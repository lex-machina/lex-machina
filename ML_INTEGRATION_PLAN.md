# ML Integration Plan

## Overview

Integrate the `lex-learning` crate (Rust with embedded Python 3.12) into the Lex Machina Tauri application. This enables automated machine learning with:

- Automated model selection (sklearn, XGBoost, LightGBM)
- Hyperparameter optimization (Optuna)
- SHAP explainability plots
- Model persistence (save/load)
- Single and batch predictions

---

## Architecture

### System Context

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        LEX MACHINA (Tauri Desktop App)                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────┐         ┌─────────────────┐         ┌──────────────┐  │
│   │  lex-processing │         │   lex-learning  │         │   Frontend   │  │
│   │   (Rust crate)  │         │   (Rust crate)  │         │  (Next.js)   │  │
│   │                 │         │                 │         │              │  │
│   │  - Data clean   │  ───►   │  - ML training  │  ◄───►  │  - ML page   │  │
│   │  - Type correct │ DataFrame│  - Predictions  │ Events/ │  - Results   │  │
│   │  - Null handle  │         │  - SHAP plots   │ Commands│  - Config    │  │
│   └─────────────────┘         └─────────────────┘         └──────────────┘  │
│           │                           │                           │          │
│           └───────────► processed_dataframe ◄─────────────────────┘          │
│                               OR                                             │
│           └───────────► original_dataframe ◄──────────────────────┘          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
CSV File ──► load_file() ──► dataframe
                                 │
                    ┌────────────┴────────────┐
                    │                         │
                    ▼                         │
          start_preprocessing()               │
                    │                         │
                    ▼                         │
          processed_dataframe                 │
                    │                         │
                    └────────────┬────────────┘
                                 │
                    User chooses data source
                                 │
                                 ▼
                         start_training()
                                 │
                                 ▼
                    TrainedModel + TrainingResult
                                 │
                    ┌────────────┴────────────┐
                    ▼                         ▼
             save_model()              predict_single()
                                       predict_batch()
```

---

## Design Decisions

| Decision                 | Choice                                | Rationale                            |
| ------------------------ | ------------------------------------- | ------------------------------------ |
| **Cancellation**         | Support via shared AtomicBool         | User can abort long training runs    |
| **Kernel init**          | Manual "Start Kernel" button          | User controls when 2-3s init happens |
| **Auto-start option**    | Settings toggle (default: off)        | Power users can auto-start           |
| **Layout**               | Sidebar + Content                     | Consistent with preprocessing page   |
| **SHAP plots**           | Full resolution, inline               | Best quality for analysis            |
| **Model files**          | .pkl extension                        | Native to Python/sklearn             |
| **Prediction input**     | Form + JSON toggle                    | Accessible + powerful                |
| **Training history**     | Keep history (like preprocessing)     | Compare different runs               |
| **Data source**          | User chooses (processed or original)  | Flexibility                          |
| **Feature selection**    | All selected by default, can deselect | Simple default, full control         |
| **Implementation order** | lex-learning → Backend → Frontend     | Core library first                   |

---

## Implementation Phases

### Phase 0: lex-learning Modifications

**Goal:** Add cancellation support to the lex-learning crate.

#### New Files

| File                  | Purpose                                    |
| --------------------- | ------------------------------------------ |
| `src/cancellation.rs` | `CancellationToken` type with `AtomicBool` |

#### Modified Files

| File                         | Changes                                |
| ---------------------------- | -------------------------------------- |
| `src/lib.rs`                 | Export `CancellationToken`             |
| `src/pipeline.rs`            | Add `.cancellation_token()` to builder |
| `src/python/callback.rs`     | Add `PyCancellationChecker` pyclass    |
| `python/.../orchestrator.py` | Check cancellation in training loop    |

#### CancellationToken API

```rust
pub struct CancellationToken {
    flag: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self;
    pub fn cancel(&self);
    pub fn is_cancelled(&self) -> bool;
    pub fn reset(&self);
    pub fn clone_flag(&self) -> Arc<AtomicBool>;  // For Python bridge
}
```

#### Pipeline Builder Addition

```rust
impl PipelineBuilder {
    pub fn cancellation_token(mut self, token: CancellationToken) -> Self;
}
```

#### Python Integration

```python
# In orchestrator.py
class Pipeline:
    def __init__(self, ..., cancel_check: Optional[Callable[[], bool]] = None):
        self._cancel_check = cancel_check

    def _check_cancelled(self) -> bool:
        if self._cancel_check:
            return self._cancel_check()
        return False

    def train(self, df):
        # At key checkpoints:
        if self._check_cancelled():
            raise TrainingCancelled("Training cancelled by user")
```

---

### Phase 1: Backend - State & Events

#### State Additions (`src-tauri/src/state.rs`)

```rust
// Add to AppState
pub trained_model: RwLock<Option<lex_learning::TrainedModel>>,
pub training_result: RwLock<Option<lex_learning::TrainingResult>>,
pub training_history: RwLock<Vec<TrainingHistoryEntry>>,
pub ml_training_in_progress: RwLock<bool>,
pub ml_cancellation_token: RwLock<lex_learning::CancellationToken>,
pub ml_runtime_initialized: RwLock<bool>,
pub ml_ui_state: RwLock<MLUIState>,

// New types
pub struct MLUIState {
    pub target_column: Option<String>,
    pub problem_type: String,
    pub excluded_columns: Vec<String>,
    pub use_processed_data: bool,
    pub config: MLConfigUIState,
    pub active_tab: String,
}

pub struct MLConfigUIState {
    pub optimize_hyperparams: bool,
    pub n_trials: u32,
    pub cv_folds: u32,
    pub test_size: f64,
    pub enable_neural_networks: bool,
    pub enable_explainability: bool,
    pub top_k_algorithms: u32,
}

pub struct TrainingHistoryEntry {
    pub id: String,
    pub timestamp: i64,
    pub config: MLConfigSnapshot,
    pub result_summary: TrainingResultSummary,
}

pub struct MLConfigSnapshot {
    pub target_column: String,
    pub problem_type: String,
    pub excluded_columns: Vec<String>,
    pub use_processed_data: bool,
    pub optimize_hyperparams: bool,
    pub n_trials: u32,
    pub cv_folds: u32,
    pub enable_explainability: bool,
}

pub struct TrainingResultSummary {
    pub best_model_name: String,
    pub test_score: f64,
    pub training_time_seconds: f64,
}

pub const MAX_TRAINING_HISTORY_ENTRIES: usize = 10;
```

#### Event Additions (`src-tauri/src/events.rs`)

```rust
// Event constants
pub const EVENT_ML_PROGRESS: &str = "ml:progress";
pub const EVENT_ML_COMPLETE: &str = "ml:complete";
pub const EVENT_ML_ERROR: &str = "ml:error";
pub const EVENT_ML_CANCELLED: &str = "ml:cancelled";
pub const EVENT_ML_KERNEL_STATUS: &str = "ml:kernel-status";

// Payloads
#[derive(Debug, Clone, Serialize)]
pub struct MLProgressPayload {
    pub stage: String,
    pub progress: f64,
    pub message: String,
    pub current_model: Option<String>,
    pub models_completed: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MLCompletePayload {
    pub best_model_name: String,
    pub test_score: f64,
    pub training_time_seconds: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MLErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MLKernelStatusPayload {
    pub status: String,  // "uninitialized", "initializing", "ready", "error"
    pub message: Option<String>,
}

// Error codes
pub mod error_codes {
    // ML Error Codes
    pub const ML_NOT_INITIALIZED: &str = "ML_NOT_INITIALIZED";
    pub const ML_NO_DATA: &str = "ML_NO_DATA";
    pub const ML_TRAINING_IN_PROGRESS: &str = "ML_TRAINING_IN_PROGRESS";
    pub const ML_NO_MODEL: &str = "ML_NO_MODEL";
    pub const ML_INVALID_CONFIG: &str = "ML_INVALID_CONFIG";
    pub const ML_TRAINING_FAILED: &str = "ML_TRAINING_FAILED";
    pub const ML_CANCELLED: &str = "ML_CANCELLED";
    pub const ML_INFERENCE_ERROR: &str = "ML_INFERENCE_ERROR";
}

// Trait extension
pub trait AppEventEmitter {
    // ... existing methods ...

    fn emit_ml_progress(&self, update: &MLProgressPayload);
    fn emit_ml_complete(&self, payload: &MLCompletePayload);
    fn emit_ml_error(&self, code: &str, message: &str);
    fn emit_ml_cancelled(&self);
    fn emit_ml_kernel_status(&self, status: &str, message: Option<&str>);
}
```

---

### Phase 2: Backend - Commands

#### New File: `src-tauri/src/commands/ml.rs`

```rust
// Request types
#[derive(Debug, Clone, Deserialize)]
pub struct MLConfigRequest {
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

// Response types
#[derive(Debug, Clone, Serialize)]
pub struct TrainingResultResponse {
    pub success: bool,
    pub best_model_name: String,
    pub metrics: lex_learning::Metrics,
    pub feature_importance: Vec<(String, f64)>,
    pub shap_plot_names: Vec<String>,
    pub model_comparison: Vec<lex_learning::ModelComparison>,
    pub training_time_seconds: f64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchPredictionResult {
    pub predictions: Vec<serde_json::Value>,
    pub probabilities: Option<Vec<HashMap<String, f64>>>,
    pub row_count: usize,
}
```

#### Commands (17 total)

| Command                    | Async | Purpose                          |
| -------------------------- | ----- | -------------------------------- |
| `is_ml_initialized`        | No    | Check if Python runtime is ready |
| `initialize_ml`            | Yes   | Start Python runtime             |
| `start_training`           | Yes   | Run ML pipeline                  |
| `cancel_training`          | No    | Set cancellation flag            |
| `get_training_result`      | No    | Get result without SHAP plots    |
| `get_shap_plot`            | No    | Get specific SHAP plot as base64 |
| `get_model_info`           | No    | Get model metadata               |
| `save_model`               | Yes   | Save to .pkl (file dialog)       |
| `load_model`               | Yes   | Load from .pkl (file dialog)     |
| `predict_single`           | No    | Single instance prediction       |
| `predict_batch`            | No    | Batch prediction on data         |
| `get_training_history`     | No    | Get training history             |
| `clear_training_history`   | No    | Clear history                    |
| `get_ml_ui_state`          | No    | Get persisted UI state           |
| `set_ml_ui_state`          | No    | Save UI state                    |
| `get_auto_start_ml_kernel` | No    | Get auto-start setting           |
| `set_auto_start_ml_kernel` | No    | Set auto-start setting           |

#### Command Registration (`lib.rs`)

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...

    // ML commands
    commands::is_ml_initialized,
    commands::initialize_ml,
    commands::start_training,
    commands::cancel_training,
    commands::get_training_result,
    commands::get_shap_plot,
    commands::get_model_info,
    commands::save_model,
    commands::load_model,
    commands::predict_single,
    commands::predict_batch,
    commands::get_training_history,
    commands::clear_training_history,
    commands::get_ml_ui_state,
    commands::set_ml_ui_state,
    commands::get_auto_start_ml_kernel,
    commands::set_auto_start_ml_kernel,
])
```

---

### Phase 3: Frontend - Types & Hook

#### Types (`types/index.ts`)

```typescript
// ML Configuration
export interface MLConfigRequest {
    target_column: string;
    problem_type: "classification" | "regression";
    excluded_columns: string[];
    use_processed_data: boolean;
    optimize_hyperparams?: boolean;
    n_trials?: number;
    cv_folds?: number;
    test_size?: number;
    enable_neural_networks?: boolean;
    enable_explainability?: boolean;
    top_k_algorithms?: number;
    algorithm?: string;
}

// Training Result (without SHAP plot bytes)
export interface TrainingResultResponse {
    success: boolean;
    best_model_name: string;
    metrics: Metrics;
    feature_importance: [string, number][];
    shap_plot_names: string[];
    model_comparison: ModelComparison[];
    training_time_seconds: number;
    warnings: string[];
}

export interface Metrics {
    cv_score?: number;
    test_score?: number;
    train_score?: number;
    accuracy?: number;
    precision?: number;
    recall?: number;
    f1_score?: number;
    roc_auc?: number;
    mse?: number;
    rmse?: number;
    mae?: number;
    r2?: number;
}

export interface ModelComparison {
    name: string;
    test_score: number;
    train_score: number;
    cv_score: number;
    training_time_seconds: number;
    hyperparameters: Record<string, unknown>;
    overfitting_risk: "low" | "medium" | "high";
}

export interface PredictionResult {
    prediction: string | number;
    probabilities?: Record<string, number>;
    confidence?: number;
}

export interface BatchPredictionResult {
    predictions: (string | number)[];
    probabilities?: Record<string, number>[];
    row_count: number;
}

export interface ModelInfo {
    model_name: string;
    problem_type: string;
    target_column: string;
    feature_names: string[];
    class_labels?: string[];
    metrics: Metrics;
    hyperparameters: Record<string, unknown>;
}

// Progress
export interface MLProgressUpdate {
    stage: string;
    progress: number;
    message: string;
    current_model?: string;
    models_completed?: [number, number];
}

// Kernel status
export type MLKernelStatus =
    | "uninitialized"
    | "initializing"
    | "ready"
    | "error";

// Training status
export type MLTrainingStatus =
    | "idle"
    | "training"
    | "completed"
    | "error"
    | "cancelled";

// UI State
export interface MLUIState {
    target_column?: string;
    problem_type: string;
    excluded_columns: string[];
    use_processed_data: boolean;
    config: MLConfigUIState;
    active_tab: string;
}

export interface MLConfigUIState {
    optimize_hyperparams: boolean;
    n_trials: number;
    cv_folds: number;
    test_size: number;
    enable_neural_networks: boolean;
    enable_explainability: boolean;
    top_k_algorithms: number;
}

// History
export interface TrainingHistoryEntry {
    id: string;
    timestamp: number;
    config: MLConfigSnapshot;
    result_summary: TrainingResultSummary;
}

export interface MLConfigSnapshot {
    target_column: string;
    problem_type: string;
    excluded_columns: string[];
    use_processed_data: boolean;
    optimize_hyperparams: boolean;
    n_trials: number;
    cv_folds: number;
    enable_explainability: boolean;
}

export interface TrainingResultSummary {
    best_model_name: string;
    test_score: number;
    training_time_seconds: number;
}
```

#### Hook (`lib/hooks/use-ml.ts`)

```typescript
export function useML() {
  // Kernel state
  const [kernelStatus, setKernelStatus] = useState<MLKernelStatus>("uninitialized");

  // Training state
  const [trainingStatus, setTrainingStatus] = useState<MLTrainingStatus>("idle");
  const [progress, setProgress] = useState<MLProgressUpdate | null>(null);
  const [result, setResult] = useState<TrainingResultResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Model state
  const [modelInfo, setModelInfo] = useState<ModelInfo | null>(null);

  // History
  const [history, setHistory] = useState<TrainingHistoryEntry[]>([]);

  // Actions
  const initializeKernel = async () => { ... };
  const startTraining = async (config: MLConfigRequest) => { ... };
  const cancelTraining = async () => { ... };
  const saveModel = async () => { ... };
  const loadModel = async () => { ... };
  const getSHAPPlot = async (name: string): Promise<string> => { ... };
  const predictSingle = async (instance: Record<string, unknown>) => { ... };
  const predictBatch = async () => { ... };
  const clearHistory = async () => { ... };

  // Event subscriptions (via useRustEvent)
  // ml:progress, ml:complete, ml:error, ml:cancelled, ml:kernel-status

  return {
    // State
    kernelStatus,
    trainingStatus,
    progress,
    result,
    error,
    modelInfo,
    history,

    // Actions
    initializeKernel,
    startTraining,
    cancelTraining,
    saveModel,
    loadModel,
    getSHAPPlot,
    predictSingle,
    predictBatch,
    clearHistory,
  };
}
```

---

### Phase 4: Frontend - Components

#### Component Structure

```
components/ml/
├── ml-sidebar.tsx              # Full sidebar content
│   ├── KernelStatusCard        # Status + "Start Kernel" button
│   ├── DataSourceSelector      # Processed vs Original toggle
│   ├── TargetColumnSelector    # Dropdown
│   ├── ProblemTypeSelector     # Classification/Regression
│   ├── FeatureSelector         # Checkbox list
│   ├── AdvancedConfig          # Collapsible config options
│   └── ActionButtons           # Train/Cancel/Save/Load
│
├── ml-content.tsx              # Main content area
│   ├── KernelNotReady          # Shown when kernel uninitialized
│   ├── NoDataLoaded            # Shown when no dataframe
│   ├── TrainingProgressPanel   # During training
│   └── ResultsPanel            # After training
│       ├── MetricsCard
│       ├── FeatureImportanceChart
│       ├── ModelComparisonTable
│       └── SHAPViewer
│
├── prediction-panel.tsx        # Prediction testing
│   ├── SinglePredictionForm
│   ├── JSONInputToggle
│   └── PredictionResult
│
├── training-history.tsx        # History view
│   └── HistoryEntry
│
└── index.ts                    # Re-exports
```

#### Page Layout

```
┌───────────────────────────────────────────────────────────────────────────────┐
│ [Lex Machina]                                                      [≡]        │
├─────────────────────────────────────────────────────────┬─────────────────────┤
│                                                         │ Kernel: ● Ready     │
│                                                         ├─────────────────────┤
│                                                         │ Data Source         │
│                                                         │ ○ Processed ● Orig  │
│                                                         ├─────────────────────┤
│                  RESULTS / PROGRESS                     │ Target Column       │
│                                                         │ [Select column ▼]   │
│          (or "Start Kernel" if uninitialized)          ├─────────────────────┤
│                                                         │ Problem Type        │
│                                                         │ ● Classification    │
│                                                         │ ○ Regression        │
│                                                         ├─────────────────────┤
│                                                         │ ▸ Features (12/14)  │
│                                                         ├─────────────────────┤
│                                                         │ ▸ Advanced Config   │
│                                                         ├─────────────────────┤
│                                                         │ [Train Model]       │
│                                                         │ [Save] [Load]       │
├─────────────────────────────────────────────────────────┴─────────────────────┤
│                                STATUS BAR                                     │
└───────────────────────────────────────────────────────────────────────────────┘
```

---

### Phase 5: Settings Integration

Add to Settings page:

```typescript
// In settings page or new "Machine Learning" section
<SettingRow
  title="Auto-start ML Kernel"
  description="Automatically initialize the Python runtime when visiting the ML page"
>
  <Switch
    checked={autoStartKernel}
    onCheckedChange={setAutoStartKernel}
  />
</SettingRow>
```

---

### Phase 6: Testing & Polish

| Test Category         | Items                                                  |
| --------------------- | ------------------------------------------------------ |
| **Unit Tests**        | CancellationToken, config conversion, state management |
| **Integration Tests** | Full training flow, save/load roundtrip, predictions   |
| **UI Tests**          | Kernel init, training progress, result display         |
| **Edge Cases**        | Cancel during training, large datasets, missing values |
| **Error Handling**    | Network errors, invalid config, corrupted models       |

---

## File Changes Summary

### New Files

| File                                      | Purpose                |
| ----------------------------------------- | ---------------------- |
| `crates/lex-learning/src/cancellation.rs` | CancellationToken type |
| `src-tauri/src/commands/ml.rs`            | ML Tauri commands      |
| `lib/hooks/use-ml.ts`                     | ML hook                |
| `components/ml/*.tsx`                     | ML components          |

### Modified Files

| File                                             | Changes                     |
| ------------------------------------------------ | --------------------------- |
| `crates/lex-learning/src/lib.rs`                 | Export CancellationToken    |
| `crates/lex-learning/src/pipeline.rs`            | Add cancellation to builder |
| `crates/lex-learning/python/.../orchestrator.py` | Check cancellation          |
| `src-tauri/src/state.rs`                         | Add ML state fields         |
| `src-tauri/src/events.rs`                        | Add ML events               |
| `src-tauri/src/commands/mod.rs`                  | Export ml module            |
| `src-tauri/src/lib.rs`                           | Register ML commands        |
| `types/index.ts`                                 | Add ML types                |
| `app/ml/page.tsx`                                | Replace placeholder         |
| `app/settings/page.tsx`                          | Add auto-start toggle       |

---

## Estimated Effort

| Phase                      | Estimated Time  |
| -------------------------- | --------------- |
| Phase 0: lex-learning mods | 2-3 hours       |
| Phase 1: State & Events    | 1-2 hours       |
| Phase 2: Commands          | 3-4 hours       |
| Phase 3: Types & Hook      | 1-2 hours       |
| Phase 4: Components        | 4-6 hours       |
| Phase 5: Settings          | 0.5 hours       |
| Phase 6: Testing           | 2-3 hours       |
| **Total**                  | **14-20 hours** |

---

## Open Items (To Discuss During Implementation)

1. **Error Messages**: What user-facing messages for each error type?
2. **Loading States**: Skeleton loaders vs spinners for async operations?
3. **Keyboard Shortcuts**: Any shortcuts for training/cancel?
4. **Model Metadata**: What additional info to show in saved model file picker?
5. **Batch Prediction Output**: How to display results? Table? Download CSV?

---

_Created: January 18, 2026_
_Status: Approved - Ready for Implementation_
