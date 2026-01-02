"""Explainability module - SHAP explanations for trained models."""

from .explainer import explain_model, save_explainability_plots

__all__ = [
    "explain_model",
    "save_explainability_plots",
]
