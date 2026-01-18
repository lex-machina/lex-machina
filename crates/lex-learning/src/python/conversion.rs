//! DataFrame and type conversion between Rust and Python.
//!
//! This module provides the core data interchange layer between Rust (Polars) and
//! Python (pandas) using Apache Arrow IPC format for efficient, type-preserving
//! transfer. It also handles conversion of configuration, results, and predictions.
//!
//! # Architecture
//!
//! The module is organized into several conversion categories:
//!
//! - **DataFrame conversion**: Bidirectional transfer between Polars and pandas
//! - **Configuration conversion**: Rust `PipelineConfig` → Python `PipelineConfig`
//! - **Result extraction**: Python training results → Rust types
//! - **Prediction handling**: JSON ↔ Python dict, prediction result extraction
//! - **Error mapping**: Python exceptions → Rust error variants
//!
//! # Arrow IPC Strategy
//!
//! DataFrames are transferred using Arrow IPC **File format** (not Stream format)
//! for these reasons:
//!
//! 1. **Polars compatibility**: Polars' `IpcWriter`/`IpcReader` use File format by default
//! 2. **Schema preservation**: File format includes the full schema in the footer
//! 3. **Type fidelity**: All supported types round-trip correctly:
//!    - `Int32`, `Int64`, `Float32`, `Float64` (numeric)
//!    - `String`/`Utf8` (text)
//!    - `Boolean` (logical)
//!
//! ## Data Flow: Rust → Python
//!
//! ```text
//! Polars DataFrame
//!       │
//!       ▼ (clone - IpcWriter needs &mut)
//! &mut DataFrame
//!       │
//!       ▼ IpcWriter::new(Cursor<Vec<u8>>).finish()
//! Arrow IPC bytes (Vec<u8>)
//!       │
//!       ▼ PyBytes::new(py, &bytes)
//! Python bytes
//!       │
//!       ▼ io.BytesIO(bytes)
//!       ▼ pyarrow.ipc.open_file(buffer).read_all()
//! PyArrow Table
//!       │
//!       ▼ table.to_pandas()
//! pandas DataFrame
//! ```
//!
//! ## Data Flow: Python → Rust
//!
//! ```text
//! pandas DataFrame
//!       │
//!       ▼ pyarrow.Table.from_pandas(df)
//! PyArrow Table
//!       │
//!       ▼ pyarrow.ipc.RecordBatchFileWriter(sink, schema)
//!       ▼ writer.write_table(table)
//!       ▼ writer.close()
//!       ▼ sink.getvalue()
//! Python bytes
//!       │
//!       ▼ PyBytes::as_bytes() → Vec<u8>
//! Arrow IPC bytes
//!       │
//!       ▼ IpcReader::new(Cursor::new(bytes)).finish()
//! Polars DataFrame
//! ```
//!
//! # Why Clone?
//!
//! The `dataframe_to_python` function clones the input DataFrame because Polars'
//! `IpcWriter::finish()` requires `&mut DataFrame`. This is a conscious API design
//! choice to keep the public interface clean (`&DataFrame`) at the cost of one
//! clone operation. For most ML workloads, this overhead is negligible compared
//! to training time.
//!
//! # Error Handling
//!
//! All conversion functions return `Result<T, LexLearningError>`. Errors are
//! categorized as:
//!
//! - [`ArrowConversionKind::Serialize`]: Failed to write Polars DataFrame to IPC
//! - [`ArrowConversionKind::Deserialize`]: Failed to read IPC bytes into Polars
//! - [`ArrowConversionKind::TypeConversion`]: Type extraction or casting failed
//! - [`LexLearningError::PythonError`]: General Python exception
//! - [`LexLearningError::InvalidData`]: Invalid input data format
//!
//! Python exceptions are mapped to specific Rust error variants via
//! [`map_python_error`], preserving error semantics across the language boundary.
//!
//! # Example
//!
//! ```ignore
//! use polars::prelude::*;
//! use pyo3::Python;
//!
//! // Rust → Python
//! let df = df! { "x" => [1, 2, 3] }.unwrap();
//! Python::attach(|py| {
//!     let pandas_df = dataframe_to_python(py, &df)?;
//!     // Use pandas_df in Python...
//!     Ok(())
//! });
//!
//! // Python → Rust  
//! Python::attach(|py| {
//!     let pandas_df = /* get from Python */;
//!     let polars_df = python_to_dataframe(py, &pandas_df)?;
//!     assert_eq!(polars_df.height(), 3);
//!     Ok(())
//! });
//! ```
//!
//! [`ArrowConversionKind::Serialize`]: crate::error::ArrowConversionKind::Serialize
//! [`ArrowConversionKind::Deserialize`]: crate::error::ArrowConversionKind::Deserialize
//! [`ArrowConversionKind::TypeConversion`]: crate::error::ArrowConversionKind::TypeConversion
//! [`LexLearningError::PythonError`]: crate::error::LexLearningError::PythonError
//! [`LexLearningError::InvalidData`]: crate::error::LexLearningError::InvalidData

use crate::config::{PipelineConfig, ProblemType};
use crate::error::{ArrowConversionKind, LexLearningError};
use crate::types::{Metrics, ModelComparison, PredictionResult, TrainingResult};
use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyBytes, PyBytesMethods, PyDict, PyList};
use std::collections::HashMap;
use std::io::Cursor;

/// Convert a Rust Polars DataFrame to a Python pandas DataFrame.
///
/// Uses Arrow IPC file format for efficient transfer. The DataFrame is
/// serialized to Arrow IPC bytes, then read in Python using PyArrow
/// and converted to a pandas DataFrame.
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `df` - The Polars DataFrame to convert
///
/// # Returns
///
/// A Python pandas DataFrame object.
///
/// # Errors
///
/// Returns an error if:
/// - Arrow IPC serialization fails ([`ArrowConversionKind::Serialize`])
/// - Python module import fails (pyarrow, io)
/// - PyArrow IPC reading fails
/// - Conversion to pandas fails
///
/// # Data Flow
///
/// ```text
/// Polars DataFrame
///       │
///       ▼ (clone - IpcWriter needs &mut)
/// &mut DataFrame
///       │
///       ▼ IpcWriter::new(Cursor<Vec<u8>>).finish()
/// Arrow IPC bytes (Vec<u8>)
///       │
///       ▼ PyBytes::new(py, &bytes)
/// Python bytes
///       │
///       ▼ io.BytesIO(bytes)
///       ▼ pyarrow.ipc.open_file(buffer).read_all()
/// PyArrow Table
///       │
///       ▼ table.to_pandas()
/// pandas DataFrame
/// ```
///
/// [`ArrowConversionKind::Serialize`]: crate::error::ArrowConversionKind::Serialize
#[must_use = "the converted pandas DataFrame should be used"]
pub fn dataframe_to_python<'py>(
    py: Python<'py>,
    df: &DataFrame,
) -> Result<Bound<'py, PyAny>, LexLearningError> {
    // Clone the DataFrame since IpcWriter::finish() needs &mut DataFrame
    let mut df_clone = df.clone();

    // Write Polars DataFrame to Arrow IPC bytes
    let mut cursor = Cursor::new(Vec::new());
    IpcWriter::new(&mut cursor)
        .finish(&mut df_clone)
        .map_err(|e| ArrowConversionKind::Serialize(e.to_string()))?;

    let ipc_bytes = cursor.into_inner();

    // Create Python bytes from the IPC data
    let py_bytes = PyBytes::new(py, &ipc_bytes);

    // Import required Python modules
    let io = py.import("io")?;
    let pyarrow_ipc = py.import("pyarrow.ipc")?;

    // Create BytesIO buffer from bytes
    let buffer = io.call_method1("BytesIO", (py_bytes,))?;

    // Read Arrow IPC file and convert to pandas
    let reader = pyarrow_ipc.call_method1("open_file", (buffer,))?;
    let table = reader.call_method0("read_all")?;
    let pandas_df = table.call_method0("to_pandas")?;

    Ok(pandas_df)
}

/// Convert a Python pandas DataFrame to a Rust Polars DataFrame.
///
/// Uses Arrow IPC file format for efficient transfer. The pandas DataFrame
/// is converted to a PyArrow Table, serialized to Arrow IPC bytes, then
/// read into a Polars DataFrame.
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `py_df` - The Python pandas DataFrame object
///
/// # Returns
///
/// A Polars DataFrame.
///
/// # Errors
///
/// Returns an error if:
/// - Python module import fails (pyarrow, io)
/// - Conversion from pandas to PyArrow Table fails
/// - Arrow IPC writing fails in Python
/// - Byte extraction fails ([`ArrowConversionKind::TypeConversion`])
/// - Arrow IPC deserialization fails ([`ArrowConversionKind::Deserialize`])
///
/// # Data Flow
///
/// ```text
/// pandas DataFrame
///       │
///       ▼ pyarrow.Table.from_pandas(df)
/// PyArrow Table
///       │
///       ▼ pyarrow.ipc.RecordBatchFileWriter(sink, schema)
///       ▼ writer.write_table(table)
///       ▼ writer.close()
///       ▼ sink.getvalue()
/// Python bytes
///       │
///       ▼ PyBytes::as_bytes() → Vec<u8>
/// Arrow IPC bytes
///       │
///       ▼ IpcReader::new(Cursor::new(bytes)).finish()
/// Polars DataFrame
/// ```
///
/// [`ArrowConversionKind::TypeConversion`]: crate::error::ArrowConversionKind::TypeConversion
/// [`ArrowConversionKind::Deserialize`]: crate::error::ArrowConversionKind::Deserialize
#[must_use = "the converted Polars DataFrame should be used"]
pub fn python_to_dataframe(
    py: Python<'_>,
    py_df: &Bound<'_, PyAny>,
) -> Result<DataFrame, LexLearningError> {
    // Import required Python modules
    let io = py.import("io")?;
    let pyarrow = py.import("pyarrow")?;
    let pyarrow_ipc = py.import("pyarrow.ipc")?;

    // Convert pandas DataFrame to PyArrow Table
    // pyarrow.Table is a class, so we need to get it first, then call from_pandas
    let table_class = pyarrow.getattr("Table")?;
    let table = table_class.call_method1("from_pandas", (py_df,))?;

    // Get the schema from the table
    let schema = table.getattr("schema")?;

    // Create a BytesIO sink for writing
    let sink = io.call_method0("BytesIO")?;

    // Create RecordBatchFileWriter and write the table
    let writer = pyarrow_ipc.call_method1("RecordBatchFileWriter", (&sink, &schema))?;
    writer.call_method1("write_table", (&table,))?;
    writer.call_method0("close")?;

    // Get the bytes from the sink
    let py_bytes = sink.call_method0("getvalue")?;

    // Extract bytes to Rust Vec<u8>
    let py_bytes_bound: &Bound<'_, PyBytes> = py_bytes.cast().map_err(|e| {
        ArrowConversionKind::TypeConversion(format!("Failed to extract bytes: {}", e))
    })?;
    let ipc_bytes = py_bytes_bound.as_bytes().to_vec();

    // Read Arrow IPC bytes into Polars DataFrame
    let cursor = Cursor::new(ipc_bytes);
    let df = IpcReader::new(cursor)
        .finish()
        .map_err(|e| ArrowConversionKind::Deserialize(e.to_string()))?;

    Ok(df)
}

/// Convert Rust [`PipelineConfig`] to Python `PipelineConfig`.
///
/// Creates a Python `PipelineConfig` object using the builder pattern,
/// mirroring the Rust configuration. All configuration fields are transferred,
/// including optional fields like `target_column` and `algorithm`.
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `config` - The Rust pipeline configuration to convert
///
/// # Returns
///
/// A Python `lex_learning.PipelineConfig` object.
///
/// # Errors
///
/// Returns an error if:
/// - The `lex_learning` Python module cannot be imported
/// - Builder method calls fail (e.g., invalid values)
/// - The final `build()` call fails validation
///
/// # Note
///
/// The `random_seed` field is cast from `u64` to `i64` for Python compatibility,
/// as Python's integers are signed. This may cause issues for seeds > `i64::MAX`,
/// but such values are rare in practice.
#[must_use = "the converted Python config should be used"]
pub fn config_to_python<'py>(
    py: Python<'py>,
    config: &PipelineConfig,
) -> Result<Bound<'py, PyAny>, LexLearningError> {
    let lex_learning = py.import("lex_learning")?;

    // Get ProblemType enum
    let problem_type_enum = lex_learning.getattr("ProblemType")?;
    let py_problem_type = match config.problem_type {
        ProblemType::Classification => problem_type_enum.getattr("CLASSIFICATION")?,
        ProblemType::Regression => problem_type_enum.getattr("REGRESSION")?,
    };

    // Build PipelineConfig using builder
    let config_class = lex_learning.getattr("PipelineConfig")?;
    let mut builder = config_class.call_method0("builder")?;

    builder = builder.call_method1("problem_type", (&py_problem_type,))?;

    if let Some(ref target) = config.target_column {
        builder = builder.call_method1("target_column", (target,))?;
    }
    if let Some(ref algo) = config.algorithm {
        builder = builder.call_method1("algorithm", (algo,))?;
    }

    // Set all numeric/boolean fields
    builder = builder.call_method1("top_k_algorithms", (config.top_k_algorithms,))?;
    builder = builder.call_method1("optimize_hyperparams", (config.optimize_hyperparams,))?;
    builder = builder.call_method1("n_trials", (config.n_trials,))?;
    builder = builder.call_method1("cv_folds", (config.cv_folds,))?;
    builder = builder.call_method1("test_size", (config.test_size,))?;
    builder = builder.call_method1("enable_neural_networks", (config.enable_neural_networks,))?;
    builder = builder.call_method1("enable_explainability", (config.enable_explainability,))?;
    builder = builder.call_method1("shap_max_samples", (config.shap_max_samples,))?;
    builder = builder.call_method1("random_seed", (config.random_seed as i64,))?;
    builder = builder.call_method1("n_jobs", (config.n_jobs,))?;

    let py_config = builder.call_method0("build")?;
    Ok(py_config)
}

/// Extract Python `TrainingResult` to Rust [`TrainingResult`].
///
/// Converts all fields from the Python `TrainingResult` dataclass to the
/// corresponding Rust struct fields, including nested types like metrics,
/// explainability data, and model comparisons.
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `py_result` - The Python `TrainingResult` object
///
/// # Returns
///
/// A Rust [`TrainingResult`] containing all training information.
///
/// # Errors
///
/// Returns an error if:
/// - Required attributes are missing from the Python object
/// - Type extraction fails for any field
/// - Nested extraction (metrics, model comparison) fails
///
/// # Extracted Fields
///
/// - `success`: Whether training completed successfully
/// - `best_model_name`: Name of the best performing model
/// - `training_time_seconds`: Total training duration
/// - `warnings`: Any warnings generated during training
/// - `metrics`: Performance metrics (via [`extract_metrics`])
/// - `feature_importance`: List of (feature_name, importance) tuples
/// - `shap_plots`: Dictionary of plot name → PNG bytes
/// - `model_comparison`: Comparison of all trained models
///
/// [`TrainingResult`]: crate::types::TrainingResult
#[must_use = "the extracted training result should be used"]
pub fn extract_training_result(
    py: Python<'_>,
    py_result: &Bound<'_, PyAny>,
) -> Result<TrainingResult, LexLearningError> {
    // Basic fields
    let success: bool = py_result.getattr("success")?.extract()?;
    let best_model_name: String = py_result.getattr("best_model_name")?.extract()?;
    let training_time_seconds: f64 = py_result.getattr("training_time_seconds")?.extract()?;
    let warnings: Vec<String> = py_result.getattr("warnings")?.extract()?;

    // Metrics
    let py_metrics = py_result.getattr("metrics")?;
    let metrics = extract_metrics(&py_metrics)?;

    // Explainability
    let explainability = py_result.getattr("explainability")?;
    let feature_importance: Vec<(String, f64)> =
        explainability.getattr("feature_importance")?.extract()?;

    // SHAP plots (extract bytes, skip if None)
    let mut shap_plots = HashMap::new();
    for (name, attr) in [
        ("summary", "summary_plot"),
        ("beeswarm", "beeswarm_plot"),
        ("feature_importance", "feature_importance_plot"),
    ] {
        if let Ok(plot) = explainability.getattr(attr)
            && !plot.is_none()
            && let Ok(bytes) = plot.extract::<Vec<u8>>()
        {
            shap_plots.insert(name.to_string(), bytes);
        }
    }

    // Model comparison
    let py_model_comparison = py_result.getattr("model_comparison")?;
    let model_comparison = extract_model_comparison(py, &py_model_comparison)?;

    Ok(TrainingResult {
        success,
        best_model_name,
        metrics,
        feature_importance,
        shap_plots,
        model_comparison,
        training_time_seconds,
        warnings,
    })
}

/// Extract Python `Metrics` to Rust [`Metrics`].
///
/// Handles both `ClassificationMetrics` and `RegressionMetrics` from Python,
/// mapping them to the unified Rust [`Metrics`] struct with optional fields.
/// Fields not present on the Python object (e.g., classification metrics
/// for a regression model) are set to `None`.
///
/// # Arguments
///
/// * `py_metrics` - The Python metrics object (ClassificationMetrics or RegressionMetrics)
///
/// # Returns
///
/// A Rust [`Metrics`] struct with all applicable fields populated.
///
/// # Errors
///
/// Returns an error if:
/// - The Python object cannot be accessed
/// - Type extraction fails for a non-None field
///
/// # Extracted Fields
///
/// **Common metrics:**
/// - `cv_score`, `test_score`, `train_score`
///
/// **Classification metrics:**
/// - `accuracy`, `precision`, `recall`, `f1_score`, `roc_auc`
///
/// **Regression metrics:**
/// - `mse`, `rmse`, `mae`, `r2`
///
/// [`Metrics`]: crate::types::Metrics
#[must_use = "the extracted metrics should be used"]
pub fn extract_metrics(py_metrics: &Bound<'_, PyAny>) -> Result<Metrics, LexLearningError> {
    // Helper to extract optional f64 field
    let get_opt_f64 = |attr: &str| -> Option<f64> {
        py_metrics.getattr(attr).ok().and_then(
            |v| {
                if v.is_none() { None } else { v.extract().ok() }
            },
        )
    };

    Ok(Metrics {
        // Common metrics
        cv_score: get_opt_f64("cv_score"),
        test_score: get_opt_f64("test_score"),
        train_score: get_opt_f64("train_score"),
        // Classification metrics
        accuracy: get_opt_f64("accuracy"),
        precision: get_opt_f64("precision"),
        recall: get_opt_f64("recall"),
        f1_score: get_opt_f64("f1_score"),
        roc_auc: get_opt_f64("roc_auc"),
        // Regression metrics
        mse: get_opt_f64("mse"),
        rmse: get_opt_f64("rmse"),
        mae: get_opt_f64("mae"),
        r2: get_opt_f64("r2"),
    })
}

/// Extract model comparison list from Python.
///
/// Iterates over the Python list of `ModelComparison` objects and extracts
/// each one into a Rust [`ModelComparison`] struct.
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `py_list` - Python list of `ModelComparison` objects
///
/// # Returns
///
/// A vector of [`ModelComparison`] structs.
///
/// # Errors
///
/// Returns an error if:
/// - The Python object is not iterable
/// - Required attributes are missing from any comparison object
/// - Hyperparameter extraction fails
///
/// [`ModelComparison`]: crate::types::ModelComparison
fn extract_model_comparison(
    py: Python<'_>,
    py_list: &Bound<'_, PyAny>,
) -> Result<Vec<ModelComparison>, LexLearningError> {
    let mut results = Vec::new();

    for item in py_list.try_iter()? {
        let item: Bound<'_, PyAny> = item?;
        results.push(ModelComparison {
            name: item.getattr("name")?.extract()?,
            test_score: item.getattr("test_score")?.extract()?,
            train_score: item.getattr("train_score")?.extract()?,
            cv_score: item.getattr("cv_score")?.extract()?,
            training_time_seconds: item.getattr("training_time_seconds")?.extract()?,
            hyperparameters: extract_hyperparameters(py, &item.getattr("hyperparameters")?)?,
            overfitting_risk: item.getattr("overfitting_risk")?.extract()?,
        });
    }

    Ok(results)
}

/// Extract hyperparameters dict from Python to Rust `HashMap`.
///
/// Converts a Python dictionary of hyperparameter names and values to a
/// Rust `HashMap<String, serde_json::Value>`. Values are recursively
/// converted using [`py_any_to_json`].
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `py_dict` - Python dictionary object
///
/// # Returns
///
/// A `HashMap` mapping parameter names to JSON values.
///
/// # Errors
///
/// Returns an error if:
/// - The Python object is not a dict or lacks `items()` method
/// - Key extraction fails (keys must be strings)
/// - Value conversion fails (via [`py_any_to_json`])
fn extract_hyperparameters(
    py: Python<'_>,
    py_dict: &Bound<'_, PyAny>,
) -> Result<HashMap<String, serde_json::Value>, LexLearningError> {
    let mut map = HashMap::new();

    let items = py_dict.call_method0("items")?;
    for item in items.try_iter()? {
        let item: Bound<'_, PyAny> = item?;
        let key: String = item.get_item(0)?.extract()?;
        let value = py_any_to_json(py, &item.get_item(1)?)?;
        map.insert(key, value);
    }

    Ok(map)
}

/// Convert a Python object to [`serde_json::Value`].
///
/// Recursively converts Python objects to their JSON equivalents. This is
/// used for hyperparameters and other dynamic values that need to be
/// serialized or stored.
///
/// # Supported Types
///
/// | Python Type | JSON Type |
/// |-------------|-----------|
/// | `None` | `null` |
/// | `bool` | `boolean` |
/// | `int` | `number` (i64) |
/// | `float` | `number` (f64) or `null` if NaN/Infinity |
/// | `str` | `string` |
/// | `list` | `array` (recursive) |
/// | `dict` | `object` (recursive, string keys only) |
/// | Other | `string` (via `str()` repr) |
///
/// # Arguments
///
/// * `_py` - Python GIL token (unused but required for recursive calls)
/// * `obj` - The Python object to convert
///
/// # Returns
///
/// A [`serde_json::Value`] representing the Python object.
///
/// # Errors
///
/// Returns an error if:
/// - Iteration over list/dict fails
/// - String representation extraction fails for fallback
///
/// # Note
///
/// Booleans are checked before integers because in Python, `bool` is a
/// subclass of `int` (i.e., `True` would extract as `1` if checked as int first).
fn py_any_to_json(
    _py: Python<'_>,
    obj: &Bound<'_, PyAny>,
) -> Result<serde_json::Value, LexLearningError> {
    // Check for None
    if obj.is_none() {
        return Ok(serde_json::Value::Null);
    }

    // Try bool first (before int, since bool is a subtype of int in Python)
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(serde_json::Value::Bool(b));
    }

    // Try int
    if let Ok(i) = obj.extract::<i64>() {
        return Ok(serde_json::Value::Number(i.into()));
    }

    // Try float
    if let Ok(f) = obj.extract::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Ok(serde_json::Value::Number(n));
        }
        // NaN or Infinity - represent as null
        return Ok(serde_json::Value::Null);
    }

    // Try string
    if let Ok(s) = obj.extract::<String>() {
        return Ok(serde_json::Value::String(s));
    }

    // Try list
    if obj.is_instance_of::<pyo3::types::PyList>() {
        let mut arr = Vec::new();
        for item in obj.try_iter()? {
            let item: Bound<'_, PyAny> = item?;
            arr.push(py_any_to_json(_py, &item)?);
        }
        return Ok(serde_json::Value::Array(arr));
    }

    // Try dict
    if obj.is_instance_of::<pyo3::types::PyDict>() {
        let mut map = serde_json::Map::new();
        let items = obj.call_method0("items")?;
        for item in items.try_iter()? {
            let item: Bound<'_, PyAny> = item?;
            let key: String = item.get_item(0)?.extract()?;
            let value = py_any_to_json(_py, &item.get_item(1)?)?;
            map.insert(key, value);
        }
        return Ok(serde_json::Value::Object(map));
    }

    // Fallback: convert to string representation
    let repr: String = obj.str()?.extract()?;
    Ok(serde_json::Value::String(repr))
}

/// Convert a [`serde_json::Value`] to a Python object.
///
/// Recursively converts JSON values to their Python equivalents. This is
/// used for passing prediction inputs and configuration values to Python.
///
/// # Supported Types
///
/// | JSON Type | Python Type |
/// |-----------|-------------|
/// | `null` | `None` |
/// | `boolean` | `bool` |
/// | `number` (integer) | `int` |
/// | `number` (float) | `float` |
/// | `string` | `str` |
/// | `array` | `list` (recursive) |
/// | `object` | `dict` (recursive) |
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `value` - The JSON value to convert
///
/// # Returns
///
/// A Python object representing the JSON value.
///
/// # Errors
///
/// Returns an error if:
/// - A number cannot be represented (neither as i64 nor f64)
/// - List/dict construction fails
#[must_use = "the converted Python object should be used"]
pub fn json_to_pyany<'py>(
    py: Python<'py>,
    value: &serde_json::Value,
) -> Result<Bound<'py, PyAny>, LexLearningError> {
    match value {
        serde_json::Value::Null => Ok(py.None().into_bound(py)),
        serde_json::Value::Bool(b) => Ok(b.into_pyobject(py)?.to_owned().into_any()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.to_owned().into_any())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.to_owned().into_any())
            } else {
                Err(LexLearningError::InvalidData(
                    "Invalid number in JSON".to_string(),
                ))
            }
        }
        serde_json::Value::String(s) => Ok(s.into_pyobject(py)?.into_any()),
        serde_json::Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                py_list.append(json_to_pyany(py, item)?)?;
            }
            Ok(py_list.into_any())
        }
        serde_json::Value::Object(map) => {
            let py_dict = PyDict::new(py);
            for (k, v) in map {
                py_dict.set_item(k, json_to_pyany(py, v)?)?;
            }
            Ok(py_dict.into_any())
        }
    }
}

/// Convert a [`serde_json::Value`] (expected to be an Object) to a Python dict.
///
/// This is a specialized version of [`json_to_pyany`] that returns a
/// `PyDict` directly. It is used for converting prediction input instances
/// where a dict is specifically required.
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `value` - The JSON value to convert (must be a JSON Object)
///
/// # Returns
///
/// A Python `dict` object.
///
/// # Errors
///
/// Returns [`LexLearningError::InvalidData`] if the value is not a JSON Object.
/// Also returns an error if nested value conversion fails.
///
/// # Example
///
/// ```ignore
/// let instance = serde_json::json!({"age": 25, "name": "Alice"});
/// Python::attach(|py| {
///     let py_dict = json_to_pydict(py, &instance)?;
///     // py_dict is now {"age": 25, "name": "Alice"} in Python
///     Ok(())
/// });
/// ```
///
/// [`LexLearningError::InvalidData`]: crate::error::LexLearningError::InvalidData
#[must_use = "the converted Python dict should be used"]
pub fn json_to_pydict<'py>(
    py: Python<'py>,
    value: &serde_json::Value,
) -> Result<Bound<'py, PyDict>, LexLearningError> {
    if let serde_json::Value::Object(map) = value {
        let py_dict = PyDict::new(py);
        for (k, v) in map {
            py_dict.set_item(k, json_to_pyany(py, v)?)?;
        }
        Ok(py_dict)
    } else {
        Err(LexLearningError::InvalidData(
            "Expected JSON object for prediction input".to_string(),
        ))
    }
}

/// Extract a Python prediction result dict to Rust [`PredictionResult`].
///
/// Converts the prediction dictionary returned by Python's `TrainedModel.predict()`
/// into a Rust struct. The dict format varies based on problem type:
///
/// - **Classification**: `{"prediction": "class", "probability": 0.85, "probabilities": {"a": 0.85, "b": 0.15}}`
/// - **Regression**: `{"prediction": 45.5}`
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `py_dict` - The Python prediction result dictionary
///
/// # Returns
///
/// A [`PredictionResult`] with:
/// - `prediction`: The predicted value (class label or numeric value)
/// - `confidence`: Optional probability for the predicted class (classification only)
/// - `probabilities`: Optional map of class → probability (classification only)
///
/// # Errors
///
/// Returns an error if:
/// - The "prediction" key is missing
/// - Value extraction fails for any field
///
/// [`PredictionResult`]: crate::types::PredictionResult
#[must_use = "the extracted prediction result should be used"]
pub fn extract_prediction_result(
    py: Python<'_>,
    py_dict: &Bound<'_, PyAny>,
) -> Result<PredictionResult, LexLearningError> {
    // Get prediction value
    let prediction = py_dict.get_item("prediction")?;
    let prediction_json = py_any_to_json(py, &prediction)?;

    // Get optional probability (confidence) - single value for predicted class
    let confidence = py_dict.get_item("probability").ok().and_then(|p| {
        if p.is_none() {
            None
        } else {
            p.extract::<f64>().ok()
        }
    });

    // Get optional full probabilities dict
    let probabilities = py_dict.get_item("probabilities").ok().and_then(|p| {
        if p.is_none() {
            None
        } else {
            // Extract as HashMap<String, f64>
            let mut map = HashMap::new();
            if let Ok(items) = p.call_method0("items")
                && let Ok(iter) = items.try_iter()
            {
                for item in iter.flatten() {
                    if let (Ok(key), Ok(value)) = (
                        item.get_item(0).and_then(|k| k.extract::<String>()),
                        item.get_item(1).and_then(|v| v.extract::<f64>()),
                    ) {
                        map.insert(key, value);
                    }
                }
            }
            if map.is_empty() { None } else { Some(map) }
        }
    });

    Ok(PredictionResult {
        prediction: prediction_json,
        probabilities,
        confidence,
    })
}

/// Map Python exceptions to specific [`LexLearningError`] variants.
///
/// This function examines the Python exception type and converts it to
/// the most appropriate Rust error variant, preserving error semantics
/// across the language boundary.
///
/// # Exception Mapping
///
/// | Python Exception | Rust Error Variant |
/// |------------------|-------------------|
/// | `InvalidDataError` | [`LexLearningError::InvalidData`] |
/// | `TargetNotFoundError` | [`LexLearningError::TargetNotFound`] |
/// | `TrainingFailedError` | [`LexLearningError::TrainingFailed`] |
/// | `CancelledError` | [`LexLearningError::Cancelled`] |
/// | `InvalidConfigError` | [`LexLearningError::InvalidConfig`] |
/// | `ValueError` | [`LexLearningError::InvalidConfig`] |
/// | `ModelNotFoundError` | [`LexLearningError::ModelNotFound`] |
/// | `InferenceError` | [`LexLearningError::InferenceError`] |
/// | `ExplainabilityError` | [`LexLearningError::ExplainabilityError`] |
/// | Other | [`LexLearningError::PythonError`] |
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `err` - The Python exception to map
///
/// # Returns
///
/// The most specific [`LexLearningError`] variant for the exception.
///
/// [`LexLearningError`]: crate::error::LexLearningError
/// [`LexLearningError::InvalidData`]: crate::error::LexLearningError::InvalidData
/// [`LexLearningError::TargetNotFound`]: crate::error::LexLearningError::TargetNotFound
/// [`LexLearningError::TrainingFailed`]: crate::error::LexLearningError::TrainingFailed
/// [`LexLearningError::Cancelled`]: crate::error::LexLearningError::Cancelled
/// [`LexLearningError::InvalidConfig`]: crate::error::LexLearningError::InvalidConfig
/// [`LexLearningError::ModelNotFound`]: crate::error::LexLearningError::ModelNotFound
/// [`LexLearningError::InferenceError`]: crate::error::LexLearningError::InferenceError
/// [`LexLearningError::ExplainabilityError`]: crate::error::LexLearningError::ExplainabilityError
/// [`LexLearningError::PythonError`]: crate::error::LexLearningError::PythonError
#[must_use = "the mapped error should be returned or handled"]
pub fn map_python_error(py: Python<'_>, err: PyErr) -> LexLearningError {
    // Get the exception type name
    let error_type = err
        .get_type(py)
        .qualname()
        .map(|s| s.to_string())
        .unwrap_or_default();

    let message = err.value(py).to_string();

    match error_type.as_str() {
        "InvalidDataError" => LexLearningError::InvalidData(message),
        "TargetNotFoundError" => LexLearningError::TargetNotFound(message),
        "TrainingFailedError" => LexLearningError::TrainingFailed(message),
        "CancelledError" => LexLearningError::Cancelled,
        "InvalidConfigError" | "ValueError" => LexLearningError::InvalidConfig(message),
        "ModelNotFoundError" => LexLearningError::ModelNotFound { path: message },
        "InferenceError" => LexLearningError::InferenceError(message),
        "ExplainabilityError" => LexLearningError::ExplainabilityError(message),
        _ => LexLearningError::PythonError { message },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::python::runtime::initialize;

    /// Helper function to create a test DataFrame with various dtypes
    fn create_test_dataframe() -> DataFrame {
        df! {
            "int32_col" => &[1i32, 2, 3, 4, 5],
            "int64_col" => &[10i64, 20, 30, 40, 50],
            "float32_col" => &[1.1f32, 2.2, 3.3, 4.4, 5.5],
            "float64_col" => &[1.11f64, 2.22, 3.33, 4.44, 5.55],
            "string_col" => &["a", "b", "c", "d", "e"],
            "bool_col" => &[true, false, true, false, true],
        }
        .expect("Failed to create test DataFrame")
    }

    /// Helper function to create an empty DataFrame with schema
    fn create_empty_dataframe() -> DataFrame {
        df! {
            "int_col" => Vec::<i32>::new(),
            "string_col" => Vec::<String>::new(),
        }
        .expect("Failed to create empty DataFrame")
    }

    #[test]
    #[ignore = "Requires Python runtime with pyarrow and pandas"]
    fn test_roundtrip_conversion() {
        // Initialize the Python runtime first to ensure proper PYTHONPATH setup
        initialize().expect("Failed to initialize Python runtime");

        Python::attach(|py| {
            let original_df = create_test_dataframe();

            // Rust -> Python -> Rust
            let py_df = dataframe_to_python(py, &original_df).expect("Failed to convert to Python");
            let result_df =
                python_to_dataframe(py, &py_df).expect("Failed to convert back to Rust");

            // Verify shape
            assert_eq!(original_df.shape(), result_df.shape());

            // Verify column names
            assert_eq!(original_df.get_column_names(), result_df.get_column_names());
        });
    }

    #[test]
    #[ignore = "Requires Python runtime with pyarrow and pandas"]
    fn test_dtype_preservation() {
        // Initialize the Python runtime first to ensure proper PYTHONPATH setup
        initialize().expect("Failed to initialize Python runtime");

        Python::attach(|py| {
            let original_df = create_test_dataframe();

            let py_df = dataframe_to_python(py, &original_df).expect("Failed to convert to Python");
            let result_df =
                python_to_dataframe(py, &py_df).expect("Failed to convert back to Rust");

            // Check that we have the expected columns
            // Note: exact dtype preservation depends on Arrow's type mapping
            assert!(result_df.column("int32_col").is_ok());
            assert!(result_df.column("int64_col").is_ok());
            assert!(result_df.column("float32_col").is_ok());
            assert!(result_df.column("float64_col").is_ok());
            assert!(result_df.column("string_col").is_ok());
            assert!(result_df.column("bool_col").is_ok());
        });
    }

    #[test]
    #[ignore = "Requires Python runtime with pyarrow and pandas"]
    fn test_empty_dataframe() {
        // Initialize the Python runtime first to ensure proper PYTHONPATH setup
        initialize().expect("Failed to initialize Python runtime");

        Python::attach(|py| {
            let original_df = create_empty_dataframe();

            let py_df = dataframe_to_python(py, &original_df)
                .expect("Failed to convert empty DataFrame to Python");
            let result_df = python_to_dataframe(py, &py_df)
                .expect("Failed to convert empty DataFrame back to Rust");

            // Verify empty shape
            assert_eq!(result_df.height(), 0);
            assert_eq!(result_df.width(), original_df.width());
        });
    }

    #[test]
    #[ignore = "Requires Python runtime with pyarrow and pandas"]
    fn test_column_name_preservation() {
        // Initialize the Python runtime first to ensure proper PYTHONPATH setup
        initialize().expect("Failed to initialize Python runtime");

        Python::attach(|py| {
            // Test with various column names including special characters
            let df = df! {
                "normal_name" => &[1, 2, 3],
                "name_with_underscore" => &[4, 5, 6],
                "Name With Spaces" => &[7, 8, 9],
                "123_numeric_start" => &[10, 11, 12],
            }
            .expect("Failed to create DataFrame with special column names");

            let py_df = dataframe_to_python(py, &df).expect("Failed to convert to Python");
            let result_df =
                python_to_dataframe(py, &py_df).expect("Failed to convert back to Rust");

            assert_eq!(df.get_column_names(), result_df.get_column_names());
        });
    }
}
