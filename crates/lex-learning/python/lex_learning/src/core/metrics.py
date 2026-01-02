"""Metrics dataclasses for model evaluation.

This module provides separate metric classes for classification and regression,
eliminating the need for nullable fields in a single combined class.
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class BaseMetrics:
    """Common metrics for all problem types."""

    cv_score: float = 0.0
    test_score: float = 0.0
    train_score: float = 0.0


@dataclass
class ClassificationMetrics(BaseMetrics):
    """Metrics for classification problems."""

    accuracy: float = 0.0
    precision: float = 0.0
    recall: float = 0.0
    f1_score: float = 0.0
    roc_auc: float | None = None  # Only for binary classification
    confusion_matrix: list[list[int]] | None = None


@dataclass
class RegressionMetrics(BaseMetrics):
    """Metrics for regression problems."""

    mse: float = 0.0
    rmse: float = 0.0
    mae: float = 0.0
    r2: float = 0.0


# Type alias for metrics - can be either classification or regression
Metrics = ClassificationMetrics | RegressionMetrics
