//! Error types for the lex-learning crate.
//!
//! This module defines [`LexLearningError`], the main error type used throughout
//! the crate. All public API functions return `Result<T, LexLearningError>`.
//!
//! # Error Handling
//!
//! Errors are designed to be:
//! - **Descriptive**: Each variant includes context about what went wrong
//! - **Actionable**: Error messages suggest how to fix the issue where possible
//! - **Mappable**: Python exceptions are automatically converted to appropriate variants
//!
//! # Example
//!
//! ```no_run
//! use lex_learning::{Pipeline, PipelineConfig, LexLearningError};
//!
//! fn train() -> Result<(), LexLearningError> {
//!     // Errors are automatically propagated with ?
//!     let config = PipelineConfig::builder()
//!         .target_column("target")
//!         .build()?;
//!     Ok(())
//! }
//! ```

use thiserror::Error;

/// Specific kinds of Arrow conversion errors.
///
/// Used to provide granular error information when converting DataFrames
/// between Rust (Polars) and Python (pandas) via Arrow IPC.
///
/// This enum is marked `#[non_exhaustive]` to allow adding new variants
/// in future versions without breaking downstream code.
#[derive(Error, Debug, Clone)]
#[non_exhaustive]
pub enum ArrowConversionKind {
    /// Failed to serialize a Polars DataFrame to Arrow IPC bytes.
    ///
    /// This typically occurs when the DataFrame contains unsupported column types.
    #[error("serialization failed: {0}")]
    Serialize(String),

    /// Failed to deserialize Arrow IPC bytes to a Polars DataFrame.
    ///
    /// This may occur if the Arrow data is corrupted or incompatible.
    #[error("deserialization failed: {0}")]
    Deserialize(String),

    /// Failed to convert types during Arrow transfer (e.g., Python ↔ Rust).
    ///
    /// This occurs when a Python type cannot be mapped to the expected Rust type.
    #[error("type conversion failed: {0}")]
    TypeConversion(String),
}

/// The main error type for lex-learning operations.
///
/// This enum covers all error conditions that can occur during:
/// - Pipeline configuration and validation
/// - Data preprocessing and validation
/// - Model training and evaluation
/// - Model inference and prediction
/// - Explainability analysis
/// - Python runtime operations
///
/// # Error Conversion
///
/// Python exceptions are automatically converted to the appropriate variant:
/// - `InvalidConfigError` / `ValueError` → [`InvalidConfig`](Self::InvalidConfig)
/// - `InvalidDataError` → [`InvalidData`](Self::InvalidData)
/// - `TargetNotFoundError` → [`TargetNotFound`](Self::TargetNotFound)
/// - `TrainingFailedError` → [`TrainingFailed`](Self::TrainingFailed)
/// - `ModelNotFoundError` → [`ModelNotFound`](Self::ModelNotFound)
/// - `InferenceError` → [`InferenceError`](Self::InferenceError)
/// - `CancelledError` → [`Cancelled`](Self::Cancelled)
/// - `ExplainabilityError` → [`ExplainabilityError`](Self::ExplainabilityError)
/// - Other Python exceptions → [`PythonError`](Self::PythonError)
///
/// This enum is marked `#[non_exhaustive]` to allow adding new variants
/// in future versions without breaking downstream code.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum LexLearningError {
    /// Invalid configuration provided to the pipeline.
    ///
    /// Check the error message for details on which configuration value is invalid
    /// and what values are accepted.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Invalid data provided for training or inference.
    ///
    /// Common causes:
    /// - DataFrame contains null values (should be handled by lex-processing)
    /// - DataFrame has insufficient rows for training
    /// - Feature types are incompatible with the selected algorithm
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// The specified target column was not found in the DataFrame.
    ///
    /// Ensure the `target_column` in your config matches a column name in the DataFrame.
    /// Column names are case-sensitive.
    #[error("Target column '{0}' not found")]
    TargetNotFound(String),

    /// Training failed due to an error in the ML pipeline.
    ///
    /// This typically means all candidate models failed to train. Check the error
    /// message for details about individual model failures.
    #[error("Training failed: {0}")]
    TrainingFailed(String),

    /// The specified model file was not found.
    ///
    /// Ensure the path is correct and the file exists. Model files should have
    /// a `.pkl` extension.
    #[error("Model not found: {path}")]
    ModelNotFound {
        /// The path that was not found.
        path: String,
    },

    /// An error occurred during inference/prediction.
    ///
    /// Common causes:
    /// - Input features don't match the model's expected features
    /// - Input contains invalid or out-of-range values
    /// - Model file is corrupted
    #[error("Inference error: {0}")]
    InferenceError(String),

    /// Training was cancelled by the user.
    ///
    /// This is not an error condition but indicates the training was intentionally
    /// stopped before completion.
    #[error("Training cancelled")]
    Cancelled,

    /// An error occurred during SHAP explainability computation.
    ///
    /// SHAP analysis can fail if:
    /// - The model type is not supported by SHAP
    /// - There's insufficient memory for the computation
    /// - The sample size is too large (adjust `shap_max_samples`)
    #[error("Explainability error: {0}")]
    ExplainabilityError(String),

    /// An error occurred in the Python runtime.
    ///
    /// This is a catch-all for Python exceptions that don't map to a specific
    /// error variant. Check the message for the original Python exception details.
    #[error("Python error: {message}")]
    PythonError {
        /// The Python exception message.
        message: String,
    },

    /// Failed to initialize the Python runtime.
    ///
    /// Common causes:
    /// - Python runtime files are missing or corrupted
    /// - Required Python packages are not installed
    /// - Environment variables (PYTHONHOME, PYTHONPATH) are incorrect
    ///
    /// Ensure [`initialize()`](crate::initialize) is called before any other operations.
    #[error("Runtime initialization failed: {0}")]
    RuntimeInit(String),

    /// Failed to convert data between Rust and Python via Arrow.
    ///
    /// See [`ArrowConversionKind`] for specific conversion error types.
    #[error("Arrow conversion error: {0}")]
    ArrowConversion(#[from] ArrowConversionKind),

    /// I/O error during file operations.
    ///
    /// This wraps standard I/O errors that occur during model save/load operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<pyo3::PyErr> for LexLearningError {
    fn from(err: pyo3::PyErr) -> Self {
        LexLearningError::PythonError {
            message: err.to_string(),
        }
    }
}

impl From<std::convert::Infallible> for LexLearningError {
    fn from(_: std::convert::Infallible) -> Self {
        // This can never happen since Infallible is uninhabited
        unreachable!("Infallible error cannot be constructed")
    }
}
