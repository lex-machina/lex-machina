"""Configuration dataclasses for lex-learning."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from typing import Self


class ProblemType(Enum):
    """Type of ML problem to solve."""

    CLASSIFICATION = "classification"
    REGRESSION = "regression"


@dataclass
class PipelineConfig:
    """Configuration for the training pipeline.

    Attributes:
        problem_type: Type of problem (classification or regression).
        target_column: Name of the target column. If None, uses last column.
        algorithm: Specific algorithm to use. If None, auto-selects top-k.
        top_k_algorithms: Number of algorithms to try when auto-selecting.
        optimize_hyperparams: Whether to use Optuna for hyperparameter optimization.
        n_trials: Number of Optuna trials per model.
        cv_folds: Number of cross-validation folds.
        test_size: Fraction of data to use for testing.
        enable_neural_networks: Whether to include neural network models.
        enable_explainability: Whether to generate SHAP explanations.
        shap_max_samples: Maximum samples for SHAP computation.
        random_seed: Random seed for reproducibility.
        n_jobs: Number of parallel jobs (-1 for all cores).
    """

    problem_type: ProblemType
    target_column: str | None = None
    algorithm: str | None = None
    top_k_algorithms: int = 3
    optimize_hyperparams: bool = True
    n_trials: int = 30
    cv_folds: int = 5
    test_size: float = 0.2
    enable_neural_networks: bool = True
    enable_explainability: bool = True
    shap_max_samples: int = 100
    random_seed: int = 42
    n_jobs: int = -1

    def __post_init__(self) -> None:
        """Validate configuration after initialization."""
        if self.top_k_algorithms < 1:
            raise ValueError("top_k_algorithms must be at least 1")
        if self.n_trials < 1:
            raise ValueError("n_trials must be at least 1")
        if self.cv_folds < 2:
            raise ValueError("cv_folds must be at least 2")
        if not 0 < self.test_size < 1:
            raise ValueError("test_size must be between 0 and 1")
        if self.shap_max_samples < 1:
            raise ValueError("shap_max_samples must be at least 1")

    @classmethod
    def builder(cls) -> PipelineConfigBuilder:
        """Create a builder for PipelineConfig."""
        return PipelineConfigBuilder()


class PipelineConfigBuilder:
    """Builder for PipelineConfig with fluent interface."""

    def __init__(self) -> None:
        self._problem_type: ProblemType | None = None
        self._target_column: str | None = None
        self._algorithm: str | None = None
        self._top_k_algorithms: int = 3
        self._optimize_hyperparams: bool = True
        self._n_trials: int = 30
        self._cv_folds: int = 5
        self._test_size: float = 0.2
        self._enable_neural_networks: bool = True
        self._enable_explainability: bool = True
        self._shap_max_samples: int = 100
        self._random_seed: int = 42
        self._n_jobs: int = -1

    def problem_type(self, value: ProblemType) -> Self:
        """Set the problem type."""
        self._problem_type = value
        return self

    def target_column(self, value: str) -> Self:
        """Set the target column name."""
        self._target_column = value
        return self

    def algorithm(self, value: str) -> Self:
        """Set a specific algorithm to use."""
        self._algorithm = value
        return self

    def top_k_algorithms(self, value: int) -> Self:
        """Set the number of top algorithms to try."""
        self._top_k_algorithms = value
        return self

    def optimize_hyperparams(self, value: bool) -> Self:
        """Enable or disable hyperparameter optimization."""
        self._optimize_hyperparams = value
        return self

    def n_trials(self, value: int) -> Self:
        """Set the number of Optuna trials."""
        self._n_trials = value
        return self

    def cv_folds(self, value: int) -> Self:
        """Set the number of cross-validation folds."""
        self._cv_folds = value
        return self

    def test_size(self, value: float) -> Self:
        """Set the test set size fraction."""
        self._test_size = value
        return self

    def enable_neural_networks(self, value: bool) -> Self:
        """Enable or disable neural network models."""
        self._enable_neural_networks = value
        return self

    def enable_explainability(self, value: bool) -> Self:
        """Enable or disable SHAP explanations."""
        self._enable_explainability = value
        return self

    def shap_max_samples(self, value: int) -> Self:
        """Set max samples for SHAP computation."""
        self._shap_max_samples = value
        return self

    def random_seed(self, value: int) -> Self:
        """Set the random seed."""
        self._random_seed = value
        return self

    def n_jobs(self, value: int) -> Self:
        """Set the number of parallel jobs."""
        self._n_jobs = value
        return self

    def build(self) -> PipelineConfig:
        """Build the PipelineConfig."""
        if self._problem_type is None:
            raise ValueError("problem_type is required")

        return PipelineConfig(
            problem_type=self._problem_type,
            target_column=self._target_column,
            algorithm=self._algorithm,
            top_k_algorithms=self._top_k_algorithms,
            optimize_hyperparams=self._optimize_hyperparams,
            n_trials=self._n_trials,
            cv_folds=self._cv_folds,
            test_size=self._test_size,
            enable_neural_networks=self._enable_neural_networks,
            enable_explainability=self._enable_explainability,
            shap_max_samples=self._shap_max_samples,
            random_seed=self._random_seed,
            n_jobs=self._n_jobs,
        )
