"""Progress reporting classes and types.

This module contains the core progress reporting infrastructure including
the TrainingStage enum, ProgressUpdate dataclass, and reporter implementations.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from enum import Enum
from typing import Protocol


class TrainingStage(Enum):
    """Stages of the training pipeline."""

    INITIALIZING = "initializing"
    PREPROCESSING = "preprocessing"
    ALGORITHM_SELECTION = "algorithm_selection"
    TRAINING = "training"
    EVALUATION = "evaluation"
    EXPLAINABILITY = "explainability"
    COMPLETE = "complete"
    FAILED = "failed"
    CANCELLED = "cancelled"


@dataclass
class ProgressUpdate:
    """Progress update during training.

    Attributes:
        stage: Current training stage.
        progress: Progress value from 0.0 to 1.0.
        message: Human-readable status message.
        current_model: Name of the model currently being trained (if any).
        models_completed: Tuple of (completed, total) models.
    """

    stage: TrainingStage
    progress: float  # 0.0 to 1.0
    message: str
    current_model: str | None = None
    models_completed: tuple[int, int] | None = None  # (completed, total)


# Type alias for progress callback
ProgressCallback = Callable[[ProgressUpdate], None]

# Type alias for cancellation check
CancellationCheck = Callable[[], bool]


class ProgressReporter(Protocol):
    """Protocol for progress reporting.

    Implement this protocol to receive progress updates during training.
    """

    def report(self, update: ProgressUpdate) -> None:
        """Report a progress update.

        Args:
            update: The progress update to report.
        """
        ...

    def is_cancelled(self) -> bool:
        """Check if the operation should be cancelled.

        Returns:
            True if the operation should be cancelled, False otherwise.
        """
        ...


class CallbackProgressReporter:
    """Progress reporter that uses callbacks.

    This reporter calls a progress callback function for each update
    and a cancellation check function to determine if training should stop.
    """

    def __init__(
        self,
        progress_callback: ProgressCallback | None = None,
        cancellation_check: CancellationCheck | None = None,
    ) -> None:
        """Initialize the callback progress reporter.

        Args:
            progress_callback: Function to call with progress updates.
            cancellation_check: Function to call to check for cancellation.
        """
        self._progress_callback = progress_callback
        self._cancellation_check = cancellation_check

    def report(self, update: ProgressUpdate) -> None:
        """Report progress update via callback."""
        if self._progress_callback is not None:
            self._progress_callback(update)

    def is_cancelled(self) -> bool:
        """Check if operation should be cancelled."""
        if self._cancellation_check is not None:
            return self._cancellation_check()
        return False


class NullProgressReporter:
    """No-op progress reporter.

    Use this when you don't need progress reporting.
    """

    def report(self, update: ProgressUpdate) -> None:
        """Do nothing."""
        pass

    def is_cancelled(self) -> bool:
        """Never cancelled."""
        return False
