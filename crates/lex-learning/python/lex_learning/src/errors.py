"""Exception hierarchy for lex-learning."""

from __future__ import annotations


class LexMLError(Exception):
    """Base exception for lex-learning."""

    pass


class InvalidConfigError(LexMLError):
    """Invalid configuration provided."""

    def __init__(self, message: str) -> None:
        self.message = message
        super().__init__(f"Invalid configuration: {message}")


class InvalidDataError(LexMLError):
    """Data validation failed."""

    def __init__(self, message: str) -> None:
        self.message = message
        super().__init__(f"Invalid data: {message}")


class TargetNotFoundError(LexMLError):
    """Target column not found in data."""

    def __init__(self, column: str, available: list[str]) -> None:
        self.column = column
        self.available = available
        super().__init__(f"Target column '{column}' not found. Available columns: {available}")


class TrainingFailedError(LexMLError):
    """All models failed to train."""

    def __init__(self, failures: dict[str, str]) -> None:
        self.failures = failures
        details = "\n".join(f"  - {name}: {error}" for name, error in failures.items())
        super().__init__(f"All models failed to train:\n{details}")


class ModelNotFoundError(LexMLError):
    """Model file not found."""

    def __init__(self, path: str) -> None:
        self.path = path
        super().__init__(f"Model file not found: {path}")


class InferenceError(LexMLError):
    """Prediction failed."""

    def __init__(self, message: str) -> None:
        self.message = message
        super().__init__(f"Inference failed: {message}")


class CancelledError(LexMLError):
    """Training was cancelled by user."""

    def __init__(self) -> None:
        super().__init__("Training was cancelled")


class ExplainabilityError(LexMLError):
    """Explainability analysis failed."""

    def __init__(self, message: str) -> None:
        self.message = message
        super().__init__(f"Explainability failed: {message}")
