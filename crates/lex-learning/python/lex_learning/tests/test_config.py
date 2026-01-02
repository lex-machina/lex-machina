"""Tests for PipelineConfig and configuration validation."""

from __future__ import annotations

import pytest

from src import PipelineConfig, PipelineConfigBuilder, ProblemType


class TestProblemType:
    """Tests for ProblemType enum."""

    def test_classification_value(self):
        """ProblemType.CLASSIFICATION has correct value."""
        assert ProblemType.CLASSIFICATION.value == "classification"

    def test_regression_value(self):
        """ProblemType.REGRESSION has correct value."""
        assert ProblemType.REGRESSION.value == "regression"

    def test_from_string(self):
        """Can create ProblemType from string value."""
        assert ProblemType("classification") == ProblemType.CLASSIFICATION
        assert ProblemType("regression") == ProblemType.REGRESSION

    def test_invalid_string(self):
        """Invalid string raises ValueError."""
        with pytest.raises(ValueError):
            ProblemType("invalid")


class TestPipelineConfig:
    """Tests for PipelineConfig dataclass."""

    def test_minimal_config(self):
        """Can create config with only required field."""
        config = PipelineConfig(problem_type=ProblemType.CLASSIFICATION)
        assert config.problem_type == ProblemType.CLASSIFICATION
        assert config.target_column is None
        assert config.algorithm is None

    def test_all_defaults(self):
        """Default values are set correctly."""
        config = PipelineConfig(problem_type=ProblemType.REGRESSION)

        assert config.top_k_algorithms == 3
        assert config.optimize_hyperparams is True
        assert config.n_trials == 30
        assert config.cv_folds == 5
        assert config.test_size == 0.2
        assert config.enable_neural_networks is True
        assert config.enable_explainability is True
        assert config.shap_max_samples == 100
        assert config.random_seed == 42
        assert config.n_jobs == -1

    def test_custom_values(self):
        """Can override all default values."""
        config = PipelineConfig(
            problem_type=ProblemType.CLASSIFICATION,
            target_column="label",
            algorithm="xgboost",
            top_k_algorithms=5,
            optimize_hyperparams=False,
            n_trials=50,
            cv_folds=10,
            test_size=0.3,
            enable_neural_networks=False,
            enable_explainability=False,
            shap_max_samples=50,
            random_seed=123,
            n_jobs=4,
        )

        assert config.target_column == "label"
        assert config.algorithm == "xgboost"
        assert config.top_k_algorithms == 5
        assert config.optimize_hyperparams is False
        assert config.n_trials == 50
        assert config.cv_folds == 10
        assert config.test_size == 0.3
        assert config.enable_neural_networks is False
        assert config.enable_explainability is False
        assert config.shap_max_samples == 50
        assert config.random_seed == 123
        assert config.n_jobs == 4

    # Validation tests
    def test_top_k_must_be_positive(self):
        """top_k_algorithms must be at least 1."""
        with pytest.raises(ValueError, match="top_k_algorithms"):
            PipelineConfig(problem_type=ProblemType.CLASSIFICATION, top_k_algorithms=0)

    def test_n_trials_must_be_positive(self):
        """n_trials must be at least 1."""
        with pytest.raises(ValueError, match="n_trials"):
            PipelineConfig(problem_type=ProblemType.CLASSIFICATION, n_trials=0)

    def test_cv_folds_must_be_at_least_2(self):
        """cv_folds must be at least 2."""
        with pytest.raises(ValueError, match="cv_folds"):
            PipelineConfig(problem_type=ProblemType.CLASSIFICATION, cv_folds=1)

    def test_test_size_bounds(self):
        """test_size must be between 0 and 1."""
        with pytest.raises(ValueError, match="test_size"):
            PipelineConfig(problem_type=ProblemType.CLASSIFICATION, test_size=0.0)

        with pytest.raises(ValueError, match="test_size"):
            PipelineConfig(problem_type=ProblemType.CLASSIFICATION, test_size=1.0)

        with pytest.raises(ValueError, match="test_size"):
            PipelineConfig(problem_type=ProblemType.CLASSIFICATION, test_size=-0.1)

        with pytest.raises(ValueError, match="test_size"):
            PipelineConfig(problem_type=ProblemType.CLASSIFICATION, test_size=1.5)

    def test_shap_max_samples_must_be_positive(self):
        """shap_max_samples must be at least 1."""
        with pytest.raises(ValueError, match="shap_max_samples"):
            PipelineConfig(problem_type=ProblemType.CLASSIFICATION, shap_max_samples=0)

    def test_builder_method(self):
        """Config has a builder() class method."""
        builder = PipelineConfig.builder()
        assert isinstance(builder, PipelineConfigBuilder)


class TestPipelineConfigBuilder:
    """Tests for PipelineConfigBuilder fluent interface."""

    def test_builder_returns_self(self):
        """All builder methods return self for chaining."""
        builder = PipelineConfigBuilder()

        assert builder.problem_type(ProblemType.CLASSIFICATION) is builder
        assert builder.target_column("target") is builder
        assert builder.algorithm("xgboost") is builder
        assert builder.top_k_algorithms(5) is builder
        assert builder.optimize_hyperparams(True) is builder
        assert builder.n_trials(30) is builder
        assert builder.cv_folds(5) is builder
        assert builder.test_size(0.2) is builder
        assert builder.enable_neural_networks(True) is builder
        assert builder.enable_explainability(True) is builder
        assert builder.shap_max_samples(100) is builder
        assert builder.random_seed(42) is builder
        assert builder.n_jobs(-1) is builder

    def test_build_requires_problem_type(self):
        """build() raises ValueError if problem_type not set."""
        builder = PipelineConfigBuilder()
        with pytest.raises(ValueError, match="problem_type is required"):
            builder.build()

    def test_fluent_chain(self):
        """Can chain all builder methods."""
        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .target_column("label")
            .algorithm("random_forest")
            .top_k_algorithms(3)
            .optimize_hyperparams(True)
            .n_trials(20)
            .cv_folds(3)
            .test_size(0.25)
            .enable_neural_networks(False)
            .enable_explainability(True)
            .shap_max_samples(50)
            .random_seed(99)
            .n_jobs(2)
            .build()
        )

        assert config.problem_type == ProblemType.CLASSIFICATION
        assert config.target_column == "label"
        assert config.algorithm == "random_forest"
        assert config.top_k_algorithms == 3
        assert config.optimize_hyperparams is True
        assert config.n_trials == 20
        assert config.cv_folds == 3
        assert config.test_size == 0.25
        assert config.enable_neural_networks is False
        assert config.enable_explainability is True
        assert config.shap_max_samples == 50
        assert config.random_seed == 99
        assert config.n_jobs == 2

    def test_minimal_classification_config(self):
        """Can build minimal classification config."""
        config = PipelineConfig.builder().problem_type(ProblemType.CLASSIFICATION).build()

        assert config.problem_type == ProblemType.CLASSIFICATION
        assert config.target_column is None  # Will use last column

    def test_minimal_regression_config(self):
        """Can build minimal regression config."""
        config = PipelineConfig.builder().problem_type(ProblemType.REGRESSION).build()

        assert config.problem_type == ProblemType.REGRESSION

    def test_specific_algorithm_config(self):
        """Can specify a specific algorithm instead of auto-select."""
        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .algorithm("xgboost")
            .build()
        )

        assert config.algorithm == "xgboost"

    def test_fast_training_config(self):
        """Can create a fast training config (no neural, no explain, few trials)."""
        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .target_column("target")
            .enable_neural_networks(False)
            .enable_explainability(False)
            .n_trials(5)
            .top_k_algorithms(1)
            .build()
        )

        assert config.enable_neural_networks is False
        assert config.enable_explainability is False
        assert config.n_trials == 5
        assert config.top_k_algorithms == 1
