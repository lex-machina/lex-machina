//! Event System for Rust → Frontend Communication
//!
//! This module defines the event system that allows Rust to push state changes
//! to the TypeScript frontend. This implements the "hybrid" communication pattern:
//! - Events: Rust pushes notifications when state changes
//! - Commands: Frontend pulls data when needed (e.g., `get_rows` for large payloads)
//!
//! # Event Flow
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                           RUST BACKEND                                  │
//! │                                                                         │
//! │   load_file() ──► emit("file:loaded", FileLoadedPayload)               │
//! │   close_file() ──► emit("file:closed", {})                             │
//! │   operations ──► emit("app:loading", LoadingPayload)                   │
//! │   errors ──► emit("app:error", ErrorPayload)                           │
//! │                                                                         │
//! │   PREPROCESSING EVENTS:                                                 │
//! │   pipeline ──► emit("preprocessing:progress", ProgressUpdate)          │
//! │   pipeline ──► emit("preprocessing:complete", PreprocessingSummary)    │
//! │   pipeline ──► emit("preprocessing:error", PreprocessingErrorPayload)  │
//! │   pipeline ──► emit("preprocessing:cancelled", {})                     │
//! │                                                                         │
//! │   SETTINGS EVENTS:                                                      │
//! │   set_theme() ──► emit("settings:theme-changed", Theme)                │
//! │                                                                         │
//! └───────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Why Events + Commands (Hybrid)?
//!
//! - **Events** are great for notifications (small payloads, fire-and-forget)
//! - **Commands** are better for large data transfers (rows, with request/response)
//! - The frontend subscribes to events to know *when* to fetch, then uses commands
//!   to fetch the actual data

use lex_processing::{PreprocessingSummary, ProgressUpdate};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::state::{FileInfo, Theme};

// ============================================================================
// EVENT NAME CONSTANTS
// ============================================================================

/// Event emitted when a file is successfully loaded.
/// Payload: `FileLoadedPayload` containing `FileInfo`
pub const EVENT_FILE_LOADED: &str = "file:loaded";

/// Event emitted when a file is closed.
/// Payload: Empty (unit type serializes to `null`)
pub const EVENT_FILE_CLOSED: &str = "file:closed";

/// Event emitted when loading state changes.
/// Payload: `LoadingPayload` with status and optional message
pub const EVENT_LOADING: &str = "app:loading";

/// Event emitted when an error occurs.
/// Payload: `ErrorPayload` with error code and message
pub const EVENT_ERROR: &str = "app:error";

// ============================================================================
// PREPROCESSING EVENT CONSTANTS
// ============================================================================

/// Event emitted during preprocessing to report progress.
/// Payload: `ProgressUpdate` from lex-processing library
/// Emitted frequently during pipeline execution.
pub const EVENT_PREPROCESSING_PROGRESS: &str = "preprocessing:progress";

/// Event emitted when preprocessing completes successfully.
/// Payload: `PreprocessingSummary` with details of what was done
pub const EVENT_PREPROCESSING_COMPLETE: &str = "preprocessing:complete";

/// Event emitted when preprocessing fails with an error.
/// Payload: `PreprocessingErrorPayload` with error code and message
pub const EVENT_PREPROCESSING_ERROR: &str = "preprocessing:error";

/// Event emitted when preprocessing is cancelled by the user.
/// Payload: Empty (unit type serializes to `null`)
pub const EVENT_PREPROCESSING_CANCELLED: &str = "preprocessing:cancelled";

// ============================================================================
// SETTINGS EVENT CONSTANTS
// ============================================================================

/// Event emitted when the theme setting changes.
/// Payload: `Theme` enum value
pub const EVENT_THEME_CHANGED: &str = "settings:theme-changed";

// ============================================================================
// ML EVENT CONSTANTS
// ============================================================================

/// Event emitted during ML training to report progress.
/// Payload: `MLProgressPayload` with stage, progress, message, model info
pub const EVENT_ML_PROGRESS: &str = "ml:progress";

/// Event emitted when ML training completes successfully.
/// Payload: `MLCompletePayload` with best model name and metrics
pub const EVENT_ML_COMPLETE: &str = "ml:complete";

/// Event emitted when ML training fails with an error.
/// Payload: `MLErrorPayload` with error code and message
pub const EVENT_ML_ERROR: &str = "ml:error";

/// Event emitted when ML training is cancelled by the user.
/// Payload: Empty (unit type serializes to `null`)
pub const EVENT_ML_CANCELLED: &str = "ml:cancelled";

/// Event emitted when ML kernel (Python runtime) status changes.
/// Payload: `MLKernelStatusPayload` with status and optional message
pub const EVENT_ML_KERNEL_STATUS: &str = "ml:kernel-status";

// ============================================================================
// EVENT PAYLOADS
// ============================================================================

/// Payload for the `file:loaded` event.
///
/// Contains full file metadata so the frontend can update its UI
/// without needing to make a separate `get_file_info` call.
#[derive(Debug, Clone, Serialize)]
pub struct FileLoadedPayload {
    /// Complete file metadata (path, name, columns, row count, etc.)
    pub file_info: FileInfo,
}

/// Payload for the `app:loading` event.
///
/// Indicates whether a long-running operation is in progress.
/// The frontend can use this to show loading indicators.
#[derive(Debug, Clone, Serialize)]
pub struct LoadingPayload {
    /// Whether loading is currently in progress
    pub is_loading: bool,
    /// Optional message describing what's happening (e.g., "Parsing CSV...")
    pub message: Option<String>,
}

/// Payload for the `app:error` event.
///
/// Contains structured error information for display in the UI.
/// The frontend typically shows this in both a toast and the status bar.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    /// Error code for programmatic handling (e.g., "FILE_NOT_FOUND")
    pub code: String,
    /// Human-readable error message for display
    pub message: String,
}

// ============================================================================
// PREPROCESSING EVENT PAYLOADS
// ============================================================================

/// Payload for the `preprocessing:error` event.
///
/// Contains structured error information specific to preprocessing failures.
/// Includes an error code that maps to `PreprocessingError` variants.
#[derive(Debug, Clone, Serialize)]
pub struct PreprocessingErrorPayload {
    /// Error code for programmatic handling (e.g., "CANCELLED", "AI_CLIENT_ERROR")
    pub code: String,
    /// Human-readable error message for display
    pub message: String,
}

// ============================================================================
// ML EVENT PAYLOADS
// ============================================================================

/// Payload for the `ml:progress` event.
///
/// Contains progress information during ML training.
#[derive(Debug, Clone, Serialize)]
pub struct MLProgressPayload {
    /// Current training stage (e.g., "initializing", "training", "evaluation")
    pub stage: String,
    /// Overall progress from 0.0 to 1.0
    pub progress: f64,
    /// Human-readable status message
    pub message: String,
    /// Name of the model currently being trained (if in training stage)
    pub current_model: Option<String>,
    /// Tuple of (completed, total) models trained
    pub models_completed: Option<(u32, u32)>,
}

/// Payload for the `ml:complete` event.
///
/// Contains summary information about a successful training run.
#[derive(Debug, Clone, Serialize)]
pub struct MLCompletePayload {
    /// Name of the best performing model
    pub best_model_name: String,
    /// Test set score (accuracy for classification, R2 for regression)
    pub test_score: f64,
    /// Total training time in seconds
    pub training_time_seconds: f64,
}

/// Payload for the `ml:error` event.
///
/// Contains structured error information specific to ML training failures.
#[derive(Debug, Clone, Serialize)]
pub struct MLErrorPayload {
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message for display
    pub message: String,
}

/// Payload for the `ml:kernel-status` event.
///
/// Contains status information about the Python ML runtime kernel.
#[derive(Debug, Clone, Serialize)]
pub struct MLKernelStatusPayload {
    /// Current kernel status
    pub status: MLKernelStatus,
    /// Optional message (e.g., error details, initialization progress)
    pub message: Option<String>,
}

/// Kernel status values for ML runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MLKernelStatus {
    /// Kernel has not been initialized yet
    Uninitialized,
    /// Kernel is currently initializing
    Initializing,
    /// Kernel is ready for training
    Ready,
    /// Kernel encountered an error
    Error,
}

// ============================================================================
// EVENT EMISSION HELPERS
// ============================================================================

/// Helper trait for emitting events with a cleaner API.
///
/// This trait extends `AppHandle` with convenient methods for emitting
/// our custom events. Using a trait keeps the code clean and allows
/// for easy testing/mocking.
///
/// # Usage
///
/// ```rust,ignore
/// use crate::events::AppEventEmitter;
///
/// fn some_command(app: AppHandle) {
///     app.emit_file_loaded(file_info);
///     app.emit_loading(true, Some("Processing..."));
///     app.emit_error("PARSE_ERROR", "Invalid CSV format");
/// }
/// ```
pub trait AppEventEmitter {
    /// Emit the `file:loaded` event with file metadata.
    fn emit_file_loaded(&self, file_info: FileInfo);

    /// Emit the `file:closed` event.
    fn emit_file_closed(&self);

    /// Emit the `app:loading` event with loading state.
    fn emit_loading(&self, is_loading: bool, message: Option<&str>);

    /// Emit the `app:error` event with error details.
    fn emit_error(&self, code: &str, message: &str);

    // -------------------------------------------------------------------------
    // Preprocessing Events
    // -------------------------------------------------------------------------

    /// Emit the `preprocessing:progress` event with progress update.
    ///
    /// This is called frequently during pipeline execution to update
    /// the progress UI. The `ProgressUpdate` struct contains stage,
    /// sub-stage, progress percentage, and message.
    fn emit_preprocessing_progress(&self, update: &ProgressUpdate);

    /// Emit the `preprocessing:complete` event with summary.
    ///
    /// Called when preprocessing finishes successfully. The summary
    /// contains details about rows/columns processed, issues found, etc.
    fn emit_preprocessing_complete(&self, summary: &PreprocessingSummary);

    /// Emit the `preprocessing:error` event with error details.
    ///
    /// Called when preprocessing fails. The error code maps to
    /// `PreprocessingError` variants for programmatic handling.
    fn emit_preprocessing_error(&self, code: &str, message: &str);

    /// Emit the `preprocessing:cancelled` event.
    ///
    /// Called when the user cancels preprocessing via the cancel button.
    fn emit_preprocessing_cancelled(&self);

    // -------------------------------------------------------------------------
    // Settings Events
    // -------------------------------------------------------------------------

    /// Emit the `settings:theme-changed` event with new theme.
    ///
    /// Called when the theme setting is changed. The frontend should
    /// immediately apply the new theme.
    fn emit_theme_changed(&self, theme: Theme);

    // -------------------------------------------------------------------------
    // ML Events
    // -------------------------------------------------------------------------

    /// Emit the `ml:progress` event with progress update.
    ///
    /// Called during training to update the progress UI.
    fn emit_ml_progress(&self, update: &MLProgressPayload);

    /// Emit the `ml:complete` event with completion summary.
    ///
    /// Called when training finishes successfully.
    fn emit_ml_complete(&self, payload: &MLCompletePayload);

    /// Emit the `ml:error` event with error details.
    ///
    /// Called when training fails.
    fn emit_ml_error(&self, code: &str, message: &str);

    /// Emit the `ml:cancelled` event.
    ///
    /// Called when the user cancels training.
    fn emit_ml_cancelled(&self);

    /// Emit the `ml:kernel-status` event with kernel status.
    ///
    /// Called when the Python ML runtime status changes.
    fn emit_ml_kernel_status(&self, status: MLKernelStatus, message: Option<&str>);
}

impl AppEventEmitter for AppHandle {
    fn emit_file_loaded(&self, file_info: FileInfo) {
        let payload = FileLoadedPayload { file_info };
        if let Err(e) = self.emit(EVENT_FILE_LOADED, payload) {
            eprintln!("Failed to emit file:loaded event: {}", e);
        }
    }

    fn emit_file_closed(&self) {
        // Emit with unit type () which serializes to null
        if let Err(e) = self.emit(EVENT_FILE_CLOSED, ()) {
            eprintln!("Failed to emit file:closed event: {}", e);
        }
    }

    fn emit_loading(&self, is_loading: bool, message: Option<&str>) {
        let payload = LoadingPayload {
            is_loading,
            message: message.map(String::from),
        };
        if let Err(e) = self.emit(EVENT_LOADING, payload) {
            eprintln!("Failed to emit app:loading event: {}", e);
        }
    }

    fn emit_error(&self, code: &str, message: &str) {
        let payload = ErrorPayload {
            code: code.to_string(),
            message: message.to_string(),
        };
        if let Err(e) = self.emit(EVENT_ERROR, payload) {
            eprintln!("Failed to emit app:error event: {}", e);
        }
    }

    // -------------------------------------------------------------------------
    // Preprocessing Events Implementation
    // -------------------------------------------------------------------------

    fn emit_preprocessing_progress(&self, update: &ProgressUpdate) {
        if let Err(e) = self.emit(EVENT_PREPROCESSING_PROGRESS, update) {
            eprintln!("Failed to emit preprocessing:progress event: {}", e);
        }
    }

    fn emit_preprocessing_complete(&self, summary: &PreprocessingSummary) {
        if let Err(e) = self.emit(EVENT_PREPROCESSING_COMPLETE, summary) {
            eprintln!("Failed to emit preprocessing:complete event: {}", e);
        }
    }

    fn emit_preprocessing_error(&self, code: &str, message: &str) {
        let payload = PreprocessingErrorPayload {
            code: code.to_string(),
            message: message.to_string(),
        };
        if let Err(e) = self.emit(EVENT_PREPROCESSING_ERROR, payload) {
            eprintln!("Failed to emit preprocessing:error event: {}", e);
        }
    }

    fn emit_preprocessing_cancelled(&self) {
        if let Err(e) = self.emit(EVENT_PREPROCESSING_CANCELLED, ()) {
            eprintln!("Failed to emit preprocessing:cancelled event: {}", e);
        }
    }

    // -------------------------------------------------------------------------
    // Settings Events Implementation
    // -------------------------------------------------------------------------

    fn emit_theme_changed(&self, theme: Theme) {
        if let Err(e) = self.emit(EVENT_THEME_CHANGED, theme) {
            eprintln!("Failed to emit settings:theme-changed event: {}", e);
        }
    }

    // -------------------------------------------------------------------------
    // ML Events Implementation
    // -------------------------------------------------------------------------

    fn emit_ml_progress(&self, update: &MLProgressPayload) {
        if let Err(e) = self.emit(EVENT_ML_PROGRESS, update) {
            eprintln!("Failed to emit ml:progress event: {}", e);
        }
    }

    fn emit_ml_complete(&self, payload: &MLCompletePayload) {
        if let Err(e) = self.emit(EVENT_ML_COMPLETE, payload) {
            eprintln!("Failed to emit ml:complete event: {}", e);
        }
    }

    fn emit_ml_error(&self, code: &str, message: &str) {
        let payload = MLErrorPayload {
            code: code.to_string(),
            message: message.to_string(),
        };
        if let Err(e) = self.emit(EVENT_ML_ERROR, payload) {
            eprintln!("Failed to emit ml:error event: {}", e);
        }
    }

    fn emit_ml_cancelled(&self) {
        if let Err(e) = self.emit(EVENT_ML_CANCELLED, ()) {
            eprintln!("Failed to emit ml:cancelled event: {}", e);
        }
    }

    fn emit_ml_kernel_status(&self, status: MLKernelStatus, message: Option<&str>) {
        let payload = MLKernelStatusPayload {
            status,
            message: message.map(String::from),
        };
        if let Err(e) = self.emit(EVENT_ML_KERNEL_STATUS, payload) {
            eprintln!("Failed to emit ml:kernel-status event: {}", e);
        }
    }
}

// ============================================================================
// ERROR CODES
// ============================================================================

/// Standard error codes for consistent error handling across the app.
///
/// Using constants instead of an enum allows for easier serialization
/// and extension without breaking changes.
pub mod error_codes {
    // -------------------------------------------------------------------------
    // File Error Codes
    // -------------------------------------------------------------------------

    /// File was not found at the specified path
    pub const FILE_NOT_FOUND: &str = "FILE_NOT_FOUND";

    /// Failed to read the file (I/O error, permissions, etc.)
    pub const FILE_READ_ERROR: &str = "FILE_READ_ERROR";

    /// Failed to parse the file (invalid CSV format, encoding, etc.)
    pub const FILE_PARSE_ERROR: &str = "FILE_PARSE_ERROR";

    /// Failed to get file metadata (size, etc.)
    pub const FILE_METADATA_ERROR: &str = "FILE_METADATA_ERROR";

    /// Generic/unknown error
    pub const UNKNOWN_ERROR: &str = "UNKNOWN_ERROR";

    // -------------------------------------------------------------------------
    // Preprocessing Error Codes
    // -------------------------------------------------------------------------

    /// Pipeline was cancelled by the user
    pub const PREPROCESSING_CANCELLED: &str = "CANCELLED";

    /// No DataFrame is loaded to preprocess
    pub const PREPROCESSING_NO_DATA: &str = "NO_DATA_LOADED";

    /// Pipeline configuration is invalid
    pub const PREPROCESSING_INVALID_CONFIG: &str = "INVALID_CONFIG";

    /// Referenced column doesn't exist in the DataFrame
    pub const PREPROCESSING_COLUMN_NOT_FOUND: &str = "COLUMN_NOT_FOUND";

    /// AI provider API call failed
    pub const PREPROCESSING_AI_ERROR: &str = "AI_CLIENT_ERROR";

    /// Polars DataFrame operation failed
    pub const PREPROCESSING_POLARS_ERROR: &str = "POLARS_ERROR";

    /// Internal preprocessing error
    pub const PREPROCESSING_INTERNAL_ERROR: &str = "INTERNAL_ERROR";

    // -------------------------------------------------------------------------
    // Settings Error Codes
    // -------------------------------------------------------------------------

    /// AI provider type is not recognized
    pub const SETTINGS_INVALID_PROVIDER: &str = "INVALID_PROVIDER";

    /// API key validation failed
    pub const SETTINGS_INVALID_API_KEY: &str = "INVALID_API_KEY";

    // -------------------------------------------------------------------------
    // ML Error Codes
    // -------------------------------------------------------------------------

    /// ML runtime is not initialized
    pub const ML_NOT_INITIALIZED: &str = "ML_NOT_INITIALIZED";

    /// No DataFrame is loaded for training
    pub const ML_NO_DATA: &str = "ML_NO_DATA";

    /// Training is already in progress
    pub const ML_TRAINING_IN_PROGRESS: &str = "ML_TRAINING_IN_PROGRESS";

    /// No trained model available
    pub const ML_NO_MODEL: &str = "ML_NO_MODEL";

    /// Invalid ML configuration
    pub const ML_INVALID_CONFIG: &str = "ML_INVALID_CONFIG";

    /// All models failed to train
    pub const ML_TRAINING_FAILED: &str = "ML_TRAINING_FAILED";

    /// Training was cancelled by user
    pub const ML_CANCELLED: &str = "ML_CANCELLED";

    /// Prediction/inference failed
    pub const ML_INFERENCE_ERROR: &str = "ML_INFERENCE_ERROR";

    /// Python runtime initialization failed
    pub const ML_RUNTIME_INIT_FAILED: &str = "ML_RUNTIME_INIT_FAILED";
}
