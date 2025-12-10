//! DataFrame Operations Commands
//!
//! This module provides commands for working with loaded DataFrames:
//! - Fetching rows for virtual scrolling
//! - Close files and releasing memory
//!
//! # Virtual Scrolling
//!
//! The `get_rows` command is the heart of our virtual scrolling implementation.
//! Instead of sending all rows to the frontend (which could be millions),
//! We only send the rows currently visible in the viewport.
//!
//! ```text
//! DataFrame (1,000,000 rows)
//! ------------------------------
//! | Row 0                      |
//! | ...                        |
//! | Row 499,999                | ← User scrolls here
//! | ─────────────────────────  |
//! | | Row 500,000           |  | ← Visible in viewport
//! | | Row 500,001           |  |
//! | | ...                   |  |
//! | | Row 500,049           |  |
//! | ─────────────────────────  |
//! | Row 500,050                |
//! | ...                        |
//! | Row 999,999                |
//! ------------------------------
//!
//! Frontend requests: get_rows(start=499,990, count=70)
//! (includes buffer rows above and below viewport)
//! ```
//!
//! # Events Emitted
//!
//! - `file:closed` - When a file is closed

use polars::prelude::AnyValue;
use serde::Serialize;
use serde_json::{Number, Value};
use tauri::{AppHandle, State};

use crate::events::AppEventEmitter;
use crate::state::AppState;

// ============================================================================
// TYPES
// ============================================================================

/// A single row of cell values.
///
/// Each cell is a `serde_json::Value` which can represent:
/// - Null
/// - Boolean
/// - Number (integer or float)
/// - String
///
/// Using JSON values to ensure type-safe serialization to the frontend.
pub type Row = Vec<serde_json::Value>;

/// Response containing rows for virual scrolling.
///
/// This struct is returned by `get_rows` and contains everything
/// the frontend needs to render the visible portion of the grid.
///
/// # Fields
///
/// * `rows` - 2D array of cell values: rows[rowIndex][colIndex]
/// * `start` - The starting row index (0-indexed) of this batch
/// * `total_rows` - Total rows in the dataset (for scrollbar calculation)
///
/// # Frontend Usage
///
/// ```typescript
/// const response = await invoke<RowsResponse>("get_rows", { start: 500000, count: 70 });
/// setRows(response.rows);
/// setVisibleStart(response.start);
/// // total_rows used for scrollbar height calculation
/// ```
#[derive(Debug, Serialize)]
pub struct RowsResponse {
    pub rows: Vec<Row>,
    pub start: usize,
    pub total_rows: usize,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Converts a Polars `AnyValue` to a JSON (`json_serde`) `Value`.
/// Polars uses `AnyValue` as a type-erased wrapper for cell values.
/// We need to convert these to JSON for serialization to the frontend.
///
/// # Type Mapping
///
/// | Polars Type | JSON Type |
/// ---------------------------
/// | Null        | null      |
/// | Boolean     | boolean   |
/// | Int8-64     | number    |
/// | UInt8-64    | number    |
/// | Float32/64  | number    |
/// | String      | string    |
/// | Other       | string    |
///
/// # Edge Cases
///
/// - NaN and Infinity floats become `null` (JSON doesn't support them)
/// - Complex types (Date, List, Struct) are stringified with `format!`
///
/// # Parameters
///
/// - `value` - The Polars `AnyValue` to convert
///
/// # Returns
///
/// A `serde::Value` suitable for JSON serialization.
fn any_value_to_json(value: AnyValue) -> serde_json::Value {
    match value {
        // Null
        AnyValue::Null => Value::Null,

        // Boolean
        AnyValue::Boolean(b) => Value::Bool(b),

        // Signed integers (all fit in JSON number)
        AnyValue::Int8(i) => Value::Number(i.into()),
        AnyValue::Int16(i) => Value::Number(i.into()),
        AnyValue::Int32(i) => Value::Number(i.into()),
        AnyValue::Int64(i) => Value::Number(i.into()),

        // Unsigned integers (all fit in JSON number)
        AnyValue::UInt8(u) => Value::Number(u.into()),
        AnyValue::UInt16(u) => Value::Number(u.into()),
        AnyValue::UInt32(u) => Value::Number(u.into()),
        AnyValue::UInt64(u) => Value::Number(u.into()),

        // Floatin point (may fail for NaN/Infinity)
        AnyValue::Float32(f) => Number::from_f64(f as f64)
            .map(Value::Number)
            .unwrap_or(Value::Null), // NaN/Infinity becomes null

        AnyValue::Float64(f) => Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),

        // Strings (both borrowed and owned)
        AnyValue::String(s) => Value::String(s.to_string()),
        AnyValue::StringOwned(s) => Value::String(s.to_string()),

        // Everything else: stringify using Display trait
        // This handles Data, DateTime, Duration, List, Struct, etc.
        _ => Value::String(format!("{}", value)),
    }
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

/// Fetches rows from the loaded `DataFrame` for virtual scrolling.
///
/// This is the core command for virtual scrolling. The frontend calls this
/// whenever the user scrolls to a region that doesn't have loaded data.
///
/// # Parameters
///
/// - `start` - Starting row index (0-indexed)
/// - `count` - Number of rows to fetch
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Some(RowsRespone) - Rows fetched successfully
/// - `None` - No file is loaded
///
/// # Bounds Handling
///
/// The command handles edge cases gracefully:
/// - `start` beyond end of data -> clamped to last valid row
/// - `count` exceeding remaining rows -> returns only available rows
/// - Empty -> returns `RowsResponse` with empty `row` vector
///
/// # Performance
/// - Uses Polars `slice()`for O(1) row access (no copying)
/// - Only converts requested rows to JSON (not entire `DataFrame`)
/// - Typical request: 50-100 rows = sub0millisecond response time
///
/// # Example
///
/// ```rust
/// // Frontend requests rows 100-149 (50 rows)
/// get_rows(100, 50, state)
/// // Returns `RowsResponse { rows: [..50 row...], start: 100, total_rows: 1,000,000 }`
/// ```
#[tauri::command]
pub fn get_rows(start: usize, count: usize, state: State<'_, AppState>) -> Option<RowsResponse> {
    let guard = state.dataframe.read();
    let loaded = guard.as_ref()?; // Returns None if no file is loaded

    let df = &loaded.df;
    let total_rows = df.height();

    // Clamp start to a valid range (0 to total_rows-1)
    // saturating_sub prevents underflow when total_rows is 0
    let start = start.min(total_rows.saturating_sub(1));

    // Calculate how many rows we can actually return
    let available = total_rows.saturating_sub(start);
    let actual_count = count.min(available);

    // Handle empty result case
    if actual_count == 0 {
        return Some(RowsResponse {
            rows: vec![],
            start,
            total_rows,
        });
    }

    // Slice the DataFrame to get only the requested rows
    // Polars slice is O(1) - it creates a view, not a copy
    let sliced = df.slice(start as i64, actual_count);

    // Convert DataFrame rows to JSON-serialized format
    let mut rows = Vec::with_capacity(actual_count);

    // Iterate through each row in the slice
    for row_idx in 0..sliced.height() {
        let mut row = Vec::with_capacity(sliced.width());

        // Iterate through each column to build the row
        for col in sliced.get_columns() {
            // Get the cell value and convert to JSON
            // If get() fails (shouldn't happen), use null
            let value = col.get(row_idx).ok().map_or(Value::Null, any_value_to_json);
            row.push(value);
        }
        rows.push(row);
    }

    Some(RowsResponse {
        rows,
        start,
        total_rows,
    })
}

/// Closes the currently loaded file and frees memory.
///
/// This command:
/// 1. Drops the `DataFrame` from state (frees memory)
/// 2. Clears the column widths from UI state
/// 3. Emits `file:closed` event to notify frontend
///
/// # Parameters
///
/// - `app` - Tauri AppHandle for emitting events
/// - `state` - Tauri-managed application state
///
/// # Events Emitted
///
/// - `file:closed` - After the file is closed and memory freed
///
/// # Memory Management
///
/// When we set `*df_guard = None` Rust's ownership system automatically
/// drops the previous `LoadedDataFrame`, which in turn drops the Polars
/// `DataFrame`, freeing all associated memory.
///
/// # Frontend usage
///
/// The frontend can either:
/// 1. Listen to `file:closed` event (reactive/event-driven approach)
/// 2. Clear state after awaiting the command (imperative approach)
///
/// ```typescript
/// // Option 1: Event-driven (preferred)
/// useRustEvent("file:closed", () => {
///   setFileInfo(null);
///   setRows([]);
/// });
/// await invoke("close_file");
///
/// // Option 2: Imperative
/// await invoke("close_file");
/// setFileInfo(null);
/// setRows([]);
/// ```
#[tauri::command]
pub fn close_file(app: AppHandle, state: State<'_, AppState>) {
    // Clear the DataFrame (releases memory)
    {
        let mut df_guard = state.dataframe.write();
        *df_guard = None; // Previous value is dropped here
    }

    // Clear column widths from UI state
    {
        let mut ui_guard = state.ui_state.write();
        ui_guard.column_widths = Vec::new();
    }

    // Emit file closed event
    app.emit_file_closed();
}
