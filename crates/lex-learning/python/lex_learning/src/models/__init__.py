"""Model registry for lex-learning.

This module provides a unified interface for creating models and their
hyperparameter search spaces for Optuna optimization.
"""

from __future__ import annotations

from typing import Any

import optuna

from ..config import ProblemType
from .boosting import BOOSTING_MODELS
from .neural import NEURAL_MODELS
from .sklearn_models import SKLEARN_MODELS

# Combine all model definitions
_ALL_MODELS: dict[str, dict[str, Any]] = {
    **SKLEARN_MODELS,
    **BOOSTING_MODELS,
    **NEURAL_MODELS,
}


def get_available_algorithms(problem_type: ProblemType) -> list[str]:
    """Get list of available algorithms for a problem type.

    Args:
        problem_type: Classification or regression.

    Returns:
        List of algorithm names.
    """
    return [
        name
        for name, config in _ALL_MODELS.items()
        if problem_type.value in config["problem_types"]
    ]


def create_model(
    name: str,
    problem_type: ProblemType,
    trial: optuna.Trial | None = None,
    random_seed: int = 42,
    n_jobs: int = -1,
) -> Any:
    """Create a model instance with optional hyperparameter suggestions.

    Args:
        name: Algorithm name (e.g., "random_forest", "xgboost").
        problem_type: Classification or regression.
        trial: Optional Optuna trial for hyperparameter suggestions.
        random_seed: Random seed for reproducibility.
        n_jobs: Number of parallel jobs.

    Returns:
        Configured model instance.

    Raises:
        ValueError: If algorithm name is unknown or not supported for problem type.
    """
    if name not in _ALL_MODELS:
        available = list(_ALL_MODELS.keys())
        raise ValueError(f"Unknown algorithm '{name}'. Available: {available}")

    config = _ALL_MODELS[name]

    if problem_type.value not in config["problem_types"]:
        raise ValueError(
            f"Algorithm '{name}' does not support {problem_type.value}. "
            f"Supported: {config['problem_types']}"
        )

    # Get the appropriate model class
    model_class = config[problem_type.value]

    # Get hyperparameters (either from Optuna or defaults)
    if trial is not None and "suggest_params" in config:
        params = config["suggest_params"](trial, problem_type)
    else:
        params = config.get("default_params", {}).copy()

    # Add common parameters
    if "random_state" in _get_model_params(model_class):
        params["random_state"] = random_seed
    if "n_jobs" in _get_model_params(model_class):
        params["n_jobs"] = n_jobs
    if "seed" in _get_model_params(model_class):
        params["seed"] = random_seed

    return model_class(**params)


def get_default_params(name: str) -> dict[str, Any]:
    """Get default parameters for an algorithm.

    Args:
        name: Algorithm name.

    Returns:
        Dictionary of default parameters.
    """
    if name not in _ALL_MODELS:
        return {}
    return _ALL_MODELS[name].get("default_params", {}).copy()


def _get_model_params(model_class: type) -> set[str]:
    """Get parameter names accepted by a model class."""
    import inspect

    try:
        sig = inspect.signature(model_class.__init__)
        return set(sig.parameters.keys()) - {"self"}
    except (ValueError, TypeError):
        return set()


__all__ = [
    "get_available_algorithms",
    "create_model",
    "get_default_params",
]
