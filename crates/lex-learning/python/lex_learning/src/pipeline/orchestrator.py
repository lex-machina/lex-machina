"""Training pipeline orchestrator.

This module provides the Pipeline class that orchestrates the ML training
process by executing a sequence of pipeline stages.
"""

from __future__ import annotations

import time
from typing import TYPE_CHECKING

import pandas as pd

from ..config import PipelineConfig
from ..core import (
    ExplainabilityResult,
    PipelineContext,
    TrainingResult,
)
from ..errors import CancelledError
from ..inference import TrainedModel
from ..progress import (
    CallbackProgressReporter,
    NullProgressReporter,
    ProgressCallback,
    ProgressReporter,
    ProgressUpdate,
    TrainingStage,
)
from .stages import (
    AlgorithmSelectionStage,
    EvaluationStage,
    ExplainabilityStage,
    ModelTrainingStage,
    PreprocessingStage,
    SplitStage,
    ValidationStage,
)

if TYPE_CHECKING:
    from typing import Self


class Pipeline:
    """Training pipeline for automated ML.

    The pipeline executes a sequence of stages:
    1. ValidationStage - Validates input data
    2. PreprocessingStage - Encodes and scales features
    3. SplitStage - Splits data into train/test sets
    4. AlgorithmSelectionStage - Selects algorithms based on data characteristics
    5. ModelTrainingStage - Trains models with hyperparameter optimization
    6. EvaluationStage - Calculates performance metrics
    7. ExplainabilityStage - Generates SHAP explanations

    Usage:
        result = Pipeline.builder() \\
            .config(config) \\
            .on_progress(lambda u: print(u.message)) \\
            .build() \\
            .train(dataframe)
    """

    def __init__(
        self,
        config: PipelineConfig,
        progress_callback: ProgressCallback | None = None,
    ) -> None:
        """Initialize pipeline.

        Use Pipeline.builder() for a fluent interface.
        """
        self._config = config
        self._progress_callback = progress_callback
        self._reporter: ProgressReporter = (
            CallbackProgressReporter(progress_callback)
            if progress_callback
            else NullProgressReporter()
        )

        # Initialize stages
        self._stages = [
            ValidationStage(),
            PreprocessingStage(),
            SplitStage(),
            AlgorithmSelectionStage(),
            ModelTrainingStage(),
            EvaluationStage(),
            ExplainabilityStage(),
        ]

    @classmethod
    def builder(cls) -> PipelineBuilder:
        """Create a builder for Pipeline."""
        return PipelineBuilder()

    def train(self, data: pd.DataFrame) -> TrainingResult:
        """Train models on the provided data.

        Args:
            data: DataFrame with features and target column.
                  Target should be the last column if target_column not specified.

        Returns:
            TrainingResult with best model and metrics.

        Raises:
            InvalidDataError: If data validation fails.
            TargetNotFoundError: If target column not found.
            TrainingFailedError: If all models fail.
            CancelledError: If training is cancelled.
        """
        start_time = time.time()

        try:
            # Initialize context
            self._report(TrainingStage.INITIALIZING, 0.0, "Initializing pipeline...")

            context = PipelineContext(
                config=self._config,
                reporter=self._reporter,
                data=data,
                start_time=start_time,
            )

            # Execute each stage
            for stage in self._stages:
                context = stage.execute(context)

            # Complete
            training_time = time.time() - start_time
            self._report(TrainingStage.COMPLETE, 1.0, "Training complete!")

            # Build result from context
            return self._build_result(context, training_time)

        except CancelledError:
            self._report(TrainingStage.CANCELLED, 0.0, "Training cancelled")
            raise
        except Exception as e:
            self._report(TrainingStage.FAILED, 0.0, f"Training failed: {e}")
            raise

    def _report(self, stage: TrainingStage, progress: float, message: str) -> None:
        """Report progress."""
        self._reporter.report(
            ProgressUpdate(
                stage=stage,
                progress=progress,
                message=message,
            )
        )

    def _build_result(self, context: PipelineContext, training_time: float) -> TrainingResult:
        """Build TrainingResult from completed context."""
        # Ensure required fields are present
        assert context.model_results is not None
        assert context.metrics is not None
        assert context.preprocessor is not None
        assert context.target_column is not None

        return TrainingResult(
            success=True,
            best_model_name=context.model_results[0].name,
            metrics=context.metrics,
            model_comparison=context.model_results,
            explainability=context.explainability or ExplainabilityResult(method="none"),
            training_time_seconds=training_time,
            warnings=context.warnings,
            _model=context.model,
            _preprocessor=context.preprocessor,
            _feature_names=context.preprocessor.feature_names,
            _class_labels=context.preprocessor.class_labels,
            _problem_type=self._config.problem_type.value,
            _target_column=context.target_column,
        )

    def create_trained_model(self, result: TrainingResult) -> TrainedModel:
        """Create a TrainedModel from a TrainingResult.

        This is a convenience method for creating a TrainedModel that can be saved.
        """
        # Get hyperparameters from the best model in model_comparison
        hyperparameters = None
        if result.model_comparison:
            # Find the best model (first one is the best)
            best_model_result = result.model_comparison[0]
            hyperparameters = best_model_result.hyperparameters

        return TrainedModel.from_training_result(
            model=result._model,
            preprocessor=result._preprocessor,
            problem_type=self._config.problem_type,
            target_column=result._target_column,
            feature_names=result._feature_names,
            class_labels=result._class_labels,
            metrics=result.metrics,
            best_model_name=result.best_model_name,
            training_time_seconds=result.training_time_seconds,
            explainability=result.explainability,
            hyperparameters=hyperparameters,
        )


class PipelineBuilder:
    """Builder for Pipeline with fluent interface."""

    def __init__(self) -> None:
        self._config: PipelineConfig | None = None
        self._progress_callback: ProgressCallback | None = None

    def config(self, config: PipelineConfig) -> Self:
        """Set the pipeline configuration."""
        self._config = config
        return self

    def on_progress(self, callback: ProgressCallback) -> Self:
        """Set the progress callback."""
        self._progress_callback = callback
        return self

    def build(self) -> Pipeline:
        """Build the Pipeline."""
        if self._config is None:
            raise ValueError("config is required")

        return Pipeline(
            config=self._config,
            progress_callback=self._progress_callback,
        )
