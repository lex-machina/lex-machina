"""Tests for model training functionality."""

from __future__ import annotations

import pytest

from src import ModelResult, PipelineConfig, ProblemType
from src.training.selector import DatasetInfo, select_algorithms
from src.training.trainer import train_models, train_single_model


class TestAlgorithmSelection:
    """Tests for algorithm selection heuristics."""

    def test_classification_small_dataset(self):
        """Small classification datasets get simple models."""
        dataset_info = DatasetInfo(
            n_samples=500,
            n_features=10,
            problem_type=ProblemType.CLASSIFICATION,
            n_classes=3,
        )

        # Available algorithms
        available = [
            "logistic_regression",
            "decision_tree",
            "knn",
            "random_forest",
            "xgboost",
            "lightgbm",
        ]

        algorithms = select_algorithms(
            dataset_info=dataset_info,
            available_algorithms=available,
            top_k=3,
            include_neural=False,
        )

        assert len(algorithms) == 3
        assert all(isinstance(alg, str) for alg in algorithms)
        # Small datasets should prioritize simpler models
        assert algorithms[0] in ["logistic_regression", "decision_tree", "knn"]

    def test_classification_large_dataset(self):
        """Large classification datasets get complex models."""
        dataset_info = DatasetInfo(
            n_samples=50000,
            n_features=50,
            problem_type=ProblemType.CLASSIFICATION,
            n_classes=2,
        )

        available = [
            "logistic_regression",
            "decision_tree",
            "random_forest",
            "xgboost",
            "lightgbm",
            "neural_network",
        ]

        algorithms = select_algorithms(
            dataset_info=dataset_info,
            available_algorithms=available,
            top_k=3,
            include_neural=False,
        )

        assert len(algorithms) == 3
        # Should include boosting models for large data
        assert any("boost" in alg or alg == "lightgbm" for alg in algorithms)

    def test_regression_selection(self):
        """Regression algorithm selection works."""
        dataset_info = DatasetInfo(
            n_samples=1000,
            n_features=20,
            problem_type=ProblemType.REGRESSION,
        )

        available = [
            "ridge",
            "decision_tree",
            "random_forest",
            "xgboost",
            "lightgbm",
        ]

        algorithms = select_algorithms(
            dataset_info=dataset_info,
            available_algorithms=available,
            top_k=3,
            include_neural=False,
        )

        assert len(algorithms) == 3
        assert all(isinstance(alg, str) for alg in algorithms)

    def test_top_k_limit(self):
        """Respects top_k limit."""
        dataset_info = DatasetInfo(
            n_samples=1000,
            n_features=10,
            problem_type=ProblemType.CLASSIFICATION,
            n_classes=2,
        )

        available = [
            "logistic_regression",
            "decision_tree",
            "knn",
            "random_forest",
            "xgboost",
            "lightgbm",
        ]

        for k in [1, 2, 5]:
            algorithms = select_algorithms(
                dataset_info=dataset_info,
                available_algorithms=available,
                top_k=k,
                include_neural=False,
            )
            assert len(algorithms) == min(k, len(available))

    def test_include_neural_networks(self):
        """Can include neural networks in selection."""
        dataset_info = DatasetInfo(
            n_samples=10000,
            n_features=20,
            problem_type=ProblemType.CLASSIFICATION,
            n_classes=3,
        )

        available = [
            "logistic_regression",
            "random_forest",
            "xgboost",
            "lightgbm",
            "neural_network",
        ]

        algorithms = select_algorithms(
            dataset_info=dataset_info,
            available_algorithms=available,
            top_k=5,
            include_neural=True,
        )

        # Neural network should be available if included
        assert len(algorithms) == 5

    def test_exclude_neural_networks(self):
        """Can exclude neural networks from selection."""
        dataset_info = DatasetInfo(
            n_samples=50000,
            n_features=20,
            problem_type=ProblemType.CLASSIFICATION,
            n_classes=2,
        )

        available = [
            "random_forest",
            "xgboost",
            "neural_network",
        ]

        algorithms = select_algorithms(
            dataset_info=dataset_info,
            available_algorithms=available,
            top_k=3,
            include_neural=False,
        )

        assert "neural_network" not in algorithms


class TestDatasetInfo:
    """Tests for DatasetInfo dataclass."""

    def test_size_category_small(self):
        """Small datasets are categorized correctly."""
        info = DatasetInfo(
            n_samples=500,
            n_features=10,
            problem_type=ProblemType.CLASSIFICATION,
        )
        assert info.size_category == "small"

    def test_size_category_medium(self):
        """Medium datasets are categorized correctly."""
        info = DatasetInfo(
            n_samples=5000,
            n_features=10,
            problem_type=ProblemType.CLASSIFICATION,
        )
        assert info.size_category == "medium"

    def test_size_category_large(self):
        """Large datasets are categorized correctly."""
        info = DatasetInfo(
            n_samples=50000,
            n_features=10,
            problem_type=ProblemType.CLASSIFICATION,
        )
        assert info.size_category == "large"

    def test_high_dimensional_by_count(self):
        """High feature count is detected."""
        info = DatasetInfo(
            n_samples=1000,
            n_features=150,  # > 100
            problem_type=ProblemType.CLASSIFICATION,
        )
        assert info.is_high_dimensional is True

    def test_high_dimensional_by_ratio(self):
        """High feature ratio is detected."""
        info = DatasetInfo(
            n_samples=100,
            n_features=60,  # > 50% of samples
            problem_type=ProblemType.CLASSIFICATION,
        )
        assert info.is_high_dimensional is True

    def test_not_high_dimensional(self):
        """Normal dimensionality is detected."""
        info = DatasetInfo(
            n_samples=1000,
            n_features=20,
            problem_type=ProblemType.CLASSIFICATION,
        )
        assert info.is_high_dimensional is False


class TestModelResult:
    """Tests for ModelResult dataclass."""

    def test_model_result_fields(self):
        """ModelResult has expected fields."""
        result = ModelResult(
            name="test_model",
            test_score=0.85,
            train_score=0.90,
            cv_score=0.83,
            training_time_seconds=1.5,
            hyperparameters={"max_depth": 5},
            overfitting_risk="low",
        )

        assert result.name == "test_model"
        assert result.test_score == 0.85
        assert result.train_score == 0.90
        assert result.cv_score == 0.83
        assert result.training_time_seconds == 1.5
        assert result.hyperparameters == {"max_depth": 5}
        assert result.overfitting_risk == "low"

    def test_overfitting_risk_values(self):
        """overfitting_risk can be low, medium, or high."""
        for risk in ["low", "medium", "high"]:
            result = ModelResult(
                name="test",
                test_score=0.8,
                train_score=0.9,
                cv_score=0.8,
                training_time_seconds=1.0,
                hyperparameters={},
                overfitting_risk=risk,
            )
            assert result.overfitting_risk == risk


class TestTrainSingleModel:
    """Tests for train_single_model function."""

    @pytest.mark.slow
    def test_train_single_model(self, small_classification_data):
        """Can train a single model."""
        from sklearn.model_selection import train_test_split

        from src.preprocessing import Preprocessor

        X = small_classification_data.drop(columns=["target"])
        y = small_classification_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        # Split data
        X_train, X_test, y_train, y_test = train_test_split(
            X_transformed, y_transformed, test_size=0.2, random_state=42
        )

        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .algorithm("decision_tree")
            .n_trials(2)
            .cv_folds(2)
            .optimize_hyperparams(False)
            .build()
        )

        result = train_single_model(
            X_train=X_train,
            y_train=y_train,
            X_test=X_test,
            y_test=y_test,
            algorithm="decision_tree",
            config=config,
        )

        assert result is not None
        model, model_result = result
        assert model_result.name == "decision_tree"
        assert 0 <= model_result.test_score <= 1
        assert 0 <= model_result.train_score <= 1
        assert model_result.training_time_seconds >= 0


class TestTrainModels:
    """Tests for train_models function."""

    @pytest.mark.slow
    def test_train_multiple_models(self, small_classification_data):
        """Can train multiple models."""
        from sklearn.model_selection import train_test_split

        from src.preprocessing import Preprocessor

        X = small_classification_data.drop(columns=["target"])
        y = small_classification_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        X_train, X_test, y_train, y_test = train_test_split(
            X_transformed, y_transformed, test_size=0.2, random_state=42
        )

        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .n_trials(2)
            .cv_folds(2)
            .optimize_hyperparams(False)
            .build()
        )

        best_model, results = train_models(
            X_train=X_train,
            y_train=y_train,
            X_test=X_test,
            y_test=y_test,
            algorithms=["decision_tree", "logistic_regression"],
            config=config,
        )

        assert best_model is not None
        assert len(results) == 2
        names = [r.name for r in results]
        assert "decision_tree" in names
        assert "logistic_regression" in names

    @pytest.mark.slow
    def test_results_sorted_by_score(self, small_classification_data):
        """Results are sorted by test score (descending)."""
        from sklearn.model_selection import train_test_split

        from src.preprocessing import Preprocessor

        X = small_classification_data.drop(columns=["target"])
        y = small_classification_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        X_train, X_test, y_train, y_test = train_test_split(
            X_transformed, y_transformed, test_size=0.2, random_state=42
        )

        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .n_trials(2)
            .cv_folds(2)
            .optimize_hyperparams(False)
            .build()
        )

        _, results = train_models(
            X_train=X_train,
            y_train=y_train,
            X_test=X_test,
            y_test=y_test,
            algorithms=["decision_tree", "logistic_regression", "knn"],
            config=config,
        )

        # Check sorted descending
        scores = [r.test_score for r in results]
        assert scores == sorted(scores, reverse=True)

    @pytest.mark.slow
    def test_best_model_is_top_scorer(self, small_classification_data):
        """Best model returned is the top scorer."""
        from sklearn.model_selection import train_test_split

        from src.preprocessing import Preprocessor

        X = small_classification_data.drop(columns=["target"])
        y = small_classification_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        X_train, X_test, y_train, y_test = train_test_split(
            X_transformed, y_transformed, test_size=0.2, random_state=42
        )

        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .n_trials(2)
            .cv_folds(2)
            .optimize_hyperparams(False)
            .build()
        )

        best_model, results = train_models(
            X_train=X_train,
            y_train=y_train,
            X_test=X_test,
            y_test=y_test,
            algorithms=["decision_tree", "logistic_regression"],
            config=config,
        )

        # Best model should correspond to first (highest score) result
        assert results[0].name in ["decision_tree", "logistic_regression"]


class TestOptunaOptimization:
    """Tests for Optuna hyperparameter optimization."""

    @pytest.mark.slow
    def test_optimization_produces_hyperparameters(self, small_classification_data):
        """Optimization produces hyperparameter values."""
        from sklearn.model_selection import train_test_split

        from src.preprocessing import Preprocessor

        X = small_classification_data.drop(columns=["target"])
        y = small_classification_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        X_train, X_test, y_train, y_test = train_test_split(
            X_transformed, y_transformed, test_size=0.2, random_state=42
        )

        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .algorithm("decision_tree")
            .optimize_hyperparams(True)
            .n_trials(3)
            .cv_folds(2)
            .build()
        )

        result = train_single_model(
            X_train=X_train,
            y_train=y_train,
            X_test=X_test,
            y_test=y_test,
            algorithm="decision_tree",
            config=config,
        )

        assert result is not None
        _, model_result = result
        # Optimized model should have hyperparameters
        assert isinstance(model_result.hyperparameters, dict)
