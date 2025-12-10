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
//! └───────────────────────────────┬─────────────────────────────────────────┘
//!                                 │ Tauri Event System
//!                                 ▼
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                         TYPESCRIPT FRONTEND                             │
//! │                                                                         │
//! │   useRustEvent("file:loaded", (payload) => { ... })                    │
//! │   useRustEvent("file:closed", () => { ... })                           │
//! │   useRustEvent("app:loading", (payload) => { ... })                    │
//! │   useRustEvent("app:error", (payload) => { ... })                      │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Why Events + Commands (Hybrid)?
//!
//! - **Events** are great for notifications (small payloads, fire-and-forget)
//! - **Commands** are better for large data transfers (rows, with request/response)
//! - The frontend subscribes to events to know *when* to fetch, then uses commands
//!   to fetch the actual data

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::state::FileInfo;

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
/// ```rust
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
}

// ============================================================================
// ERROR CODES
// ============================================================================

/// Standard error codes for consistent error handling across the app.
///
/// Using constants instead of an enum allows for easier serialization
/// and extension without breaking changes.
pub mod error_codes {
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
}
