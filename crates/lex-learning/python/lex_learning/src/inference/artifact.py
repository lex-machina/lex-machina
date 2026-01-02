"""Model artifact serialization and deserialization."""

from __future__ import annotations

import pickle
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import TYPE_CHECKING, Any

from ..config import ProblemType
from ..core import ExplainabilityResult, Metrics
from ..errors import ModelNotFoundError

if TYPE_CHECKING:
    from ..preprocessing import Preprocessor

# Current artifact version
ARTIFACT_VERSION = "1.0"


@dataclass
class ModelArtifact:
    """Complete model artifact for serialization.

    Contains everything needed to recreate a trained model for inference:
    - The trained sklearn-compatible model
    - The fitted preprocessor
    - Metadata (problem type, feature names, etc.)
    - Training metrics and explainability results
    """

    version: str
    model: Any
    preprocessor: Preprocessor
    problem_type: ProblemType
    target_column: str
    feature_names: list[str]
    class_labels: list[str] | None
    metrics: Metrics
    best_model_name: str
    trained_at: str
    training_time_seconds: float
    explainability: ExplainabilityResult
    hyperparameters: dict[str, Any] | None = None


def create_artifact(
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
) -> ModelArtifact:
    """Create a new model artifact.

    Args:
        model: Trained sklearn-compatible model.
        preprocessor: Fitted preprocessor.
        problem_type: Classification or regression.
        target_column: Name of target column.
        feature_names: List of input feature names.
        class_labels: Class labels for classification (None for regression).
        metrics: Training metrics.
        best_model_name: Name of the model algorithm.
        training_time_seconds: Time taken to train.
        explainability: SHAP explainability results.
        hyperparameters: Best model hyperparameters.

    Returns:
        ModelArtifact ready for serialization.
    """
    return ModelArtifact(
        version=ARTIFACT_VERSION,
        model=model,
        preprocessor=preprocessor,
        problem_type=problem_type,
        target_column=target_column,
        feature_names=feature_names,
        class_labels=class_labels,
        metrics=metrics,
        best_model_name=best_model_name,
        trained_at=datetime.now().isoformat(),
        training_time_seconds=training_time_seconds,
        explainability=explainability,
        hyperparameters=hyperparameters,
    )


def save_artifact(artifact: ModelArtifact, path: str | Path) -> None:
    """Save a model artifact to disk.

    Args:
        artifact: The model artifact to save.
        path: Path to save the artifact (typically .pkl extension).
    """
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)

    with open(path, "wb") as f:
        pickle.dump(artifact, f, protocol=pickle.HIGHEST_PROTOCOL)


def load_artifact(path: str | Path) -> ModelArtifact:
    """Load a model artifact from disk.

    Args:
        path: Path to the saved artifact.

    Returns:
        Loaded ModelArtifact.

    Raises:
        ModelNotFoundError: If the file does not exist.
    """
    path = Path(path)

    if not path.exists():
        raise ModelNotFoundError(str(path))

    with open(path, "rb") as f:
        artifact = pickle.load(f)

    # Version validation could be added here in the future
    # For now, we trust the artifact format

    return artifact
