"""Plot generation utilities for SHAP explainability."""

from __future__ import annotations

import io
import logging
from pathlib import Path
from typing import Any

import matplotlib

matplotlib.use("Agg")  # Non-interactive backend

import matplotlib.pyplot as plt
import numpy as np
from numpy.typing import NDArray

from ..core import ExplainabilityResult
from .shap_strategies import get_shap_module

logger = logging.getLogger(__name__)


def render_plot_to_bytes(fig: Any = None) -> bytes | None:
    """Render the current matplotlib figure to PNG bytes.

    Args:
        fig: Optional figure to render. If None, uses current figure.

    Returns:
        PNG bytes or None if rendering fails.
    """
    try:
        if fig is not None:
            plt.figure(fig.number)
        plt.tight_layout()

        buf = io.BytesIO()
        plt.savefig(buf, format="png", dpi=150, bbox_inches="tight")
        buf.seek(0)
        result = buf.getvalue()
        plt.close("all")

        return result

    except Exception as e:
        logger.warning(f"Plot rendering failed: {e}")
        plt.close("all")
        return None


def generate_summary_plot(
    shap_values: NDArray[Any],
    X: NDArray[Any],
    feature_names: list[str],
) -> bytes | None:
    """Generate SHAP summary plot (dot plot) as PNG bytes.

    Args:
        shap_values: SHAP values array (samples x features).
        X: Original data.
        feature_names: Names of features.

    Returns:
        PNG bytes or None if generation fails.
    """
    shap = get_shap_module()
    if shap is None:
        return None

    try:
        plt.figure(figsize=(10, 8))
        shap.summary_plot(
            shap_values,
            X,
            feature_names=feature_names,
            show=False,
            plot_type="dot",
        )
        return render_plot_to_bytes()

    except Exception as e:
        logger.warning(f"Summary plot generation failed: {e}")
        plt.close("all")
        return None


def generate_bar_plot(
    shap_values: NDArray[Any],
    feature_names: list[str],
) -> bytes | None:
    """Generate SHAP bar plot (mean absolute values) as PNG bytes.

    Args:
        shap_values: SHAP values array (samples x features).
        feature_names: Names of features.

    Returns:
        PNG bytes or None if generation fails.
    """
    try:
        # Calculate mean absolute SHAP values
        mean_abs_shap = np.abs(shap_values).mean(axis=0)

        # Sort by importance
        sorted_idx = np.argsort(mean_abs_shap)[::-1]
        sorted_names = [feature_names[i] for i in sorted_idx]
        sorted_values = mean_abs_shap[sorted_idx]

        # Create bar plot
        plt.figure(figsize=(10, 8))
        y_pos = np.arange(len(sorted_names))
        plt.barh(y_pos, sorted_values[::-1], align="center", color="#1E88E5")
        plt.yticks(y_pos, sorted_names[::-1])
        plt.xlabel("Mean |SHAP value|")
        plt.title("Feature Importance (SHAP)")

        return render_plot_to_bytes()

    except Exception as e:
        logger.warning(f"Bar plot generation failed: {e}")
        plt.close("all")
        return None


def generate_waterfall_plot(
    shap_values: NDArray[Any],
    X: NDArray[Any],
    feature_names: list[str],
    base_value: float,
) -> bytes | None:
    """Generate SHAP waterfall plot for first instance as PNG bytes.

    Args:
        shap_values: SHAP values array (samples x features).
        X: Original data.
        feature_names: Names of features.
        base_value: Expected/base value for the model.

    Returns:
        PNG bytes or None if generation fails.
    """
    shap = get_shap_module()
    if shap is None:
        return None

    try:
        # Create explanation for first instance
        explanation = shap.Explanation(
            values=shap_values[0],
            base_values=base_value,
            data=X[0],
            feature_names=feature_names,
        )

        plt.figure(figsize=(10, 8))
        shap.plots.waterfall(explanation, show=False)

        return render_plot_to_bytes()

    except Exception as e:
        logger.warning(f"Waterfall plot generation failed: {e}")
        plt.close("all")
        return None


def save_plots_to_disk(
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
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    saved_paths: list[Path] = []

    plot_mapping = [
        (result.summary_plot, f"{prefix}_summary.png"),
        (result.beeswarm_plot, f"{prefix}_importance_bar.png"),
        (result.feature_importance_plot, f"{prefix}_waterfall.png"),
    ]

    for plot_bytes, filename in plot_mapping:
        if plot_bytes:
            path = output_dir / filename
            path.write_bytes(plot_bytes)
            saved_paths.append(path)
            logger.info(f"Saved plot to {path}")

    return saved_paths


def calculate_feature_importance(
    shap_values: NDArray[Any],
    feature_names: list[str],
) -> list[tuple[str, float]]:
    """Calculate feature importance from SHAP values.

    Args:
        shap_values: SHAP values array (samples x features).
        feature_names: Names of features.

    Returns:
        List of (feature_name, importance) tuples, sorted by importance descending.
    """
    try:
        # Mean absolute SHAP value per feature
        importance = np.abs(shap_values).mean(axis=0)

        # Ensure importance is 1D
        if len(importance.shape) > 1:
            importance = importance.flatten()

        # Normalize to sum to 1
        total = float(importance.sum())
        if total > 0:
            importance = importance / total

        # Create sorted list
        result = []
        for i in range(min(len(feature_names), len(importance))):
            result.append((feature_names[i], float(importance[i])))
        result.sort(key=lambda x: x[1], reverse=True)

        return result

    except Exception as e:
        logger.warning(f"Feature importance calculation failed: {e}")
        return []
