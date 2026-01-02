"""Data preprocessing for ML training.

Handles encoding of categorical features and scaling of numeric features.
"""

from __future__ import annotations

from typing import Any, cast

import numpy as np
import pandas as pd
from numpy.typing import NDArray
from scipy import sparse
from sklearn.compose import ColumnTransformer
from sklearn.pipeline import Pipeline as SklearnPipeline
from sklearn.preprocessing import LabelEncoder, OneHotEncoder, StandardScaler

from ..config import ProblemType


class Preprocessor:
    """Preprocesses data for ML training.

    Handles:
    - Encoding categorical features (OneHotEncoder)
    - Scaling numeric features (StandardScaler)
    - Encoding target variable for classification (LabelEncoder)

    The preprocessor must be fit on training data before transforming.
    It is saved with the model for consistent inference.
    """

    def __init__(self, problem_type: ProblemType) -> None:
        """Initialize preprocessor.

        Args:
            problem_type: Type of ML problem (classification or regression).
        """
        self.problem_type = problem_type
        self._feature_transformer: ColumnTransformer | None = None
        self._target_encoder: LabelEncoder | None = None
        self._feature_names: list[str] = []
        self._numeric_features: list[str] = []
        self._categorical_features: list[str] = []
        self._class_labels: list[str] | None = None
        self._is_fitted = False

    @property
    def is_fitted(self) -> bool:
        """Whether the preprocessor has been fitted."""
        return self._is_fitted

    @property
    def feature_names(self) -> list[str]:
        """Original feature names before transformation."""
        return self._feature_names.copy()

    @property
    def class_labels(self) -> list[str] | None:
        """Class labels for classification problems."""
        return self._class_labels.copy() if self._class_labels else None

    @property
    def transformed_feature_names(self) -> list[str]:
        """Feature names after transformation (includes one-hot encoded names)."""
        if not self._is_fitted or self._feature_transformer is None:
            return []
        try:
            return list(self._feature_transformer.get_feature_names_out())
        except Exception:
            # Fallback if get_feature_names_out fails
            return self._feature_names.copy()

    def fit(
        self,
        X: pd.DataFrame,
        y: pd.Series,
    ) -> Preprocessor:
        """Fit the preprocessor on training data.

        Args:
            X: Feature DataFrame.
            y: Target Series.

        Returns:
            Self for method chaining.
        """
        self._feature_names = [str(c) for c in X.columns]
        self._identify_column_types(X)
        self._build_feature_transformer()

        if self._feature_transformer is not None:
            self._feature_transformer.fit(X)

        # Fit target encoder for classification
        if self.problem_type == ProblemType.CLASSIFICATION:
            self._target_encoder = LabelEncoder()
            self._target_encoder.fit(y)
            # classes_ is always an ndarray after fit
            encoder_classes: Any = self._target_encoder.classes_
            self._class_labels = [str(c) for c in encoder_classes]

        self._is_fitted = True
        return self

    def transform(
        self,
        X: pd.DataFrame,
        y: pd.Series | None = None,
    ) -> tuple[NDArray[Any], NDArray[Any] | None]:
        """Transform features and optionally target.

        Args:
            X: Feature DataFrame.
            y: Optional target Series.

        Returns:
            Tuple of (transformed_X, transformed_y).
            transformed_y is None if y is not provided.

        Raises:
            ValueError: If preprocessor is not fitted.
        """
        if not self._is_fitted or self._feature_transformer is None:
            raise ValueError("Preprocessor must be fitted before transform")

        X_result = self._feature_transformer.transform(X)
        # Ensure we have a dense numpy array
        if sparse.issparse(X_result):
            X_transformed: NDArray[Any] = X_result.toarray()  # type: ignore[union-attr]
        else:
            X_transformed = np.asarray(X_result)

        y_transformed: NDArray[Any] | None = None
        if y is not None:
            if self.problem_type == ProblemType.CLASSIFICATION and self._target_encoder:
                y_transformed = self._target_encoder.transform(y)
            else:
                y_transformed = np.asarray(y.values)

        return X_transformed, y_transformed

    def fit_transform(
        self,
        X: pd.DataFrame,
        y: pd.Series,
    ) -> tuple[NDArray[Any], NDArray[Any]]:
        """Fit and transform in one step.

        Args:
            X: Feature DataFrame.
            y: Target Series.

        Returns:
            Tuple of (transformed_X, transformed_y).
        """
        self.fit(X, y)
        X_transformed, y_transformed = self.transform(X, y)
        assert y_transformed is not None  # We know y is not None here
        return X_transformed, y_transformed

    def inverse_transform_target(self, y: NDArray[Any]) -> NDArray[Any]:
        """Inverse transform target values (for classification).

        Args:
            y: Encoded target values.

        Returns:
            Original target values.
        """
        if self.problem_type == ProblemType.CLASSIFICATION and self._target_encoder:
            return cast(NDArray[Any], self._target_encoder.inverse_transform(y.astype(int)))
        return y

    def _identify_column_types(self, X: pd.DataFrame) -> None:
        """Identify numeric and categorical columns."""
        self._numeric_features = []
        self._categorical_features = []

        for col in X.columns:
            if pd.api.types.is_numeric_dtype(X[col]):
                self._numeric_features.append(str(col))
            else:
                self._categorical_features.append(str(col))

    def _build_feature_transformer(self) -> None:
        """Build the sklearn ColumnTransformer."""
        transformers: list[tuple[str, SklearnPipeline, list[str]]] = []

        if self._numeric_features:
            numeric_pipeline = SklearnPipeline(
                [
                    ("scaler", StandardScaler()),
                ]
            )
            transformers.append(("numeric", numeric_pipeline, self._numeric_features))

        if self._categorical_features:
            categorical_pipeline = SklearnPipeline(
                [
                    ("encoder", OneHotEncoder(handle_unknown="ignore", sparse_output=False)),
                ]
            )
            transformers.append(("categorical", categorical_pipeline, self._categorical_features))

        self._feature_transformer = ColumnTransformer(
            transformers=transformers,
            remainder="drop",  # Drop any columns not in our lists
        )
