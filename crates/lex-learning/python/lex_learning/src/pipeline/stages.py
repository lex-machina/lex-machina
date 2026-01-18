"""Pipeline stages implementing the PipelineStage protocol.

Each stage is a single-responsibility class that operates on PipelineContext,
performing one step of the ML training pipeline.
"""

from __future__ import annotations

from typing import Any

import numpy as np
from sklearn.metrics import (
    accuracy_score,
    confusion_matrix,
    f1_score,
    mean_absolute_error,
    mean_squared_error,
    precision_score,
    r2_score,
    recall_score,
    roc_auc_score,
)
from sklearn.model_selection import train_test_split

from ..config import ProblemType
from ..core import (
    ClassificationMetrics,
    ExplainabilityResult,
    PipelineContext,
    RegressionMetrics,
)
from ..errors import InvalidDataError, TargetNotFoundError
from ..explainability import explain_model
from ..models import get_available_algorithms
from ..preprocessing import Preprocessor
from ..progress import ProgressUpdate, TrainingStage
from ..training import DatasetInfo, select_algorithms, train_models


class ValidationStage:
    """Validates input data and prepares features/target split.

    Checks for:
    - Target column existence
    - Null values
    - Minimum sample count

    Sets on context: X, y, target_column
    """

    def execute(self, context: PipelineContext) -> PipelineContext:
        """Execute validation stage."""
        context.reporter.report(
            ProgressUpdate(
                stage=TrainingStage.PREPROCESSING,
                progress=0.05,
                message="Validating data...",
            )
        )

        if context.data is None:
            raise InvalidDataError("No data provided to pipeline")

        data = context.data

        # Determine target column
        target_column = context.config.target_column
        if target_column is None:
            target_column = str(data.columns[-1])

        # Check target exists
        if target_column not in data.columns:
            raise TargetNotFoundError(target_column, list(data.columns))

        # Check for nulls
        if data.isnull().any().any():
            null_cols = data.columns[data.isnull().any()].tolist()
            raise InvalidDataError(f"Data contains null values in columns: {null_cols}")

        # Check minimum samples
        if len(data) < 10:
            raise InvalidDataError(f"Data must have at least 10 samples, got {len(data)}")

        # Split features and target
        context.X = data.drop(columns=[target_column])
        context.y = data[target_column]
        context.target_column = target_column

        return context


class PreprocessingStage:
    """Applies preprocessing transformations to features.

    Handles:
    - Categorical encoding (OneHot)
    - Numeric scaling (Standard)
    - Target encoding (for classification)

    Sets on context: preprocessor, X_transformed (via preprocessor)
    """

    def execute(self, context: PipelineContext) -> PipelineContext:
        """Execute preprocessing stage."""
        context.reporter.report(
            ProgressUpdate(
                stage=TrainingStage.PREPROCESSING,
                progress=0.08,
                message="Preprocessing data...",
            )
        )

        if context.X is None or context.y is None:
            raise InvalidDataError("Validation stage must run before preprocessing")

        preprocessor = Preprocessor(context.config.problem_type)
        X_transformed, y_transformed = preprocessor.fit_transform(context.X, context.y)

        context.preprocessor = preprocessor
        # Store transformed data temporarily for split stage
        context._X_transformed = X_transformed
        context._y_transformed = y_transformed

        return context


class SplitStage:
    """Splits data into training and test sets.

    Uses stratified split for classification problems.

    Sets on context: X_train, X_test, y_train, y_test
    """

    def execute(self, context: PipelineContext) -> PipelineContext:
        """Execute split stage."""
        context.reporter.report(
            ProgressUpdate(
                stage=TrainingStage.PREPROCESSING,
                progress=0.10,
                message="Splitting data...",
            )
        )

        # Get transformed data from preprocessing stage
        X_transformed = getattr(context, "_X_transformed", None)
        y_transformed = getattr(context, "_y_transformed", None)

        if X_transformed is None or y_transformed is None:
            raise InvalidDataError("Preprocessing stage must run before split")

        X_train, X_test, y_train, y_test = train_test_split(
            X_transformed,
            y_transformed,
            test_size=context.config.test_size,
            random_state=context.config.random_seed,
            stratify=y_transformed
            if context.config.problem_type == ProblemType.CLASSIFICATION
            else None,
        )

        context.X_train = X_train
        context.X_test = X_test
        context.y_train = y_train
        context.y_test = y_test

        # Clean up temporary attributes
        if hasattr(context, "_X_transformed"):
            delattr(context, "_X_transformed")
        if hasattr(context, "_y_transformed"):
            delattr(context, "_y_transformed")

        return context


class AlgorithmSelectionStage:
    """Selects algorithms to train based on dataset characteristics.

    Uses heuristics to choose the best algorithms for the data size
    and problem type.

    Sets on context: algorithms
    """

    def execute(self, context: PipelineContext) -> PipelineContext:
        """Execute algorithm selection stage."""
        context.reporter.report(
            ProgressUpdate(
                stage=TrainingStage.ALGORITHM_SELECTION,
                progress=0.15,
                message="Selecting algorithms...",
            )
        )

        if context.X_train is None or context.y_train is None:
            raise InvalidDataError("Split stage must run before algorithm selection")

        # If specific algorithm requested
        if context.config.algorithm:
            context.algorithms = [context.config.algorithm]
            return context

        # Get available algorithms
        available = get_available_algorithms(context.config.problem_type)

        # Filter neural networks if disabled
        if not context.config.enable_neural_networks:
            available = [a for a in available if a != "neural_network"]

        # Create dataset info
        n_classes = (
            len(np.unique(context.y_train))
            if context.config.problem_type == ProblemType.CLASSIFICATION
            else None
        )
        dataset_info = DatasetInfo(
            n_samples=context.X_train.shape[0],
            n_features=context.X_train.shape[1],
            problem_type=context.config.problem_type,
            n_classes=n_classes,
        )

        # Select top-k algorithms
        context.algorithms = select_algorithms(
            dataset_info,
            available,
            top_k=context.config.top_k_algorithms,
            include_neural=context.config.enable_neural_networks,
        )

        return context


class ModelTrainingStage:
    """Trains selected algorithms with hyperparameter optimization.

    Uses Optuna for hyperparameter tuning and cross-validation
    for model evaluation.

    Sets on context: model, model_results
    """

    def execute(self, context: PipelineContext) -> PipelineContext:
        """Execute training stage."""
        if context.algorithms is None:
            raise InvalidDataError("Algorithm selection stage must run before training")

        if (
            context.X_train is None
            or context.y_train is None
            or context.X_test is None
            or context.y_test is None
        ):
            raise InvalidDataError("Split stage must run before training")

        context.reporter.report(
            ProgressUpdate(
                stage=TrainingStage.TRAINING,
                progress=0.20,
                message=f"Training {len(context.algorithms)} models...",
            )
        )

        best_model, model_results = train_models(
            context.X_train,
            context.y_train,
            context.X_test,
            context.y_test,
            context.algorithms,
            context.config,
            context.reporter,
        )

        context.model = best_model
        context.model_results = model_results

        return context


class EvaluationStage:
    """Evaluates the best model on the test set.

    Calculates appropriate metrics based on problem type:
    - Classification: accuracy, precision, recall, F1, ROC-AUC
    - Regression: MSE, RMSE, MAE, R2

    Sets on context: metrics
    """

    def execute(self, context: PipelineContext) -> PipelineContext:
        """Execute evaluation stage."""
        context.reporter.report(
            ProgressUpdate(
                stage=TrainingStage.EVALUATION,
                progress=0.90,
                message="Calculating metrics...",
            )
        )

        if context.model is None or context.X_test is None or context.y_test is None:
            raise InvalidDataError("Training stage must run before evaluation")

        context.metrics = self._calculate_metrics(
            context.model,
            context.X_test,
            context.y_test,
            context.config.problem_type,
        )

        return context

    def _calculate_metrics(
        self,
        model: Any,
        X_test: np.ndarray,
        y_test: np.ndarray,
        problem_type: ProblemType,
    ) -> ClassificationMetrics | RegressionMetrics:
        """Calculate evaluation metrics."""
        y_pred = model.predict(X_test)

        if problem_type == ProblemType.CLASSIFICATION:
            accuracy = float(accuracy_score(y_test, y_pred))
            precision = float(precision_score(y_test, y_pred, average="weighted", zero_division=0))
            recall = float(recall_score(y_test, y_pred, average="weighted", zero_division=0))
            f1 = float(f1_score(y_test, y_pred, average="weighted", zero_division=0))

            # ROC AUC (binary classification only)
            roc_auc = None
            if len(np.unique(y_test)) == 2 and hasattr(model, "predict_proba"):
                y_proba = model.predict_proba(X_test)[:, 1]
                roc_auc = float(roc_auc_score(y_test, y_proba))

            # Confusion matrix
            cm = confusion_matrix(y_test, y_pred).tolist()

            return ClassificationMetrics(
                accuracy=accuracy,
                precision=precision,
                recall=recall,
                f1_score=f1,
                roc_auc=roc_auc,
                confusion_matrix=cm,
                test_score=accuracy,
            )

        else:
            mse = float(mean_squared_error(y_test, y_pred))
            rmse = float(np.sqrt(mse))
            mae = float(mean_absolute_error(y_test, y_pred))
            r2 = float(r2_score(y_test, y_pred))

            return RegressionMetrics(
                mse=mse,
                rmse=rmse,
                mae=mae,
                r2=r2,
                test_score=r2,
            )


class ExplainabilityStage:
    """Generates SHAP explainability plots and feature importance.

    Uses appropriate SHAP explainer based on model type:
    - TreeExplainer for tree-based models
    - LinearExplainer for linear models
    - KernelExplainer as fallback

    Sets on context: explainability
    """

    def execute(self, context: PipelineContext) -> PipelineContext:
        """Execute explainability stage."""
        if not context.config.enable_explainability:
            context.explainability = ExplainabilityResult(method="disabled")
            return context

        context.reporter.report(
            ProgressUpdate(
                stage=TrainingStage.EXPLAINABILITY,
                progress=0.95,
                message="Generating explanations...",
            )
        )

        if context.model is None or context.X_test is None or context.preprocessor is None:
            raise InvalidDataError("Training stage must run before explainability")

        context.explainability = explain_model(
            context.model,
            context.X_test,
            context.preprocessor.transformed_feature_names,
            context.config.problem_type,
            context.config.shap_max_samples,
        )

        return context


# Default stage order for the pipeline
DEFAULT_STAGES: list[
    type[
        ValidationStage
        | PreprocessingStage
        | SplitStage
        | AlgorithmSelectionStage
        | ModelTrainingStage
        | EvaluationStage
        | ExplainabilityStage
    ]
] = [
    ValidationStage,
    PreprocessingStage,
    SplitStage,
    AlgorithmSelectionStage,
    ModelTrainingStage,
    EvaluationStage,
    ExplainabilityStage,
]
