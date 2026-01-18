//! Trained model wrapper for inference and serialization.
//!
//! This module provides [`TrainedModel`], which wraps a Python ML model and enables:
//!
//! - **Single-instance prediction** via [`predict()`](TrainedModel::predict)
//! - **Batch prediction** via [`predict_batch()`](TrainedModel::predict_batch)
//! - **Serialization** via [`save()`](TrainedModel::save), [`load()`](TrainedModel::load),
//!   [`to_bytes()`](TrainedModel::to_bytes), and [`from_bytes()`](TrainedModel::from_bytes)
//! - **Introspection** via [`get_info()`](TrainedModel::get_info), [`metrics()`](TrainedModel::metrics),
//!   and property accessors
//!
//! # Lifecycle
//!
//! A `TrainedModel` is created in one of two ways:
//!
//! 1. **From training**: Call [`Pipeline::create_trained_model()`](crate::Pipeline::create_trained_model)
//!    after training completes
//! 2. **From disk**: Call [`TrainedModel::load()`] to load a previously saved model
//!
//! # Example
//!
//! ```rust,ignore
//! use lex_learning::{Pipeline, PipelineConfig, ProblemType, TrainedModel};
//! use polars::prelude::*;
//!
//! // After training...
//! let model = pipeline.create_trained_model()?;
//!
//! // Save for later use
//! model.save("model.pkl")?;
//!
//! // Make predictions
//! let prediction = model.predict(&serde_json::json!({
//!     "Age": 25,
//!     "Fare": 50.0
//! }))?;
//!
//! println!("Predicted: {:?}", prediction.prediction);
//! if let Some(probs) = &prediction.probabilities {
//!     println!("Probabilities: {:?}", probs);
//! }
//!
//! // Later, load and use
//! let loaded = TrainedModel::load("model.pkl")?;
//! let batch_results = loaded.predict_batch(&new_data)?;
//! ```
//!
//! # Thread Safety
//!
//! `TrainedModel` holds a `Py<PyAny>` which is `Send` but not `Sync`. The model can be
//! moved between threads but should not be shared across threads without synchronization.
//! All methods acquire the Python GIL internally.
//!
//! # Security
//!
//! Models are serialized using Python's pickle format. **Only load models from trusted
//! sources**, as pickle can execute arbitrary code during deserialization.

use crate::config::ProblemType;
use crate::error::LexLearningError;
use crate::python::conversion;
use crate::types::{Metrics, ModelInfo, PredictionResult};
use polars::prelude::DataFrame;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

/// A trained machine learning model ready for inference.
///
/// This struct wraps a Python `TrainedModel` instance and provides a Rust-native API
/// for making predictions, saving/loading models, and inspecting model properties.
///
/// The underlying Python model is held in memory via [`Py<PyAny>`], which allows
/// the model to persist across GIL acquisitions.
///
/// # Creating a TrainedModel
///
/// ```rust,ignore
/// // From training pipeline
/// let model = pipeline.create_trained_model()?;
///
/// // From saved file
/// let model = TrainedModel::load("model.pkl")?;
///
/// // From bytes (e.g., database storage)
/// let model = TrainedModel::from_bytes(&bytes)?;
/// ```
///
/// # Serialization Formats
///
/// | Method | Use Case |
/// |--------|----------|
/// | [`save()`](Self::save) / [`load()`](Self::load) | File-based persistence |
/// | [`to_bytes()`](Self::to_bytes) / [`from_bytes()`](Self::from_bytes) | Database or network transfer |
pub struct TrainedModel {
    /// The Python `TrainedModel` instance.
    ///
    /// This is a GIL-independent reference that can be moved between threads.
    /// Use `.bind(py)` to get a GIL-bound reference for calling methods.
    py_model: Py<PyAny>,
}

// Manual Debug implementation since Py<PyAny> doesn't provide useful debug info
impl fmt::Debug for TrainedModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Try to get model info if Python is initialized
        let info = Python::attach(|py| {
            let bound = self.py_model.bind(py);
            let model_name = bound
                .getattr("best_model_name")
                .and_then(|v| v.extract::<String>())
                .unwrap_or_else(|_| "<unknown>".to_string());
            let problem_type = bound
                .getattr("problem_type")
                .and_then(|v| v.getattr("value"))
                .and_then(|v| v.extract::<String>())
                .unwrap_or_else(|_| "<unknown>".to_string());
            (model_name, problem_type)
        });

        f.debug_struct("TrainedModel")
            .field("model_name", &info.0)
            .field("problem_type", &info.1)
            .finish()
    }
}

impl TrainedModel {
    /// Creates a `TrainedModel` from a Python object.
    ///
    /// This is an internal constructor used by [`Pipeline::create_trained_model()`](crate::Pipeline::create_trained_model).
    ///
    /// # Arguments
    ///
    /// * `py_model` - A Python `TrainedModel` instance wrapped in [`Py<PyAny>`].
    pub(crate) fn from_py_object(py_model: Py<PyAny>) -> Self {
        Self { py_model }
    }

    /// Loads a trained model from a pickle file.
    ///
    /// This deserializes a model that was previously saved with [`save()`](Self::save).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.pkl` file. Can be absolute or relative.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist ([`LexLearningError::ModelNotFound`])
    /// - The file cannot be read or is corrupted ([`LexLearningError::InferenceError`])
    /// - The file contains incompatible pickle data ([`LexLearningError::PythonError`])
    ///
    /// # Security
    ///
    /// **Only load models from trusted sources.** Pickle files can execute arbitrary
    /// code during deserialization. Loading a malicious pickle file could compromise
    /// your system.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let model = TrainedModel::load("models/classifier.pkl")?;
    /// println!("Loaded model: {}", model.best_model_name()?);
    /// ```
    #[must_use = "returns the loaded model; use it or handle the error"]
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LexLearningError> {
        let path = path.as_ref();

        // Check file exists first (Rust-side validation)
        if !path.exists() {
            return Err(LexLearningError::ModelNotFound {
                path: path.display().to_string(),
            });
        }

        let path_str = path.to_string_lossy().to_string();

        Python::attach(|py| {
            let lex_learning = py.import("lex_learning")?;
            let trained_model_class = lex_learning.getattr("TrainedModel")?;

            let py_model = trained_model_class
                .call_method1("load", (path_str,))
                .map_err(|e| conversion::map_python_error(py, e))?;

            Ok(TrainedModel::from_py_object(py_model.unbind()))
        })
    }

    /// Saves the model to a pickle file.
    ///
    /// The saved model can later be loaded with [`load()`](Self::load).
    ///
    /// # Arguments
    ///
    /// * `path` - Destination path for the `.pkl` file. Parent directories must exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The parent directory does not exist
    /// - The file cannot be written (permissions, disk full, etc.)
    /// - Serialization fails ([`LexLearningError::PythonError`])
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// model.save("models/classifier.pkl")?;
    /// model.save("/absolute/path/to/model.pkl")?;
    /// ```
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), LexLearningError> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        Python::attach(|py| {
            self.py_model
                .bind(py)
                .call_method1("save", (path_str,))
                .map_err(|e| conversion::map_python_error(py, e))?;
            Ok(())
        })
    }

    /// Exports the model as a byte vector.
    ///
    /// This serializes the entire model (including preprocessing artifacts and
    /// trained weights) to bytes using Python's pickle protocol. Useful for:
    ///
    /// - Storing models in databases
    /// - Sending models over the network
    /// - Embedding models in other binary formats
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails ([`LexLearningError::PythonError`]).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let bytes = model.to_bytes()?;
    /// println!("Model size: {} bytes", bytes.len());
    ///
    /// // Store in database, send over network, etc.
    /// database.store("model_v1", &bytes)?;
    /// ```
    #[must_use = "returns serialized model bytes; use them or handle the error"]
    pub fn to_bytes(&self) -> Result<Vec<u8>, LexLearningError> {
        Python::attach(|py| {
            let pickle = py.import("pickle")?;
            let py_bytes = pickle
                .call_method1("dumps", (self.py_model.bind(py),))
                .map_err(|e| conversion::map_python_error(py, e))?;

            let bytes: Vec<u8> = py_bytes.extract()?;
            Ok(bytes)
        })
    }

    /// Loads a model from a byte slice.
    ///
    /// This deserializes a model that was previously serialized with [`to_bytes()`](Self::to_bytes).
    ///
    /// # Arguments
    ///
    /// * `bytes` - The serialized model bytes from [`to_bytes()`](Self::to_bytes).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The bytes are not valid pickle data ([`LexLearningError::PythonError`])
    /// - The pickle protocol version is incompatible
    /// - The deserialized object is not a valid `TrainedModel`
    ///
    /// # Security
    ///
    /// **Only load bytes from trusted sources.** Pickle can execute arbitrary code
    /// during deserialization.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Retrieve bytes from database
    /// let bytes = database.get("model_v1")?;
    /// let model = TrainedModel::from_bytes(&bytes)?;
    ///
    /// // Use for inference
    /// let prediction = model.predict(&instance)?;
    /// ```
    #[must_use = "returns the loaded model; use it or handle the error"]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LexLearningError> {
        Python::attach(|py| {
            let pickle = py.import("pickle")?;
            let py_bytes = pyo3::types::PyBytes::new(py, bytes);

            let py_model = pickle
                .call_method1("loads", (py_bytes,))
                .map_err(|e| conversion::map_python_error(py, e))?;

            Ok(TrainedModel::from_py_object(py_model.unbind()))
        })
    }

    /// Makes a prediction for a single instance.
    ///
    /// This is the primary method for real-time inference on individual data points.
    /// For batch predictions, use [`predict_batch()`](Self::predict_batch) instead.
    ///
    /// # Arguments
    ///
    /// * `instance` - A JSON object where keys are feature names and values are
    ///   feature values. The feature names must match those used during training.
    ///
    /// # Returns
    ///
    /// A [`PredictionResult`] containing:
    /// - `prediction`: The predicted value (class label or numeric value)
    /// - `probabilities`: Class probabilities (classification only)
    /// - `confidence`: Prediction confidence (classification only)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required features are missing from the instance
    /// - Feature values have incompatible types
    /// - The model fails to make a prediction
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use serde_json::json;
    ///
    /// // Classification example
    /// let prediction = model.predict(&json!({
    ///     "Age": 25,
    ///     "Sex": "male",
    ///     "Fare": 7.25
    /// }))?;
    ///
    /// println!("Predicted class: {}", prediction.prediction);
    /// if let Some(probs) = &prediction.probabilities {
    ///     println!("Probabilities: {:?}", probs);
    /// }
    /// if let Some(conf) = prediction.confidence {
    ///     println!("Confidence: {:.2}%", conf * 100.0);
    /// }
    ///
    /// // Regression example
    /// let prediction = model.predict(&json!({
    ///     "sqft": 2000,
    ///     "bedrooms": 3,
    ///     "location": "downtown"
    /// }))?;
    /// println!("Predicted price: ${}", prediction.prediction);
    /// ```
    #[must_use = "returns the prediction result; use it or handle the error"]
    pub fn predict(
        &self,
        instance: &serde_json::Value,
    ) -> Result<PredictionResult, LexLearningError> {
        Python::attach(|py| {
            // Convert JSON to Python dict
            let py_dict = conversion::json_to_pydict(py, instance)?;

            // Call predict on Python model
            let py_result = self
                .py_model
                .bind(py)
                .call_method1("predict", (py_dict,))
                .map_err(|e| conversion::map_python_error(py, e))?;

            // Extract the prediction result
            conversion::extract_prediction_result(py, &py_result)
        })
    }

    /// Makes predictions for multiple instances in a batch.
    ///
    /// This is more efficient than calling [`predict()`](Self::predict) in a loop
    /// when you have many instances to score.
    ///
    /// # Arguments
    ///
    /// * `df` - A Polars [`DataFrame`] containing the instances to predict.
    ///   Column names must match the feature names used during training.
    ///   The target column should **not** be included.
    ///
    /// # Returns
    ///
    /// A new [`DataFrame`] with prediction columns added:
    /// - `prediction`: The predicted values
    /// - `probability_<class>`: Per-class probabilities (classification only)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required feature columns are missing
    /// - Column types are incompatible
    /// - DataFrame conversion fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use polars::prelude::*;
    ///
    /// let new_data = df! {
    ///     "Age" => &[25, 30, 35],
    ///     "Fare" => &[50.0, 75.0, 100.0],
    ///     "Sex" => &["male", "female", "male"],
    /// }?;
    ///
    /// let results = model.predict_batch(&new_data)?;
    /// println!("Predictions:\n{}", results);
    /// ```
    #[must_use = "returns DataFrame with predictions; use it or handle the error"]
    pub fn predict_batch(&self, df: &DataFrame) -> Result<DataFrame, LexLearningError> {
        Python::attach(|py| {
            // Convert Polars DataFrame to pandas DataFrame
            let pandas_df = conversion::dataframe_to_python(py, df)?;

            // Call predict_batch on Python model
            let py_result = self
                .py_model
                .bind(py)
                .call_method1("predict_batch", (&pandas_df,))
                .map_err(|e| conversion::map_python_error(py, e))?;

            // Convert back to Polars DataFrame
            conversion::python_to_dataframe(py, &py_result)
        })
    }

    /// Returns the problem type this model was trained for.
    ///
    /// # Returns
    ///
    /// - [`ProblemType::Classification`] for classifiers
    /// - [`ProblemType::Regression`] for regressors
    ///
    /// # Errors
    ///
    /// Returns an error if the Python model's problem type attribute is invalid.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// match model.problem_type()? {
    ///     ProblemType::Classification => println!("This is a classifier"),
    ///     ProblemType::Regression => println!("This is a regressor"),
    /// }
    /// ```
    #[must_use = "returns the problem type; use it or handle the error"]
    pub fn problem_type(&self) -> Result<ProblemType, LexLearningError> {
        Python::attach(|py| {
            let py_problem_type = self.py_model.bind(py).getattr("problem_type")?;
            let value: String = py_problem_type.getattr("value")?.extract()?;

            match value.as_str() {
                "classification" => Ok(ProblemType::Classification),
                "regression" => Ok(ProblemType::Regression),
                other => Err(LexLearningError::InvalidConfig(format!(
                    "Unknown problem type: '{}'. Expected 'classification' or 'regression'.",
                    other
                ))),
            }
        })
    }

    /// Returns the name of the target column.
    ///
    /// This is the column that the model predicts.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let target = model.target_column()?;
    /// println!("This model predicts: {}", target);
    /// ```
    #[must_use = "returns the target column name; use it or handle the error"]
    pub fn target_column(&self) -> Result<String, LexLearningError> {
        Python::attach(|py| {
            let value: String = self.py_model.bind(py).getattr("target_column")?.extract()?;
            Ok(value)
        })
    }

    /// Returns the names of the feature columns used by the model.
    ///
    /// These are the columns that must be present when making predictions.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let features = model.feature_names()?;
    /// println!("Required features: {:?}", features);
    ///
    /// // Verify input data has required columns
    /// for feature in &features {
    ///     assert!(input_df.column(feature).is_ok(), "Missing feature: {}", feature);
    /// }
    /// ```
    #[must_use = "returns feature names; use them or handle the error"]
    pub fn feature_names(&self) -> Result<Vec<String>, LexLearningError> {
        Python::attach(|py| {
            let value: Vec<String> = self.py_model.bind(py).getattr("feature_names")?.extract()?;
            Ok(value)
        })
    }

    /// Returns the class labels for classification models.
    ///
    /// # Returns
    ///
    /// - `Some(labels)` for classification models, where `labels` are the unique
    ///   class names in the order used by the model
    /// - `None` for regression models
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(labels) = model.class_labels()? {
    ///     println!("Classes: {:?}", labels);
    ///     // e.g., ["No", "Yes"] or ["cat", "dog", "bird"]
    /// }
    /// ```
    #[must_use = "returns class labels; use them or handle the error"]
    pub fn class_labels(&self) -> Result<Option<Vec<String>>, LexLearningError> {
        Python::attach(|py| {
            let py_value = self.py_model.bind(py).getattr("class_labels")?;
            if py_value.is_none() {
                Ok(None)
            } else {
                let value: Vec<String> = py_value.extract()?;
                Ok(Some(value))
            }
        })
    }

    /// Returns the name of the best-performing algorithm.
    ///
    /// This is the algorithm that was selected during training based on
    /// cross-validation performance.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let name = model.best_model_name()?;
    /// println!("Best algorithm: {}", name);
    /// // e.g., "xgboost", "random_forest", "lightgbm"
    /// ```
    #[must_use = "returns model name; use it or handle the error"]
    pub fn best_model_name(&self) -> Result<String, LexLearningError> {
        Python::attach(|py| {
            let value: String = self
                .py_model
                .bind(py)
                .getattr("best_model_name")?
                .extract()?;
            Ok(value)
        })
    }

    /// Returns the performance metrics from training.
    ///
    /// The available metrics depend on the problem type:
    ///
    /// **Classification metrics:**
    /// - `accuracy`, `precision`, `recall`, `f1_score`, `roc_auc`
    ///
    /// **Regression metrics:**
    /// - `mse`, `rmse`, `mae`, `r2`
    ///
    /// **Common metrics (both):**
    /// - `cv_score`, `test_score`, `train_score`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let metrics = model.metrics()?;
    ///
    /// // Classification
    /// if let Some(acc) = metrics.accuracy {
    ///     println!("Accuracy: {:.2}%", acc * 100.0);
    /// }
    ///
    /// // Regression
    /// if let Some(r2) = metrics.r2 {
    ///     println!("R² score: {:.4}", r2);
    /// }
    /// ```
    #[must_use = "returns metrics; use them or handle the error"]
    pub fn metrics(&self) -> Result<Metrics, LexLearningError> {
        Python::attach(|py| {
            let py_metrics = self.py_model.bind(py).getattr("metrics")?;
            conversion::extract_metrics(&py_metrics)
        })
    }

    /// Returns the feature importance scores.
    ///
    /// Feature importance indicates how much each feature contributes to the
    /// model's predictions. Higher values indicate more important features.
    ///
    /// # Returns
    ///
    /// A vector of `(feature_name, importance_score)` tuples, sorted by
    /// importance in descending order.
    ///
    /// # Note
    ///
    /// Feature importance is only available if explainability was enabled
    /// during training. Returns an empty vector if unavailable.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let importance = model.feature_importance()?;
    /// println!("Top 5 most important features:");
    /// for (name, score) in importance.iter().take(5) {
    ///     println!("  {}: {:.4}", name, score);
    /// }
    /// ```
    #[must_use = "returns feature importance; use it or handle the error"]
    pub fn feature_importance(&self) -> Result<Vec<(String, f64)>, LexLearningError> {
        Python::attach(|py| {
            let value: Vec<(String, f64)> = self
                .py_model
                .bind(py)
                .getattr("feature_importance")?
                .extract()?;
            Ok(value)
        })
    }

    /// Returns comprehensive information about the model.
    ///
    /// This is a convenience method that aggregates all model metadata into
    /// a single [`ModelInfo`] struct. Useful for logging, display, or
    /// serialization.
    ///
    /// # Returns
    ///
    /// A [`ModelInfo`] struct containing:
    /// - Model algorithm name
    /// - Problem type
    /// - Target column
    /// - Feature names
    /// - Class labels (classification only)
    /// - Performance metrics
    /// - Hyperparameters
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let info = model.get_info()?;
    /// println!("Model: {} ({})", info.model_name, info.problem_type);
    /// println!("Target: {}", info.target_column);
    /// println!("Features: {:?}", info.feature_names);
    /// println!("Hyperparameters: {:?}", info.hyperparameters);
    /// ```
    #[must_use = "returns model info; use it or handle the error"]
    pub fn get_info(&self) -> Result<ModelInfo, LexLearningError> {
        Python::attach(|py| {
            let py_info = self
                .py_model
                .bind(py)
                .call_method0("get_info")
                .map_err(|e| conversion::map_python_error(py, e))?;

            // Extract fields from the dict
            let model_name: String = py_info.get_item("best_model_name")?.extract()?;
            let problem_type: String = py_info.get_item("problem_type")?.extract()?;
            let target_column: String = py_info.get_item("target_column")?.extract()?;
            let feature_names: Vec<String> = py_info.get_item("feature_names")?.extract()?;

            // class_labels might be None
            let class_labels_item = py_info.get_item("class_labels")?;
            let class_labels: Option<Vec<String>> = if class_labels_item.is_none() {
                None
            } else {
                Some(class_labels_item.extract()?)
            };

            // Extract metrics dict and convert to Metrics struct
            let py_metrics = py_info.get_item("metrics")?;
            let metrics = extract_metrics_from_dict(&py_metrics)?;

            // Extract hyperparameters
            let py_hyperparams = py_info.get_item("hyperparameters")?;
            let hyperparameters = if py_hyperparams.is_none() {
                HashMap::new()
            } else {
                extract_hyperparameters_from_dict(py, &py_hyperparams)?
            };

            Ok(ModelInfo {
                model_name,
                problem_type,
                target_column,
                feature_names,
                class_labels,
                metrics,
                hyperparameters,
            })
        })
    }
}

/// Extracts [`Metrics`] from a Python dictionary.
///
/// This is used when deserializing model info from Python's `get_info()` method,
/// which returns metrics as a plain dict rather than a Metrics object.
///
/// # Arguments
///
/// * `py_dict` - A Python dictionary containing metric key-value pairs.
///
/// # Returns
///
/// A [`Metrics`] struct with optional fields populated from the dictionary.
/// Missing keys result in `None` for the corresponding field.
fn extract_metrics_from_dict(py_dict: &Bound<'_, PyAny>) -> Result<Metrics, LexLearningError> {
    // Helper closure to safely extract optional f64 values
    let get_opt_f64 = |key: &str| -> Option<f64> {
        py_dict
            .get_item(key)
            .ok()
            .and_then(|v| if v.is_none() { None } else { v.extract().ok() })
    };

    Ok(Metrics {
        cv_score: get_opt_f64("cv_score"),
        test_score: get_opt_f64("test_score"),
        train_score: get_opt_f64("train_score"),
        accuracy: get_opt_f64("accuracy"),
        precision: get_opt_f64("precision"),
        recall: get_opt_f64("recall"),
        f1_score: get_opt_f64("f1_score"),
        roc_auc: get_opt_f64("roc_auc"),
        mse: get_opt_f64("mse"),
        rmse: get_opt_f64("rmse"),
        mae: get_opt_f64("mae"),
        r2: get_opt_f64("r2"),
    })
}

/// Extracts hyperparameters from a Python dictionary into a JSON-compatible map.
///
/// Hyperparameters are stored as arbitrary key-value pairs where values can be
/// strings, numbers, booleans, lists, or nested dicts.
///
/// # Arguments
///
/// * `py` - Python GIL token.
/// * `py_dict` - A Python dictionary containing hyperparameter key-value pairs.
///
/// # Returns
///
/// A `HashMap` where keys are parameter names and values are JSON-compatible values.
fn extract_hyperparameters_from_dict(
    py: Python<'_>,
    py_dict: &Bound<'_, PyAny>,
) -> Result<HashMap<String, serde_json::Value>, LexLearningError> {
    let mut map = HashMap::new();

    let items = py_dict.call_method0("items")?;
    for item in items.try_iter()? {
        let item: Bound<'_, PyAny> = item?;
        let key: String = item.get_item(0)?.extract()?;
        let value = py_any_to_json_value(py, &item.get_item(1)?)?;
        map.insert(key, value);
    }

    Ok(map)
}

/// Converts a Python value to a [`serde_json::Value`].
///
/// This handles the common Python types that appear in hyperparameters:
/// - `None` → `Value::Null`
/// - `bool` → `Value::Bool`
/// - `int` → `Value::Number`
/// - `float` → `Value::Number`
/// - `str` → `Value::String`
/// - `list` → `Value::Array`
/// - `dict` → `Value::Object`
/// - Other types → `Value::String` (using `str()` representation)
///
/// # Arguments
///
/// * `_py` - Python GIL token (unused but required for context).
/// * `obj` - The Python object to convert.
///
/// # Note
///
/// Boolean extraction is attempted before integer extraction because in Python,
/// `bool` is a subclass of `int`, so `True` would extract as `1` if we checked
/// int first.
fn py_any_to_json_value(
    _py: Python<'_>,
    obj: &Bound<'_, PyAny>,
) -> Result<serde_json::Value, LexLearningError> {
    if obj.is_none() {
        return Ok(serde_json::Value::Null);
    }

    // Try bool first (before int, since bool is a subtype of int in Python)
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(serde_json::Value::Bool(b));
    }

    if let Ok(i) = obj.extract::<i64>() {
        return Ok(serde_json::Value::Number(i.into()));
    }

    if let Ok(f) = obj.extract::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Ok(serde_json::Value::Number(n));
        }
        return Ok(serde_json::Value::Null);
    }

    if let Ok(s) = obj.extract::<String>() {
        return Ok(serde_json::Value::String(s));
    }

    if obj.is_instance_of::<pyo3::types::PyList>() {
        let mut arr = Vec::new();
        for item in obj.try_iter()? {
            let item: Bound<'_, PyAny> = item?;
            arr.push(py_any_to_json_value(_py, &item)?);
        }
        return Ok(serde_json::Value::Array(arr));
    }

    if obj.is_instance_of::<pyo3::types::PyDict>() {
        let mut map = serde_json::Map::new();
        let items = obj.call_method0("items")?;
        for item in items.try_iter()? {
            let item: Bound<'_, PyAny> = item?;
            let key: String = item.get_item(0)?.extract()?;
            let value = py_any_to_json_value(_py, &item.get_item(1)?)?;
            map.insert(key, value);
        }
        return Ok(serde_json::Value::Object(map));
    }

    // Fallback: convert to string representation
    let repr: String = obj.str()?.extract()?;
    Ok(serde_json::Value::String(repr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_nonexistent_file() {
        // Should return ModelNotFound error without calling Python
        let result = TrainedModel::load("/nonexistent/path/model.pkl");
        assert!(matches!(
            result,
            Err(LexLearningError::ModelNotFound { .. })
        ));
    }

    #[test]
    #[ignore = "Requires Python runtime with ML libraries"]
    fn test_create_trained_model_and_predict() {
        use crate::config::{PipelineConfig, ProblemType};
        use crate::pipeline::Pipeline;
        use polars::prelude::*;

        crate::initialize().expect("Failed to initialize Python");

        // Create a simple classification dataset
        let df = df! {
            "Age" => &[22i64, 38, 26, 35, 28, 19, 40, 66, 28, 42],
            "Fare" => &[7.25f64, 71.28, 7.92, 53.10, 8.05, 8.46, 27.72, 10.50, 7.23, 52.00],
            "Pclass" => &[3i64, 1, 3, 1, 3, 3, 1, 3, 3, 1],
            "Survived" => &[0i64, 1, 1, 1, 0, 0, 0, 0, 1, 1],
        }
        .unwrap();

        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Classification)
            .target_column("Survived")
            .optimize_hyperparams(false)
            .enable_explainability(false)
            .enable_neural_networks(false)
            .top_k_algorithms(1)
            .cv_folds(2)
            .build()
            .unwrap();

        let mut pipeline = Pipeline::builder().config(config).build().unwrap();
        let _result = pipeline.train(&df).expect("Training should succeed");

        // Create trained model
        let model = pipeline
            .create_trained_model()
            .expect("Should create trained model");

        // Test property accessors
        assert_eq!(model.problem_type().unwrap(), ProblemType::Classification);
        assert_eq!(model.target_column().unwrap(), "Survived");
        assert!(!model.feature_names().unwrap().is_empty());
        assert!(model.class_labels().unwrap().is_some());
        assert!(!model.best_model_name().unwrap().is_empty());

        // Test metrics
        let metrics = model.metrics().unwrap();
        assert!(metrics.accuracy.is_some());

        // Test feature importance (may be empty if explainability is disabled)
        let _importance = model.feature_importance().unwrap();
        // Not asserting non-empty since explainability is disabled in test

        // Test get_info
        let info = model.get_info().unwrap();
        assert_eq!(info.target_column, "Survived");
        assert_eq!(info.problem_type, "classification");

        // Test single prediction
        let prediction = model
            .predict(&serde_json::json!({
                "Age": 25,
                "Fare": 50.0,
                "Pclass": 1
            }))
            .expect("Prediction should succeed");

        assert!(!prediction.prediction.is_null());
        // Classification should have probabilities
        assert!(prediction.probabilities.is_some());
        assert!(prediction.confidence.is_some());
    }

    #[test]
    #[ignore = "Requires Python runtime with ML libraries"]
    fn test_predict_batch() {
        use crate::config::{PipelineConfig, ProblemType};
        use crate::pipeline::Pipeline;
        use polars::prelude::*;

        crate::initialize().expect("Failed to initialize Python");

        // Create a simple regression dataset
        let df = df! {
            "feature1" => &[1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
            "feature2" => &[2.0f64, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0],
            "target" => &[3.0f64, 6.0, 9.0, 12.0, 15.0, 18.0, 21.0, 24.0, 27.0, 30.0],
        }
        .unwrap();

        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Regression)
            .target_column("target")
            .optimize_hyperparams(false)
            .enable_explainability(false)
            .enable_neural_networks(false)
            .top_k_algorithms(1)
            .cv_folds(2)
            .build()
            .unwrap();

        let mut pipeline = Pipeline::builder().config(config).build().unwrap();
        let _result = pipeline.train(&df).expect("Training should succeed");

        let model = pipeline
            .create_trained_model()
            .expect("Should create trained model");

        // Create new data for batch prediction
        let new_data = df! {
            "feature1" => &[11.0f64, 12.0, 13.0],
            "feature2" => &[22.0f64, 24.0, 26.0],
        }
        .unwrap();

        let predictions = model
            .predict_batch(&new_data)
            .expect("Batch prediction should succeed");

        // Should have a prediction column added
        assert!(predictions.column("prediction").is_ok());
        assert_eq!(predictions.height(), 3);
    }

    #[test]
    #[ignore = "Requires Python runtime with ML libraries"]
    fn test_save_and_load() {
        use crate::config::{PipelineConfig, ProblemType};
        use crate::pipeline::Pipeline;
        use polars::prelude::*;
        use std::fs;

        crate::initialize().expect("Failed to initialize Python");

        // Create and train
        let df = df! {
            "feature1" => &[1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
            "feature2" => &[2.0f64, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0],
            "target" => &[3.0f64, 6.0, 9.0, 12.0, 15.0, 18.0, 21.0, 24.0, 27.0, 30.0],
        }
        .unwrap();

        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Regression)
            .target_column("target")
            .optimize_hyperparams(false)
            .enable_explainability(false)
            .enable_neural_networks(false)
            .top_k_algorithms(1)
            .cv_folds(2)
            .build()
            .unwrap();

        let mut pipeline = Pipeline::builder().config(config).build().unwrap();
        let _result = pipeline.train(&df).expect("Training should succeed");

        let model = pipeline
            .create_trained_model()
            .expect("Should create trained model");

        // Save
        let temp_path = "/tmp/test_model.pkl";
        model.save(temp_path).expect("Save should succeed");

        // Verify file exists
        assert!(std::path::Path::new(temp_path).exists());

        // Load
        let loaded_model = TrainedModel::load(temp_path).expect("Load should succeed");

        // Verify properties match
        assert_eq!(
            loaded_model.problem_type().unwrap(),
            model.problem_type().unwrap()
        );
        assert_eq!(
            loaded_model.target_column().unwrap(),
            model.target_column().unwrap()
        );
        assert_eq!(
            loaded_model.feature_names().unwrap(),
            model.feature_names().unwrap()
        );

        // Cleanup
        fs::remove_file(temp_path).ok();
    }

    #[test]
    #[ignore = "Requires Python runtime with ML libraries"]
    fn test_to_bytes_and_from_bytes() {
        use crate::config::{PipelineConfig, ProblemType};
        use crate::pipeline::Pipeline;
        use polars::prelude::*;

        crate::initialize().expect("Failed to initialize Python");

        // Create and train
        let df = df! {
            "feature1" => &[1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
            "feature2" => &[2.0f64, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0],
            "target" => &[3.0f64, 6.0, 9.0, 12.0, 15.0, 18.0, 21.0, 24.0, 27.0, 30.0],
        }
        .unwrap();

        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Regression)
            .target_column("target")
            .optimize_hyperparams(false)
            .enable_explainability(false)
            .enable_neural_networks(false)
            .top_k_algorithms(1)
            .cv_folds(2)
            .build()
            .unwrap();

        let mut pipeline = Pipeline::builder().config(config).build().unwrap();
        let _result = pipeline.train(&df).expect("Training should succeed");

        let model = pipeline
            .create_trained_model()
            .expect("Should create trained model");

        // Serialize to bytes
        let bytes = model.to_bytes().expect("to_bytes should succeed");
        assert!(!bytes.is_empty());

        // Deserialize from bytes
        let restored_model = TrainedModel::from_bytes(&bytes).expect("from_bytes should succeed");

        // Verify properties match
        assert_eq!(
            restored_model.problem_type().unwrap(),
            model.problem_type().unwrap()
        );
        assert_eq!(
            restored_model.target_column().unwrap(),
            model.target_column().unwrap()
        );

        // Verify prediction works
        let prediction = restored_model
            .predict(&serde_json::json!({
                "feature1": 11.0,
                "feature2": 22.0
            }))
            .expect("Prediction should succeed");

        assert!(!prediction.prediction.is_null());
    }
}
