"""Core types and protocols for lex-learning.

This module contains the foundational types, metrics, and protocols
used throughout the library.
"""

from __future__ import annotations

from .metrics import (
    BaseMetrics,
    ClassificationMetrics,
    Metrics,
    RegressionMetrics,
)
from .protocols import PipelineContext, PipelineStage
from .types import (
    ExplainabilityResult,
    ModelResult,
    TrainingBundle,
    TrainingResult,
)

__all__ = [
    # Metrics
    "BaseMetrics",
    "ClassificationMetrics",
    "RegressionMetrics",
    "Metrics",
    # Types
    "ModelResult",
    "ExplainabilityResult",
    "TrainingResult",
    "TrainingBundle",
    # Protocols
    "PipelineStage",
    "PipelineContext",
]
