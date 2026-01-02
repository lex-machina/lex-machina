"""Pipeline module for ML training orchestration.

This module provides the Pipeline class and related components for
orchestrating the ML training process.
"""

from __future__ import annotations

from .orchestrator import Pipeline, PipelineBuilder
from .stages import (
    DEFAULT_STAGES,
    AlgorithmSelectionStage,
    EvaluationStage,
    ExplainabilityStage,
    ModelTrainingStage,
    PreprocessingStage,
    SplitStage,
    ValidationStage,
)

__all__ = [
    # Main orchestrator
    "Pipeline",
    "PipelineBuilder",
    # Stages
    "ValidationStage",
    "PreprocessingStage",
    "SplitStage",
    "AlgorithmSelectionStage",
    "ModelTrainingStage",
    "EvaluationStage",
    "ExplainabilityStage",
    "DEFAULT_STAGES",
]
