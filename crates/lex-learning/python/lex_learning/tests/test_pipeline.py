"""Tests for full training pipeline integration."""

from __future__ import annotations

import pytest

from src import (
    ClassificationMetrics,
    Pipeline,
    PipelineConfig,
    ProblemType,
    ProgressUpdate,
    RegressionMetrics,
    TrainingStage,
)
from src.errors import InvalidDataError, TargetNotFoundError


class TestPipelineBuilder:
    """Tests for Pipeline.builder() interface."""

    def test_builder_requires_config(self):
        """build() raises if config not set."""
        with pytest.raises(ValueError, match="config is required"):
            Pipeline.builder().build()

    def test_builder_with_config_only(self, classification_config):
        """Can build pipeline with just config."""
        pipeline = Pipeline.builder().config(classification_config).build()
        assert pipeline is not None

    def test_builder_with_progress_callback(self, classification_config):
        """Can build pipeline with progress callback."""
        updates = []

        def callback(update: ProgressUpdate) -> None:
            updates.append(update)

        pipeline = Pipeline.builder().config(classification_config).on_progress(callback).build()

        assert pipeline is not None

    def test_builder_returns_self(self, classification_config):
        """Builder methods return self for chaining."""
        builder = Pipeline.builder()

        assert builder.config(classification_config) is builder
        assert builder.on_progress(lambda u: None) is builder


class TestPipelineValidation:
    """Tests for pipeline data validation."""

    def test_validates_no_null_values(self, classification_config):
        """Pipeline rejects data with null values."""
        import pandas as pd

        data = pd.DataFrame(
            {"a": [1, 2, None, 4], "b": [5, 6, 7, 8], "target": [0, 1, 0, 1]}  # Null value
        )

        pipeline = Pipeline.builder().config(classification_config).build()

        with pytest.raises(InvalidDataError):
            pipeline.train(data)

    def test_validates_target_exists(self):
        """Pipeline rejects data when target column not found."""
        import pandas as pd

        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .target_column("nonexistent_column")
            .build()
        )

        data = pd.DataFrame({"a": [1, 2, 3], "b": [4, 5, 6], "target": [0, 1, 0]})

        pipeline = Pipeline.builder().config(config).build()

        with pytest.raises(TargetNotFoundError):
            pipeline.train(data)

    def test_validates_minimum_samples(self, classification_config):
        """Pipeline rejects data with too few samples."""
        import pandas as pd

        # Only 5 samples (below minimum of 10)
        data = pd.DataFrame({"a": [1, 2, 3, 4, 5], "target": [0, 1, 0, 1, 0]})

        pipeline = Pipeline.builder().config(classification_config).build()

        with pytest.raises(InvalidDataError):
            pipeline.train(data)


class TestPipelineClassification:
    """Tests for classification training."""

    @pytest.mark.slow
    @pytest.mark.integration
    def test_train_classification(self, small_classification_data, classification_config):
        """Can train a classification model end-to-end."""
        pipeline = Pipeline.builder().config(classification_config).build()

        result = pipeline.train(small_classification_data)

        assert result.success is True
        assert result.best_model_name is not None
        assert isinstance(result.metrics, ClassificationMetrics)
        assert len(result.model_comparison) > 0
        assert result.training_time_seconds > 0

    @pytest.mark.slow
    @pytest.mark.integration
    def test_classification_metrics(self, small_classification_data, classification_config):
        """Classification returns correct metrics."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        metrics = result.metrics
        assert isinstance(metrics, ClassificationMetrics)
        assert 0 <= metrics.accuracy <= 1
        assert 0 <= metrics.precision <= 1
        assert 0 <= metrics.recall <= 1
        assert 0 <= metrics.f1_score <= 1
        assert 0 <= metrics.cv_score <= 1
        assert 0 <= metrics.test_score <= 1

    @pytest.mark.slow
    @pytest.mark.integration
    def test_binary_classification(self, binary_classification_data):
        """Can train binary classification."""
        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .target_column("target")
            .algorithm("logistic_regression")
            .n_trials(2)
            .cv_folds(2)
            .enable_neural_networks(False)
            .enable_explainability(False)
            .build()
        )

        pipeline = Pipeline.builder().config(config).build()
        result = pipeline.train(binary_classification_data)

        assert result.success is True
        # Binary classification may have ROC-AUC
        assert isinstance(result.metrics, ClassificationMetrics)


class TestPipelineRegression:
    """Tests for regression training."""

    @pytest.mark.slow
    @pytest.mark.integration
    def test_train_regression(self, small_regression_data, regression_config):
        """Can train a regression model end-to-end."""
        pipeline = Pipeline.builder().config(regression_config).build()

        result = pipeline.train(small_regression_data)

        assert result.success is True
        assert result.best_model_name is not None
        assert isinstance(result.metrics, RegressionMetrics)

    @pytest.mark.slow
    @pytest.mark.integration
    def test_regression_metrics(self, small_regression_data, regression_config):
        """Regression returns correct metrics."""
        pipeline = Pipeline.builder().config(regression_config).build()
        result = pipeline.train(small_regression_data)

        metrics = result.metrics
        assert isinstance(metrics, RegressionMetrics)
        assert metrics.mse >= 0
        assert metrics.rmse >= 0
        assert metrics.mae >= 0
        # R2 can be negative for bad models
        assert isinstance(metrics.r2, float)


class TestPipelineProgress:
    """Tests for progress reporting."""

    @pytest.mark.slow
    @pytest.mark.integration
    def test_progress_callback_called(self, small_classification_data, classification_config):
        """Progress callback is called during training."""
        updates = []

        def callback(update: ProgressUpdate) -> None:
            updates.append(update)

        pipeline = Pipeline.builder().config(classification_config).on_progress(callback).build()

        pipeline.train(small_classification_data)

        assert len(updates) > 0
        # Should have at least initializing and complete
        stages = [u.stage for u in updates]
        assert TrainingStage.INITIALIZING in stages
        assert TrainingStage.COMPLETE in stages

    @pytest.mark.slow
    @pytest.mark.integration
    def test_progress_increases(self, small_classification_data, classification_config):
        """Progress generally increases during training."""
        progresses = []

        def callback(update: ProgressUpdate) -> None:
            progresses.append(update.progress)

        pipeline = Pipeline.builder().config(classification_config).on_progress(callback).build()

        pipeline.train(small_classification_data)

        # Progress should reach 1.0 at the end
        assert progresses[-1] == 1.0
        # Most progress values should be increasing
        increasing_count = sum(
            1 for i in range(1, len(progresses)) if progresses[i] >= progresses[i - 1]
        )
        assert increasing_count > len(progresses) * 0.8  # 80% should be increasing

    @pytest.mark.slow
    @pytest.mark.integration
    def test_progress_update_fields(self, small_classification_data, classification_config):
        """Progress updates have expected fields."""
        updates = []

        def callback(update: ProgressUpdate) -> None:
            updates.append(update)

        pipeline = Pipeline.builder().config(classification_config).on_progress(callback).build()

        pipeline.train(small_classification_data)

        for update in updates:
            assert isinstance(update.stage, TrainingStage)
            assert isinstance(update.progress, float)
            assert 0 <= update.progress <= 1
            assert isinstance(update.message, str)


class TestPipelineModelComparison:
    """Tests for model comparison in results."""

    @pytest.mark.slow
    @pytest.mark.integration
    def test_model_comparison_populated(self, small_classification_data):
        """Model comparison contains trained models."""
        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .target_column("target")
            .top_k_algorithms(2)
            .n_trials(2)
            .cv_folds(2)
            .enable_neural_networks(False)
            .enable_explainability(False)
            .build()
        )

        pipeline = Pipeline.builder().config(config).build()
        result = pipeline.train(small_classification_data)

        assert len(result.model_comparison) >= 1

    @pytest.mark.slow
    @pytest.mark.integration
    def test_model_comparison_sorted(self, small_classification_data):
        """Model comparison is sorted by test score descending."""
        config = (
            PipelineConfig.builder()
            .problem_type(ProblemType.CLASSIFICATION)
            .target_column("target")
            .top_k_algorithms(2)
            .n_trials(2)
            .cv_folds(2)
            .enable_neural_networks(False)
            .enable_explainability(False)
            .build()
        )

        pipeline = Pipeline.builder().config(config).build()
        result = pipeline.train(small_classification_data)

        if len(result.model_comparison) > 1:
            scores = [m.test_score for m in result.model_comparison]
            assert scores == sorted(scores, reverse=True)

    @pytest.mark.slow
    @pytest.mark.integration
    def test_best_model_is_first(self, small_classification_data, classification_config):
        """Best model name matches first in comparison."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        assert result.best_model_name == result.model_comparison[0].name


class TestPipelineCreateTrainedModel:
    """Tests for create_trained_model method."""

    @pytest.mark.slow
    @pytest.mark.integration
    def test_create_trained_model(self, small_classification_data, classification_config):
        """Can create TrainedModel from result."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        assert trained_model is not None
        assert trained_model.problem_type == ProblemType.CLASSIFICATION
        assert trained_model.target_column == "target"
        assert len(trained_model.feature_names) > 0

    @pytest.mark.slow
    @pytest.mark.integration
    def test_created_model_can_predict(self, small_classification_data, classification_config):
        """Created TrainedModel can make predictions."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        # Get a sample for prediction
        sample = small_classification_data.drop(columns=["target"]).iloc[0].to_dict()
        prediction = trained_model.predict(sample)

        assert "prediction" in prediction
