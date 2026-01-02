"""Hyperparameter optimization using Optuna."""

from __future__ import annotations

import warnings
from typing import Any

import numpy as np
import optuna
from numpy.typing import NDArray
from sklearn.model_selection import cross_val_score

from ..config import PipelineConfig, ProblemType
from ..errors import CancelledError
from ..models import create_model
from ..progress import ProgressReporter

# Suppress Optuna logging
optuna.logging.set_verbosity(optuna.logging.WARNING)


def optimize_hyperparameters(
    X_train: NDArray[Any],
    y_train: NDArray[Any],
    algorithm: str,
    config: PipelineConfig,
    reporter: ProgressReporter,
) -> tuple[Any, dict[str, Any]]:
    """Optimize hyperparameters using Optuna.

    Args:
        X_train: Training features.
        y_train: Training target.
        algorithm: Algorithm name to optimize.
        config: Pipeline configuration.
        reporter: Progress reporter for cancellation checks.

    Returns:
        Tuple of (best_model, best_params).

    Raises:
        CancelledError: If training is cancelled.
    """
    # Determine scoring metric
    if config.problem_type == ProblemType.CLASSIFICATION:
        scoring = "accuracy"
        direction = "maximize"
    else:
        scoring = "neg_mean_squared_error"
        direction = "maximize"  # neg_mse, so maximize

    best_params: dict[str, Any] = {}

    def objective(trial: optuna.Trial) -> float:
        nonlocal best_params

        if reporter.is_cancelled():
            raise CancelledError()

        model = create_model(
            algorithm,
            config.problem_type,
            trial=trial,
            random_seed=config.random_seed,
            n_jobs=config.n_jobs,
        )

        # Cross-validation score
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")
            scores = cross_val_score(
                model,
                X_train,
                y_train,
                cv=config.cv_folds,
                scoring=scoring,
                n_jobs=1,  # Already using n_jobs in model
            )

        score = float(np.mean(scores))

        # Track best params
        if best_params == {} or (
            direction == "maximize" and score > trial.study.best_value
            if trial.study.best_trial is not None
            else True
        ):
            best_params = trial.params.copy()

        return score

    # Create and run study
    study = optuna.create_study(direction=direction)

    try:
        study.optimize(
            objective,
            n_trials=config.n_trials,
            show_progress_bar=False,
            catch=(Exception,),
        )
    except CancelledError:
        raise

    # Create final model with best params
    best_params = study.best_params if study.best_trial else {}
    best_model = create_model(
        algorithm,
        config.problem_type,
        trial=None,
        random_seed=config.random_seed,
        n_jobs=config.n_jobs,
    )

    # Apply best params manually
    for key, value in best_params.items():
        if hasattr(best_model, key):
            setattr(best_model, key, value)

    return best_model, best_params
