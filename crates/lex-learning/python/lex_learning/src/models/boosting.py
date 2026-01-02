"""XGBoost and LightGBM model definitions and hyperparameter spaces."""

from __future__ import annotations

from typing import Any

import optuna

from ..config import ProblemType

# Lazy imports to handle optional dependencies
_xgboost_available = True
_lightgbm_available = True

try:
    import xgboost as xgb
except ImportError:
    _xgboost_available = False
    xgb = None  # type: ignore[assignment]

try:
    import lightgbm as lgb
except ImportError:
    _lightgbm_available = False
    lgb = None  # type: ignore[assignment]


def _suggest_xgboost(trial: optuna.Trial, problem_type: ProblemType) -> dict[str, Any]:
    """Suggest hyperparameters for XGBoost optimization.

    Args:
        trial: Optuna trial for hyperparameter suggestion.
        problem_type: Classification or regression.

    Returns:
        Dictionary of suggested hyperparameters.
    """
    params: dict[str, Any] = {
        "n_estimators": trial.suggest_int("n_estimators", 50, 300),
        "learning_rate": trial.suggest_float("learning_rate", 0.01, 0.3, log=True),
        "max_depth": trial.suggest_int("max_depth", 2, 10),
        "min_child_weight": trial.suggest_int("min_child_weight", 1, 10),
        "subsample": trial.suggest_float("subsample", 0.6, 1.0),
        "colsample_bytree": trial.suggest_float("colsample_bytree", 0.6, 1.0),
        "reg_alpha": trial.suggest_float("reg_alpha", 1e-8, 10.0, log=True),
        "reg_lambda": trial.suggest_float("reg_lambda", 1e-8, 10.0, log=True),
        "verbosity": 0,
    }

    if problem_type == ProblemType.CLASSIFICATION:
        params["eval_metric"] = "logloss"
    else:
        params["eval_metric"] = "rmse"

    return params


def _suggest_lightgbm(trial: optuna.Trial, problem_type: ProblemType) -> dict[str, Any]:
    """Suggest hyperparameters for LightGBM optimization.

    Args:
        trial: Optuna trial for hyperparameter suggestion.
        problem_type: Classification or regression (unused, same params for both).

    Returns:
        Dictionary of suggested hyperparameters.
    """
    del problem_type  # Unused, same params for both problem types
    params: dict[str, Any] = {
        "n_estimators": trial.suggest_int("n_estimators", 50, 300),
        "learning_rate": trial.suggest_float("learning_rate", 0.01, 0.3, log=True),
        "max_depth": trial.suggest_int("max_depth", 2, 15),
        "num_leaves": trial.suggest_int("num_leaves", 20, 150),
        "min_child_samples": trial.suggest_int("min_child_samples", 5, 100),
        "subsample": trial.suggest_float("subsample", 0.6, 1.0),
        "colsample_bytree": trial.suggest_float("colsample_bytree", 0.6, 1.0),
        "reg_alpha": trial.suggest_float("reg_alpha", 1e-8, 10.0, log=True),
        "reg_lambda": trial.suggest_float("reg_lambda", 1e-8, 10.0, log=True),
        "verbose": -1,
    }
    return params


# Build model registry dynamically based on available libraries
BOOSTING_MODELS: dict[str, dict[str, Any]] = {}

if _xgboost_available and xgb is not None:
    BOOSTING_MODELS["xgboost"] = {
        "problem_types": ["classification", "regression"],
        "classification": xgb.XGBClassifier,
        "regression": xgb.XGBRegressor,
        "suggest_params": _suggest_xgboost,
        "default_params": {
            "n_estimators": 100,
            "learning_rate": 0.1,
            "max_depth": 6,
            "verbosity": 0,
        },
    }

if _lightgbm_available and lgb is not None:
    BOOSTING_MODELS["lightgbm"] = {
        "problem_types": ["classification", "regression"],
        "classification": lgb.LGBMClassifier,
        "regression": lgb.LGBMRegressor,
        "suggest_params": _suggest_lightgbm,
        "default_params": {
            "n_estimators": 100,
            "learning_rate": 0.1,
            "max_depth": -1,
            "verbose": -1,
        },
    }
