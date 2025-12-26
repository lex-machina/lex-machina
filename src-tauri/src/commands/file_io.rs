//! File I/O Commands
//!
//! This module handles loading CSV files into memory using Polars.
//! It's responsible for:
//! - Reading CSV files from disk
//! - Parsing them into Polars DataFrames
//! - Extracting metadata (column names, types, row counts)
//! - Auto-calculating optimal column widths
//! - Storing loaded data in application state
//! - Emitting events to notify the frontend of state changes
//!
//! # Why Polars?
//!
//! Polars is a fast DataFrame library written in Rust. Benefits:
//! - Extremely fast CSV parsing (multi-threaded, vectorized)
//! - Low memory usage (columnar storage, lazy evaluation)
//! - Rich type inference (auto-detects column types)
//! - Handles large files (millions of rows)
//!
//! # Error Handling
//!
//! Uses a custom `FileError` enum that implements `Serialize` so errors
//! can be sent to the frontend as JSON for user-friendly error messages.
//!
//! # Events Emitted
//!
//! - `app:loading` - When loading starts/ends
//! - `file:loaded` - When file is successfully loaded
//! - `app:error` - When an error occurs

use polars::prelude::*;
use std::{fs, path::Path};
use tauri::{AppHandle, State};

use crate::events::{AppEventEmitter, error_codes};

use crate::state::{AppState, ColumnInfo, FileInfo, LoadedDataFrame};

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Custom error type for file operations.
///
/// Each variant represents a different failure mode with a descriptive message.
/// Implements `Serialize` so errors can be sent to the frontend as JSON.
/// # Variants
///
/// - `NotFound` - File doesn't exist at the specified path
/// - `ReadError` - Failed to open/read the file (permissions, I/O error)
/// - `ParseError` - CSV parsing failed (malformed CSV, encoding issues)
/// - `MetadataError` - Failed to get file system metadata (size, etc.)
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Failed to read file: {0}")]
    ReadError(String),

    #[error("Failed to parse CSV: {0}")]
    ParseError(String),

    #[error("Failed to get file metadata: {0}")]
    MetadataError(String),
}

/// Manual `Serialize` implementation for FileError.
///
/// Tauri requires commant to return types to be serializable.
/// We serialize errors as simple strings containing the error message.
/// This allows the fronend to display them in toast notifications.
impl serde::Serialize for FileError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Convert error to string using Display trait (from thiserror)
        serializer.serialize_str(&self.to_string())
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Calculates the optimal display width for a column based on its content.
///
/// This function samples data from the column and calculates a pixel width
/// that will comfortably display most values without excessive truncation.
///
/// # Algorithm
///
/// 1. Get the header length (column name)
/// 2. Sample N rows from the column
/// 3. Find the maximum string length among samples
/// 4. Calculate pixel widths: `max_chars * CHAR_WIDTH + PADDING`
/// 5. Clamp to `MIN_WIDTH..MAX_WIDTH` range
///
/// # Parameters
///
/// - `df` - The DataFrame containing the column
/// - `col_name` - Name of the column to measure
/// - `sample_size` - Number of rows to sample (e.g., 100)
///
/// # Returns
///
/// Width in pixels (f32), clamped between 80 and 400 pixels.
///
/// # Why Sample?
///
/// Scanning all rows would be slow for large datasets. Sampling gives
/// a good estimate quickly. 100 samples is usually sufficient.
fn calculate_column_width(df: &DataFrame, col_name: &str, sample_size: usize) -> f32 {
    // Constants for width calculation
    const CHAR_WIDTH: f32 = 8.0; // Average character width in pixels (monospace-ish)
    const PADDING: f32 = 24.0; // Padding for cell borders and internal spacing
    const MIN_WIDTH: f32 = 80.0; // Minimum usable column width
    const MAX_WIDTH: f32 = 400.0; // Maximum width (prevents one column dominating)

    // Start with header length as baseline
    let header_len = col_name.len();

    // Sample content length from the column
    let content_max_len = if let Ok(col) = df.column(col_name) {
        // Don't sample more rows than exist
        let sample_count = sample_size.min(col.len());
        let mut max_len = 0usize;

        // Check each sample value's string representation length
        for i in 0..sample_count {
            if let Ok(val) = col.get(i) {
                // Format the value as it would appear in the UI
                let val_str = format!("{}", val);
                max_len = max_len.max(val_str.len());
            }
        }
        max_len
    } else {
        0
    };

    // Use whichever is longer: header or content
    let max_chars = header_len.max(content_max_len);

    // Calculate pixel width
    let width = (max_chars as f32 * CHAR_WIDTH) + PADDING;

    width.clamp(MIN_WIDTH, MAX_WIDTH)
}

/// Reads a CSV file from disk into a Polars DataFrame.
///
/// Uses Polars' `CsvReadOptions` for efficient, configurable CSV parsing.
///
/// # Configuration
///
/// - `with_has_header(true)` - First row contains column names
/// - `with_infer_schema_length(Some(1000))` - Sample 1000 rows row type inference
///
/// # Parameters
///
/// - `path` - File system path to the CSV file
///
/// # Returns
///
/// - `Ok(DataFrame)` - Successfully parsed DataFrame
/// - `Err(FileError)` - If file doesn't exist, can't be read, or parsing fails
///
/// # Type Inference
///
/// Polars samples the first N rows (1000 here) to infer column types.
/// This means a column that looks like integers in the first 1000 rows
/// but has string later might cause issues. We use generous sample size.
fn read_csv_file(path: &str) -> Result<DataFrame, FileError> {
    let path = Path::new(path);

    // Check if the file exists before attempting to read
    // This gives a clearer error message than a generic I/O error
    if !path.exists() {
        return Err(FileError::NotFound(path.display().to_string()));
    }

    // Configure and execute CSV reading
    CsvReadOptions::default()
        // First row is headers (column names)
        .with_has_header(true)
        // Sample 1000 rows for type inference (balance between accuracy and speed)
        .with_infer_schema_length(Some(1000))
        // Create a reader with the file path
        .try_into_reader_with_file_path(Some(path.into()))
        .map_err(|e| FileError::ReadError(e.to_string()))?
        // Execute the read and parse
        .finish()
        .map_err(|e| FileError::ParseError(e.to_string()))
}

/// Extract column metadata from a DataFrame.
///
/// Iterates through all columns and builds a `ColumnInfo` struct for each,
/// containing the information the UI needs to render column headers and
/// display column details in the sidebar.
///
/// # Parameters
///
/// - `df` - Reference to the DataFrame to analyze
///
/// # Returns
///
/// Vector of `ColumnInfo` structs, one per column, in teh column order.
fn extract_column_info(df: &DataFrame) -> Vec<ColumnInfo> {
    df.get_columns()
        .iter()
        .map(|col| {
            // Count null values (useful for data quality display)
            let null_count = col.null_count();

            // Calculate optimal display width for this column
            let width = calculate_column_width(df, col.name().as_str(), 100);

            ColumnInfo {
                name: col.name().to_string(),
                // Format dtype using Debug trait (gives "Int64", "Float64", etc.)
                dtype: format!("{:?}", col.dtype()),
                null_count,
                width,
            }
        })
        .collect()
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

/// Loads a CSV file into application state and returns file metadata.
///
/// This is the main command for opening file. It:
/// 1. Emits `app:loading` event (loading started)
/// 2. Reads file metadata (size)
/// 3. Parses the CSV using Polars
/// 4. Extracts column information
/// 5. Stores the DataFrame in application state
/// 6. Emits `file:loaded` event with FileInfo
/// 7. Emits `app:loading` event (loading ended)
/// 8. Returns `FileInfo` for UI to render
///
/// # Parameters
///
/// - `app` - Tauri AppHandle for emitting events
/// - `path` - Full file system path to the CSV file
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Ok(FileInfo)` - File successfully loaded, returns metadata
/// - `Err(FileError)` - Loading failed, returns error message
///
/// # Events Emitted
///
/// - `app:loading { is_loading: true, message: "Loading file..." }` - On start
/// - `file:loaded { file_info: FileInfo }` - On success
/// - `app:loading { is_loading: false, message: null }` - On complete
/// - `app:error { code, message }` - On error (before returning Err)
///
/// # Frontend Usage
///
/// The frontend can either:
/// 1. Listen to `file:loaded` event (reactive/event-driven approach)
/// 2. Await the returned `FileInfo` (imperative approach)
///
/// ```typescript
/// // Option 1: Event-driven (preferred for Rust Supremacy)
/// useRustEvent("file:loaded", (payload) => setFileInfo(payload.file_info));
/// await invoke("load_file", { path: "/path/to/data.csv" });
///
/// // Option 2: Imperative (still works)
/// const info = await invoke<FileInfo>("load_file", { path: "/path/to/data.csv" });
/// setFileInfo(info);
/// ```
///
/// # State Updates
///
/// This command updates two pieces of state:
/// 1. `dataframe` - Stores the loaded `DataFrame` and `FileInfo`
/// 2. `ui_state.column_widths` - Initialize column widths from auto-calculation
#[tauri::command]
pub async fn load_file(
    app: AppHandle,
    path: String,
    state: State<'_, AppState>,
) -> Result<FileInfo, FileError> {
    // Emit loading started event
    app.emit_loading(true, Some("Loading file..."));

    let file_path = Path::new(&path);

    // Get file metadata (primarily for file size display)
    let metadata = fs::metadata(file_path).map_err(|e| {
        let error = FileError::MetadataError(e.to_string());
        app.emit_error(error_codes::FILE_METADATA_ERROR, &error.to_string());
        app.emit_loading(false, None);
        error
    })?;

    // Extract just the filename (without path) for display
    let file_name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Emit progress update
    app.emit_loading(true, Some("Parsing CSV..."));

    // Read and parse the CSV file
    let df = read_csv_file(&path).inspect_err(|e| {
        let error_code = match e {
            FileError::NotFound(_) => error_codes::FILE_NOT_FOUND,
            FileError::ReadError(_) => error_codes::FILE_READ_ERROR,
            FileError::ParseError(_) => error_codes::FILE_PARSE_ERROR,
            FileError::MetadataError(_) => error_codes::FILE_METADATA_ERROR,
        };
        app.emit_error(error_code, &e.to_string());
        app.emit_loading(false, None);
    })?;

    // Extract column metadata with auto-calculated widths
    let columns = extract_column_info(&df);
    let column_widths = columns.iter().map(|c| c.width).collect();

    // Build FileInfo struct
    let file_info = FileInfo {
        path: path.clone(),
        name: file_name,
        size_bytes: metadata.len(),
        row_count: df.height(),   // Number of rows in the DataFrame
        column_count: df.width(), // Number of columns in the DataFrame
        columns,
    };

    // Store DataFrame in application state
    // Using a block to limit the scope of the write lock
    {
        let mut df_guard = state.dataframe.write();
        *df_guard = Some(LoadedDataFrame {
            df,
            file_info: file_info.clone(), // Clone because we return it
        });
    }

    // Update UI state with calculated column widths
    {
        let mut ui_guard = state.ui_state.write();
        ui_guard.column_widths = column_widths;
    }

    // Emit file loaded event (frontend can react to this)
    app.emit_file_loaded(file_info.clone());

    // Emit loading complete
    app.emit_loading(false, None);

    Ok(file_info)
}

/// Returns metadata for currently loaded file, if any
///
/// This command allows the frontend to query file info without
/// re-loading the file. Useful for refreshing UI state.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Some(FileInfo)` - If a file is currently loaded
/// - `None` - If no file is loaded
///
/// # Note
///
/// This is a synchronous command (not `async`) because it only
/// reads from memory with no I/O operations
#[tauri::command]
pub fn get_file_info(state: State<'_, AppState>) -> Option<FileInfo> {
    // Acquire read lock on dataframe state
    let guard = state.dataframe.read();
    // Map the Option<LoadedDataFrame> to Option<FileInfo>
    guard.as_ref().map(|loaded| loaded.file_info.clone())
}
