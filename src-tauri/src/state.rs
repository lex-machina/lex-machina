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

use lex_learning::{CancellationToken as MLCancellationToken, TrainingResult};
use lex_processing::{
    CancellationToken, ColumnProfile, DataQualityIssue, DatasetProfile, PipelineConfig,
    PreprocessingSummary,
};
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
/// * `sidebar_collapsed` - Whether the sidebar is collapsed (vertical nav only)
/// * `column_widths` - Vector of column widths in pixels (one per column)
/// * `grid_scroll` - Current scroll position of the data grid
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::UIState`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIState {
    pub sidebar_width: f32,
    pub sidebar_collapsed: bool,
    pub column_widths: Vec<f32>,
    pub grid_scroll: GridScrollPosition,
}

impl Default for UIState {
    /// Creates a default UI state with reasonable initial values.
    ///
    /// - Sidebar width: 280px
    /// - Sidebar collapsed: false (expanded by default)
    /// - Column widths: empty (populated when a file loads)
    /// - Grid scroll: top-left (0, 0)
    fn default() -> Self {
        Self {
            sidebar_width: 280.0,
            sidebar_collapsed: false,
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

/// Navigation bar position setting.
///
/// Controls where the navigation bar is positioned in the UI.
/// Defaults to Merged, which combines nav with the right sidebar.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::NavBarPosition`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NavBarPosition {
    /// Navigation bar on the left side (vertical, always visible)
    Left,
    /// Navigation bar on the right side (vertical, always visible)
    Right,
    /// Navigation merged with right sidebar (horizontal when expanded, vertical when collapsed)
    #[default]
    Merged,
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
// ML STATE TYPES
// ============================================================================

/// UI state for the ML page.
///
/// This captures the user's ML configuration so it persists across navigation.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::MLUIState`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MLUIState {
    /// true = automatic mode, false = manual mode
    pub smart_mode: bool,
    /// Selected target column for prediction
    pub target_column: Option<String>,
    /// Problem type ("classification" or "regression")
    pub problem_type: String,
    /// Columns to exclude from features
    pub excluded_columns: Vec<String>,
    /// Whether to use processed or original data
    pub use_processed_data: bool,
    /// ML configuration settings
    pub config: MLConfigUIState,
    /// Active tab in the results panel
    pub active_tab: String,
}

/// ML configuration UI state.
///
/// Mirrors the frontend MLConfigUIState type.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::MLConfigUIState`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfigUIState {
    /// Whether to optimize hyperparameters with Optuna
    pub optimize_hyperparams: bool,
    /// Number of Optuna trials for hyperparameter optimization
    pub n_trials: u32,
    /// Number of cross-validation folds
    pub cv_folds: u32,
    /// Train/test split ratio (0.0-1.0)
    pub test_size: f64,
    /// Whether to include neural networks in model selection
    pub enable_neural_networks: bool,
    /// Whether to compute SHAP explainability plots
    pub enable_explainability: bool,
    /// Number of top algorithms to compare
    pub top_k_algorithms: u32,
    /// Optional algorithm override
    pub algorithm: Option<String>,
}

impl Default for MLConfigUIState {
    fn default() -> Self {
        Self {
            optimize_hyperparams: true,
            n_trials: 10,
            cv_folds: 5,
            test_size: 0.2,
            enable_neural_networks: false,
            enable_explainability: true,
            top_k_algorithms: 3,
            algorithm: None,
        }
    }
}

// ============================================================================
// ANALYSIS STATE TYPES
// ============================================================================

/// Dataset selection for analysis results.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::AnalysisDataset`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisDataset {
    Original,
    Processed,
}

/// UI state for the analysis page.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::AnalysisUIState`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisUIState {
    /// Whether to use processed data for analysis
    pub use_processed_data: bool,
    /// Active tab in the analysis workspace
    pub active_tab: String,
    /// Selected column for focused analysis
    pub selected_column: Option<String>,
}

impl Default for AnalysisUIState {
    fn default() -> Self {
        Self {
            use_processed_data: false,
            active_tab: "overview".to_string(),
            selected_column: None,
        }
    }
}

/// High-level summary of the dataset analysis.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::AnalysisSummary`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub rows: usize,
    pub columns: usize,
    pub memory_bytes: u64,
    pub duplicate_count: usize,
    pub duplicate_percentage: f64,
    pub total_missing_cells: usize,
    pub total_missing_percentage: f64,
    pub type_distribution: Vec<TypeDistributionEntry>,
}

/// Type distribution entry for dataset summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDistributionEntry {
    pub dtype: String,
    pub count: usize,
    pub percentage: f64,
}

/// Histogram bin for numeric and text distributions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBin {
    pub start: f64,
    pub end: f64,
    pub count: usize,
}

/// Box plot summary values for numeric columns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxPlotSummary {
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
}

/// Simple category count entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCount {
    pub value: String,
    pub count: usize,
    pub percentage: f64,
}

/// Time series bin for datetime columns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBin {
    pub label: String,
    pub count: usize,
}

/// Statistical test result entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalTestResult {
    pub test: String,
    pub statistic: f64,
    pub p_value: f64,
    pub df: Option<f64>,
    pub effect_size: Option<f64>,
    pub notes: Option<String>,
}

/// Numeric column analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericColumnStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub variance: f64,
    pub iqr: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub outliers_iqr: usize,
    pub outliers_robust_z: usize,
    pub histogram: Vec<HistogramBin>,
    pub box_plot: BoxPlotSummary,
    pub normality_tests: Vec<StatisticalTestResult>,
}

/// Categorical column analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoricalColumnStats {
    pub cardinality: usize,
    pub entropy: f64,
    pub gini: f64,
    pub imbalance_ratio: f64,
    pub top_values: Vec<CategoryCount>,
}

/// Text column analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextColumnStats {
    pub min_length: usize,
    pub max_length: usize,
    pub mean_length: f64,
    pub median_length: f64,
    pub empty_percentage: f64,
    pub whitespace_percentage: f64,
    pub unique_token_count: usize,
    pub length_histogram: Vec<HistogramBin>,
}

/// Datetime column analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateTimeColumnStats {
    pub min: String,
    pub max: String,
    pub range_days: f64,
    pub granularity: String,
    pub time_bins: Vec<TimeBin>,
}

/// Per-column analysis entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisColumnStats {
    pub profile: ColumnProfile,
    pub numeric: Option<NumericColumnStats>,
    pub categorical: Option<CategoricalColumnStats>,
    pub text: Option<TextColumnStats>,
    pub datetime: Option<DateTimeColumnStats>,
}

/// Missingness analysis summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingnessAnalysis {
    pub total_missing_cells: usize,
    pub total_missing_percentage: f64,
    pub per_column: Vec<MissingnessColumn>,
    pub co_missing_matrix: HeatmapMatrix,
}

/// Missingness entry for a single column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingnessColumn {
    pub column: String,
    pub missing_count: usize,
    pub missing_percentage: f64,
}

/// Heatmap matrix structure for correlations/associations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapMatrix {
    pub x_labels: Vec<String>,
    pub y_labels: Vec<String>,
    pub values: Vec<Vec<f64>>,
    pub p_values: Option<Vec<Vec<f64>>>,
}

/// Correlation analysis results for numeric columns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationAnalysis {
    pub numeric_columns: Vec<String>,
    pub pearson: HeatmapMatrix,
    pub spearman: HeatmapMatrix,
    pub top_pairs: Vec<CorrelationPair>,
}

/// Correlation pair entry for top correlations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationPair {
    pub column_x: String,
    pub column_y: String,
    pub method: String,
    pub estimate: f64,
    pub p_value: f64,
}

/// Association analysis for categorical and mixed column pairs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationAnalysis {
    pub categorical_columns: Vec<String>,
    pub cramers_v: HeatmapMatrix,
    pub chi_square: HeatmapMatrix,
    pub numeric_categorical: Vec<NumericCategoricalAssociation>,
}

/// Numeric-categorical association results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericCategoricalAssociation {
    pub numeric_column: String,
    pub categorical_column: String,
    pub anova: Option<StatisticalTestResult>,
    pub variance_test: Option<StatisticalTestResult>,
    pub kruskal: Option<StatisticalTestResult>,
    pub t_test: Option<StatisticalTestResult>,
    pub mann_whitney: Option<StatisticalTestResult>,
}

/// Full analysis result cached in state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub dataset: AnalysisDataset,
    pub generated_at: String,
    pub duration_ms: u64,
    pub summary: AnalysisSummary,
    pub dataset_profile: DatasetProfile,
    pub columns: Vec<AnalysisColumnStats>,
    pub missingness: MissingnessAnalysis,
    pub correlations: CorrelationAnalysis,
    pub associations: AssociationAnalysis,
    pub quality_issues: Vec<DataQualityIssue>,
}

/// Cached analysis results for original and processed datasets.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisCache {
    pub original: Option<AnalysisResult>,
    pub processed: Option<AnalysisResult>,
}

/// A snapshot of ML configuration for history entries.
///
/// Mirrors the frontend MLConfigSnapshot type.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::MLConfigSnapshot`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfigSnapshot {
    /// Target column for prediction
    pub target_column: String,
    /// Problem type ("classification" or "regression")
    pub problem_type: String,
    /// Columns excluded from features
    pub excluded_columns: Vec<String>,
    /// Whether processed or original data was used
    pub use_processed_data: bool,
    /// Whether hyperparameter optimization was enabled
    pub optimize_hyperparams: bool,
    /// Number of Optuna trials
    pub n_trials: u32,
    /// Number of CV folds
    pub cv_folds: u32,
    /// Whether SHAP explainability was enabled
    pub enable_explainability: bool,
    /// Number of top algorithms to compare
    pub top_k_algorithms: u32,
    /// Optional algorithm override
    pub algorithm: Option<String>,
}

/// Summary of a training result for history.
///
/// Mirrors the frontend TrainingResultSummary type.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::TrainingResultSummary`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingResultSummary {
    /// Name of the best model
    pub best_model_name: String,
    /// Test set score (accuracy, R2, etc.)
    pub test_score: f64,
    /// Total training time in seconds
    pub training_time_seconds: f64,
}

/// An entry in the training history.
///
/// Created each time training is run to allow users to view past results.
///
/// # Session-Only Storage
///
/// History is stored in memory only. Maximum 10 entries are kept.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::TrainingHistoryEntry`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingHistoryEntry {
    /// Unique identifier (UUID)
    pub id: String,
    /// Unix timestamp when training completed
    pub timestamp: i64,
    /// Configuration used for this training run
    pub config: MLConfigSnapshot,
    /// Summary of the training result
    pub result_summary: TrainingResultSummary,
}

/// Maximum number of training history entries to keep.
pub const MAX_TRAINING_HISTORY_ENTRIES: usize = 10;

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
/// * `nav_bar_position` - Navigation bar position setting
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

    /// Navigation bar position setting (Left, Right, or Merged).
    /// Defaults to Merged (combined with right sidebar).
    pub nav_bar_position: RwLock<NavBarPosition>,

    /// UI state for the preprocessing page.
    /// Persists selected columns, row range, and config across navigation.
    /// Session-only: not persisted to disk.
    pub preprocessing_ui_state: RwLock<PreprocessingUIState>,

    /// Cached analysis results for original and processed datasets.
    /// Session-only: not persisted to disk.
    pub analysis_results: RwLock<AnalysisCache>,

    /// UI state for the analysis page.
    /// Persists dataset toggle, active tab, and column focus.
    /// Session-only: not persisted to disk.
    pub analysis_ui_state: RwLock<AnalysisUIState>,

    // ============================================================================
    // ML STATE FIELDS
    // ============================================================================
    /// Currently trained model (if training has completed).
    /// Used for predictions and model persistence.
    pub trained_model: RwLock<Option<lex_learning::TrainedModel>>,

    /// Result from the most recent training run.
    /// Contains metrics, feature importance, and SHAP plots (raw PNG bytes).
    pub training_result: RwLock<Option<TrainingResult>>,

    /// History of training runs.
    /// Maximum [`MAX_TRAINING_HISTORY_ENTRIES`] entries, oldest removed first.
    /// Session-only: not persisted to disk.
    pub training_history: RwLock<Vec<TrainingHistoryEntry>>,

    /// Whether ML training is currently in progress.
    /// Used to prevent concurrent training runs and show UI state.
    pub ml_training_in_progress: RwLock<bool>,

    /// Cancellation token for ML training.
    /// Can be used to cancel training from the UI.
    pub ml_cancellation_token: RwLock<MLCancellationToken>,

    /// Whether the Python ML runtime has been initialized.
    /// Set to true after successful runtime initialization.
    pub ml_runtime_initialized: RwLock<bool>,

    /// UI state for the ML page.
    /// Persists target column, config, and settings across navigation.
    /// Session-only: not persisted to disk.
    pub ml_ui_state: RwLock<MLUIState>,
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
            nav_bar_position: RwLock::new(NavBarPosition::default()),
            preprocessing_ui_state: RwLock::new(PreprocessingUIState::default()),
            analysis_results: RwLock::new(AnalysisCache::default()),
            analysis_ui_state: RwLock::new(AnalysisUIState::default()),
            trained_model: RwLock::new(None),
            training_result: RwLock::new(None),
            training_history: RwLock::new(Vec::new()),
            ml_training_in_progress: RwLock::new(false),
            ml_cancellation_token: RwLock::new(MLCancellationToken::new()),
            ml_runtime_initialized: RwLock::new(false),
            ml_ui_state: RwLock::new(MLUIState::default()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
