//! Application State Management
//!
//! This module defines the core state structures that hold all application data.
//! Following the "Rust Supremacy" principle, all state lives here in Rust - the
//! TypeScript frontend is purely a renderer with no business logic.
//!
//! # Architecture
//!
//! ```text
//! -------------------------------------------------------------------
//! |                        AppState                                 |
//! |  ---------------------------  -------------------------------   |
//! |  |  dataframe: RwLock      |  |  ui_state: RwLock           |   |
//! |  |  ┌───────────────────┐  |  |  ┌───────────────────────┐  |   |
//! |  |  | LoadedDataFrame   |  |  |  | UIState               |  |   |
//! |  |  | - df: DataFrame   |  |  |  | - sidebar_width       |  |   |
//! |  |  | - file_info       |  |  |  | - column_widths       |  |   |
//! |  |  └───────────────────┘  |  |  | - grid_scroll         |  |   |
//! |  |                         |  |  └───────────────────────┘  |   |
//! |  ---------------------------  -------------------------------   |
//! -------------------------------------------------------------------
//! ```
//!
//! # Thread Safety
//!
//! All state is wrapped in `RwLock` from `parking_lot` (faster than std).
//! This allows safe concurrent access from multiple Tauri command handlers.
//!
//! # Serialization
//!
//! Structs with `#[derive(Serialize, Deserialize)]` are automatically
//! converted to/from JSON when passed to the TypeScript frontend via Tauri IPC.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

// ============================================================================
// COLUMN & FILE METADATA
// ============================================================================

/// Information about a single column in the DataFrame.
///
/// This struct is serialized to JSON and sent to the frontend when a file
/// is loaded. It contains everything the UI needs to render a column header
/// and display column info in the sidebar.
///
/// # Fields
///
/// * `name` - Column name from the CSV header (or auto-generated like "column_1")
/// * `dtype` - Polars data type as string ("Int64", "Float64", String)
/// * `null_count` - Number of null/missing values (useful for data quality display)
/// * `width` - Suggested display width in pixels (auto-calculated based on content)
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::ColumnInfo`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub dtype: String,
    pub null_count: usize,
    pub width: f32,
}

/// Metadata about a loaded CSV file.
///
/// Returned by the `load_file` command after successfully loading a CSV.
/// Contains everything the UI needs to render the grid structure and sidebar
/// without having to query for individual pieces.
///
/// # Fields
///
/// * `path` - Full absolute path to the file (for display in sidebar)
/// * `name` - Just the filename without path (e.g., "data.csv")
/// * `size_bytes` - File size in bytes (for display as "1.2 MB" etc.)
/// * `row_count` - Total number of rows (for virtual scroll and status bar)
/// * `column_count` - Total number of columns
/// * `columns` - Detailed info for each column
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::FileInfo`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub row_count: usize,
    pub column_count: usize,
    pub columns: Vec<ColumnInfo>,
}

// ============================================================================
// DATAFRAME CONTAINER
// ============================================================================

/// Container for a loaded DataFrame and it's associated metadata.
///
/// This struct bundles the Polars `DataFrame` with its `FileInfo` so we can
/// access both from a single state lookup. The `DataFrame` itself is not
/// serializeable (too large, complex structure), but `FileInfo` is.
///
/// # Why Bundle Together?
///
/// When we load a file, we compute `FileInfo` once and store it alongside
/// the `DataFrame`. This avoids re-computing metadata on every request.
///
/// # Fields
///
/// * `df` - The actual Polars `DataFrame` (not serialized)
/// * `file_info` - Cached metadata about the file/columns
pub struct LoadedDataFrame {
    /// The Polars `DataFrame` containing all the data.
    /// This is where the actual CSV data lives in memory.
    pub df: polars::prelude::DataFrame,

    /// Cached File metadata.
    /// Computed once when loading, served from cache thereafter.
    pub file_info: FileInfo,
}

// ============================================================================
// UI STATE
// ============================================================================

/// Grid scroll position state.
///
/// Tracks the current scroll position of the data grid so it can be
/// restored when navigating between pages or reloading.
///
/// # Fields
///
/// * `row_index` - Current top row index (vertical scroll position)
/// * `scroll_left` - Horizontal scroll offset in pixels
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::GridScrollPosition`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GridScrollPosition {
    pub row_index: usize,
    pub scroll_left: f32,
}

/// UI layout state that persists across interactions.
///
/// This captures the user preference for UI layout that should be remembered.
/// Currently in-memory only.
///
/// # Fields
///
/// * `sidebar_width` - Width of the right sidebar in pixels
/// * `column_widths` - Vector of column widths in pixels (one per column)
/// * `grid_scroll` - Current scroll position of the data grid
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::UIState`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIState {
    pub sidebar_width: f32,
    pub column_widths: Vec<f32>,
    pub grid_scroll: GridScrollPosition,
}

impl Default for UIState {
    /// Creates a default UI state with reasonable initial values.
    ///
    /// - Sidebar width: 280px
    /// - Column widths: empty (populated when a file loads)
    /// - Grid scroll: top-left (0, 0)
    fn default() -> Self {
        Self {
            sidebar_width: 280.0,
            column_widths: Vec::new(),
            grid_scroll: GridScrollPosition::default(),
        }
    }
}

// ============================================================================
// APPLICATION STATE
// ============================================================================

/// Global Application State - the single source of truth.
///
/// This struct is managed by Tauri and injected into all command handlers
/// via the `State` extractor. It contains all application data.
///
/// # Thread Safety
///
/// All fields are wrapped in `RwLock` for safe concurrent access:
/// - Multiple readers can access simultaneously
/// - Writers get exclusive access
/// `parking_lot::RwLock` is faster than `std::sync::RwLock`
///
/// # Usage in Commands
///
/// ```rust
/// #[tauri::command]
/// async fn my_command(state: State<'_, AppState>) -> Result<(), String> {
///     let df_guard = state.dataframe.read();
///     // ... use of df_guard...
/// }
/// ```
///
/// # Fields
///
/// * `dataframe` - The currently loaded `DataFrame` (or none if nothing is loaded)
/// * `ui_state` - User's UI layout preferences
pub struct AppState {
    /// Currently loaded `DataFrame` with metadata.
    /// `None` when no file is loaded, `Some(LoadedDataFrame)` after loading
    pub dataframe: RwLock<Option<LoadedDataFrame>>,

    /// UI layout state (sidebar width, column widths).
    /// Always has a value (initialized with defaults).
    pub ui_state: RwLock<UIState>,
}

impl AppState {
    /// Creates a new `AppState` with no loaded `DataFrame` and default UI settings.
    pub fn new() -> Self {
        Self {
            dataframe: RwLock::new(None),
            ui_state: RwLock::new(UIState::default()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
