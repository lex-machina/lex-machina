"""SHAP explainer strategies using the Strategy pattern.

Each strategy wraps a specific SHAP explainer type and provides a unified
interface for computing SHAP values.
"""

from __future__ import annotations

import logging
import warnings
from abc import ABC, abstractmethod
from typing import TYPE_CHECKING, Any

import numpy as np
from numpy.typing import NDArray

from ..config import ProblemType

if TYPE_CHECKING:
    pass

logger = logging.getLogger(__name__)

# Lazy SHAP import
_shap: Any = None
_shap_available = True

try:
    import shap as _shap
except ImportError:
    _shap_available = False


def is_shap_available() -> bool:
    """Check if SHAP is available."""
    return _shap_available and _shap is not None


def get_shap_module() -> Any:
    """Get the SHAP module (or None if not available)."""
    return _shap


class ShapExplainerStrategy(ABC):
    """Abstract base class for SHAP explainer strategies."""

    @property
    @abstractmethod
    def name(self) -> str:
        """Name of this strategy."""
        ...

    @abstractmethod
    def can_explain(self, model: Any) -> bool:
        """Check if this strategy can explain the given model.

        Args:
            model: The trained model to explain.

        Returns:
            True if this strategy can handle the model.
        """
        ...

    @abstractmethod
    def compute_shap_values(
        self,
        model: Any,
        X: NDArray[Any],
        problem_type: ProblemType,
    ) -> tuple[NDArray[Any], float] | None:
        """Compute SHAP values for the model.

        Args:
            model: The trained model.
            X: Data to explain.
            problem_type: Classification or regression.

        Returns:
            Tuple of (shap_values, base_value) or None if computation fails.
        """
        ...


class TreeExplainerStrategy(ShapExplainerStrategy):
    """Strategy for tree-based models (fastest)."""

    @property
    def name(self) -> str:
        return "TreeExplainer"

    def can_explain(self, model: Any) -> bool:
        """Check if model is tree-based."""
        if not is_shap_available():
            return False

        tree_model_types = (
            "RandomForestClassifier",
            "RandomForestRegressor",
            "GradientBoostingClassifier",
            "GradientBoostingRegressor",
            "ExtraTreesClassifier",
            "ExtraTreesRegressor",
            "DecisionTreeClassifier",
            "DecisionTreeRegressor",
            "XGBClassifier",
            "XGBRegressor",
            "LGBMClassifier",
            "LGBMRegressor",
        )
        model_type = type(model).__name__
        return model_type in tree_model_types

    def compute_shap_values(
        self,
        model: Any,
        X: NDArray[Any],
        problem_type: ProblemType,
    ) -> tuple[NDArray[Any], float] | None:
        if _shap is None:
            return None

        try:
            with warnings.catch_warnings():
                warnings.simplefilter("ignore")

                explainer = _shap.TreeExplainer(model)
                shap_values = explainer.shap_values(X)

                # Get base value
                base_value = 0.0
                if hasattr(explainer, "expected_value"):
                    ev = explainer.expected_value
                    if isinstance(ev, (list, np.ndarray)):
                        base_value = float(ev[1]) if len(ev) > 1 else float(ev[0])
                    else:
                        base_value = float(ev)

                # Handle list output (classification)
                if isinstance(shap_values, list):
                    shap_values = shap_values[1] if len(shap_values) == 2 else shap_values[0]

                return np.asarray(shap_values), base_value

        except Exception as e:
            logger.debug(f"TreeExplainer failed: {e}")
            return None


class LinearExplainerStrategy(ShapExplainerStrategy):
    """Strategy for linear models."""

    @property
    def name(self) -> str:
        return "LinearExplainer"

    def can_explain(self, model: Any) -> bool:
        """Check if model is linear."""
        if not is_shap_available():
            return False

        linear_model_types = (
            "LogisticRegression",
            "LinearRegression",
            "Ridge",
            "Lasso",
            "ElasticNet",
            "SGDClassifier",
            "SGDRegressor",
        )
        model_type = type(model).__name__
        return model_type in linear_model_types

    def compute_shap_values(
        self,
        model: Any,
        X: NDArray[Any],
        problem_type: ProblemType,
    ) -> tuple[NDArray[Any], float] | None:
        if _shap is None:
            return None

        try:
            with warnings.catch_warnings():
                warnings.simplefilter("ignore")

                explainer = _shap.LinearExplainer(model, X)
                shap_values = explainer.shap_values(X)

                base_value = 0.0
                if hasattr(explainer, "expected_value"):
                    ev = explainer.expected_value
                    if isinstance(ev, np.ndarray):
                        base_value = float(ev[0]) if len(ev) > 0 else 0.0
                    else:
                        base_value = float(ev)

                return np.asarray(shap_values), base_value

        except Exception as e:
            logger.debug(f"LinearExplainer failed: {e}")
            return None


class UnifiedExplainerStrategy(ShapExplainerStrategy):
    """Strategy using SHAP's unified Explainer API."""

    @property
    def name(self) -> str:
        return "Explainer"

    def can_explain(self, model: Any) -> bool:
        """Unified explainer can try any model."""
        return is_shap_available()

    def compute_shap_values(
        self,
        model: Any,
        X: NDArray[Any],
        problem_type: ProblemType,
    ) -> tuple[NDArray[Any], float] | None:
        if _shap is None:
            return None

        try:
            with warnings.catch_warnings():
                warnings.simplefilter("ignore")

                # Use small background sample
                background = X[: min(50, len(X))]
                explainer = _shap.Explainer(model, background)
                explanation = explainer(X)
                shap_values = explanation.values

                base_value = 0.0
                if hasattr(explanation, "base_values"):
                    bv = explanation.base_values
                    base_value = float(bv.mean()) if isinstance(bv, np.ndarray) else float(bv)

                return np.asarray(shap_values), base_value

        except Exception as e:
            logger.debug(f"Explainer failed: {e}")
            return None


class KernelExplainerStrategy(ShapExplainerStrategy):
    """Strategy using KernelExplainer (universal but slow)."""

    @property
    def name(self) -> str:
        return "KernelExplainer"

    def can_explain(self, model: Any) -> bool:
        """Kernel explainer can explain any model with predict/predict_proba."""
        if not is_shap_available():
            return False
        return hasattr(model, "predict")

    def compute_shap_values(
        self,
        model: Any,
        X: NDArray[Any],
        problem_type: ProblemType,
    ) -> tuple[NDArray[Any], float] | None:
        if _shap is None:
            return None

        try:
            with warnings.catch_warnings():
                warnings.simplefilter("ignore")

                # Sample background data
                background = _shap.sample(X, min(50, len(X)))

                # Choose prediction function based on problem type
                if problem_type == ProblemType.CLASSIFICATION and hasattr(model, "predict_proba"):

                    def predict_fn(x: NDArray[Any]) -> NDArray[Any]:
                        proba = model.predict_proba(x)
                        return proba[:, 1] if proba.shape[1] == 2 else proba[:, 0]

                    explainer = _shap.KernelExplainer(predict_fn, background)
                else:
                    explainer = _shap.KernelExplainer(model.predict, background)

                shap_values = explainer.shap_values(X, nsamples=100)

                base_value = 0.0
                if hasattr(explainer, "expected_value"):
                    ev = explainer.expected_value
                    base_value = float(ev) if not isinstance(ev, np.ndarray) else float(ev.mean())

                return np.asarray(shap_values), base_value

        except Exception as e:
            logger.debug(f"KernelExplainer failed: {e}")
            return None


def get_explainer_chain() -> list[ShapExplainerStrategy]:
    """Get the chain of explainer strategies to try, in order of preference.

    Returns:
        List of strategies ordered from fastest/most specialized to slowest/most general.
    """
    return [
        TreeExplainerStrategy(),
        LinearExplainerStrategy(),
        UnifiedExplainerStrategy(),
        KernelExplainerStrategy(),
    ]


def compute_shap_values_with_chain(
    model: Any,
    X: NDArray[Any],
    problem_type: ProblemType,
) -> tuple[NDArray[Any], float, str] | None:
    """Try each explainer strategy in order until one succeeds.

    Args:
        model: The trained model.
        X: Data to explain.
        problem_type: Classification or regression.

    Returns:
        Tuple of (shap_values, base_value, strategy_name) or None if all fail.
    """
    for strategy in get_explainer_chain():
        if strategy.can_explain(model):
            logger.debug(f"Trying {strategy.name} for {type(model).__name__}")
            result = strategy.compute_shap_values(model, X, problem_type)
            if result is not None:
                shap_values, base_value = result
                logger.info(f"Successfully computed SHAP values with {strategy.name}")
                return shap_values, base_value, strategy.name

    logger.warning("All SHAP explainer strategies failed")
    return None
