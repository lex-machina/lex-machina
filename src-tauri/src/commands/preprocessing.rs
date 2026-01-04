//! Preprocessing Commands
//!
//! This module provides Tauri commands for data preprocessing operations:
//! - Starting and cancelling preprocessing pipelines
//! - Managing preprocessing history
//! - Fetching processed data for virtual scrolling
//!
//! # Pipeline Execution
//!
//! Preprocessing runs in a background thread via `tauri::async_runtime::spawn_blocking`
//! to keep the UI responsive. Progress updates are emitted as events.
//!
//! ```text
//! Frontend                          Rust Backend
//! ─────────                         ────────────
//!     │                                   │
//!     │  invoke("start_preprocessing")    │
//!     │──────────────────────────────────►│
//!     │                                   │ spawn_blocking()
//!     │                                   │──────────┐
//!     │   event: "preprocessing:progress" │          │ Pipeline
//!     │◄──────────────────────────────────│          │ running
//!     │   event: "preprocessing:progress" │          │
//!     │◄──────────────────────────────────│          │
//!     │          ...                      │          │
//!     │   event: "preprocessing:complete" │◄─────────┘
//!     │◄──────────────────────────────────│
//!     │                                   │
//! ```
//!
//! # Cancellation
//!
//! The user can cancel a running pipeline via the `cancel_preprocessing` command.
//! This sets a flag in the `CancellationToken` that the pipeline checks periodically.

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Local;
use lex_processing::{
    Pipeline, PipelineConfig, PipelineResult, PreprocessingError, ProgressReporter, ProgressUpdate,
    ai::{AIProvider, GeminiProvider, OpenRouterProvider},
};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value, json};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::events::AppEventEmitter;
use crate::state::{
    AIProviderType, AppState, ColumnInfo, FileInfo, LoadedDataFrame, MAX_HISTORY_ENTRIES,
    PreprocessingConfigSnapshot, PreprocessingHistoryEntry,
};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Request to start preprocessing.
///
/// This struct is sent from the frontend when the user clicks "Start Processing".
/// It contains the configuration for the preprocessing pipeline.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::PreprocessingRequest`
#[derive(Debug, Clone, Deserialize)]
pub struct PreprocessingRequest {
    /// Columns selected for preprocessing (empty = all columns)
    pub selected_columns: Vec<String>,
    /// Optional row range to process (start, end indices)
    pub row_range: Option<(usize, usize)>,
    /// Pipeline configuration options
    pub config: PipelineConfigRequest,
}

/// Pipeline configuration from the frontend.
///
/// This mirrors `PipelineConfig` but uses frontend-friendly types.
/// Converted to `PipelineConfig` before running the pipeline.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::PipelineConfigRequest`
#[derive(Debug, Clone, Deserialize)]
pub struct PipelineConfigRequest {
    /// Threshold for dropping columns with too many missing values (0.0-1.0)
    pub missing_column_threshold: f64,
    /// Threshold for dropping rows with too many missing values (0.0-1.0)
    pub missing_row_threshold: f64,
    /// Strategy for handling outliers: "Cap", "Remove", "Median", "Keep"
    pub outlier_strategy: String,
    /// Method for imputing numeric values: "Mean", "Median", "Knn", "Zero", "Drop"
    pub numeric_imputation: String,
    /// Method for imputing categorical values: "Mode", "Constant", "Drop"
    pub categorical_imputation: String,
    /// Whether to enable automatic type correction
    pub enable_type_correction: bool,
    /// Whether to remove duplicate rows
    pub remove_duplicates: bool,
    /// Number of neighbors for KNN imputation
    pub knn_neighbors: usize,
    /// Whether to use AI for preprocessing decisions
    pub use_ai_decisions: bool,
    /// Optional target column for ML task detection
    pub target_column: Option<String>,
}

/// A single row of cell values for virtual scrolling.
pub type Row = Vec<serde_json::Value>;

/// Response containing rows from the processed DataFrame.
///
/// Same structure as `RowsResponse` in dataframe.rs but for processed data.
#[derive(Debug, Serialize)]
pub struct ProcessedRowsResponse {
    pub rows: Vec<Row>,
    pub start: usize,
    pub total_rows: usize,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Converts a Polars `AnyValue` to a JSON `Value`.
fn any_value_to_json(value: AnyValue) -> serde_json::Value {
    match value {
        AnyValue::Null => Value::Null,
        AnyValue::Boolean(b) => Value::Bool(b),
        AnyValue::Int8(i) => Value::Number(i.into()),
        AnyValue::Int16(i) => Value::Number(i.into()),
        AnyValue::Int32(i) => Value::Number(i.into()),
        AnyValue::Int64(i) => Value::Number(i.into()),
        AnyValue::UInt8(u) => Value::Number(u.into()),
        AnyValue::UInt16(u) => Value::Number(u.into()),
        AnyValue::UInt32(u) => Value::Number(u.into()),
        AnyValue::UInt64(u) => Value::Number(u.into()),
        AnyValue::Float32(f) => Number::from_f64(f as f64)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        AnyValue::Float64(f) => Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        AnyValue::String(s) => Value::String(s.to_string()),
        AnyValue::StringOwned(s) => Value::String(s.to_string()),
        _ => Value::String(format!("{}", value)),
    }
}

/// Converts frontend config request to lex-processing PipelineConfig.
fn convert_config(req: &PipelineConfigRequest) -> Result<PipelineConfig, PreprocessingError> {
    use lex_processing::{CategoricalImputation, NumericImputation, OutlierStrategy};

    let outlier_strategy = match req.outlier_strategy.to_lowercase().as_str() {
        "cap" => OutlierStrategy::Cap,
        "remove" => OutlierStrategy::Remove,
        "median" => OutlierStrategy::Median,
        "keep" => OutlierStrategy::Keep,
        other => {
            return Err(PreprocessingError::InvalidConfig(format!(
                "Unknown outlier strategy: {}",
                other
            )));
        }
    };

    let numeric_imputation = match req.numeric_imputation.to_lowercase().as_str() {
        "mean" => NumericImputation::Mean,
        "median" => NumericImputation::Median,
        "knn" => NumericImputation::Knn,
        "zero" => NumericImputation::Zero,
        "drop" => NumericImputation::Drop,
        other => {
            return Err(PreprocessingError::InvalidConfig(format!(
                "Unknown numeric imputation: {}",
                other
            )));
        }
    };

    let categorical_imputation = match req.categorical_imputation.to_lowercase().as_str() {
        "mode" => CategoricalImputation::Mode,
        "constant" => CategoricalImputation::Constant,
        "drop" => CategoricalImputation::Drop,
        other => {
            return Err(PreprocessingError::InvalidConfig(format!(
                "Unknown categorical imputation: {}",
                other
            )));
        }
    };

    let mut builder = PipelineConfig::builder()
        .missing_column_threshold(req.missing_column_threshold)
        .missing_row_threshold(req.missing_row_threshold)
        .outlier_strategy(outlier_strategy)
        .numeric_imputation(numeric_imputation)
        .categorical_imputation(categorical_imputation)
        .enable_type_correction(req.enable_type_correction)
        .remove_duplicates(req.remove_duplicates)
        .knn_neighbors(req.knn_neighbors)
        .use_ai_decisions(req.use_ai_decisions)
        .generate_reports(false) // Don't write JSON reports in Tauri mode
        .save_to_disk(false); // Don't write files - keep in memory only

    // Only set target column if provided
    if let Some(ref target) = req.target_column {
        builder = builder.target_column(target.clone());
    }

    builder
        .build()
        .map_err(|e| PreprocessingError::InvalidConfig(e.to_string()))
}

/// Creates a FileInfo struct from a processed DataFrame.
fn create_processed_file_info(df: &DataFrame, original_path: &str) -> FileInfo {
    let columns: Vec<ColumnInfo> = df
        .get_columns()
        .iter()
        .map(|col| {
            let name = col.name().to_string();
            let dtype = format!("{:?}", col.dtype());
            let null_count = col.null_count();
            // Calculate suggested width based on column name and type
            let base_width = (name.len() * 10).max(80) as f32;
            let width = base_width.min(300.0);

            ColumnInfo {
                name,
                dtype,
                null_count,
                width,
            }
        })
        .collect();

    FileInfo {
        path: format!("{} (processed)", original_path),
        name: "Processed Data".to_string(),
        size_bytes: 0, // Not applicable for in-memory data
        row_count: df.height(),
        column_count: df.width(),
        columns,
    }
}

/// Generates a unique ID for history entries.
fn generate_history_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("prep_{}", timestamp)
}

// ============================================================================
// PROGRESS REPORTER
// ============================================================================

/// Progress reporter that emits Tauri events.
///
/// This struct implements `ProgressReporter` and forwards all progress
/// updates to the frontend via Tauri events.
struct TauriProgressReporter {
    app: AppHandle,
}

impl TauriProgressReporter {
    fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl ProgressReporter for TauriProgressReporter {
    fn report(&self, update: ProgressUpdate) {
        self.app.emit_preprocessing_progress(&update);
    }
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

/// Starts the preprocessing pipeline on the loaded DataFrame.
///
/// This command runs the preprocessing pipeline in a background thread,
/// emitting progress events as it runs. When complete, the processed
/// DataFrame is stored in `AppState::processed_dataframe` and a history
/// entry is created.
///
/// # Parameters
///
/// - `app` - Tauri AppHandle for emitting events
/// - `request` - Preprocessing configuration from the frontend
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Ok(PipelineResult)` - Preprocessing completed successfully
/// - `Err(PreprocessingError)` - Preprocessing failed or was cancelled
///
/// # Events Emitted
///
/// - `preprocessing:progress` - Repeatedly during execution
/// - `preprocessing:complete` - On successful completion
/// - `preprocessing:error` - On failure
/// - `preprocessing:cancelled` - If cancelled by user
///
/// # State Changes
///
/// - Clears `last_preprocessing_result` before starting
/// - Stores summary in `last_preprocessing_result` on completion
/// - Stores processed DataFrame in `processed_dataframe` on completion
/// - Adds entry to `preprocessing_history` on completion
#[tauri::command]
pub async fn start_preprocessing(
    app: AppHandle,
    request: PreprocessingRequest,
    state: State<'_, AppState>,
) -> Result<PipelineResult, PreprocessingError> {
    // Clear previous result before starting new preprocessing
    *state.last_preprocessing_result.write() = None;

    // Get the source DataFrame
    let (source_df, original_path) = {
        let guard = state.dataframe.read();
        let loaded = guard.as_ref().ok_or(PreprocessingError::NoDataLoaded)?;
        (loaded.df.clone(), loaded.file_info.path.clone())
    };

    // Apply column selection if specified
    let df = if request.selected_columns.is_empty() {
        source_df
    } else {
        source_df
            .select(&request.selected_columns)
            .map_err(PreprocessingError::Polars)?
    };

    // Apply row range if specified
    let df = if let Some((start, end)) = request.row_range {
        let len = (end - start).min(df.height());
        df.slice(start as i64, len)
    } else {
        df
    };

    // Convert frontend config to pipeline config
    let config = convert_config(&request.config)?;

    // Get or create cancellation token
    let token = {
        let token = state.preprocessing_token.read().clone();
        token.reset(); // Reset in case it was previously cancelled
        token
    };

    // Get AI provider config if available (read before spawn_blocking)
    let ai_provider_config = state.ai_provider_config.read().clone();

    // Create progress reporter
    let reporter = Arc::new(TauriProgressReporter::new(app.clone()));

    // Clone values needed for the blocking task
    let app_clone = app.clone();
    let config_clone = request.config.clone();
    let selected_columns = request.selected_columns.clone();
    let row_range = request.row_range;

    // Run pipeline in blocking task (CPU-bound work)
    let result = tauri::async_runtime::spawn_blocking(move || {
        // Create AI provider if configured and AI decisions are enabled
        let ai_provider: Option<Arc<dyn AIProvider>> = if config.use_ai_decisions {
            ai_provider_config.and_then(|cfg| match cfg.provider {
                AIProviderType::OpenRouter => OpenRouterProvider::new(&cfg.api_key)
                    .ok()
                    .map(|p| Arc::new(p) as Arc<dyn AIProvider>),
                AIProviderType::Gemini => GeminiProvider::new(&cfg.api_key)
                    .ok()
                    .map(|p| Arc::new(p) as Arc<dyn AIProvider>),
                AIProviderType::None => None,
            })
        } else {
            None
        };

        // Build pipeline with optional AI provider
        let mut builder = Pipeline::builder()
            .config(config)
            .progress_reporter(reporter as Arc<dyn ProgressReporter>)
            .cancellation_token(token);

        if let Some(provider) = ai_provider {
            builder = builder.ai_provider(provider);
        }

        let pipeline = builder
            .build()
            .map_err(|e| PreprocessingError::InvalidConfig(e.to_string()))?;
        pipeline.process(df)
    })
    .await
    .map_err(|e| PreprocessingError::Internal(format!("Task join error: {}", e)))?;

    // Handle the result
    match result {
        Ok(pipeline_result) => {
            // Store the processed DataFrame
            if let Some(ref cleaned_df) = pipeline_result.dataframe {
                let file_info = create_processed_file_info(cleaned_df, &original_path);
                let loaded = LoadedDataFrame {
                    df: cleaned_df.clone(),
                    file_info,
                };
                *state.processed_dataframe.write() = Some(loaded);
            }

            // Store summary and create history entry
            if let Some(ref summary) = pipeline_result.summary {
                // Store as last result (persists across navigation)
                *state.last_preprocessing_result.write() = Some(summary.clone());

                let config_for_snapshot = convert_config(&config_clone)?;
                let mut config_snapshot = PreprocessingConfigSnapshot::from(&config_for_snapshot);
                config_snapshot.selected_columns = selected_columns;
                config_snapshot.row_range = row_range;

                let entry = PreprocessingHistoryEntry {
                    id: generate_history_id(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64,
                    config: config_snapshot,
                    summary: summary.clone(),
                };

                // Add to history (limit to MAX_HISTORY_ENTRIES)
                let mut history = state.preprocessing_history.write();
                history.insert(0, entry);
                if history.len() > MAX_HISTORY_ENTRIES {
                    history.truncate(MAX_HISTORY_ENTRIES);
                }

                // Emit completion event
                app_clone.emit_preprocessing_complete(summary);
            }

            Ok(pipeline_result)
        }
        Err(e) => {
            if e.is_cancelled() {
                app_clone.emit_preprocessing_cancelled();
            } else {
                app_clone.emit_preprocessing_error(e.error_code(), &e.to_string());
            }
            Err(e)
        }
    }
}

/// Cancels the currently running preprocessing pipeline.
///
/// This sets the cancellation flag on the token. The pipeline checks this
/// flag periodically and will stop with a `Cancelled` error.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
///
/// # Notes
///
/// Cancellation is not immediate - the pipeline will stop at the next
/// checkpoint. The `preprocessing:cancelled` event will be emitted
/// when the pipeline actually stops.
#[tauri::command]
pub fn cancel_preprocessing(state: State<'_, AppState>) {
    state.preprocessing_token.read().cancel();
}

/// Gets the preprocessing history.
///
/// Returns a list of previous preprocessing runs, newest first.
/// Maximum of `MAX_HISTORY_ENTRIES` (10) entries are kept.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// Vector of history entries, newest first.
#[tauri::command]
pub fn get_preprocessing_history(state: State<'_, AppState>) -> Vec<PreprocessingHistoryEntry> {
    state.preprocessing_history.read().clone()
}

/// Clears all preprocessing history.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
#[tauri::command]
pub fn clear_preprocessing_history(state: State<'_, AppState>) {
    state.preprocessing_history.write().clear();
}

/// Loads a history entry into the current processed data view.
///
/// This command retrieves a preprocessing result from history by its ID
/// and makes it the current processed data. This allows users to view
/// and compare previous preprocessing results.
///
/// # Parameters
///
/// - `entry_id` - UUID of the history entry to load
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Ok(())` if the entry was loaded successfully
/// - `Err(String)` if the entry was not found or loading failed
///
/// # Note
///
/// Currently, history entries only store the summary and config, not the
/// actual processed DataFrame. To fully implement this feature, we would
/// need to either:
/// 1. Store the processed DataFrame in history (memory intensive)
/// 2. Re-run preprocessing with the stored config
///
/// For now, this returns an error indicating the limitation.
#[tauri::command]
pub fn load_history_entry(entry_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let history = state.preprocessing_history.read();

    // Check if the entry exists
    let entry_exists = history.iter().any(|entry| entry.id == entry_id);

    if !entry_exists {
        return Err(format!("History entry '{}' not found", entry_id));
    }

    // Note: Currently we don't store the processed DataFrame in history entries
    // because it would be very memory intensive. To load a previous result,
    // we would need to re-run preprocessing with the stored config.
    //
    // For now, return an informative error.
    Err("Loading history entries is not yet supported. History entries currently only store the configuration and summary, not the processed data. To view different results, please re-run preprocessing with the desired configuration.".to_string())
}

/// Gets file info for the processed DataFrame.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Some(FileInfo)` if processed data exists
/// - `None` if no preprocessing has been done
#[tauri::command]
pub fn get_processed_file_info(state: State<'_, AppState>) -> Option<FileInfo> {
    state
        .processed_dataframe
        .read()
        .as_ref()
        .map(|loaded| loaded.file_info.clone())
}

/// Fetches rows from the processed DataFrame for virtual scrolling.
///
/// Same interface as `get_rows` but operates on the processed data.
///
/// # Parameters
///
/// - `start` - Starting row index (0-indexed)
/// - `count` - Number of rows to fetch
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Some(ProcessedRowsResponse)` - Rows fetched successfully
/// - `None` - No processed data exists
#[tauri::command]
pub fn get_processed_rows(
    start: usize,
    count: usize,
    state: State<'_, AppState>,
) -> Option<ProcessedRowsResponse> {
    let guard = state.processed_dataframe.read();
    let loaded = guard.as_ref()?;

    let df = &loaded.df;
    let total_rows = df.height();

    let start = start.min(total_rows.saturating_sub(1));
    let available = total_rows.saturating_sub(start);
    let actual_count = count.min(available);

    if actual_count == 0 {
        return Some(ProcessedRowsResponse {
            rows: vec![],
            start,
            total_rows,
        });
    }

    let sliced = df.slice(start as i64, actual_count);
    let mut rows = Vec::with_capacity(actual_count);

    for row_idx in 0..sliced.height() {
        let mut row = Vec::with_capacity(sliced.width());
        for col in sliced.get_columns() {
            let value = col.get(row_idx).ok().map_or(Value::Null, any_value_to_json);
            row.push(value);
        }
        rows.push(row);
    }

    Some(ProcessedRowsResponse {
        rows,
        start,
        total_rows,
    })
}

/// Clears the processed DataFrame from memory.
///
/// Call this when the user wants to discard the processed data
/// and start fresh.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
#[tauri::command]
pub fn clear_processed_data(state: State<'_, AppState>) {
    *state.processed_dataframe.write() = None;
}

/// Gets the last preprocessing result summary.
///
/// This returns the summary from the most recent preprocessing run.
/// The result persists across navigation until the user dismisses it
/// or starts a new preprocessing run.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Some(PreprocessingSummary)` if a preprocessing result exists
/// - `None` if no preprocessing has been done or result was dismissed
///
/// # Frontend Usage
///
/// ```typescript
/// // Load on component mount to restore persisted result
/// useEffect(() => {
///   invoke<PreprocessingSummary | null>("get_last_preprocessing_result")
///     .then((result) => {
///       if (result) {
///         setSummary(result);
///         setStatus("completed");
///       }
///     });
/// }, []);
/// ```
#[tauri::command]
pub fn get_last_preprocessing_result(
    state: State<'_, AppState>,
) -> Option<lex_processing::PreprocessingSummary> {
    state.last_preprocessing_result.read().clone()
}

/// Clears the last preprocessing result.
///
/// Call this when the user dismisses the results panel.
/// This does NOT clear the preprocessing history or the processed DataFrame.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
///
/// # Frontend Usage
///
/// ```typescript
/// const handleDismiss = async () => {
///   await invoke("clear_last_preprocessing_result");
///   setSummary(null);
///   setStatus("idle");
/// };
/// ```
#[tauri::command]
pub fn clear_last_preprocessing_result(state: State<'_, AppState>) {
    *state.last_preprocessing_result.write() = None;
}

// ============================================================================
// EXPORT TYPES AND COMMANDS
// ============================================================================

/// Result of exporting processed data.
///
/// # Mirrors
///
/// TypeScript: `types/index.ts::ExportResult`
#[derive(Debug, Serialize)]
pub struct ExportResult {
    /// Path to the exported CSV file
    pub csv_path: String,
    /// Path to the exported JSON report file
    pub report_path: String,
}

/// Exports the processed DataFrame to a CSV file with a JSON report.
///
/// Opens a native save dialog for the user to choose the export location.
/// The default filename is `{original_name}_processed_{YYYYMMDD_HHMMSS}.csv`.
/// A JSON report with preprocessing metadata is saved alongside the CSV.
///
/// # Parameters
///
/// - `app` - Tauri AppHandle for the file dialog
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Ok(ExportResult)` - Export completed successfully with paths to both files
/// - `Err(String)` - Export failed or was cancelled by user
///
/// # Errors
///
/// Returns an error if:
/// - No processed data exists
/// - User cancels the save dialog
/// - File write fails
///
/// # Frontend Usage
///
/// ```typescript
/// try {
///     const result = await invoke<ExportResult>("export_processed_data");
///     toast.success(`Exported to ${result.csv_path}`);
/// } catch (e) {
///     // User cancelled or export failed
///     if (e !== "Export cancelled by user") {
///         toast.error(e);
///     }
/// }
/// ```
#[tauri::command]
pub async fn export_processed_data(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ExportResult, String> {
    // Check if processed data exists
    let processed_guard = state.processed_dataframe.read();
    if processed_guard.is_none() {
        return Err("No processed data to export".to_string());
    }
    drop(processed_guard);

    // Get original filename for default name
    let original_stem = {
        let guard = state.dataframe.read();
        guard
            .as_ref()
            .map(|loaded| {
                Path::new(&loaded.file_info.path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("data")
                    .to_string()
            })
            .unwrap_or_else(|| "data".to_string())
    };

    // Generate default filename with timestamp
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let default_filename = format!("{}_processed_{}.csv", original_stem, timestamp);

    // Open native save dialog
    let file_path = app
        .dialog()
        .file()
        .add_filter("CSV Files", &["csv"])
        .set_file_name(&default_filename)
        .blocking_save_file();

    let csv_path = match file_path {
        Some(path) => path.to_string(),
        None => return Err("Export cancelled by user".to_string()),
    };

    // Write CSV file
    {
        let guard = state.processed_dataframe.read();
        let loaded = guard.as_ref().ok_or("Processed data no longer available")?;

        let mut df = loaded.df.clone();
        let file = File::create(&csv_path).map_err(|e| format!("Failed to create file: {}", e))?;

        CsvWriter::new(file)
            .finish(&mut df)
            .map_err(|e| format!("Failed to write CSV: {}", e))?;
    }

    // Generate JSON report path (same location, .json extension)
    let report_path = csv_path.replace(".csv", "_report.json");

    // Build JSON report
    let report = {
        let processed_guard = state.processed_dataframe.read();
        let original_guard = state.dataframe.read();
        let history_guard = state.preprocessing_history.read();

        let processed_info = processed_guard.as_ref().map(|l| &l.file_info);
        let original_info = original_guard.as_ref().map(|l| &l.file_info);
        let latest_summary = history_guard.first().map(|e| &e.summary);

        json!({
            "exported_at": Local::now().to_rfc3339(),
            "original_file": original_info.map(|info| json!({
                "path": info.path,
                "name": info.name,
                "row_count": info.row_count,
                "column_count": info.column_count,
            })),
            "processed_file": processed_info.map(|info| json!({
                "path": csv_path,
                "row_count": info.row_count,
                "column_count": info.column_count,
                "columns": info.columns.iter().map(|c| json!({
                    "name": c.name,
                    "dtype": c.dtype,
                    "null_count": c.null_count,
                })).collect::<Vec<_>>(),
            })),
            "preprocessing_summary": latest_summary,
        })
    };

    // Write JSON report
    let mut report_file =
        File::create(&report_path).map_err(|e| format!("Failed to create report file: {}", e))?;
    report_file
        .write_all(serde_json::to_string_pretty(&report).unwrap().as_bytes())
        .map_err(|e| format!("Failed to write report: {}", e))?;

    Ok(ExportResult {
        csv_path,
        report_path,
    })
}

// ============================================================================
// PREPROCESSING UI STATE PERSISTENCE
// ============================================================================

/// Gets the persisted preprocessing UI state.
///
/// Returns the saved column selection, row range, and configuration
/// so it can be restored when navigating back to the preprocessing page.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// The saved preprocessing UI state (may have empty columns if no file loaded)
#[tauri::command]
pub fn get_preprocessing_ui_state(
    state: State<'_, AppState>,
) -> crate::state::PreprocessingUIState {
    state.preprocessing_ui_state.read().clone()
}

/// Saves the preprocessing UI state for persistence across navigation.
///
/// Call this whenever the user changes their selection or configuration
/// on the preprocessing page.
///
/// # Parameters
///
/// - `ui_state` - The current UI state to save
/// - `state` - Tauri-managed application state
#[tauri::command]
pub fn set_preprocessing_ui_state(
    ui_state: crate::state::PreprocessingUIState,
    state: State<'_, AppState>,
) {
    *state.preprocessing_ui_state.write() = ui_state;
}
