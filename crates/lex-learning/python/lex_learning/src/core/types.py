"""Result types for lex-learning.

This module contains the dataclasses for training results, model results,
explainability outputs, and internal bundles for passing data between stages.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from ..preprocessing import Preprocessor

from .metrics import Metrics


@dataclass
class ModelResult:
    """Result for a single trained model."""

    name: str
    test_score: float
    train_score: float
    cv_score: float
    training_time_seconds: float
    hyperparameters: dict[str, Any]
    overfitting_risk: str  # "low", "medium", "high"


@dataclass
class ExplainabilityResult:
    """SHAP explainability outputs."""

    # PNG bytes for plots
    summary_plot: bytes | None = None
    beeswarm_plot: bytes | None = None
    feature_importance_plot: bytes | None = None

    # Feature importance as (feature_name, importance_value) pairs
    feature_importance: list[tuple[str, float]] = field(default_factory=list)

    # Method used for explainability
    method: str = "shap"


@dataclass
class TrainingResult:
    """Complete training result.

    This is the public-facing result returned by Pipeline.train().
    It contains all information about the training process and results.
    """

    success: bool
    best_model_name: str
    metrics: Metrics
    model_comparison: list[ModelResult]
    explainability: ExplainabilityResult
    training_time_seconds: float
    warnings: list[str] = field(default_factory=list)

    # Internal data for saving (not part of public API)
    # These are prefixed with underscore to indicate they're internal
    _model: Any = field(default=None, repr=False)
    _preprocessor: Any = field(default=None, repr=False)
    _feature_names: list[str] = field(default_factory=list, repr=False)
    _class_labels: list[str] | None = field(default=None, repr=False)
    _problem_type: str = field(default="classification", repr=False)
    _target_column: str = field(default="target", repr=False)


@dataclass
class TrainingBundle:
    """Internal bundle for passing model and preprocessor between stages.

    This separates internal training artifacts from the public TrainingResult.
    Used by pipeline stages to pass data without exposing internal details.
    """

    model: Any
    preprocessor: Preprocessor
    feature_names: list[str]
    class_labels: list[str] | None
    problem_type: str
    target_column: str
