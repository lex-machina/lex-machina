//!
//! Tauri commands for machine learning operations.
//!
//! This module provides commands for:
//! - Initializing the ML Python runtime
//! - Training models with lex-learning
//! - Cancelling training runs
//! - Retrieving results and SHAP plots
//! - Saving/loading models
//! - Running predictions
//! - Persisting ML UI state
//!
//! The heavy ML work runs in a blocking task to keep the UI responsive.

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use lex_learning::{
    LexLearningError, Pipeline, PipelineConfig, PredictionResult, ProblemType, ProgressUpdate,
    TrainedModel, TrainingResult,
};
use polars::prelude::{AnyValue, DataFrame};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_store::StoreExt;

use crate::commands::settings::{SETTINGS_STORE, store_keys};
use crate::events::{self, AppEventEmitter, MLCompletePayload, MLKernelStatus, MLProgressPayload};
use crate::state::{
    AppState, MAX_TRAINING_HISTORY_ENTRIES, MLConfigSnapshot, MLUIState, TrainingHistoryEntry,
    TrainingResultSummary,
};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct MLConfigRequest {
    pub smart_mode: bool,
    pub target_column: String,
    pub problem_type: String,
    pub excluded_columns: Vec<String>,
    pub use_processed_data: bool,
    pub optimize_hyperparams: Option<bool>,
    pub n_trials: Option<u32>,
    pub cv_folds: Option<u32>,
    pub test_size: Option<f64>,
    pub enable_neural_networks: Option<bool>,
    pub enable_explainability: Option<bool>,
    pub top_k_algorithms: Option<u32>,
    pub algorithm: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrainingResultResponse {
    pub success: bool,
    pub best_model_name: String,
    pub metrics: lex_learning::Metrics,
    pub feature_importance: Vec<(String, f64)>,
    pub shap_plots: HashMap<String, String>,
    pub model_comparison: Vec<lex_learning::ModelComparison>,
    pub training_time_seconds: f64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchPredictionResult {
    pub predictions: Vec<Value>,
    pub probabilities: Option<Vec<HashMap<String, f64>>>,
    pub row_count: usize,
}

// ============================================================================
// HELPERS
// ============================================================================

enum DataSelectionError {
    NoData(String),
    InvalidConfig(String),
}

impl DataSelectionError {
    fn message(&self) -> String {
        match self {
            Self::NoData(message) | Self::InvalidConfig(message) => message.clone(),
        }
    }
}

fn parse_problem_type(value: &str) -> Result<ProblemType, LexLearningError> {
    match value.trim().to_lowercase().as_str() {
        "classification" => Ok(ProblemType::Classification),
        "regression" => Ok(ProblemType::Regression),
        other => Err(LexLearningError::InvalidConfig(format!(
            "Unknown problem type: {}",
            other
        ))),
    }
}

fn build_pipeline_config(request: &MLConfigRequest) -> Result<PipelineConfig, LexLearningError> {
    if request.target_column.trim().is_empty() {
        return Err(LexLearningError::InvalidConfig(
            "Target column is required".to_string(),
        ));
    }

    let problem_type = parse_problem_type(&request.problem_type)?;
    let mut builder = PipelineConfig::builder()
        .problem_type(problem_type)
        .target_column(request.target_column.trim().to_string());

    if !request.smart_mode {
        if let Some(optimize) = request.optimize_hyperparams {
            builder = builder.optimize_hyperparams(optimize);
        }
        if let Some(trials) = request.n_trials {
            builder = builder.n_trials(trials);
        }
        if let Some(folds) = request.cv_folds {
            builder = builder.cv_folds(folds);
        }
        if let Some(test_size) = request.test_size {
            builder = builder.test_size(test_size);
        }
        if let Some(enable_nn) = request.enable_neural_networks {
            builder = builder.enable_neural_networks(enable_nn);
        }
        if let Some(enable_explainability) = request.enable_explainability {
            builder = builder.enable_explainability(enable_explainability);
        }
        if let Some(top_k) = request.top_k_algorithms {
            builder = builder.top_k_algorithms(top_k);
        }
        if let Some(algorithm) = request.algorithm.clone()
            && !algorithm.trim().is_empty()
        {
            builder = builder.algorithm(algorithm);
        }
    }

    builder.build()
}

fn any_value_to_json(value: AnyValue) -> Value {
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
        AnyValue::Float32(f) => serde_json::Number::from_f64(f as f64)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        AnyValue::Float64(f) => serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        AnyValue::String(s) => Value::String(s.to_string()),
        AnyValue::StringOwned(s) => Value::String(s.to_string()),
        _ => Value::String(format!("{}", value)),
    }
}

fn any_value_to_f64(value: AnyValue) -> Option<f64> {
    match value {
        AnyValue::Float32(f) => Some(f as f64),
        AnyValue::Float64(f) => Some(f),
        AnyValue::Int8(i) => Some(f64::from(i)),
        AnyValue::Int16(i) => Some(f64::from(i)),
        AnyValue::Int32(i) => Some(f64::from(i)),
        AnyValue::Int64(i) => Some(i as f64),
        AnyValue::UInt8(u) => Some(f64::from(u)),
        AnyValue::UInt16(u) => Some(f64::from(u)),
        AnyValue::UInt32(u) => Some(f64::from(u)),
        AnyValue::UInt64(u) => Some(u as f64),
        _ => None,
    }
}

fn training_result_to_response(
    result: &TrainingResult,
    include_shap: bool,
) -> TrainingResultResponse {
    let shap_plots = if include_shap {
        result
            .shap_plots
            .iter()
            .map(|(name, bytes)| (name.clone(), STANDARD.encode(bytes)))
            .collect()
    } else {
        HashMap::new()
    };

    TrainingResultResponse {
        success: result.success,
        best_model_name: result.best_model_name.clone(),
        metrics: result.metrics.clone(),
        feature_importance: result.feature_importance.clone(),
        shap_plots,
        model_comparison: result.model_comparison.clone(),
        training_time_seconds: result.training_time_seconds,
        warnings: result.warnings.clone(),
    }
}

fn map_lex_learning_error(err: &LexLearningError) -> (&'static str, String) {
    match err {
        LexLearningError::RuntimeInit(message) => {
            (events::error_codes::ML_RUNTIME_INIT_FAILED, message.clone())
        }
        LexLearningError::InvalidConfig(message) => {
            (events::error_codes::ML_INVALID_CONFIG, message.clone())
        }
        LexLearningError::InvalidData(message) => {
            (events::error_codes::ML_INVALID_CONFIG, message.clone())
        }
        LexLearningError::TargetNotFound(message) => {
            (events::error_codes::ML_INVALID_CONFIG, message.clone())
        }
        LexLearningError::TrainingFailed(message) => {
            (events::error_codes::ML_TRAINING_FAILED, message.clone())
        }
        LexLearningError::Cancelled => (
            events::error_codes::ML_CANCELLED,
            "Training cancelled".to_string(),
        ),
        LexLearningError::ModelNotFound { path } => (
            events::error_codes::ML_NO_MODEL,
            format!("Model not found: {}", path),
        ),
        LexLearningError::InferenceError(message) => {
            (events::error_codes::ML_INFERENCE_ERROR, message.clone())
        }
        LexLearningError::ExplainabilityError(message) => {
            (events::error_codes::ML_TRAINING_FAILED, message.clone())
        }
        LexLearningError::PythonError { message } => {
            (events::error_codes::ML_TRAINING_FAILED, message.clone())
        }
        LexLearningError::ArrowConversion(err) => {
            (events::error_codes::ML_INVALID_CONFIG, err.to_string())
        }
        LexLearningError::Io(err) => (events::error_codes::ML_TRAINING_FAILED, err.to_string()),
        _ => (events::error_codes::ML_TRAINING_FAILED, err.to_string()),
    }
}

fn generate_history_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("ml_{}", timestamp)
}

fn select_dataframe_for_training(
    request: &MLConfigRequest,
    state: &State<'_, AppState>,
) -> Result<DataFrame, DataSelectionError> {
    let source_df = if request.use_processed_data {
        let guard = state.processed_dataframe.read();
        let loaded = guard
            .as_ref()
            .ok_or_else(|| DataSelectionError::NoData("No processed data loaded".to_string()))?;
        loaded.df.clone()
    } else {
        let guard = state.dataframe.read();
        let loaded = guard
            .as_ref()
            .ok_or_else(|| DataSelectionError::NoData("No data loaded".to_string()))?;
        loaded.df.clone()
    };

    if request.excluded_columns.is_empty() {
        return Ok(source_df);
    }

    let excluded: HashSet<String> = request
        .excluded_columns
        .iter()
        .map(|col| col.trim().to_string())
        .collect();

    let column_names = source_df.get_column_names();
    for column in &excluded {
        if !column_names
            .iter()
            .any(|name| name.as_str() == column.as_str())
        {
            return Err(DataSelectionError::InvalidConfig(format!(
                "Excluded column not found: {}",
                column
            )));
        }
    }

    let kept_columns: Vec<String> = column_names
        .iter()
        .filter(|name| !excluded.contains(name.as_str()))
        .map(|name| name.to_string())
        .collect();

    source_df
        .select(&kept_columns)
        .map_err(|e| DataSelectionError::InvalidConfig(format!("Failed to exclude columns: {}", e)))
}

fn select_dataframe_for_prediction(state: &State<'_, AppState>) -> Result<DataFrame, String> {
    let use_processed = state.ml_ui_state.read().use_processed_data;
    if use_processed {
        let guard = state.processed_dataframe.read();
        let loaded = guard
            .as_ref()
            .ok_or_else(|| "No processed data loaded".to_string())?;
        Ok(loaded.df.clone())
    } else {
        let guard = state.dataframe.read();
        let loaded = guard.as_ref().ok_or_else(|| "No data loaded".to_string())?;
        Ok(loaded.df.clone())
    }
}

fn set_training_in_progress(state: &State<'_, AppState>, value: bool) {
    *state.ml_training_in_progress.write() = value;
}

// ============================================================================
// KERNEL COMMANDS
// ============================================================================

#[tauri::command]
pub fn is_ml_initialized(state: State<'_, AppState>) -> bool {
    let initialized = lex_learning::is_initialized();
    *state.ml_runtime_initialized.write() = initialized;
    initialized
}

#[tauri::command]
pub async fn initialize_ml(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if lex_learning::is_initialized() {
        *state.ml_runtime_initialized.write() = true;
        app.emit_ml_kernel_status(MLKernelStatus::Ready, None);
        return Ok(());
    }

    app.emit_ml_kernel_status(MLKernelStatus::Initializing, None);

    let result = tauri::async_runtime::spawn_blocking(lex_learning::initialize)
        .await
        .map_err(|e| format!("Initialization task failed: {}", e))?;

    match result {
        Ok(()) => {
            *state.ml_runtime_initialized.write() = true;
            app.emit_ml_kernel_status(MLKernelStatus::Ready, None);
            Ok(())
        }
        Err(err) => {
            let message = err.to_string();
            *state.ml_runtime_initialized.write() = false;
            app.emit_ml_kernel_status(MLKernelStatus::Error, Some(&message));
            Err(message)
        }
    }
}

// ============================================================================
// TRAINING COMMANDS
// ============================================================================

#[tauri::command]
pub async fn start_training(
    app: AppHandle,
    request: MLConfigRequest,
    state: State<'_, AppState>,
) -> Result<TrainingResultResponse, String> {
    if !lex_learning::is_initialized() {
        let message = "ML runtime not initialized".to_string();
        app.emit_ml_error(events::error_codes::ML_NOT_INITIALIZED, &message);
        return Err(message);
    }

    if request.target_column.trim().is_empty() {
        let message = "Target column is required".to_string();
        app.emit_ml_error(events::error_codes::ML_INVALID_CONFIG, &message);
        return Err(message);
    }

    if request
        .excluded_columns
        .iter()
        .any(|col| col == request.target_column.trim())
    {
        let message = "Target column cannot be excluded".to_string();
        app.emit_ml_error(events::error_codes::ML_INVALID_CONFIG, &message);
        return Err(message);
    }

    {
        let mut guard = state.ml_training_in_progress.write();
        if *guard {
            let message = "Training already in progress".to_string();
            app.emit_ml_error(events::error_codes::ML_TRAINING_IN_PROGRESS, &message);
            return Err(message);
        }
        *guard = true;
    }

    *state.training_result.write() = None;
    *state.trained_model.write() = None;

    let token = {
        let token = state.ml_cancellation_token.read().clone();
        token.reset();
        token
    };

    let df = match select_dataframe_for_training(&request, &state) {
        Ok(df) => df,
        Err(err) => {
            let message = err.message();
            let code = match err {
                DataSelectionError::NoData(_) => events::error_codes::ML_NO_DATA,
                DataSelectionError::InvalidConfig(_) => events::error_codes::ML_INVALID_CONFIG,
            };
            set_training_in_progress(&state, false);
            app.emit_ml_error(code, &message);
            return Err(message);
        }
    };

    let config = match build_pipeline_config(&request) {
        Ok(config) => config,
        Err(err) => {
            let (code, message) = map_lex_learning_error(&err);
            set_training_in_progress(&state, false);
            app.emit_ml_error(code, &message);
            return Err(message);
        }
    };

    let config_snapshot_source = config.clone();
    let app_clone = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let progress_callback = move |update: ProgressUpdate| {
            let payload = MLProgressPayload {
                stage: update.stage.as_str().to_string(),
                progress: update.progress,
                message: update.message,
                current_model: update.current_model,
                models_completed: update.models_completed,
            };
            app_clone.emit_ml_progress(&payload);
        };

        let mut pipeline = Pipeline::builder()
            .config(config)
            .on_progress(progress_callback)
            .cancellation_token(token)
            .build()?;

        let training_result = pipeline.train(&df)?;
        let trained_model = pipeline.create_trained_model()?;
        Ok::<(TrainingResult, TrainedModel), LexLearningError>((training_result, trained_model))
    })
    .await
    .map_err(|e| {
        set_training_in_progress(&state, false);
        format!("Training task failed: {}", e)
    })?;

    match result {
        Ok((training_result, trained_model)) => {
            let summary = TrainingResultSummary {
                best_model_name: training_result.best_model_name.clone(),
                test_score: training_result.metrics.test_score.unwrap_or(0.0),
                training_time_seconds: training_result.training_time_seconds,
            };

            let config_snapshot = MLConfigSnapshot {
                target_column: request.target_column.trim().to_string(),
                problem_type: config_snapshot_source.problem_type.as_str().to_string(),
                excluded_columns: request.excluded_columns.clone(),
                use_processed_data: request.use_processed_data,
                optimize_hyperparams: config_snapshot_source.optimize_hyperparams,
                n_trials: config_snapshot_source.n_trials,
                cv_folds: config_snapshot_source.cv_folds,
                enable_explainability: config_snapshot_source.enable_explainability,
                top_k_algorithms: config_snapshot_source.top_k_algorithms,
                algorithm: config_snapshot_source.algorithm.clone(),
            };

            let entry = TrainingHistoryEntry {
                id: generate_history_id(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
                config: config_snapshot,
                result_summary: summary.clone(),
            };

            {
                let mut history = state.training_history.write();
                history.insert(0, entry);
                if history.len() > MAX_TRAINING_HISTORY_ENTRIES {
                    history.truncate(MAX_TRAINING_HISTORY_ENTRIES);
                }
            }

            *state.training_result.write() = Some(training_result.clone());
            *state.trained_model.write() = Some(trained_model);

            let completion_payload = MLCompletePayload {
                best_model_name: summary.best_model_name.clone(),
                test_score: summary.test_score,
                training_time_seconds: summary.training_time_seconds,
            };
            app.emit_ml_complete(&completion_payload);

            set_training_in_progress(&state, false);
            Ok(training_result_to_response(
                &training_result,
                config_snapshot_source.enable_explainability,
            ))
        }
        Err(err) => {
            let (code, message) = map_lex_learning_error(&err);
            if matches!(err, LexLearningError::Cancelled) {
                app.emit_ml_cancelled();
            } else {
                app.emit_ml_error(code, &message);
            }
            set_training_in_progress(&state, false);
            Err(message)
        }
    }
}

#[tauri::command]
pub fn cancel_training(state: State<'_, AppState>) {
    let token = state.ml_cancellation_token.read().clone();
    token.cancel();
}

#[tauri::command]
pub fn get_training_result(state: State<'_, AppState>) -> Result<TrainingResultResponse, String> {
    let guard = state.training_result.read();
    let result = guard
        .as_ref()
        .ok_or_else(|| "No training result available".to_string())?;
    Ok(training_result_to_response(result, true))
}

#[tauri::command]
pub fn get_shap_plot(name: String, state: State<'_, AppState>) -> Result<String, String> {
    let guard = state.training_result.read();
    let result = guard
        .as_ref()
        .ok_or_else(|| "No training result available".to_string())?;

    let bytes = result
        .shap_plots
        .get(&name)
        .ok_or_else(|| format!("SHAP plot not found: {}", name))?;

    Ok(STANDARD.encode(bytes))
}

#[tauri::command]
pub fn get_model_info(state: State<'_, AppState>) -> Result<lex_learning::ModelInfo, String> {
    let guard = state.trained_model.read();
    let model = guard
        .as_ref()
        .ok_or_else(|| "No trained model available".to_string())?;

    model
        .get_info()
        .map_err(|err| map_lex_learning_error(&err).1)
}

#[tauri::command]
pub async fn save_model(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let default_filename = state
        .training_result
        .read()
        .as_ref()
        .map(|result| format!("{}_model.pkl", result.best_model_name))
        .unwrap_or_else(|| "model.pkl".to_string());

    let file_path = app
        .dialog()
        .file()
        .add_filter("Model Files", &["pkl"])
        .set_file_name(&default_filename)
        .blocking_save_file();

    let path = match file_path {
        Some(path) => path.to_string(),
        None => return Err("Save cancelled by user".to_string()),
    };

    let guard = state.trained_model.read();
    let model = guard
        .as_ref()
        .ok_or_else(|| "No trained model available".to_string())?;

    model
        .save(&path)
        .map_err(|err| map_lex_learning_error(&err).1)?;

    Ok(path)
}

#[tauri::command]
pub async fn load_model(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<lex_learning::ModelInfo, String> {
    if !lex_learning::is_initialized() {
        return Err("ML runtime not initialized".to_string());
    }

    let file_path = app
        .dialog()
        .file()
        .add_filter("Model Files", &["pkl"])
        .blocking_pick_file();

    let path = match file_path {
        Some(path) => path.to_string(),
        None => return Err("Load cancelled by user".to_string()),
    };

    let model = TrainedModel::load(&path).map_err(|err| map_lex_learning_error(&err).1)?;
    let info = model
        .get_info()
        .map_err(|err| map_lex_learning_error(&err).1)?;

    *state.trained_model.write() = Some(model);
    *state.training_result.write() = None;

    Ok(info)
}

#[tauri::command]
pub fn predict_single(
    instance: Value,
    state: State<'_, AppState>,
) -> Result<PredictionResult, String> {
    if !lex_learning::is_initialized() {
        return Err("ML runtime not initialized".to_string());
    }

    let guard = state.trained_model.read();
    let model = guard
        .as_ref()
        .ok_or_else(|| "No trained model available".to_string())?;

    model
        .predict(&instance)
        .map_err(|err| map_lex_learning_error(&err).1)
}

#[tauri::command]
pub fn predict_batch(state: State<'_, AppState>) -> Result<BatchPredictionResult, String> {
    if !lex_learning::is_initialized() {
        return Err("ML runtime not initialized".to_string());
    }

    let df = select_dataframe_for_prediction(&state)?;

    let guard = state.trained_model.read();
    let model = guard
        .as_ref()
        .ok_or_else(|| "No trained model available".to_string())?;

    let model_info = model
        .get_info()
        .map_err(|err| map_lex_learning_error(&err).1)?;

    let df = df
        .select(&model_info.feature_names)
        .map_err(|e| format!("Failed to select feature columns: {}", e))?;

    let prediction_df = model
        .predict_batch(&df)
        .map_err(|err| map_lex_learning_error(&err).1)?;

    let row_count = prediction_df.height();
    let prediction_series = prediction_df
        .column("prediction")
        .map_err(|e| format!("Missing prediction column: {}", e))?;

    let predictions = (0..row_count)
        .map(|idx| prediction_series.get(idx))
        .collect::<polars::prelude::PolarsResult<Vec<_>>>()
        .map_err(|e| format!("Failed to read predictions: {}", e))?
        .into_iter()
        .map(any_value_to_json)
        .collect::<Vec<_>>();

    let probability_columns: Vec<_> = prediction_df
        .get_columns()
        .iter()
        .filter(|col| col.name().starts_with("probability_"))
        .map(|col| (col.name().to_string(), col.clone()))
        .collect();

    let probabilities = if probability_columns.is_empty() {
        None
    } else {
        let mut rows = Vec::with_capacity(row_count);
        for idx in 0..row_count {
            let mut row = HashMap::new();
            for (name, col) in &probability_columns {
                let class_label = name.trim_start_matches("probability_");
                let value = col
                    .get(idx)
                    .map_err(|e| format!("Failed to read probability: {}", e))?;
                if let Some(prob) = any_value_to_f64(value) {
                    row.insert(class_label.to_string(), prob);
                }
            }
            rows.push(row);
        }
        Some(rows)
    };

    Ok(BatchPredictionResult {
        predictions,
        probabilities,
        row_count,
    })
}

// ============================================================================
// HISTORY + UI STATE
// ============================================================================

#[tauri::command]
pub fn get_training_history(state: State<'_, AppState>) -> Vec<TrainingHistoryEntry> {
    state.training_history.read().clone()
}

#[tauri::command]
pub fn clear_training_history(state: State<'_, AppState>) {
    state.training_history.write().clear();
}

#[tauri::command]
pub fn get_ml_ui_state(state: State<'_, AppState>) -> MLUIState {
    state.ml_ui_state.read().clone()
}

#[tauri::command]
pub fn set_ml_ui_state(ui_state: MLUIState, state: State<'_, AppState>) {
    *state.ml_ui_state.write() = ui_state;
}

// ============================================================================
// SETTINGS (AUTO-START)
// ============================================================================

#[tauri::command]
pub fn get_auto_start_ml_kernel(app: AppHandle) -> Result<bool, String> {
    let store = app
        .store(SETTINGS_STORE)
        .map_err(|e| format!("Failed to open settings store: {}", e))?;

    Ok(store
        .get(store_keys::AUTO_START_ML_KERNEL)
        .and_then(|value| value.as_bool())
        .unwrap_or(false))
}

#[tauri::command]
pub fn set_auto_start_ml_kernel(auto_start: bool, app: AppHandle) -> Result<(), String> {
    let store = app
        .store(SETTINGS_STORE)
        .map_err(|e| format!("Failed to open settings store: {}", e))?;

    store.set(store_keys::AUTO_START_ML_KERNEL, json!(auto_start));
    store
        .save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    Ok(())
}
