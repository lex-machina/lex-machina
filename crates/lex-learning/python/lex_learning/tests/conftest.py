"""Shared test fixtures and utilities for lex-learning tests."""

from __future__ import annotations

import tempfile
from pathlib import Path
from typing import Any

import numpy as np
import pandas as pd
import pytest

from src import PipelineConfig, ProblemType

# =============================================================================
# Test Data Fixtures
# =============================================================================


@pytest.fixture
def classification_data() -> pd.DataFrame:
    """Create a simple classification dataset (Iris-like).

    Returns:
        DataFrame with numeric features and categorical target.
    """
    np.random.seed(42)
    n_samples = 150

    # Generate features
    data = {
        "feature_a": np.random.randn(n_samples),
        "feature_b": np.random.randn(n_samples) * 2,
        "feature_c": np.random.randn(n_samples) + 1,
        "feature_d": np.random.randn(n_samples) - 1,
    }

    # Generate target based on features (simple linear combination)
    score = data["feature_a"] + data["feature_b"] * 0.5 - data["feature_c"]
    data["target"] = pd.cut(score, bins=3, labels=["class_0", "class_1", "class_2"]).astype(str)

    return pd.DataFrame(data)


@pytest.fixture
def binary_classification_data() -> pd.DataFrame:
    """Create a binary classification dataset.

    Returns:
        DataFrame with numeric features and binary target.
    """
    np.random.seed(42)
    n_samples = 200

    data = {
        "age": np.random.randint(18, 80, n_samples),
        "income": np.random.exponential(50000, n_samples),
        "score": np.random.randn(n_samples) * 100 + 500,
    }

    # Binary target based on simple rule
    prob = 1 / (1 + np.exp(-(data["income"] / 50000 - 1)))
    data["target"] = (np.random.random(n_samples) < prob).astype(int)

    return pd.DataFrame(data)


@pytest.fixture
def regression_data() -> pd.DataFrame:
    """Create a simple regression dataset.

    Returns:
        DataFrame with numeric features and continuous target.
    """
    np.random.seed(42)
    n_samples = 200

    # Generate features
    data = {
        "size": np.random.uniform(500, 5000, n_samples),
        "bedrooms": np.random.randint(1, 6, n_samples),
        "age": np.random.uniform(0, 50, n_samples),
        "location_score": np.random.uniform(1, 10, n_samples),
    }

    # Target: house price based on features
    noise = np.random.randn(n_samples) * 10000
    data["price"] = (
        data["size"] * 100
        + data["bedrooms"] * 20000
        - data["age"] * 1000
        + data["location_score"] * 15000
        + noise
    )

    return pd.DataFrame(data)


@pytest.fixture
def mixed_types_data() -> pd.DataFrame:
    """Create a dataset with mixed numeric and categorical features.

    Returns:
        DataFrame with both numeric and categorical features.
    """
    np.random.seed(42)
    n_samples = 150

    data = {
        "numeric_1": np.random.randn(n_samples),
        "numeric_2": np.random.uniform(0, 100, n_samples),
        "category_1": np.random.choice(["A", "B", "C"], n_samples),
        "category_2": np.random.choice(["X", "Y"], n_samples),
        "target": np.random.choice([0, 1], n_samples),
    }

    return pd.DataFrame(data)


@pytest.fixture
def small_classification_data() -> pd.DataFrame:
    """Create a very small classification dataset for fast tests.

    Returns:
        DataFrame with 50 samples for quick testing.
    """
    np.random.seed(42)
    n_samples = 50

    data = {
        "x1": np.random.randn(n_samples),
        "x2": np.random.randn(n_samples),
        "target": np.random.choice(["yes", "no"], n_samples),
    }

    return pd.DataFrame(data)


@pytest.fixture
def small_regression_data() -> pd.DataFrame:
    """Create a very small regression dataset for fast tests.

    Returns:
        DataFrame with 50 samples for quick testing.
    """
    np.random.seed(42)
    n_samples = 50

    x1 = np.random.randn(n_samples)
    x2 = np.random.randn(n_samples)

    data = {
        "x1": x1,
        "x2": x2,
        "target": x1 * 2 + x2 * 3 + np.random.randn(n_samples) * 0.5,
    }

    return pd.DataFrame(data)


# =============================================================================
# Configuration Fixtures
# =============================================================================


@pytest.fixture
def classification_config() -> PipelineConfig:
    """Create a minimal classification config for fast testing."""
    return (
        PipelineConfig.builder()
        .problem_type(ProblemType.CLASSIFICATION)
        .target_column("target")
        .top_k_algorithms(1)
        .n_trials(2)
        .cv_folds(2)
        .enable_neural_networks(False)
        .enable_explainability(False)
        .build()
    )


@pytest.fixture
def regression_config() -> PipelineConfig:
    """Create a minimal regression config for fast testing."""
    return (
        PipelineConfig.builder()
        .problem_type(ProblemType.REGRESSION)
        .target_column("target")
        .top_k_algorithms(1)
        .n_trials(2)
        .cv_folds(2)
        .enable_neural_networks(False)
        .enable_explainability(False)
        .build()
    )


@pytest.fixture
def explainability_config() -> PipelineConfig:
    """Create a config with explainability enabled."""
    return (
        PipelineConfig.builder()
        .problem_type(ProblemType.CLASSIFICATION)
        .target_column("target")
        .algorithm("decision_tree")  # Fast, SHAP-compatible
        .n_trials(2)
        .cv_folds(2)
        .enable_neural_networks(False)
        .enable_explainability(True)
        .shap_max_samples(20)
        .build()
    )


# =============================================================================
# Utility Fixtures
# =============================================================================


@pytest.fixture
def temp_dir() -> Path:
    """Create a temporary directory for test outputs.

    Yields:
        Path to temporary directory (cleaned up after test).
    """
    with tempfile.TemporaryDirectory() as tmpdir:
        yield Path(tmpdir)


@pytest.fixture
def progress_tracker() -> dict[str, Any]:
    """Create a progress tracker for testing callbacks.

    Returns:
        Dictionary to store progress updates.
    """
    tracker: dict[str, Any] = {
        "updates": [],
        "stages": [],
        "final_progress": 0.0,
    }
    return tracker


def make_progress_callback(tracker: dict[str, Any]):
    """Create a progress callback that stores updates in the tracker."""
    from src import ProgressUpdate

    def callback(update: ProgressUpdate) -> None:
        tracker["updates"].append(update)
        tracker["stages"].append(update.stage)
        tracker["final_progress"] = update.progress

    return callback


# =============================================================================
# Markers
# =============================================================================


def pytest_configure(config):
    """Register custom markers."""
    config.addinivalue_line(
        "markers", "slow: marks tests as slow (deselect with '-m \"not slow\"')"
    )
    config.addinivalue_line(
        "markers", "integration: marks tests as integration tests (full pipeline)"
    )
