"""TensorFlow/Keras neural network model definitions.

Neural networks are wrapped in sklearn-compatible classes for
integration with the training pipeline.
"""

from __future__ import annotations

from typing import Any

import numpy as np
import optuna
from numpy.typing import NDArray
from sklearn.base import BaseEstimator, ClassifierMixin, RegressorMixin

from ..config import ProblemType

# Lazy import TensorFlow
_tensorflow_available = True
_tf: Any = None
_keras: Any = None

try:
    import tensorflow as _tf  # noqa: N812
    from tensorflow import keras as _keras  # noqa: N812
except ImportError:
    _tensorflow_available = False


class KerasClassifier(BaseEstimator, ClassifierMixin):
    """Sklearn-compatible wrapper for Keras classification models."""

    def __init__(
        self,
        hidden_layers: tuple[int, ...] = (64, 32),
        dropout_rate: float = 0.2,
        learning_rate: float = 0.001,
        epochs: int = 100,
        batch_size: int = 32,
        early_stopping_patience: int = 10,
        random_state: int = 42,
        verbose: int = 0,
    ) -> None:
        self.hidden_layers = hidden_layers
        self.dropout_rate = dropout_rate
        self.learning_rate = learning_rate
        self.epochs = epochs
        self.batch_size = batch_size
        self.early_stopping_patience = early_stopping_patience
        self.random_state = random_state
        self.verbose = verbose
        self._model: Any = None
        self._n_features: int = 0
        self._n_classes: int = 0
        self.classes_: NDArray[Any] | None = None

    def _build_model(self, n_features: int, n_classes: int) -> Any:
        """Build the Keras model."""
        if not _tensorflow_available or _keras is None:
            raise ImportError("TensorFlow is required for neural network models")

        _tf.random.set_seed(self.random_state)

        model = _keras.Sequential()
        model.add(_keras.layers.Input(shape=(n_features,)))

        for units in self.hidden_layers:
            model.add(_keras.layers.Dense(units, activation="relu"))
            model.add(_keras.layers.Dropout(self.dropout_rate))

        if n_classes == 2:
            model.add(_keras.layers.Dense(1, activation="sigmoid"))
            loss = "binary_crossentropy"
        else:
            model.add(_keras.layers.Dense(n_classes, activation="softmax"))
            loss = "sparse_categorical_crossentropy"

        optimizer = _keras.optimizers.Adam(learning_rate=self.learning_rate)
        model.compile(optimizer=optimizer, loss=loss, metrics=["accuracy"])

        return model

    def fit(self, X: NDArray[Any], y: NDArray[Any]) -> KerasClassifier:
        """Fit the model."""
        if not _tensorflow_available or _keras is None:
            raise ImportError("TensorFlow is required for neural network models")

        self._n_features = X.shape[1]
        self.classes_ = np.unique(y)
        self._n_classes = len(self.classes_)

        self._model = self._build_model(self._n_features, self._n_classes)

        callbacks = [
            _keras.callbacks.EarlyStopping(
                monitor="val_loss",
                patience=self.early_stopping_patience,
                restore_best_weights=True,
            )
        ]

        # Convert labels for binary classification
        y_train = y.copy()
        if self._n_classes == 2:
            y_train = (y == self.classes_[1]).astype(int)

        self._model.fit(
            X,
            y_train,
            epochs=self.epochs,
            batch_size=self.batch_size,
            validation_split=0.1,
            callbacks=callbacks,
            verbose=self.verbose,
        )

        return self

    def predict(self, X: NDArray[Any]) -> NDArray[Any]:
        """Predict class labels."""
        if self._model is None or self.classes_ is None:
            raise ValueError("Model must be fitted before prediction")

        proba = self._model.predict(X, verbose=0)

        if self._n_classes == 2:
            predictions = (proba.flatten() > 0.5).astype(int)
        else:
            predictions = np.argmax(proba, axis=1)

        return self.classes_[predictions]

    def predict_proba(self, X: NDArray[Any]) -> NDArray[Any]:
        """Predict class probabilities."""
        if self._model is None:
            raise ValueError("Model must be fitted before prediction")

        proba = self._model.predict(X, verbose=0)

        if self._n_classes == 2:
            proba = proba.flatten()
            return np.column_stack([1 - proba, proba])

        return proba


class KerasRegressor(BaseEstimator, RegressorMixin):
    """Sklearn-compatible wrapper for Keras regression models."""

    def __init__(
        self,
        hidden_layers: tuple[int, ...] = (64, 32),
        dropout_rate: float = 0.2,
        learning_rate: float = 0.001,
        epochs: int = 100,
        batch_size: int = 32,
        early_stopping_patience: int = 10,
        random_state: int = 42,
        verbose: int = 0,
    ) -> None:
        self.hidden_layers = hidden_layers
        self.dropout_rate = dropout_rate
        self.learning_rate = learning_rate
        self.epochs = epochs
        self.batch_size = batch_size
        self.early_stopping_patience = early_stopping_patience
        self.random_state = random_state
        self.verbose = verbose
        self._model: Any = None
        self._n_features: int = 0

    def _build_model(self, n_features: int) -> Any:
        """Build the Keras model."""
        if not _tensorflow_available or _keras is None:
            raise ImportError("TensorFlow is required for neural network models")

        _tf.random.set_seed(self.random_state)

        model = _keras.Sequential()
        model.add(_keras.layers.Input(shape=(n_features,)))

        for units in self.hidden_layers:
            model.add(_keras.layers.Dense(units, activation="relu"))
            model.add(_keras.layers.Dropout(self.dropout_rate))

        model.add(_keras.layers.Dense(1))

        optimizer = _keras.optimizers.Adam(learning_rate=self.learning_rate)
        model.compile(optimizer=optimizer, loss="mse", metrics=["mae"])

        return model

    def fit(self, X: NDArray[Any], y: NDArray[Any]) -> KerasRegressor:
        """Fit the model."""
        if not _tensorflow_available or _keras is None:
            raise ImportError("TensorFlow is required for neural network models")

        self._n_features = X.shape[1]
        self._model = self._build_model(self._n_features)

        callbacks = [
            _keras.callbacks.EarlyStopping(
                monitor="val_loss",
                patience=self.early_stopping_patience,
                restore_best_weights=True,
            )
        ]

        self._model.fit(
            X,
            y,
            epochs=self.epochs,
            batch_size=self.batch_size,
            validation_split=0.1,
            callbacks=callbacks,
            verbose=self.verbose,
        )

        return self

    def predict(self, X: NDArray[Any]) -> NDArray[Any]:
        """Predict values."""
        if self._model is None:
            raise ValueError("Model must be fitted before prediction")

        return self._model.predict(X, verbose=0).flatten()


def _suggest_neural_network(trial: optuna.Trial, _: ProblemType) -> dict[str, Any]:
    """Suggest hyperparameters for neural network."""
    n_layers = trial.suggest_int("n_layers", 1, 4)
    hidden_layers = tuple(trial.suggest_int(f"units_layer_{i}", 16, 256) for i in range(n_layers))

    return {
        "hidden_layers": hidden_layers,
        "dropout_rate": trial.suggest_float("dropout_rate", 0.0, 0.5),
        "learning_rate": trial.suggest_float("learning_rate", 1e-4, 1e-2, log=True),
        "batch_size": trial.suggest_categorical("batch_size", [16, 32, 64, 128]),
        "epochs": 100,
        "early_stopping_patience": 10,
    }


# Build model registry based on TensorFlow availability
NEURAL_MODELS: dict[str, dict[str, Any]] = {}

if _tensorflow_available:
    NEURAL_MODELS["neural_network"] = {
        "problem_types": ["classification", "regression"],
        "classification": KerasClassifier,
        "regression": KerasRegressor,
        "suggest_params": _suggest_neural_network,
        "default_params": {
            "hidden_layers": (64, 32),
            "dropout_rate": 0.2,
            "learning_rate": 0.001,
            "epochs": 100,
            "batch_size": 32,
        },
    }
