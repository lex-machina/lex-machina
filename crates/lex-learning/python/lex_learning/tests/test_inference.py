"""Tests for model inference (save/load/predict)."""

from __future__ import annotations

import pytest

from src import Pipeline, PipelineConfig, ProblemType, TrainedModel
from src.errors import InferenceError, ModelNotFoundError


class TestTrainedModelSaveLoad:
    """Tests for saving and loading models."""

    @pytest.mark.slow
    @pytest.mark.integration
    def test_save_model(self, small_classification_data, classification_config, temp_dir):
        """Can save a trained model."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)
        model_path = temp_dir / "model.pkl"

        trained_model.save(model_path)

        assert model_path.exists()
        assert model_path.stat().st_size > 0

    @pytest.mark.slow
    @pytest.mark.integration
    def test_load_model(self, small_classification_data, classification_config, temp_dir):
        """Can load a saved model."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)
        model_path = temp_dir / "model.pkl"
        trained_model.save(model_path)

        loaded_model = TrainedModel.load(model_path)

        assert loaded_model is not None
        assert loaded_model.problem_type == ProblemType.CLASSIFICATION
        assert loaded_model.target_column == "target"

    @pytest.mark.slow
    @pytest.mark.integration
    def test_loaded_model_has_same_properties(
        self, small_classification_data, classification_config, temp_dir
    ):
        """Loaded model has same properties as original."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        original = pipeline.create_trained_model(result)
        model_path = temp_dir / "model.pkl"
        original.save(model_path)

        loaded = TrainedModel.load(model_path)

        assert loaded.problem_type == original.problem_type
        assert loaded.target_column == original.target_column
        assert loaded.feature_names == original.feature_names
        assert loaded.best_model_name == original.best_model_name

    def test_load_nonexistent_raises(self, temp_dir):
        """Loading non-existent file raises ModelNotFoundError."""
        with pytest.raises(ModelNotFoundError):
            TrainedModel.load(temp_dir / "nonexistent.pkl")

    @pytest.mark.slow
    @pytest.mark.integration
    def test_save_load_regression(self, small_regression_data, regression_config, temp_dir):
        """Can save and load regression model."""
        pipeline = Pipeline.builder().config(regression_config).build()
        result = pipeline.train(small_regression_data)

        trained_model = pipeline.create_trained_model(result)
        model_path = temp_dir / "regression_model.pkl"
        trained_model.save(model_path)

        loaded = TrainedModel.load(model_path)

        assert loaded.problem_type == ProblemType.REGRESSION


class TestTrainedModelPredict:
    """Tests for single prediction."""

    @pytest.mark.slow
    @pytest.mark.integration
    def test_predict_single(self, small_classification_data, classification_config, temp_dir):
        """Can make single prediction."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        sample = small_classification_data.drop(columns=["target"]).iloc[0].to_dict()
        prediction = trained_model.predict(sample)

        assert "prediction" in prediction
        assert prediction["prediction"] in ["yes", "no"]

    @pytest.mark.slow
    @pytest.mark.integration
    def test_predict_classification_has_full_probabilities(
        self, small_classification_data, classification_config, temp_dir
    ):
        """Classification prediction includes full probabilities dict."""
        # Use a model that supports predict_proba
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
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        sample = small_classification_data.drop(columns=["target"]).iloc[0].to_dict()
        prediction = trained_model.predict(sample)

        assert "prediction" in prediction
        assert "probabilities" in prediction
        assert isinstance(prediction["probabilities"], dict)
        # Should have probability for each class
        assert set(prediction["probabilities"].keys()) == {"yes", "no"}
        # Probabilities should sum to ~1
        total_prob = sum(prediction["probabilities"].values())
        assert abs(total_prob - 1.0) < 0.01

    @pytest.mark.slow
    @pytest.mark.integration
    def test_predict_classification_has_probability(
        self, small_classification_data, classification_config, temp_dir
    ):
        """Classification prediction includes probability (for supported models)."""
        # Use a model that supports predict_proba
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
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        sample = small_classification_data.drop(columns=["target"]).iloc[0].to_dict()
        prediction = trained_model.predict(sample)

        assert "prediction" in prediction
        if "probability" in prediction:
            assert 0 <= prediction["probability"] <= 1

    @pytest.mark.slow
    @pytest.mark.integration
    def test_predict_regression(self, small_regression_data, regression_config, temp_dir):
        """Can make regression prediction."""
        pipeline = Pipeline.builder().config(regression_config).build()
        result = pipeline.train(small_regression_data)

        trained_model = pipeline.create_trained_model(result)

        sample = small_regression_data.drop(columns=["target"]).iloc[0].to_dict()
        prediction = trained_model.predict(sample)

        assert "prediction" in prediction
        assert isinstance(prediction["prediction"], float)

    @pytest.mark.slow
    @pytest.mark.integration
    def test_predict_missing_feature_raises(self, small_classification_data, classification_config):
        """Missing features in input raises InferenceError."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        # Missing 'x2' feature
        sample = {"x1": 0.5}

        with pytest.raises(InferenceError):
            trained_model.predict(sample)


class TestTrainedModelBatchPredict:
    """Tests for batch prediction."""

    @pytest.mark.slow
    @pytest.mark.integration
    def test_predict_batch_dataframe(self, small_classification_data, classification_config):
        """Can make batch predictions from DataFrame."""
        import pandas as pd

        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        # Use first 5 rows for prediction
        test_data = small_classification_data.drop(columns=["target"]).head(5)
        predictions = trained_model.predict_batch(test_data)

        assert isinstance(predictions, pd.DataFrame)
        assert "prediction" in predictions.columns
        assert len(predictions) == 5

    @pytest.mark.slow
    @pytest.mark.integration
    def test_predict_batch_csv(self, small_classification_data, classification_config, temp_dir):
        """Can make batch predictions from CSV file."""
        import pandas as pd

        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        # Save test data to CSV
        test_data = small_classification_data.drop(columns=["target"]).head(5)
        csv_path = temp_dir / "test_data.csv"
        test_data.to_csv(csv_path, index=False)

        predictions = trained_model.predict_batch(csv_path)

        assert isinstance(predictions, pd.DataFrame)
        assert "prediction" in predictions.columns
        assert len(predictions) == 5

    @pytest.mark.slow
    @pytest.mark.integration
    def test_predict_batch_preserves_original_columns(
        self, small_classification_data, classification_config
    ):
        """Batch prediction preserves original columns."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        test_data = small_classification_data.drop(columns=["target"]).head(5)
        predictions = trained_model.predict_batch(test_data)

        # Should have original columns plus prediction
        assert "x1" in predictions.columns
        assert "x2" in predictions.columns
        assert "prediction" in predictions.columns


class TestTrainedModelInfo:
    """Tests for model information retrieval."""

    @pytest.mark.slow
    @pytest.mark.integration
    def test_get_info(self, small_classification_data, classification_config):
        """Can get model info dictionary."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)
        info = trained_model.get_info()

        assert isinstance(info, dict)
        assert "version" in info
        assert "problem_type" in info
        assert "target_column" in info
        assert "best_model_name" in info
        assert "feature_names" in info
        assert "trained_at" in info
        assert "training_time_seconds" in info
        assert "metrics" in info
        assert "hyperparameters" in info

    @pytest.mark.slow
    @pytest.mark.integration
    def test_get_info_has_hyperparameters(self, small_classification_data, classification_config):
        """Model info includes hyperparameters dict."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)
        info = trained_model.get_info()

        # hyperparameters should be a dict (or None for simple models)
        assert "hyperparameters" in info
        if info["hyperparameters"] is not None:
            assert isinstance(info["hyperparameters"], dict)

    @pytest.mark.slow
    @pytest.mark.integration
    def test_hyperparameters_property(self, small_classification_data, classification_config):
        """Model has hyperparameters property."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        # hyperparameters should be a dict or None
        hyperparams = trained_model.hyperparameters
        if hyperparams is not None:
            assert isinstance(hyperparams, dict)

    @pytest.mark.slow
    @pytest.mark.integration
    def test_get_info_classification_metrics(
        self, small_classification_data, classification_config
    ):
        """Classification model info includes classification metrics."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)
        info = trained_model.get_info()

        metrics = info["metrics"]
        assert "accuracy" in metrics
        assert "precision" in metrics
        assert "recall" in metrics
        assert "f1_score" in metrics
        assert "test_score" in metrics
        assert "cv_score" in metrics

    @pytest.mark.slow
    @pytest.mark.integration
    def test_get_info_regression_metrics(self, small_regression_data, regression_config):
        """Regression model info includes regression metrics."""
        pipeline = Pipeline.builder().config(regression_config).build()
        result = pipeline.train(small_regression_data)

        trained_model = pipeline.create_trained_model(result)
        info = trained_model.get_info()

        metrics = info["metrics"]
        assert "mse" in metrics
        assert "rmse" in metrics
        assert "mae" in metrics
        assert "r2" in metrics

    @pytest.mark.slow
    @pytest.mark.integration
    def test_properties(self, small_classification_data, classification_config):
        """Model properties work correctly."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        assert trained_model.problem_type == ProblemType.CLASSIFICATION
        assert trained_model.target_column == "target"
        assert isinstance(trained_model.feature_names, list)
        assert isinstance(trained_model.best_model_name, str)
        assert trained_model.metrics is not None

    @pytest.mark.slow
    @pytest.mark.integration
    def test_class_labels_for_classification(
        self, small_classification_data, classification_config
    ):
        """Classification models have class labels."""
        pipeline = Pipeline.builder().config(classification_config).build()
        result = pipeline.train(small_classification_data)

        trained_model = pipeline.create_trained_model(result)

        labels = trained_model.class_labels
        assert labels is not None
        assert set(labels) == {"yes", "no"}

    @pytest.mark.slow
    @pytest.mark.integration
    def test_class_labels_none_for_regression(self, small_regression_data, regression_config):
        """Regression models have None for class labels."""
        pipeline = Pipeline.builder().config(regression_config).build()
        result = pipeline.train(small_regression_data)

        trained_model = pipeline.create_trained_model(result)

        assert trained_model.class_labels is None
