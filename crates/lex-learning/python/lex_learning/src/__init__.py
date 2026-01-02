"""lex-learning: Automated ML training library.

This library provides a simple interface for training machine learning models
with automatic algorithm selection, hyperparameter optimization, and explainability.

Example usage:
    from lex_learning import Pipeline, PipelineConfig, ProblemType

    config = PipelineConfig.builder() \\
        .problem_type(ProblemType.CLASSIFICATION) \\
        .target_column("Survived") \\
        .build()

    result = Pipeline.builder() \\
        .config(config) \\
        .on_progress(lambda u: print(f"{u.progress:.0%} - {u.message}")) \\
        .build() \\
        .train(dataframe)

    # Save model
    trained_model = pipeline.create_trained_model(result)
    trained_model.save("model.pkl")

    # Load and predict
    model = TrainedModel.load("model.pkl")
    prediction = model.predict({"Age": 25, "Sex": "male"})
"""

from __future__ import annotations

# Configuration
from .config import PipelineConfig, PipelineConfigBuilder, ProblemType

# Types (from core module)
from .core import (
    BaseMetrics,
    ClassificationMetrics,
    ExplainabilityResult,
    Metrics,
    ModelResult,
    PipelineContext,
    PipelineStage,
    RegressionMetrics,
    TrainingBundle,
    TrainingResult,
)

# Errors
from .errors import (
    CancelledError,
    ExplainabilityError,
    InferenceError,
    InvalidConfigError,
    InvalidDataError,
    LexMLError,
    ModelNotFoundError,
    TargetNotFoundError,
    TrainingFailedError,
)

# Model
from .inference import TrainedModel

# Pipeline
from .pipeline import Pipeline, PipelineBuilder

# Progress
from .progress import (
    CallbackProgressReporter,
    NullProgressReporter,
    ProgressCallback,
    ProgressReporter,
    ProgressUpdate,
    TrainingStage,
)

__version__ = "0.1.0"

__all__ = [
    # Version
    "__version__",
    # Configuration
    "PipelineConfig",
    "PipelineConfigBuilder",
    "ProblemType",
    # Pipeline
    "Pipeline",
    "PipelineBuilder",
    # Model
    "TrainedModel",
    # Progress
    "TrainingStage",
    "ProgressUpdate",
    "ProgressCallback",
    "ProgressReporter",
    "CallbackProgressReporter",
    "NullProgressReporter",
    # Types (from core)
    "BaseMetrics",
    "ClassificationMetrics",
    "RegressionMetrics",
    "Metrics",
    "ModelResult",
    "ExplainabilityResult",
    "TrainingResult",
    "TrainingBundle",
    "PipelineStage",
    "PipelineContext",
    # Errors
    "LexMLError",
    "InvalidConfigError",
    "InvalidDataError",
    "TargetNotFoundError",
    "TrainingFailedError",
    "ModelNotFoundError",
    "InferenceError",
    "CancelledError",
    "ExplainabilityError",
]
