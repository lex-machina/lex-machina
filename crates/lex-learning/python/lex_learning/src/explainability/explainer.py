"""SHAP explainability for trained models."""

from __future__ import annotations

import logging
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from ..config import ProblemType
from ..core import ExplainabilityResult
from .plots import (
    calculate_feature_importance,
    generate_bar_plot,
    generate_summary_plot,
    generate_waterfall_plot,
    save_plots_to_disk,
)
from .shap_strategies import compute_shap_values_with_chain, is_shap_available

logger = logging.getLogger(__name__)


def explain_model(
    model: Any,
    X_sample: NDArray[Any],
    feature_names: list[str],
    problem_type: ProblemType,
    max_samples: int = 100,
) -> ExplainabilityResult:
    """Generate SHAP explanations for a trained model.

    Args:
        model: Trained sklearn-compatible model.
        X_sample: Sample of data to explain.
        feature_names: Names of features.
        problem_type: Classification or regression.
        max_samples: Maximum samples to use for explanation.

    Returns:
        ExplainabilityResult with plots and feature importance.
    """
    if not is_shap_available():
        logger.warning("SHAP not available, skipping explainability")
        return ExplainabilityResult(method="none")

    try:
        # Limit samples
        if len(X_sample) > max_samples:
            indices = np.random.choice(len(X_sample), max_samples, replace=False)
            X_sample = X_sample[indices]

        # Compute SHAP values using strategy chain
        result = compute_shap_values_with_chain(model, X_sample, problem_type)

        if result is None:
            logger.warning("Failed to compute SHAP values with all strategies")
            return ExplainabilityResult(method="shap_failed")

        shap_values, base_value, strategy_name = result

        # Ensure shap_values is 2D (samples x features)
        shap_values = _ensure_2d_shap_values(shap_values)

        if shap_values is None or len(shap_values.shape) != 2:
            logger.warning(
                f"Invalid SHAP values shape: {shap_values.shape if shap_values is not None else None}"
            )
            return ExplainabilityResult(method="shap_failed")

        logger.info(f"SHAP values shape: {shap_values.shape}, X_sample shape: {X_sample.shape}")

        # Generate plots
        summary_plot = generate_summary_plot(shap_values, X_sample, feature_names)
        bar_plot = generate_bar_plot(shap_values, feature_names)
        waterfall_plot = generate_waterfall_plot(shap_values, X_sample, feature_names, base_value)

        # Calculate feature importance
        feature_importance = calculate_feature_importance(shap_values, feature_names)

        return ExplainabilityResult(
            summary_plot=summary_plot,
            beeswarm_plot=bar_plot,  # Using bar plot for compatibility
            feature_importance_plot=waterfall_plot,
            feature_importance=feature_importance,
            method="shap",
        )

    except Exception as e:
        logger.warning(f"Explainability failed: {e}", exc_info=True)
        return ExplainabilityResult(method="failed")


def save_explainability_plots(
    result: ExplainabilityResult,
    output_dir: str | Path,
    prefix: str = "shap",
) -> list[Path]:
    """Save explainability plots to disk.

    Args:
        result: ExplainabilityResult containing plot bytes.
        output_dir: Directory to save plots.
        prefix: Prefix for plot filenames.

    Returns:
        List of paths to saved plots.
    """
    return save_plots_to_disk(result, output_dir, prefix)


def _ensure_2d_shap_values(shap_values: Any) -> NDArray[Any] | None:
    """Ensure SHAP values are 2D (samples x features).

    Handles various SHAP output formats:
    - 3D arrays from multiclass classification
    - List outputs from older SHAP versions

    Args:
        shap_values: Raw SHAP values in any format.

    Returns:
        2D numpy array (samples x features) or None.
    """
    if shap_values is None:
        return None

    shap_values = np.asarray(shap_values)

    # If 3D (samples x features x classes), take one class
    if len(shap_values.shape) == 3:
        # For binary classification, use positive class (index 1)
        # For multiclass, use class 0 or average
        if shap_values.shape[2] == 2:
            shap_values = shap_values[:, :, 1]
        else:
            # Average across classes for multiclass
            shap_values = shap_values.mean(axis=2)

    # If it's a list (old SHAP format), convert
    if isinstance(shap_values, list):
        if len(shap_values) == 2:
            shap_values = np.asarray(shap_values[1])
        else:
            shap_values = np.asarray(shap_values[0])

    return shap_values
