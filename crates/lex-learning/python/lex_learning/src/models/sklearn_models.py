"""Scikit-learn model definitions and hyperparameter spaces."""

from __future__ import annotations

from typing import Any

import optuna
from sklearn.ensemble import (
    ExtraTreesClassifier,
    ExtraTreesRegressor,
    GradientBoostingClassifier,
    GradientBoostingRegressor,
    RandomForestClassifier,
    RandomForestRegressor,
)
from sklearn.linear_model import (
    Lasso,
    LinearRegression,
    LogisticRegression,
    Ridge,
)
from sklearn.neighbors import KNeighborsClassifier, KNeighborsRegressor
from sklearn.svm import SVC, SVR
from sklearn.tree import DecisionTreeClassifier, DecisionTreeRegressor

from ..config import ProblemType


def _suggest_logistic_regression(trial: optuna.Trial, _: ProblemType) -> dict[str, Any]:
    return {
        "C": trial.suggest_float("C", 0.001, 100.0, log=True),
        "solver": trial.suggest_categorical("solver", ["lbfgs", "saga"]),
        "max_iter": 1000,
    }


def _suggest_decision_tree(trial: optuna.Trial, _: ProblemType) -> dict[str, Any]:
    return {
        "max_depth": trial.suggest_int("max_depth", 2, 32),
        "min_samples_split": trial.suggest_int("min_samples_split", 2, 20),
        "min_samples_leaf": trial.suggest_int("min_samples_leaf", 1, 10),
    }


def _suggest_random_forest(trial: optuna.Trial, _: ProblemType) -> dict[str, Any]:
    return {
        "n_estimators": trial.suggest_int("n_estimators", 50, 300),
        "max_depth": trial.suggest_int("max_depth", 3, 20),
        "min_samples_split": trial.suggest_int("min_samples_split", 2, 20),
        "min_samples_leaf": trial.suggest_int("min_samples_leaf", 1, 10),
    }


def _suggest_gradient_boosting(trial: optuna.Trial, _: ProblemType) -> dict[str, Any]:
    return {
        "n_estimators": trial.suggest_int("n_estimators", 50, 300),
        "learning_rate": trial.suggest_float("learning_rate", 0.01, 0.3, log=True),
        "max_depth": trial.suggest_int("max_depth", 2, 10),
        "min_samples_split": trial.suggest_int("min_samples_split", 2, 20),
        "min_samples_leaf": trial.suggest_int("min_samples_leaf", 1, 10),
        "subsample": trial.suggest_float("subsample", 0.6, 1.0),
    }


def _suggest_extra_trees(trial: optuna.Trial, _: ProblemType) -> dict[str, Any]:
    return {
        "n_estimators": trial.suggest_int("n_estimators", 50, 300),
        "max_depth": trial.suggest_int("max_depth", 3, 20),
        "min_samples_split": trial.suggest_int("min_samples_split", 2, 20),
        "min_samples_leaf": trial.suggest_int("min_samples_leaf", 1, 10),
    }


def _suggest_knn(trial: optuna.Trial, _: ProblemType) -> dict[str, Any]:
    return {
        "n_neighbors": trial.suggest_int("n_neighbors", 3, 30),
        "weights": trial.suggest_categorical("weights", ["uniform", "distance"]),
        "metric": trial.suggest_categorical("metric", ["euclidean", "manhattan"]),
    }


def _suggest_svm(trial: optuna.Trial, problem_type: ProblemType) -> dict[str, Any]:
    params: dict[str, Any] = {
        "C": trial.suggest_float("C", 0.1, 100.0, log=True),
        "kernel": trial.suggest_categorical("kernel", ["rbf", "linear"]),
    }
    if params["kernel"] == "rbf":
        params["gamma"] = trial.suggest_categorical("gamma", ["scale", "auto"])
    if problem_type == ProblemType.CLASSIFICATION:
        params["probability"] = True  # Enable probability predictions
    return params


def _suggest_ridge(trial: optuna.Trial, _: ProblemType) -> dict[str, Any]:
    return {
        "alpha": trial.suggest_float("alpha", 0.001, 100.0, log=True),
    }


def _suggest_lasso(trial: optuna.Trial, _: ProblemType) -> dict[str, Any]:
    return {
        "alpha": trial.suggest_float("alpha", 0.001, 100.0, log=True),
        "max_iter": 2000,
    }


# Model registry
SKLEARN_MODELS: dict[str, dict[str, Any]] = {
    "logistic_regression": {
        "problem_types": ["classification"],
        "classification": LogisticRegression,
        "suggest_params": _suggest_logistic_regression,
        "default_params": {"C": 1.0, "solver": "lbfgs", "max_iter": 1000},
    },
    "linear_regression": {
        "problem_types": ["regression"],
        "regression": LinearRegression,
        "suggest_params": lambda t, p: {},
        "default_params": {},
    },
    "ridge": {
        "problem_types": ["regression"],
        "regression": Ridge,
        "suggest_params": _suggest_ridge,
        "default_params": {"alpha": 1.0},
    },
    "lasso": {
        "problem_types": ["regression"],
        "regression": Lasso,
        "suggest_params": _suggest_lasso,
        "default_params": {"alpha": 1.0, "max_iter": 2000},
    },
    "decision_tree": {
        "problem_types": ["classification", "regression"],
        "classification": DecisionTreeClassifier,
        "regression": DecisionTreeRegressor,
        "suggest_params": _suggest_decision_tree,
        "default_params": {"max_depth": 10, "min_samples_split": 2, "min_samples_leaf": 1},
    },
    "random_forest": {
        "problem_types": ["classification", "regression"],
        "classification": RandomForestClassifier,
        "regression": RandomForestRegressor,
        "suggest_params": _suggest_random_forest,
        "default_params": {"n_estimators": 100, "max_depth": 10},
    },
    "gradient_boosting": {
        "problem_types": ["classification", "regression"],
        "classification": GradientBoostingClassifier,
        "regression": GradientBoostingRegressor,
        "suggest_params": _suggest_gradient_boosting,
        "default_params": {"n_estimators": 100, "learning_rate": 0.1, "max_depth": 3},
    },
    "extra_trees": {
        "problem_types": ["classification", "regression"],
        "classification": ExtraTreesClassifier,
        "regression": ExtraTreesRegressor,
        "suggest_params": _suggest_extra_trees,
        "default_params": {"n_estimators": 100, "max_depth": 10},
    },
    "knn": {
        "problem_types": ["classification", "regression"],
        "classification": KNeighborsClassifier,
        "regression": KNeighborsRegressor,
        "suggest_params": _suggest_knn,
        "default_params": {"n_neighbors": 5, "weights": "uniform"},
    },
    "svm": {
        "problem_types": ["classification"],
        "classification": SVC,
        "suggest_params": _suggest_svm,
        "default_params": {"C": 1.0, "kernel": "rbf", "probability": True},
    },
    "svr": {
        "problem_types": ["regression"],
        "regression": SVR,
        "suggest_params": _suggest_svm,
        "default_params": {"C": 1.0, "kernel": "rbf"},
    },
}
