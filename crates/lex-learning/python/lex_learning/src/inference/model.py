"""TrainedModel class for inference."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import numpy as np
import pandas as pd

from ..config import ProblemType
from ..core import ClassificationMetrics, ExplainabilityResult, Metrics
from ..errors import InferenceError
from ..preprocessing import Preprocessor
from .artifact import (
    ModelArtifact,
    create_artifact,
    load_artifact,
    save_artifact,
)


class TrainedModel:
    """Trained model for inference.

    This class wraps a trained model along with its preprocessor and metadata,
    providing a unified interface for saving, loading, and making predictions.

    Example:
        # Load a saved model
        model = TrainedModel.load("model.pkl")

        # Make a single prediction
        result = model.predict({"Age": 25, "Sex": "male"})
        print(result)  # {"prediction": "survived", "probability": 0.85}

        # Make batch predictions
        predictions = model.predict_batch("new_data.csv")
        predictions.to_csv("output.csv")
    """

    def __init__(self, artifact: ModelArtifact) -> None:
        """Initialize from a model artifact.

        Use TrainedModel.load() to load from a file, or TrainedModel.from_training_result()
        to create from a TrainingResult.
        """
        self._artifact = artifact

    @property
    def problem_type(self) -> ProblemType:
        """Get the problem type (classification or regression)."""
        return self._artifact.problem_type

    @property
    def target_column(self) -> str:
        """Get the name of the target column."""
        return self._artifact.target_column

    @property
    def feature_names(self) -> list[str]:
        """Get the names of input features."""
        return self._artifact.feature_names.copy()

    @property
    def class_labels(self) -> list[str] | None:
        """Get class labels for classification problems."""
        return self._artifact.class_labels.copy() if self._artifact.class_labels else None

    @property
    def metrics(self) -> Metrics:
        """Get training metrics."""
        return self._artifact.metrics

    @property
    def best_model_name(self) -> str:
        """Get the name of the best model algorithm."""
        return self._artifact.best_model_name

    @property
    def feature_importance(self) -> list[tuple[str, float]]:
        """Get feature importance from explainability."""
        return self._artifact.explainability.feature_importance.copy()

    @property
    def hyperparameters(self) -> dict[str, Any] | None:
        """Get the hyperparameters of the best model."""
        return self._artifact.hyperparameters.copy() if self._artifact.hyperparameters else None

    def save(self, path: str | Path) -> None:
        """Save the model artifact to a file.

        Args:
            path: Path to save the model (typically .pkl extension).
        """
        save_artifact(self._artifact, path)

    @classmethod
    def load(cls, path: str | Path) -> TrainedModel:
        """Load a model artifact from a file.

        Args:
            path: Path to the saved model.

        Returns:
            TrainedModel instance.

        Raises:
            ModelNotFoundError: If the file does not exist.
        """
        artifact = load_artifact(path)
        return cls(artifact)

    def predict(self, instance: dict[str, Any]) -> dict[str, Any]:
        """Make a prediction for a single instance.

        Args:
            instance: Dictionary mapping feature names to values.

        Returns:
            Dictionary with prediction and optionally probabilities.
            For classification: {
                "prediction": "class_label",
                "probability": 0.85,
                "probabilities": {"class_a": 0.85, "class_b": 0.15}
            }
            For regression: {"prediction": 45.5}

        Raises:
            InferenceError: If prediction fails.
        """
        try:
            # Convert to DataFrame
            df = pd.DataFrame([instance])

            # Ensure columns are in the right order
            df = self._prepare_dataframe(df)

            # Transform features
            X, _ = self._artifact.preprocessor.transform(df)

            # Make prediction
            pred = self._artifact.model.predict(X)[0]

            if self._artifact.problem_type == ProblemType.CLASSIFICATION:
                # Get original class label
                pred_label = self._artifact.preprocessor.inverse_transform_target(np.array([pred]))[
                    0
                ]

                result: dict[str, Any] = {"prediction": pred_label}

                # Add probabilities if available
                if hasattr(self._artifact.model, "predict_proba"):
                    proba = self._artifact.model.predict_proba(X)[0]
                    # Get probability of predicted class
                    pred_idx = int(pred)
                    result["probability"] = float(proba[pred_idx])

                    # Add full probabilities dict mapping class labels to probabilities
                    if self._artifact.class_labels:
                        result["probabilities"] = {
                            label: float(proba[i])
                            for i, label in enumerate(self._artifact.class_labels)
                        }

                return result

            else:
                return {"prediction": float(pred)}

        except Exception as e:
            raise InferenceError(f"Prediction failed: {e}") from e

    def predict_batch(
        self,
        data: pd.DataFrame | str | Path,
    ) -> pd.DataFrame:
        """Make predictions for multiple instances.

        Args:
            data: DataFrame or path to CSV file.

        Returns:
            DataFrame with original data plus "prediction" column.

        Raises:
            InferenceError: If prediction fails.
        """
        try:
            # Load data if path
            df = pd.read_csv(data) if isinstance(data, (str, Path)) else data.copy()

            # Prepare DataFrame
            df_features = self._prepare_dataframe(df)

            # Transform features
            X, _ = self._artifact.preprocessor.transform(df_features)

            # Make predictions
            predictions = self._artifact.model.predict(X)

            # Inverse transform for classification
            if self._artifact.problem_type == ProblemType.CLASSIFICATION:
                predictions = self._artifact.preprocessor.inverse_transform_target(predictions)

            # Add predictions to original DataFrame
            df["prediction"] = predictions

            return df

        except Exception as e:
            raise InferenceError(f"Batch prediction failed: {e}") from e

    def _prepare_dataframe(self, df: pd.DataFrame) -> pd.DataFrame:
        """Prepare DataFrame for prediction.

        Ensures columns are in the right order and handles missing features.
        """
        # Check for missing features
        missing = set(self._artifact.feature_names) - set(df.columns)
        if missing:
            raise InferenceError(f"Missing features: {missing}")

        # Select and order columns - cast to satisfy type checker
        result: pd.DataFrame = df[self._artifact.feature_names]  # type: ignore[assignment]
        return result

    @classmethod
    def from_training_result(
        cls,
        model: Any,
        preprocessor: Preprocessor,
        problem_type: ProblemType,
        target_column: str,
        feature_names: list[str],
        class_labels: list[str] | None,
        metrics: Metrics,
        best_model_name: str,
        training_time_seconds: float,
        explainability: ExplainabilityResult,
        hyperparameters: dict[str, Any] | None = None,
    ) -> TrainedModel:
        """Create a TrainedModel from training results.

        This is used by the Pipeline to create a TrainedModel after training.
        """
        artifact = create_artifact(
            model=model,
            preprocessor=preprocessor,
            problem_type=problem_type,
            target_column=target_column,
            feature_names=feature_names,
            class_labels=class_labels,
            metrics=metrics,
            best_model_name=best_model_name,
            training_time_seconds=training_time_seconds,
            explainability=explainability,
            hyperparameters=hyperparameters,
        )

        return cls(artifact)

    def get_info(self) -> dict[str, Any]:
        """Get model information as a dictionary.

        Returns:
            Dictionary with model metadata, metrics, and hyperparameters.
        """
        metrics = self._artifact.metrics

        # Build metrics dict based on type
        if isinstance(metrics, ClassificationMetrics):
            metrics_dict = {
                "accuracy": metrics.accuracy,
                "precision": metrics.precision,
                "recall": metrics.recall,
                "f1_score": metrics.f1_score,
                "roc_auc": metrics.roc_auc,
                "cv_score": metrics.cv_score,
                "test_score": metrics.test_score,
            }
        else:
            # RegressionMetrics
            metrics_dict = {
                "mse": metrics.mse,
                "rmse": metrics.rmse,
                "mae": metrics.mae,
                "r2": metrics.r2,
                "cv_score": metrics.cv_score,
                "test_score": metrics.test_score,
            }

        return {
            "version": self._artifact.version,
            "problem_type": self._artifact.problem_type.value,
            "target_column": self._artifact.target_column,
            "best_model_name": self._artifact.best_model_name,
            "feature_names": self._artifact.feature_names,
            "class_labels": self._artifact.class_labels,
            "trained_at": self._artifact.trained_at,
            "training_time_seconds": self._artifact.training_time_seconds,
            "metrics": metrics_dict,
            "feature_importance": self._artifact.explainability.feature_importance,
            "hyperparameters": self._artifact.hyperparameters,
        }
