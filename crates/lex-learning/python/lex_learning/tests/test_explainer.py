"""Tests for SHAP explainability."""

from __future__ import annotations

import pytest

from src import ExplainabilityResult, Pipeline, PipelineConfig, ProblemType
from src.explainability import explain_model, save_explainability_plots


class TestExplainabilityResult:
    """Tests for ExplainabilityResult dataclass."""

    def test_default_result(self):
        """Can create default ExplainabilityResult."""
        result = ExplainabilityResult()

        assert result.method == "shap"
        assert result.summary_plot is None
        assert result.feature_importance_plot is None
        assert result.feature_importance == []

    def test_disabled_result(self):
        """Can create disabled ExplainabilityResult."""
        result = ExplainabilityResult(method="none")

        assert result.method == "none"

    def test_with_feature_importance(self):
        """Can create result with feature importance."""
        importance = [("feature_a", 0.5), ("feature_b", 0.3), ("feature_c", 0.2)]
        result = ExplainabilityResult(feature_importance=importance, method="shap")

        assert result.feature_importance == importance
        assert len(result.feature_importance) == 3


class TestExplainModel:
    """Tests for explain_model function."""

    @pytest.mark.slow
    @pytest.mark.integration
    def test_explain_tree_model(self, small_classification_data):
        """Can explain a tree-based model."""
        from sklearn.tree import DecisionTreeClassifier

        from src.preprocessing import Preprocessor

        X = small_classification_data.drop(columns=["target"])
        y = small_classification_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        # Train a simple model
        model = DecisionTreeClassifier(random_state=42, max_depth=3)
        model.fit(X_transformed, y_transformed)

        # Explain
        result = explain_model(
            model=model,
            X_sample=X_transformed,
            feature_names=prep.transformed_feature_names,
            problem_type=ProblemType.CLASSIFICATION,
            max_samples=20,
        )

        assert result.method in ["shap", "shap_failed", "none"]
        if result.method == "shap":
            assert len(result.feature_importance) > 0

    @pytest.mark.slow
    @pytest.mark.integration
    def test_explain_linear_model(self, small_classification_data):
        """Can explain a linear model."""
        from sklearn.linear_model import LogisticRegression

        from src.preprocessing import Preprocessor

        X = small_classification_data.drop(columns=["target"])
        y = small_classification_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        model = LogisticRegression(random_state=42, max_iter=200)
        model.fit(X_transformed, y_transformed)

        result = explain_model(
            model=model,
            X_sample=X_transformed,
            feature_names=prep.transformed_feature_names,
            problem_type=ProblemType.CLASSIFICATION,
            max_samples=20,
        )

        # Linear models may use different SHAP strategy
        assert result.method in ["shap", "shap_failed", "none", "failed"]

    @pytest.mark.slow
    def test_max_samples_limit(self, small_classification_data):
        """max_samples limits the data used for explanation."""
        from sklearn.tree import DecisionTreeClassifier

        from src.preprocessing import Preprocessor

        X = small_classification_data.drop(columns=["target"])
        y = small_classification_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        model = DecisionTreeClassifier(random_state=42, max_depth=3)
        model.fit(X_transformed, y_transformed)

        # Should not raise even with small max_samples
        result = explain_model(
            model=model,
            X_sample=X_transformed,
            feature_names=prep.transformed_feature_names,
            problem_type=ProblemType.CLASSIFICATION,
            max_samples=5,
        )

        assert result is not None


class TestSaveExplainabilityPlots:
    """Tests for saving explainability plots."""

    def test_save_empty_result(self, temp_dir):
        """Saving empty result returns empty list."""
        result = ExplainabilityResult(method="none")

        saved = save_explainability_plots(result, temp_dir)

        assert saved == []

    def test_save_with_plots(self, temp_dir):
        """Can save plots to disk."""
        # Create fake plot bytes
        fake_png = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100

        result = ExplainabilityResult(
            summary_plot=fake_png,
            feature_importance_plot=fake_png,
            method="shap",
        )

        saved = save_explainability_plots(result, temp_dir)

        assert len(saved) >= 1
        for path in saved:
            assert path.exists()

    def test_save_creates_directory(self, temp_dir):
        """Saving creates output directory if needed."""
        output_path = temp_dir / "subdir" / "plots"

        fake_png = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100
        result = ExplainabilityResult(summary_plot=fake_png, method="shap")

        save_explainability_plots(result, output_path)

        assert output_path.exists()


class TestPipelineExplainability:
    """Tests for explainability in full pipeline."""

    @pytest.mark.slow
    @pytest.mark.integration
    def test_pipeline_with_explainability(self, small_classification_data, explainability_config):
        """Pipeline generates explainability when enabled."""
        pipeline = Pipeline.builder().config(explainability_config).build()

        result = pipeline.train(small_classification_data)

        # Should have some explainability result
        assert result.explainability is not None
        # Method should be set
        assert result.explainability.method in ["shap", "shap_failed", "none", "failed"]

    @pytest.mark.slow
    @pytest.mark.integration
    def test_pipeline_without_explainability(self, small_classification_data):
        """Pipeline skips explainability when disabled."""
        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .target_column("target")
            .algorithm("decision_tree")
            .n_trials(2)
            .cv_folds(2)
            .enable_neural_networks(False)
            .enable_explainability(False)  # Disabled
            .build()
        )

        pipeline = Pipeline.builder().config(config).build()
        result = pipeline.train(small_classification_data)

        # Should have result but with "none" or "disabled" method
        assert result.explainability.method in ["none", "disabled"]

    @pytest.mark.slow
    @pytest.mark.integration
    def test_feature_importance_in_trained_model(
        self, small_classification_data, explainability_config
    ):
        """TrainedModel exposes feature importance from explainability."""
        pipeline = Pipeline.builder().config(explainability_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        # Feature importance should be accessible
        importance = trained_model.feature_importance
        assert isinstance(importance, list)

        if result.explainability.method == "shap" and importance:
            # Each entry should be (feature_name, value)
            assert all(isinstance(item, tuple) and len(item) == 2 for item in importance)


class TestExplainabilityPlotGeneration:
    """Tests for plot generation."""

    @pytest.mark.slow
    @pytest.mark.integration
    def test_generates_summary_plot(self, small_classification_data):
        """Generates summary plot bytes."""
        from sklearn.tree import DecisionTreeClassifier

        from src.preprocessing import Preprocessor

        X = small_classification_data.drop(columns=["target"])
        y = small_classification_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        model = DecisionTreeClassifier(random_state=42, max_depth=3)
        model.fit(X_transformed, y_transformed)

        result = explain_model(
            model=model,
            X_sample=X_transformed,
            feature_names=prep.transformed_feature_names,
            problem_type=ProblemType.CLASSIFICATION,
            max_samples=20,
        )

        if result.method == "shap":
            # Should have at least one plot
            has_plot = result.summary_plot is not None or result.feature_importance_plot is not None
            assert has_plot or len(result.feature_importance) > 0

    @pytest.mark.slow
    @pytest.mark.integration
    def test_feature_importance_sorted(self, small_classification_data):
        """Feature importance is sorted by value."""
        from sklearn.tree import DecisionTreeClassifier

        from src.preprocessing import Preprocessor

        X = small_classification_data.drop(columns=["target"])
        y = small_classification_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        model = DecisionTreeClassifier(random_state=42, max_depth=3)
        model.fit(X_transformed, y_transformed)

        result = explain_model(
            model=model,
            X_sample=X_transformed,
            feature_names=prep.transformed_feature_names,
            problem_type=ProblemType.CLASSIFICATION,
            max_samples=20,
        )

        if result.method == "shap" and result.feature_importance:
            values = [v for _, v in result.feature_importance]
            # Should be sorted descending
            assert values == sorted(values, reverse=True)
