"""Protocols and shared interfaces for the pipeline stages.

This module defines the PipelineStage protocol that all pipeline stages implement,
along with the PipelineContext dataclass that flows through the pipeline.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any, Protocol

import pandas as pd
from numpy.typing import NDArray

if TYPE_CHECKING:
    from ..config import PipelineConfig
    from ..preprocessing import Preprocessor
    from ..progress import ProgressReporter
    from .metrics import Metrics
    from .types import ExplainabilityResult, ModelResult


@dataclass
class PipelineContext:
    """Context object that flows through pipeline stages.

    This contains all the state needed by pipeline stages, avoiding
    the need to pass many parameters between stages.
    """

    # Configuration (always present)
    config: PipelineConfig
    reporter: ProgressReporter

    # Raw data (set by validation stage)
    data: pd.DataFrame | None = None
    X: pd.DataFrame | None = None
    y: pd.Series | None = None
    target_column: str | None = None

    # Split data (set by split stage)
    X_train: NDArray[Any] | None = None
    y_train: NDArray[Any] | None = None
    X_test: NDArray[Any] | None = None
    y_test: NDArray[Any] | None = None

    # Preprocessor (set by preprocessing stage)
    preprocessor: Preprocessor | None = None

    # Algorithm selection (set by selection stage)
    algorithms: list[str] | None = None

    # Training results (set by training stage)
    model: Any = None
    model_results: list[ModelResult] | None = None

    # Evaluation results (set by evaluation stage)
    metrics: Metrics | None = None

    # Explainability results (set by explainability stage)
    explainability: ExplainabilityResult | None = None

    # Warnings accumulated during pipeline execution
    warnings: list[str] = field(default_factory=list)

    # Training start time for calculating total duration
    start_time: float = 0.0


class PipelineStage(Protocol):
    """Protocol for pipeline stages.

    Each stage takes a PipelineContext, performs its work, and returns
    the (possibly modified) context. This allows stages to be composed
    and executed in sequence.
    """

    def execute(self, context: PipelineContext) -> PipelineContext:
        """Execute the pipeline stage.

        Args:
            context: The current pipeline context.

        Returns:
            The updated pipeline context.

        Raises:
            Various exceptions depending on the stage.
        """
        ...
