"""Tests for Preprocessor (encoding and scaling)."""

from __future__ import annotations

import numpy as np
import pandas as pd
import pytest

from src import ProblemType
from src.preprocessing import Preprocessor


class TestPreprocessorInit:
    """Tests for Preprocessor initialization."""

    def test_init_classification(self):
        """Can initialize for classification."""
        prep = Preprocessor(ProblemType.CLASSIFICATION)
        assert prep.problem_type == ProblemType.CLASSIFICATION
        assert prep.is_fitted is False

    def test_init_regression(self):
        """Can initialize for regression."""
        prep = Preprocessor(ProblemType.REGRESSION)
        assert prep.problem_type == ProblemType.REGRESSION
        assert prep.is_fitted is False

    def test_properties_before_fit(self):
        """Properties return empty/None before fitting."""
        prep = Preprocessor(ProblemType.CLASSIFICATION)

        assert prep.feature_names == []
        assert prep.class_labels is None
        assert prep.transformed_feature_names == []


class TestPreprocessorNumeric:
    """Tests for preprocessing numeric features."""

    def test_fit_numeric_only(self):
        """Can fit on numeric-only data."""
        X = pd.DataFrame({"a": [1.0, 2.0, 3.0], "b": [4.0, 5.0, 6.0]})
        y = pd.Series([0, 1, 0])

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        prep.fit(X, y)

        assert prep.is_fitted is True
        assert prep.feature_names == ["a", "b"]

    def test_transform_numeric_scales(self):
        """Numeric features are scaled (StandardScaler)."""
        X = pd.DataFrame({"a": [0.0, 10.0, 20.0], "b": [100.0, 200.0, 300.0]})
        y = pd.Series([0, 1, 0])

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        # StandardScaler centers data (mean=0, std=1)
        assert X_transformed.shape == (3, 2)
        # Check that values are scaled (not original values)
        assert not np.allclose(X_transformed[:, 0], [0.0, 10.0, 20.0])
        # Mean should be approximately 0 for each feature
        assert np.abs(X_transformed.mean(axis=0)).max() < 1e-10

    def test_fit_transform_returns_correct_shapes(self):
        """fit_transform returns arrays with correct shapes."""
        X = pd.DataFrame({"a": np.random.randn(100), "b": np.random.randn(100)})
        y = pd.Series(np.random.randint(0, 2, 100))

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        assert X_transformed.shape == (100, 2)
        assert y_transformed.shape == (100,)


class TestPreprocessorCategorical:
    """Tests for preprocessing categorical features."""

    def test_fit_categorical_only(self):
        """Can fit on categorical-only data."""
        X = pd.DataFrame({"cat1": ["A", "B", "A", "C"], "cat2": ["X", "Y", "X", "Y"]})
        y = pd.Series([0, 1, 0, 1])

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        prep.fit(X, y)

        assert prep.is_fitted is True
        assert prep.feature_names == ["cat1", "cat2"]

    def test_transform_categorical_one_hot(self):
        """Categorical features are one-hot encoded."""
        X = pd.DataFrame({"cat": ["A", "B", "C", "A"]})
        y = pd.Series([0, 1, 0, 1])

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, _ = prep.fit_transform(X, y)

        # One-hot encoding creates 3 columns (A, B, C)
        assert X_transformed.shape == (4, 3)
        # First row should be [1, 0, 0] for "A"
        assert X_transformed[0].sum() == 1.0  # One-hot: exactly one 1
        assert X_transformed[1].sum() == 1.0
        assert X_transformed[2].sum() == 1.0
        assert X_transformed[3].sum() == 1.0


class TestPreprocessorMixed:
    """Tests for preprocessing mixed numeric and categorical features."""

    def test_fit_mixed_types(self, mixed_types_data):
        """Can fit on mixed numeric and categorical data."""
        X = mixed_types_data.drop(columns=["target"])
        y = mixed_types_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        prep.fit(X, y)

        assert prep.is_fitted is True
        assert "numeric_1" in prep.feature_names
        assert "category_1" in prep.feature_names

    def test_transform_mixed_types(self, mixed_types_data):
        """Mixed data is transformed correctly."""
        X = mixed_types_data.drop(columns=["target"])
        y = mixed_types_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        # Should have: 2 numeric + one-hot encoded categoricals
        # category_1 has 3 values (A, B, C) -> 3 columns
        # category_2 has 2 values (X, Y) -> 2 columns
        # Total: 2 + 3 + 2 = 7 columns
        assert X_transformed.shape[0] == len(X)
        assert X_transformed.shape[1] == 7

    def test_transformed_feature_names(self, mixed_types_data):
        """transformed_feature_names includes one-hot encoded names."""
        X = mixed_types_data.drop(columns=["target"])
        y = mixed_types_data["target"]

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        prep.fit(X, y)

        names = prep.transformed_feature_names
        assert len(names) == 7
        # Should include original numeric names
        assert any("numeric" in n for n in names)


class TestPreprocessorTarget:
    """Tests for target encoding/decoding."""

    def test_classification_target_encoded(self):
        """Classification targets are label encoded."""
        X = pd.DataFrame({"a": [1, 2, 3, 4]})
        y = pd.Series(["cat", "dog", "cat", "bird"])

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        # Should be encoded as integers
        assert y_transformed.dtype in [np.int32, np.int64]
        assert set(y_transformed) == {0, 1, 2}  # 3 unique classes

    def test_regression_target_not_encoded(self):
        """Regression targets are not encoded."""
        X = pd.DataFrame({"a": [1, 2, 3, 4]})
        y = pd.Series([1.5, 2.5, 3.5, 4.5])

        prep = Preprocessor(ProblemType.REGRESSION)
        X_transformed, y_transformed = prep.fit_transform(X, y)

        # Should remain as float
        assert np.allclose(y_transformed, [1.5, 2.5, 3.5, 4.5])

    def test_class_labels_stored(self):
        """Class labels are stored after fitting classification."""
        X = pd.DataFrame({"a": [1, 2, 3]})
        y = pd.Series(["yes", "no", "yes"])

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        prep.fit(X, y)

        labels = prep.class_labels
        assert labels is not None
        assert set(labels) == {"yes", "no"}

    def test_inverse_transform_target(self):
        """Can inverse transform encoded targets."""
        X = pd.DataFrame({"a": [1, 2, 3, 4]})
        y = pd.Series(["A", "B", "C", "A"])

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        prep.fit(X, y)

        # Encode then decode
        _, y_encoded = prep.transform(X, y)
        y_decoded = prep.inverse_transform_target(y_encoded)

        # Should get back original labels
        assert list(y_decoded) == ["A", "B", "C", "A"]


class TestPreprocessorTransform:
    """Tests for transform behavior."""

    def test_transform_without_fit_raises(self):
        """transform() raises if not fitted."""
        X = pd.DataFrame({"a": [1, 2, 3]})

        prep = Preprocessor(ProblemType.CLASSIFICATION)

        with pytest.raises(ValueError, match="must be fitted"):
            prep.transform(X)

    def test_transform_without_y(self):
        """Can transform without y (for inference)."""
        X = pd.DataFrame({"a": [1, 2, 3], "b": [4, 5, 6]})
        y = pd.Series([0, 1, 0])

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        prep.fit(X, y)

        X_new = pd.DataFrame({"a": [7, 8], "b": [9, 10]})
        X_transformed, y_transformed = prep.transform(X_new)

        assert X_transformed.shape == (2, 2)
        assert y_transformed is None

    def test_transform_new_data(self):
        """Can transform new data after fitting."""
        X_train = pd.DataFrame({"a": [1, 2, 3], "b": [4, 5, 6]})
        y_train = pd.Series([0, 1, 0])

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        prep.fit(X_train, y_train)

        X_test = pd.DataFrame({"a": [10, 20], "b": [40, 50]})
        X_transformed, _ = prep.transform(X_test)

        assert X_transformed.shape == (2, 2)

    def test_unknown_category_handled(self):
        """Unknown categories in new data are handled gracefully."""
        X_train = pd.DataFrame({"cat": ["A", "B", "A"]})
        y_train = pd.Series([0, 1, 0])

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        prep.fit(X_train, y_train)

        # "C" was not in training data
        X_test = pd.DataFrame({"cat": ["A", "C"]})
        X_transformed, _ = prep.transform(X_test)

        # Should not raise - OneHotEncoder handles unknown with zeros
        assert X_transformed.shape == (2, 2)  # A, B columns
        # "C" should have all zeros (unknown)
        assert X_transformed[1].sum() == 0.0


class TestPreprocessorFeatureNames:
    """Tests for feature name handling."""

    def test_feature_names_preserved(self):
        """Original feature names are preserved."""
        X = pd.DataFrame({"my_feature": [1, 2, 3], "another_one": [4, 5, 6]})
        y = pd.Series([0, 1, 0])

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        prep.fit(X, y)

        assert prep.feature_names == ["my_feature", "another_one"]

    def test_feature_names_is_copy(self):
        """feature_names returns a copy (not the internal list)."""
        X = pd.DataFrame({"a": [1, 2, 3]})
        y = pd.Series([0, 1, 0])

        prep = Preprocessor(ProblemType.CLASSIFICATION)
        prep.fit(X, y)

        names = prep.feature_names
        names.append("should_not_affect_internal")

        assert "should_not_affect_internal" not in prep.feature_names
