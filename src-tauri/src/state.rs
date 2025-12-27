//! Application State Management
//!
//! This module defines the core state structures that hold all application data.
//! Following the "Rust Supremacy" principle, all state lives here in Rust - the
//! TypeScript frontend is purely a renderer with no business logic.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                              AppState                                       │
//! ├─────────────────────────────────┬───────────────────────────────────────────┤
//! │  dataframe: RwLock              │  processed_dataframe: RwLock              │
//! │  ┌────────────────────────────┐ │  ┌────────────────────────────┐           │
//! │  │ LoadedDataFrame            │ │  │ LoadedDataFrame            │           │
//! │  │ - df: DataFrame            │ │  │ - df: DataFrame (cleaned)  │           │
//! │  │ - file_info: FileInfo      │ │  │ - file_info: FileInfo      │           │
//! │  └────────────────────────────┘ │  └────────────────────────────┘           │
//! ├─────────────────────────────────┼───────────────────────────────────────────┤
//! │  ui_state: RwLock               │  preprocessing_history: RwLock            │
//! │  ┌────────────────────────────┐ │  ┌────────────────────────────┐           │
//! │  │ UIState                    │ │  │ Vec<HistoryEntry> (max 10) │           │
//! │  │ - sidebar_width            │ │  │ - id, timestamp            │           │
//! │  │ - column_widths            │ │  │ - config, summary          │           │
//! │  │ - grid_scroll              │ │  └────────────────────────────┘           │
//! │  └────────────────────────────┘ │                                           │
//! ├─────────────────────────────────┼───────────────────────────────────────────┤
//! │  ai_provider_config: RwLock     │  preprocessing_token: RwLock              │
//! │  ┌────────────────────────────┐ │  ┌────────────────────────────┐           │
//! │  │ AIProviderConfig           │ │  │ CancellationToken          │           │
//! │  │ - provider: AIProviderType │ │  │ (thread-safe cancel)       │           │
//! │  │ - api_key: String          │ │  └────────────────────────────┘           │
//! │  └────────────────────────────┘ │                                           │
//! ├─────────────────────────────────┼───────────────────────────────────────────┤
//! │  last_preprocessing_result:     │  theme: RwLock<Theme>                     │
//! │  RwLock<Option<Summary>>        │  (System | Light | Dark)                  │
//! │  (persists across navigation)   │                                           │
//! └─────────────────────────────────┴───────────────────────────────────────────┘
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
//!
//! # Session-Only State
//!
//! Some state is intentionally session-only (not persisted to disk):
//! - `ai_provider_config` - API keys should not be stored
//! - `preprocessing_history` - Transient processing history
//! - `processed_dataframe` - Can be regenerated from source

use lex_processing::{CancellationToken, PipelineConfig, PreprocessingSummary};
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
// SETTINGS STATE
// ============================================================================

/// Supported AI provider types for preprocessing decisions.
///
/// The AI provider is used during preprocessing to make intelligent
/// decisions about how to handle data quality issues (e.g., which
/// imputation method to use for a column).
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::AIProviderType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AIProviderType {
    /// No AI provider - use rule-based decisions only
    #[default]
    None,
    /// OpenRouter API (supports multiple LLM models)
    OpenRouter,
    /// Google Gemini API
    Gemini,
}

/// Configuration for an AI provider.
///
/// Stores the provider type and API key for use in preprocessing.
/// Note: API keys are stored in memory only (session-only, not persisted).
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::AIProviderConfig`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderConfig {
    /// The type of AI provider
    pub provider: AIProviderType,
    /// API key for the provider (session-only, not persisted)
    pub api_key: String,
}

/// Application theme setting.
///
/// Controls the visual appearance of the application.
/// Defaults to System, which follows the OS preference.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::Theme`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Follow the operating system's theme preference
    #[default]
    System,
    /// Always use light theme
    Light,
    /// Always use dark theme
    Dark,
}

// ============================================================================
// PREPROCESSING STATE
// ============================================================================

/// A snapshot of preprocessing configuration for history entries.
///
/// This captures the configuration used for a preprocessing run so it
/// can be displayed in the history and potentially reused.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::PreprocessingConfigSnapshot`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingConfigSnapshot {
    /// Columns that were selected for preprocessing
    pub selected_columns: Vec<String>,
    /// Row range that was processed (start, end indices)
    pub row_range: Option<(usize, usize)>,
    /// Threshold for dropping columns with too many missing values (0.0-1.0)
    pub missing_column_threshold: f64,
    /// Threshold for dropping rows with too many missing values (0.0-1.0)
    pub missing_row_threshold: f64,
    /// Strategy used for handling outliers
    pub outlier_strategy: String,
    /// Method used for imputing numeric missing values
    pub numeric_imputation: String,
    /// Method used for imputing categorical missing values
    pub categorical_imputation: String,
    /// Whether type correction was enabled
    pub enable_type_correction: bool,
    /// Whether duplicate removal was enabled
    pub remove_duplicates: bool,
    /// Number of neighbors used for KNN imputation
    pub knn_neighbors: usize,
    /// Whether AI-guided decisions were used
    pub use_ai_decisions: bool,
    /// Target column if specified
    pub target_column: Option<String>,
}

impl From<&PipelineConfig> for PreprocessingConfigSnapshot {
    /// Creates a config snapshot from a `PipelineConfig`.
    ///
    /// Note: `selected_columns` and `row_range` must be set separately
    /// as they are not part of `PipelineConfig`.
    fn from(config: &PipelineConfig) -> Self {
        Self {
            selected_columns: Vec::new(), // Set separately
            row_range: None,              // Set separately
            missing_column_threshold: config.missing_column_threshold,
            missing_row_threshold: config.missing_row_threshold,
            outlier_strategy: format!("{:?}", config.outlier_strategy),
            numeric_imputation: format!("{:?}", config.default_numeric_imputation),
            categorical_imputation: format!("{:?}", config.default_categorical_imputation),
            enable_type_correction: config.enable_type_correction,
            remove_duplicates: config.remove_duplicates,
            knn_neighbors: config.knn_neighbors,
            use_ai_decisions: config.use_ai_decisions,
            target_column: config.target_column.clone(),
        }
    }
}

/// An entry in the preprocessing history.
///
/// Each time preprocessing is run, an entry is created to allow users
/// to view past results and reload previous processed datasets.
///
/// # Session-Only Storage
///
/// History is stored in memory only. Maximum 10 entries are kept,
/// with oldest entries removed when the limit is exceeded.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::PreprocessingHistoryEntry`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingHistoryEntry {
    /// Unique identifier for this history entry (UUID)
    pub id: String,
    /// Unix timestamp when preprocessing was completed
    pub timestamp: i64,
    /// Configuration used for this preprocessing run
    pub config: PreprocessingConfigSnapshot,
    /// Summary of what the preprocessing accomplished
    pub summary: PreprocessingSummary,
}

/// Maximum number of preprocessing history entries to keep.
pub const MAX_HISTORY_ENTRIES: usize = 10;

// ============================================================================
// PREPROCESSING UI STATE
// ============================================================================

/// UI state for the preprocessing page.
///
/// This captures the user's selections on the preprocessing page so they
/// persist across navigation. Stored in memory only (session-only).
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::PreprocessingUIState`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreprocessingUIState {
    /// Selected column names for preprocessing
    pub selected_columns: Vec<String>,
    /// Row range to process (start, end indices), or None for all rows
    pub row_range: Option<(usize, usize)>,
    /// Pipeline configuration settings
    pub config: PreprocessingUIConfig,
    /// Active tab in the results panel ("results" or "history")
    pub active_results_tab: String,
}

/// Pipeline configuration for the preprocessing UI.
///
/// Mirrors the frontend PipelineConfigRequest type.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::PipelineConfigRequest`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingUIConfig {
    /// Threshold for dropping columns with too many missing values (0.0-1.0)
    pub missing_column_threshold: f64,
    /// Threshold for dropping rows with too many missing values (0.0-1.0)
    pub missing_row_threshold: f64,
    /// Strategy for handling outliers
    pub outlier_strategy: String,
    /// Method for imputing numeric missing values
    pub numeric_imputation: String,
    /// Method for imputing categorical missing values
    pub categorical_imputation: String,
    /// Whether to enable type correction
    pub enable_type_correction: bool,
    /// Whether to remove duplicate rows
    pub remove_duplicates: bool,
    /// Number of neighbors for KNN imputation
    pub knn_neighbors: usize,
    /// Whether to use AI-guided decisions
    pub use_ai_decisions: bool,
    /// Target column for ML task detection
    pub target_column: Option<String>,
}

impl Default for PreprocessingUIConfig {
    fn default() -> Self {
        Self {
            missing_column_threshold: 0.7,
            missing_row_threshold: 0.5,
            outlier_strategy: "cap".to_string(),
            numeric_imputation: "median".to_string(),
            categorical_imputation: "mode".to_string(),
            enable_type_correction: true,
            remove_duplicates: true,
            knn_neighbors: 5,
            use_ai_decisions: true, // Smart mode on by default
            target_column: None,
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
///   `parking_lot::RwLock` is faster than `std::sync::RwLock`
///
/// # Usage in Commands
///
/// ```rust,ignore
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
/// * `ai_provider_config` - AI provider configuration (session-only)
/// * `preprocessing_token` - Cancellation token for running preprocessing
/// * `preprocessing_history` - History of preprocessing runs (max 10, session-only)
/// * `processed_dataframe` - The most recent preprocessed DataFrame
/// * `last_preprocessing_result` - Summary from the most recent preprocessing run
/// * `theme` - Application theme setting
pub struct AppState {
    /// Currently loaded `DataFrame` with metadata.
    /// `None` when no file is loaded, `Some(LoadedDataFrame)` after loading
    pub dataframe: RwLock<Option<LoadedDataFrame>>,

    /// UI layout state (sidebar width, column widths).
    /// Always has a value (initialized with defaults).
    pub ui_state: RwLock<UIState>,

    /// AI provider configuration for preprocessing decisions.
    /// `None` means no AI provider is configured (rule-based only).
    /// Session-only: not persisted to disk.
    pub ai_provider_config: RwLock<Option<AIProviderConfig>>,

    /// Cancellation token for the currently running preprocessing pipeline.
    /// Can be used to cancel preprocessing from the UI.
    pub preprocessing_token: RwLock<CancellationToken>,

    /// History of preprocessing runs.
    /// Maximum [`MAX_HISTORY_ENTRIES`] entries, oldest removed first.
    /// Session-only: not persisted to disk.
    pub preprocessing_history: RwLock<Vec<PreprocessingHistoryEntry>>,

    /// The most recently preprocessed `DataFrame` with metadata.
    /// `None` until preprocessing is run, then contains the cleaned data.
    pub processed_dataframe: RwLock<Option<LoadedDataFrame>>,

    /// Summary from the most recent preprocessing run.
    /// Persists across navigation until user dismisses or starts new run.
    /// Separate from history - dismissing this doesn't clear history.
    pub last_preprocessing_result: RwLock<Option<PreprocessingSummary>>,

    /// Application theme setting (System, Light, or Dark).
    /// Defaults to System (follows OS preference).
    pub theme: RwLock<Theme>,

    /// UI state for the preprocessing page.
    /// Persists selected columns, row range, and config across navigation.
    /// Session-only: not persisted to disk.
    pub preprocessing_ui_state: RwLock<PreprocessingUIState>,
}

impl AppState {
    /// Creates a new `AppState` with no loaded `DataFrame` and default settings.
    pub fn new() -> Self {
        Self {
            dataframe: RwLock::new(None),
            ui_state: RwLock::new(UIState::default()),
            ai_provider_config: RwLock::new(None),
            preprocessing_token: RwLock::new(CancellationToken::new()),
            preprocessing_history: RwLock::new(Vec::new()),
            processed_dataframe: RwLock::new(None),
            last_preprocessing_result: RwLock::new(None),
            theme: RwLock::new(Theme::default()),
            preprocessing_ui_state: RwLock::new(PreprocessingUIState::default()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
