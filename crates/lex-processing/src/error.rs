//! Custom error types for the data preprocessing pipeline.
//!
//! This module provides a comprehensive error hierarchy using `thiserror`
//! for better error handling and context throughout the pipeline.
//!
//! Errors are serializable for Tauri IPC compatibility, allowing them to be
//! sent to the frontend for display.

use serde::Serialize;
use serde::ser::SerializeStruct;
use thiserror::Error;

/// The main error type for the preprocessing pipeline.
#[derive(Error, Debug)]
pub enum PreprocessingError {
    /// Pipeline was cancelled by user.
    #[error("Pipeline cancelled")]
    Cancelled,

    /// Column was not found in the dataset.
    #[error("Column '{0}' not found in dataset")]
    ColumnNotFound(String),

    /// Invalid configuration provided.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// No valid values found in a column for computation.
    #[error("No valid values found in column '{0}'")]
    NoValidValues(String),

    /// Type conversion failed.
    #[error("Failed to convert column '{column}' to {target_type}: {reason}")]
    TypeConversionFailed {
        column: String,
        target_type: String,
        reason: String,
    },

    /// Data profiling failed.
    #[error("Failed to profile dataset: {0}")]
    ProfilingFailed(String),

    /// Data cleaning failed.
    #[error("Failed to clean data: {0}")]
    CleaningFailed(String),

    /// Imputation failed.
    #[error("Failed to impute missing values in column '{column}': {reason}")]
    ImputationFailed { column: String, reason: String },

    /// AI client error.
    #[error("AI client error: {0}")]
    AiClientError(String),

    /// Report generation failed.
    #[error("Failed to generate report: {0}")]
    ReportGenerationFailed(String),

    /// No data loaded in the application.
    #[error("No data loaded")]
    NoDataLoaded,

    /// Internal error (e.g., thread join failure).
    #[error("Internal error: {0}")]
    Internal(String),

    /// IO error wrapper.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Polars error wrapper.
    #[error("Polars error: {0}")]
    Polars(#[from] polars::error::PolarsError),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP request error (for AI client, only with "ai" feature).
    #[cfg(feature = "ai")]
    #[error("HTTP request error: {0}")]
    HttpRequest(#[from] reqwest::Error),

    /// Generic error with context.
    #[error("{context}: {source}")]
    WithContext {
        context: String,
        #[source]
        source: Box<PreprocessingError>,
    },
}

impl PreprocessingError {
    /// Add context to an error.
    pub fn with_context(self, context: impl Into<String>) -> Self {
        PreprocessingError::WithContext {
            context: context.into(),
            source: Box::new(self),
        }
    }

    /// Get error code for frontend handling.
    ///
    /// These codes can be used by the frontend to handle specific error types
    /// differently (e.g., showing a different UI for cancellation vs. failure).
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Cancelled => "CANCELLED",
            Self::ColumnNotFound(_) => "COLUMN_NOT_FOUND",
            Self::InvalidConfig(_) => "INVALID_CONFIG",
            Self::NoValidValues(_) => "NO_VALID_VALUES",
            Self::TypeConversionFailed { .. } => "TYPE_CONVERSION_FAILED",
            Self::ProfilingFailed(_) => "PROFILING_FAILED",
            Self::CleaningFailed(_) => "CLEANING_FAILED",
            Self::ImputationFailed { .. } => "IMPUTATION_FAILED",
            Self::AiClientError(_) => "AI_CLIENT_ERROR",
            Self::ReportGenerationFailed(_) => "REPORT_GENERATION_FAILED",
            Self::NoDataLoaded => "NO_DATA_LOADED",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Io(_) => "IO_ERROR",
            Self::Polars(_) => "POLARS_ERROR",
            Self::Json(_) => "JSON_ERROR",
            #[cfg(feature = "ai")]
            Self::HttpRequest(_) => "HTTP_REQUEST_ERROR",
            Self::WithContext { source, .. } => source.error_code(),
        }
    }

    /// Check if this error represents a cancellation.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    /// Check if this error is recoverable (i.e., not a fundamental failure).
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Cancelled | Self::NoDataLoaded | Self::InvalidConfig(_)
        )
    }
}

/// Serialize implementation for Tauri IPC compatibility.
///
/// Errors are serialized as a struct with `code` and `message` fields,
/// making them easy to handle in the frontend.
impl Serialize for PreprocessingError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("PreprocessingError", 2)?;
        state.serialize_field("code", &self.error_code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

/// Result type alias for preprocessing operations.
pub type Result<T> = std::result::Result<T, PreprocessingError>;

/// Extension trait for adding context to Results.
pub trait ResultExt<T> {
    /// Add context to an error result.
    fn context(self, context: impl Into<String>) -> Result<T>;
}

impl<T> ResultExt<T> for Result<T> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| e.with_context(context))
    }
}

impl<T> ResultExt<T> for std::result::Result<T, polars::error::PolarsError> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| PreprocessingError::Polars(e).with_context(context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code() {
        assert_eq!(PreprocessingError::Cancelled.error_code(), "CANCELLED");
        assert_eq!(
            PreprocessingError::ColumnNotFound("test".to_string()).error_code(),
            "COLUMN_NOT_FOUND"
        );
    }

    #[test]
    fn test_is_cancelled() {
        assert!(PreprocessingError::Cancelled.is_cancelled());
        assert!(!PreprocessingError::NoDataLoaded.is_cancelled());
    }

    #[test]
    fn test_is_recoverable() {
        assert!(PreprocessingError::Cancelled.is_recoverable());
        assert!(PreprocessingError::NoDataLoaded.is_recoverable());
        assert!(!PreprocessingError::CleaningFailed("error".to_string()).is_recoverable());
    }

    #[test]
    fn test_error_serialization() {
        let error = PreprocessingError::ColumnNotFound("Age".to_string());
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("COLUMN_NOT_FOUND"));
        assert!(json.contains("Age"));
    }

    #[test]
    fn test_with_context() {
        let error =
            PreprocessingError::ColumnNotFound("test".to_string()).with_context("During profiling");
        assert!(error.to_string().contains("During profiling"));
        assert_eq!(error.error_code(), "COLUMN_NOT_FOUND"); // Preserves original code
    }
}
