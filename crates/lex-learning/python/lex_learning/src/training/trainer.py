"""Model training orchestration."""

from __future__ import annotations

import logging
import time
import warnings
from typing import Any

import numpy as np
from numpy.typing import NDArray
from sklearn.base import clone
from sklearn.model_selection import cross_val_score

from ..config import PipelineConfig, ProblemType
from ..core import ModelResult
from ..errors import CancelledError, TrainingFailedError
from ..models import create_model, get_default_params
from ..progress import (
    NullProgressReporter,
    ProgressReporter,
    ProgressUpdate,
    TrainingStage,
)
from .optimizer import optimize_hyperparameters

logger = logging.getLogger(__name__)


def train_single_model(
    X_train: NDArray[Any],
    y_train: NDArray[Any],
    X_test: NDArray[Any],
    y_test: NDArray[Any],
    algorithm: str,
    config: PipelineConfig,
    progress_reporter: ProgressReporter | None = None,
) -> tuple[Any, ModelResult] | None:
    """Train a single model with optional hyperparameter optimization.

    Args:
        X_train: Training features.
        y_train: Training target.
        X_test: Test features.
        y_test: Test target.
        algorithm: Algorithm name.
        config: Pipeline configuration.
        progress_reporter: Optional progress reporter.

    Returns:
        Tuple of (trained_model, result) or None if training failed.
    """
    reporter = progress_reporter or NullProgressReporter()
    start_time = time.time()

    try:
        if config.optimize_hyperparams:
            model, best_params = optimize_hyperparameters(
                X_train,
                y_train,
                algorithm,
                config,
                reporter,
            )
        else:
            # Use default parameters
            best_params = get_default_params(algorithm)
            model = create_model(
                algorithm,
                config.problem_type,
                trial=None,
                random_seed=config.random_seed,
                n_jobs=config.n_jobs,
            )

        # Fit model on full training data
        model.fit(X_train, y_train)

        # Calculate scores
        train_score = _score_model(model, X_train, y_train, config.problem_type)
        test_score = _score_model(model, X_test, y_test, config.problem_type)
        cv_score = _cross_validate(model, X_train, y_train, config)

        training_time = time.time() - start_time

        # Assess overfitting risk
        overfitting_risk = _assess_overfitting(train_score, test_score)

        result = ModelResult(
            name=algorithm,
            test_score=test_score,
            train_score=train_score,
            cv_score=cv_score,
            training_time_seconds=training_time,
            hyperparameters=best_params,
            overfitting_risk=overfitting_risk,
        )

        return model, result

    except Exception as e:
        logger.warning(f"Training {algorithm} failed: {e}")
        return None


def train_models(
    X_train: NDArray[Any],
    y_train: NDArray[Any],
    X_test: NDArray[Any],
    y_test: NDArray[Any],
    algorithms: list[str],
    config: PipelineConfig,
    progress_reporter: ProgressReporter | None = None,
) -> tuple[Any, list[ModelResult]]:
    """Train multiple models and return the best one.

    Args:
        X_train: Training features.
        y_train: Training target.
        X_test: Test features.
        y_test: Test target.
        algorithms: List of algorithm names to try.
        config: Pipeline configuration.
        progress_reporter: Optional progress reporter.

    Returns:
        Tuple of (best_model, list_of_all_results).

    Raises:
        TrainingFailedError: If all models fail to train.
        CancelledError: If training is cancelled.
    """
    reporter = progress_reporter or NullProgressReporter()
    results: list[ModelResult] = []
    models: dict[str, Any] = {}
    failures: dict[str, str] = {}

    n_algorithms = len(algorithms)

    for i, algorithm in enumerate(algorithms):
        # Check for cancellation
        if reporter.is_cancelled():
            raise CancelledError()

        # Report progress
        progress = 0.2 + (0.65 * i / n_algorithms)  # 20% to 85%
        reporter.report(
            ProgressUpdate(
                stage=TrainingStage.TRAINING,
                progress=progress,
                message=f"Training {algorithm}...",
                current_model=algorithm,
                models_completed=(i, n_algorithms),
            )
        )

        try:
            result = train_single_model(
                X_train,
                y_train,
                X_test,
                y_test,
                algorithm,
                config,
                reporter,
            )

            if result is not None:
                model, model_result = result
                models[algorithm] = model
                results.append(model_result)
            else:
                failures[algorithm] = "Training returned no result"

        except CancelledError:
            raise
        except Exception as e:
            failures[algorithm] = str(e)
            logger.warning(f"Training {algorithm} failed: {e}")

    if not results:
        raise TrainingFailedError(failures)

    # Sort by test score (descending)
    results.sort(key=lambda r: r.test_score, reverse=True)

    # Get best model
    best_algorithm = results[0].name
    best_model = models[best_algorithm]

    return best_model, results


def _score_model(
    model: Any,
    X: NDArray[Any],
    y: NDArray[Any],
    problem_type: ProblemType,
) -> float:
    """Score a model on given data."""
    if problem_type == ProblemType.CLASSIFICATION:
        return float(model.score(X, y))  # accuracy
    else:
        # R2 score for regression
        return float(model.score(X, y))


def _cross_validate(
    model: Any,
    X: NDArray[Any],
    y: NDArray[Any],
    config: PipelineConfig,
) -> float:
    """Perform cross-validation and return mean score."""
    scoring = "accuracy" if config.problem_type == ProblemType.CLASSIFICATION else "r2"

    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        # Clone the model to avoid fitting the original
        model_clone = clone(model)
        scores = cross_val_score(
            model_clone,
            X,
            y,
            cv=config.cv_folds,
            scoring=scoring,
            n_jobs=config.n_jobs,
        )

    return float(np.mean(scores))


def _assess_overfitting(train_score: float, test_score: float) -> str:
    """Assess overfitting risk based on train/test score gap."""
    gap = train_score - test_score

    if gap < 0.05:
        return "low"
    elif gap < 0.15:
        return "medium"
    else:
        return "high"
