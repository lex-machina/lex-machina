"""Heuristic algorithm selection based on dataset characteristics."""

from __future__ import annotations

from dataclasses import dataclass

from ..config import ProblemType


@dataclass
class DatasetInfo:
    """Information about a dataset for algorithm selection."""

    n_samples: int
    n_features: int
    problem_type: ProblemType
    n_classes: int | None = None  # For classification only

    @property
    def size_category(self) -> str:
        """Categorize dataset size."""
        if self.n_samples < 1000:
            return "small"
        elif self.n_samples < 10000:
            return "medium"
        else:
            return "large"

    @property
    def is_high_dimensional(self) -> bool:
        """Check if dataset has many features relative to samples."""
        return self.n_features > 100 or self.n_features > self.n_samples * 0.5


# Priority lists for different dataset sizes and problem types
CLASSIFICATION_PRIORITIES = {
    "small": [
        "logistic_regression",
        "decision_tree",
        "knn",
        "random_forest",
        "gradient_boosting",
    ],
    "medium": [
        "random_forest",
        "xgboost",
        "gradient_boosting",
        "logistic_regression",
        "lightgbm",
        "extra_trees",
    ],
    "large": [
        "lightgbm",
        "xgboost",
        "random_forest",
        "neural_network",
        "gradient_boosting",
        "extra_trees",
    ],
}

REGRESSION_PRIORITIES = {
    "small": [
        "ridge",
        "decision_tree",
        "knn",
        "random_forest",
        "gradient_boosting",
    ],
    "medium": [
        "random_forest",
        "xgboost",
        "gradient_boosting",
        "ridge",
        "lightgbm",
        "extra_trees",
    ],
    "large": [
        "lightgbm",
        "xgboost",
        "random_forest",
        "neural_network",
        "gradient_boosting",
        "extra_trees",
    ],
}


def select_algorithms(
    dataset_info: DatasetInfo,
    available_algorithms: list[str],
    top_k: int = 3,
    include_neural: bool = True,
) -> list[str]:
    """Select top algorithms based on dataset characteristics.

    Args:
        dataset_info: Information about the dataset.
        available_algorithms: List of available algorithm names.
        top_k: Number of algorithms to select.
        include_neural: Whether to include neural networks.

    Returns:
        List of selected algorithm names (up to top_k).
    """
    # Get priority list based on problem type and dataset size
    if dataset_info.problem_type == ProblemType.CLASSIFICATION:
        priority_list = CLASSIFICATION_PRIORITIES[dataset_info.size_category]
    else:
        priority_list = REGRESSION_PRIORITIES[dataset_info.size_category]

    # Filter by available algorithms
    available_set = set(available_algorithms)

    # Filter neural networks if not included
    if not include_neural:
        available_set.discard("neural_network")

    # High dimensional data: prefer tree-based models
    if dataset_info.is_high_dimensional:
        # Boost tree-based models in priority
        tree_models = ["random_forest", "extra_trees", "xgboost", "lightgbm", "gradient_boosting"]
        priority_list = [m for m in tree_models if m in priority_list] + [
            m for m in priority_list if m not in tree_models
        ]

    # Select top-k from priority list that are available
    selected = []
    for algo in priority_list:
        if algo in available_set and algo not in selected:
            selected.append(algo)
            if len(selected) >= top_k:
                break

    # If we don't have enough, add remaining available algorithms
    if len(selected) < top_k:
        for algo in available_algorithms:
            if algo not in selected and (include_neural or algo != "neural_network"):
                selected.append(algo)
                if len(selected) >= top_k:
                    break

    return selected
