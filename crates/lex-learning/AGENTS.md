# AGENTS.md - lex-learning (Machine Learning Training Library)

> **SYSTEM INSTRUCTION:** This file contains the master context, architecture, API design, and implementation details for the `lex-learning` library. Read this before generating code or planning tasks.

---

## 0. Core Principles

### ASK BEFORE ACTING

**Do NOT make assumptions on decisions that matter. The user is here to help, not just to request.**

Communicate and ask when:
- Requirements are ambiguous or incomplete
- Trade-offs exist between different approaches
- Scope is unclear (what's in/out)
- User preference matters (design, UX, naming)
- You're unsure about the right approach
- Implementation details could go multiple ways

Do NOT ask about: obvious file locations, following existing patterns, trivial details.

### GET LATEST DOCUMENTATION

**Always use MCP tools to get the latest documentation for libraries before implementing.**

Key libraries to check documentation for:
- **PyO3** (Context7 ID: `/pyo3/pyo3`) - Rust-Python bindings
- **Polars** (Context7 ID: `/pola-rs/polars`) - DataFrames and Arrow
- **thiserror** (Context7 ID: `/dtolnay/thiserror`) - Error handling
- **pyo3-polars** (Context7 ID: `/websites/rs_pyo3-polars_pyo3_polars`) - PyDataFrame wrappers

Use `resolve-library-id` first, then `get-library-docs` with appropriate topics.

### USER AVAILABILITY

**The user (Montaser) is available to answer questions.** If you are uncertain about:
- Architecture decisions
- API design choices
- Implementation approach
- Priority of features
- Trade-offs between options

**ASK.** Do not guess or assume. The user prefers to be consulted on important decisions rather than have to fix incorrect assumptions later.

---

## 1. Project Identity

| Field | Value |
|-------|-------|
| **Name** | lex-learning |
| **Location** | `/home/shush/dev/projects/lex-learning` |
| **Type** | Rust crate with embedded Python (PyO3) |
| **Part Of** | Lex Machina (Graduation Thesis - B.Sc. Data Science) |
| **Authors** | Montaser Amoor & Rita Basbous |
| **Purpose** | Automated ML training + inference with explainability |

---

## 2. System Context

### Where lex-learning Fits

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     LEX MACHINA (Tauri Desktop App)                      │
│                         Cargo Workspace                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   ┌─────────────────────┐         ┌─────────────────────────────────┐   │
│   │   lex-processing    │         │        lex-learning             │   │
│   │   (Rust crate)      │         │        (Rust crate)             │   │
│   │                     │         │                                 │   │
│   │   - Data cleaning   │         │   - Rust API (PyO3 bindings)    │   │
│   │   - Type correction │  ───►   │   - Bundles Python 3.12 runtime │   │
│   │   - Null handling   │  data   │   - Bundles ML libraries        │   │
│   │   - Outlier removal │         │   - Calls Python lex_learning   │   │
│   └─────────────────────┘         └─────────────────────────────────┘   │
│           ▲                                      │                       │
│           │                                      │                       │
│      Polars DataFrame                       TrainingResult               │
│                                             TrainedModel                 │
│                                             Predictions                  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Data Contract

**Input from lex-processing (via lex-machina):**
- Polars DataFrame with **NO nulls**
- **NO duplicate rows**
- **NO datetime columns** (removed by lex-processing)
- **NO identifier columns** (excluded by lex-processing)
- **Target column is LAST**
- Column types: `Int32`, `Int64`, `Float32`, `Float64`, `String`, `Boolean`
- Provided: `problem_type` ("classification" or "regression")
- Provided: `target_column` name

**Output from lex-learning:**
- Trained model (kept in Python memory, with option to export as `.pkl`)
- Training metrics (accuracy, F1, R2, etc.)
- SHAP explainability plots (PNG bytes)
- Inference capability (batch and single-instance)

---

## 3. Architecture

### Directory Structure

```
lex-learning/
├── src/                           # Rust crate source
│   ├── lib.rs                     # Public Rust API exports
│   ├── config.rs                  # PipelineConfig, ProblemType
│   ├── error.rs                   # LexLearningError enum (thiserror)
│   ├── types.rs                   # TrainingResult, ModelResult, Metrics, etc.
│   ├── pipeline.rs                # Pipeline struct with builder pattern
│   ├── model.rs                   # TrainedModel (holds Py<PyAny>, predict methods)
│   ├── progress.rs                # TrainingStage, ProgressUpdate, callback handling
│   └── python/
│       ├── mod.rs                 # Python initialization and module loading
│       ├── runtime.rs             # PYTHONHOME/PATH setup, runtime extraction
│       └── conversion.rs          # DataFrame ↔ PyObject conversions via Arrow
│
├── python/                        # Bundled Python source (embedded at compile time)
│   └── lex_learning/              # The Python library
│       ├── src/                   # Main library package
│       │   ├── __init__.py        # Public API exports
│       │   ├── config.py          # PipelineConfig, ProblemType
│       │   ├── errors.py          # Exception hierarchy
│       │   ├── core/              # Core types and protocols
│       │   ├── pipeline/          # Training pipeline orchestration
│       │   ├── preprocessing/     # Data preprocessing
│       │   ├── training/          # Model training
│       │   ├── models/            # Model registry
│       │   ├── explainability/    # SHAP explainability
│       │   ├── inference/         # Model loading and prediction
│       │   └── progress/          # Progress reporting
│       ├── tests/                 # Test suite (112 tests)
│       ├── cli.py                 # CLI entry point
│       ├── pyproject.toml         # Python dependencies
│       └── requirements.txt       # Pip requirements
│
├── runtime/                       # Platform-specific Python runtimes (pre-committed)
│   └── python/                    # python-build-standalone CPython 3.12
│       ├── linux-x86_64/          # Linux x86_64 runtime
│       ├── linux-aarch64/         # Linux ARM64 runtime
│       ├── darwin-x86_64/         # macOS Intel runtime
│       ├── darwin-aarch64/        # macOS Apple Silicon runtime
│       └── windows-x86_64/        # Windows x86_64 runtime
│
├── build.rs                       # Build script for Python setup
├── Cargo.toml                     # Rust dependencies
├── Makefile                       # Development commands
└── AGENTS.md                      # This file
```

### Component Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Rust API (lex-learning crate)                    │
│   PipelineConfig → Pipeline → TrainingResult → TrainedModel             │
└─────────────────────────────────────────────────────────────────────────┘
                              │ PyO3
                              ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    Python Library (lex_learning)                         │
│                                                                          │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐              │
│  │Preprocess│   │ Selector │   │ Trainer  │   │Explainer │              │
│  │          │   │          │   │          │   │          │              │
│  │- Encode  │   │- Heuristic│  │- Optuna  │   │- SHAP    │              │
│  │- Scale   │   │  selection│  │- CV      │   │- Plots   │              │
│  └──────────┘   └──────────┘   └──────────┘   └──────────┘              │
│                                     │                                    │
│                      ┌──────────────┼──────────────┐                     │
│                      ▼              ▼              ▼                     │
│                 ┌────────┐    ┌────────┐    ┌────────┐                   │
│                 │sklearn │    │Boosting│    │ Neural │                   │
│                 │ models │    │XGB/LGBM│    │  Keras │                   │
│                 └────────┘    └────────┘    └────────┘                   │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Rust Public API

### 4.1 Quick Start

```rust
use lex_learning::{Pipeline, PipelineConfig, ProblemType, TrainedModel};
use polars::prelude::*;

// Initialize Python runtime (call once at startup)
lex_learning::initialize()?;

// Configure the pipeline
let config = PipelineConfig::builder()
    .problem_type(ProblemType::Classification)
    .target_column("Survived")
    .build()?;

// Build and run the pipeline
let pipeline = Pipeline::builder()
    .config(config)
    .on_progress(|u| println!("{:.0}% - {}", u.progress * 100.0, u.message))
    .build()?;

let result = pipeline.train(&dataframe)?;

// Keep model in memory for inference
let model = pipeline.create_trained_model(&result)?;

// Single prediction
let prediction = model.predict(&serde_json::json!({"Age": 25, "Sex": "male"}))?;

// Batch prediction
let predictions_df = model.predict_batch(&new_data_df)?;

// Optional: save to disk
model.save("model.pkl")?;
```

### 4.2 Core Types (Rust)

```rust
// src/config.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemType {
    Classification,
    Regression,
}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub problem_type: ProblemType,
    pub target_column: Option<String>,
    pub algorithm: Option<String>,
    pub top_k_algorithms: u32,
    pub optimize_hyperparams: bool,
    pub n_trials: u32,
    pub cv_folds: u32,
    pub test_size: f64,
    pub enable_neural_networks: bool,
    pub enable_explainability: bool,
    pub shap_max_samples: u32,
    pub random_seed: u64,
    pub n_jobs: i32,
}
```

### 4.3 Error Handling

```rust
// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexLearningError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Target column '{0}' not found")]
    TargetNotFound(String),
    
    #[error("Training failed: {0}")]
    TrainingFailed(String),
    
    #[error("Model not found: {path}")]
    ModelNotFound { path: String },
    
    #[error("Inference error: {0}")]
    InferenceError(String),
    
    #[error("Training cancelled")]
    Cancelled,
    
    #[error("Explainability error: {0}")]
    ExplainabilityError(String),
    
    #[error("Python error: {message}")]
    PythonError { message: String },
    
    #[error("Runtime initialization failed: {0}")]
    RuntimeInit(String),
    
    #[error("Arrow conversion error: {0}")]
    ArrowConversion(String),
}
```

### 4.4 TrainedModel API

```rust
// src/model.rs
pub struct TrainedModel {
    py_model: Py<PyAny>,  // Holds Python TrainedModel instance in memory
}

impl TrainedModel {
    /// Load a model from a .pkl file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LexLearningError>;
    
    /// Save the model to a .pkl file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), LexLearningError>;
    
    /// Export model as bytes (for custom storage)
    pub fn to_bytes(&self) -> Result<Vec<u8>, LexLearningError>;
    
    /// Load model from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LexLearningError>;
    
    /// Single instance prediction
    pub fn predict(&self, instance: &serde_json::Value) -> Result<PredictionResult, LexLearningError>;
    
    /// Batch prediction from DataFrame
    pub fn predict_batch(&self, df: &DataFrame) -> Result<DataFrame, LexLearningError>;
    
    // Property accessors
    pub fn problem_type(&self) -> Result<ProblemType, LexLearningError>;
    pub fn target_column(&self) -> Result<String, LexLearningError>;
    pub fn feature_names(&self) -> Result<Vec<String>, LexLearningError>;
    pub fn class_labels(&self) -> Result<Option<Vec<String>>, LexLearningError>;
    pub fn best_model_name(&self) -> Result<String, LexLearningError>;
    pub fn metrics(&self) -> Result<Metrics, LexLearningError>;
    pub fn feature_importance(&self) -> Result<Vec<(String, f64)>, LexLearningError>;
    pub fn get_info(&self) -> Result<ModelInfo, LexLearningError>;
}
```

### 4.5 Progress Reporting

```rust
// src/progress.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainingStage {
    Initializing,
    Preprocessing,
    AlgorithmSelection,
    Training,
    Evaluation,
    Explainability,
    Complete,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub stage: TrainingStage,
    pub progress: f64,  // 0.0 to 1.0
    pub message: String,
    pub current_model: Option<String>,
    pub models_completed: Option<(u32, u32)>,  // (completed, total)
}

pub type ProgressCallback = Arc<dyn Fn(ProgressUpdate) + Send + Sync>;
```

---

## 5. Python Library API

The Python library (`python/lex_learning/`) is fully implemented with 112 passing tests.

### Python Quick Start

```python
from lex_learning import Pipeline, PipelineConfig, ProblemType, TrainedModel

# Configure
config = PipelineConfig.builder() \
    .problem_type(ProblemType.CLASSIFICATION) \
    .target_column("Survived") \
    .build()

# Train
pipeline = Pipeline.builder() \
    .config(config) \
    .on_progress(lambda u: print(f"{u.progress:.0%} - {u.message}")) \
    .build()

result = pipeline.train(dataframe)

# Save model
trained_model = pipeline.create_trained_model(result)
trained_model.save("model.pkl")

# Load and predict
model = TrainedModel.load("model.pkl")
prediction = model.predict({"Age": 25, "Sex": "male"})
```

---

## 6. DataFrame Conversion via Arrow

Data flows between Rust (Polars) and Python (pandas) using Arrow IPC for zero-copy transfer.

```rust
// src/python/conversion.rs
use polars::prelude::*;
use pyo3::prelude::*;

/// Convert Rust Polars DataFrame to Python pandas DataFrame via Arrow
pub fn dataframe_to_python<'py>(
    py: Python<'py>,
    df: &DataFrame,
) -> PyResult<Bound<'py, PyAny>> {
    // 1. Convert Polars DataFrame to Arrow IPC bytes
    // 2. In Python: read with pyarrow, convert to pandas
    // ...
}

/// Convert Python pandas DataFrame to Rust Polars DataFrame via Arrow
pub fn python_to_dataframe(
    py: Python<'_>,
    py_df: &Bound<'_, PyAny>,
) -> Result<DataFrame, LexLearningError> {
    // 1. Convert pandas to PyArrow Table
    // 2. Write to IPC bytes
    // 3. Read into Polars
    // ...
}
```

### Phase 5 Implementation Details (Finalized)

**Decisions:**
- Use Arrow IPC **File format** (not Stream) - matches Polars' default `IpcWriter`/`IpcReader`
- Clone DataFrame in `dataframe_to_python()` - keeps API clean with `&DataFrame`
- Nested error variants for granular error handling

**Error Types (add to `src/error.rs`):**

```rust
/// Specific kinds of Arrow conversion errors
#[derive(Error, Debug)]
pub enum ArrowConversionKind {
    #[error("serialization failed: {0}")]
    Serialize(String),
    #[error("deserialization failed: {0}")]
    Deserialize(String),
    #[error("type conversion failed: {0}")]
    TypeConversion(String),
}
```

**Data Flow - Rust to Python:**

```
Polars DataFrame
      │
      ▼ (clone)
&mut DataFrame
      │
      ▼ IpcWriter::new(Cursor<Vec<u8>>).finish()
Arrow IPC bytes (Vec<u8>)
      │
      ▼ PyBytes::new(py, &bytes)
Python bytes
      │
      ▼ io.BytesIO(bytes)
      ▼ pyarrow.ipc.open_file(buffer).read_all()
PyArrow Table
      │
      ▼ table.to_pandas()
pandas DataFrame
```

**Data Flow - Python to Rust:**

```
pandas DataFrame
      │
      ▼ pyarrow.Table.from_pandas(df)
PyArrow Table
      │
      ▼ pyarrow.ipc.RecordBatchFileWriter(sink, schema)
      ▼ writer.write_table(table)
      ▼ sink.getvalue()
Python bytes
      │
      ▼ PyBytes::as_bytes() → Vec<u8>
Arrow IPC bytes
      │
      ▼ IpcReader::new(Cursor::new(bytes)).finish()
Polars DataFrame
```

**Test Cases:**
1. Round-trip conversion (Rust → Python → Rust)
2. All supported dtypes (Int32, Int64, Float32, Float64, String, Boolean)
3. Empty DataFrame edge case
4. Column name preservation

---

## 7. Python Runtime Management

### Bundled Python Strategy

- **Runtime**: python-build-standalone CPython 3.12 (TensorFlow compatibility)
- **Location**: `runtime/python/` directory next to executable
- **Archives**: Pre-committed platform-specific archives in repo
- **Python Source**: Embedded in Rust binary via `include_str!`

### Build Script (build.rs)

```rust
// build.rs
fn main() {
    // 1. Detect target platform
    // 2. Ensure Python runtime is extracted
    // 3. Install pip dependencies if needed
    // 4. Set PYO3_PYTHON environment variable
    // 5. Export LEX_PYTHON_DIR for runtime use
}
```

### Runtime Initialization

```rust
// src/python/runtime.rs
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize Python runtime. Must be called before any Python operations.
pub fn initialize() -> Result<(), LexLearningError> {
    INIT.call_once(|| {
        // 1. Find Python runtime directory
        // 2. Set PYTHONHOME and PYTHONPATH
        // 3. Call pyo3::prepare_freethreaded_python()
    });
    Ok(())
}
```

### Critical Fix: sys.executable for Embedded Python

**Issue:** When running Rust integration tests that use the embedded Python runtime for ML training, tests fail with cryptic errors from joblib/loky worker processes.

**Problem:** When PyO3 embeds Python, `sys.executable` points to the Rust binary (host executable), not the Python interpreter. This breaks joblib/loky which spawn worker processes using `sys.executable`.

**Symptoms:**
```
AssertionError: SRE module mismatch
AttributeError: module '_thread' has no attribute '_set_sentinel'
The executor underlying Parallel has been shutdown
Training failed: All models failed to train
```

**Root Cause:** joblib/loky runs `sys.executable -m joblib.externals.loky.backend.popen_loky_posix` to spawn workers. When `sys.executable` points to the Rust binary, this fails catastrophically. The worker processes try to execute the Rust test binary as if it were a Python interpreter, leading to module mismatch errors and threading failures.

**Solution:** Set `sys.executable` to the correct Python interpreter path AFTER `Python::initialize()` but BEFORE importing any modules that use joblib/multiprocessing:

```rust
// src/python/runtime.rs - in do_initialize()

// 4. Initialize Python interpreter
pyo3::Python::initialize();

// 5. Fix sys.executable to point to the bundled Python interpreter
// This MUST happen before any imports that use joblib/multiprocessing
fix_sys_executable(&runtime_dir)?;

// 6. Now safe to import lex_learning (which imports sklearn, joblib, etc.)
verify_python_setup()?;

/// Fix sys.executable to point to the bundled Python interpreter.
fn fix_sys_executable(runtime_dir: &PathBuf) -> Result<(), String> {
    #[cfg(windows)]
    let python_exe = runtime_dir.join("python.exe");
    #[cfg(not(windows))]
    let python_exe = runtime_dir.join("bin").join("python3");

    pyo3::Python::attach(|py| {
        let sys = py.import("sys")?;
        let exe_path = python_exe.to_str().unwrap();
        sys.setattr("executable", exe_path)?;
        sys.setattr("_base_executable", exe_path)?;
        Ok(())
    })
}
```

**Why this matters:**
- Preserves full parallelism (`n_jobs=-1` works correctly)
- Training uses all available CPU cores as designed
- No performance degradation

---

## 8. Implementation Plan

### Decisions Made

| Decision | Choice |
|----------|--------|
| **API Design** | Mirror Python API in Rust (builder pattern) |
| **Data Transfer** | Arrow-based (Polars ↔ pandas via IPC) |
| **Python Runtime** | python-build-standalone, next to executable |
| **Dependencies** | Install at build time, pre-commit archives |
| **Model Storage** | Keep in Python memory, with option to export |
| **Async** | Not needed in lex-learning (handle in lex-machina) |
| **Python Source** | Embedded in Rust binary via `include_str!` |
| **Runtime Archives** | Pre-commit platform-specific archives to repo |
| **Model Format** | Keep pickle |
| **DataFrame Library** | Keep pandas in Python |

### Implementation Phases

| Phase | Task | Priority | Status |
|-------|------|----------|--------|
| **1** | Set up project structure and Cargo.toml | High | DONE |
| **2** | Implement error types with thiserror | High | DONE |
| **3** | Implement config types and builder | High | DONE |
| **4** | Implement Python runtime initialization | High | DONE |
| **5** | Implement DataFrame ↔ Python conversion via Arrow | High | DONE |
| **6** | Implement Pipeline and PipelineBuilder | High | DONE |
| **7** | Implement TrainedModel (predict, save, load) | High | DONE |
| **8** | Implement progress callback bridge | Medium | DONE |
| **9** | Write build.rs for Python runtime setup | Medium | DONE |
| **10** | Add `to_bytes()`/`from_bytes()` to Rust TrainedModel | Medium | DONE |
| **11** | Test on Linux | High | DONE |
| **12** | Test on macOS (x86_64 and aarch64) | Medium | DEFERRED |
| **13** | Test on Windows | Medium | DEFERRED |
| **14** | Integration tests with lex-machina | Low | TODO |
| **15** | Code review and quality assurance | High | TODO |

### Phase 15: Code Review and Quality Assurance

**Goal:** Systematically review the entire codebase to ensure correctness, idiomatic Rust/Python, best practices, proper documentation, and full functionality.

**Approach:** Sequential execution, default clippy strictness, thorough Python review, essential documentation.

| Sub-Phase | Task | Focus | Status |
|-----------|------|-------|--------|
| **15.1** | Verify Build and Tests | Run cargo check/test/clippy, Python tests | DONE |
| **15.2** | Review Error Handling | `src/error.rs` - variants, messages, From impls | TODO |
| **15.3** | Review Configuration | `src/config.rs` - builder pattern, validation | TODO |
| **15.4** | Review Types | `src/types.rs` - struct design, derives | TODO |
| **15.5** | Review Progress Types | `src/progress.rs` - enum, callback types | TODO |
| **15.6** | Review Pipeline | `src/pipeline.rs` - builder, train(), errors | TODO |
| **15.7** | Review TrainedModel | `src/model.rs` - Py<PyAny>, save/load, predict | TODO |
| **15.8** | Review Python Runtime | `src/python/runtime.rs` - init, sys.executable | TODO |
| **15.9** | Review Arrow Conversion | `src/python/conversion.rs` - IPC, type mapping | TODO |
| **15.10** | Review Callback Bridge | `src/python/callback.rs` - pyclass, __call__ | TODO |
| **15.11** | Review Embedded Python | `src/python/embedded.rs` - file completeness | TODO |
| **15.12** | Review Public API | `src/lib.rs` - exports, docs | TODO |
| **15.13** | Review Build Config | `build.rs`, `Cargo.toml`, `.cargo/config.toml` | TODO |
| **15.14** | Review Python Library | Full review of `python/lex_learning/src/` | TODO |
| **15.15** | Documentation Review | Doc comments, AGENTS.md, README.md | TODO |
| **15.16** | Integration Test Verification | End-to-end training, save/load, predict | TODO |

#### Sub-Phase Details

**15.1 - Verify Build and Tests:**
- `cargo check` - compilation succeeds
- `cargo test` - 19 unit tests pass
- `cargo test -- --ignored` - 18 integration tests pass
- `cargo clippy` - no warnings
- `cargo doc` - documentation builds
- Python tests: `cd python/lex_learning && uv run pytest -v` - 112 tests pass

**15.2 - Review Error Handling (`src/error.rs`, 93 lines):**
- Error variant coverage for all failure modes
- Error messages are descriptive and actionable
- `From<PyErr>` and `From<Infallible>` implementations
- Consider `#[non_exhaustive]` for future extensibility

**15.3 - Review Configuration (`src/config.rs`, 239 lines):**
- Builder pattern is idiomatic Rust
- Validation is comprehensive (test_size bounds, cv_folds > 1, etc.)
- Default values are sensible
- Add Serde derives if needed for serialization

**15.4 - Review Types (`src/types.rs`, 115 lines):**
- Struct field visibility (pub vs pub(crate))
- Derive macros (Debug, Clone, Serialize, etc.)
- Optional vs required fields
- Consider `#[non_exhaustive]` for public structs

**15.5 - Review Progress Types (`src/progress.rs`, 124 lines):**
- TrainingStage enum completeness
- Progress callback type ergonomics
- Thread safety considerations

**15.6 - Review Pipeline (`src/pipeline.rs`, 428 lines):**
- Builder pattern correctness
- Error handling in `train()` method
- Progress callback wiring
- `create_trained_model()` error handling
- Memory management of training results

**15.7 - Review TrainedModel (`src/model.rs`, 678 lines):**
- `Py<PyAny>` lifecycle management
- `save()`/`load()` error handling
- `to_bytes()`/`from_bytes()` correctness
- `predict()` single instance handling
- `predict_batch()` DataFrame conversion
- Property accessors error handling

**15.8 - Review Python Runtime (`src/python/runtime.rs`, 423 lines):**
- Initialization is idempotent (OnceLock usage)
- `sys.executable` fix applied correctly
- PYTHONHOME/PYTHONPATH setup
- Platform detection
- Error messages for runtime not found

**15.9 - Review Arrow Conversion (`src/python/conversion.rs`, 684 lines):**
- Rust→Python DataFrame via Arrow IPC
- Python→Rust DataFrame via Arrow IPC
- Config to Python dict conversion
- Training result extraction
- Error mapping from Python exceptions

**15.10 - Review Callback Bridge (`src/python/callback.rs`, 134 lines):**
- `#[pyclass]` implementation correctness
- `__call__` method handles all ProgressUpdate fields
- Thread safety with Arc callback

**15.11 - Review Embedded Python (`src/python/embedded.rs`, 187 lines):**
- All Python source files are embedded
- File paths are correct
- No missing files

**15.12 - Review Public API (`src/lib.rs`, 68 lines):**
- All public types are re-exported
- Documentation is present
- `initialize()` and `is_initialized()` are correct

**15.13 - Review Build Config:**
- `build.rs` - platform detection, linker paths, rpath
- `Cargo.toml` - dependencies, features, edition
- `.cargo/config.toml` - PYO3_PYTHON path

**15.14 - Review Python Library (`python/lex_learning/src/`, ~2,618 lines):**
- Code structure and organization
- Error handling patterns
- Type hints completeness
- Docstrings on public API
- Known issues to address:
  - Empty `TYPE_CHECKING` blocks cleanup
  - Dynamic attribute creation in `stages.py`
  - `KerasClassifier`/`KerasRegressor` duplication
  - Deprecated XGBoost `use_label_encoder` parameter
  - Pickle security warning visibility
  - Import inside function (`models/__init__.py`)

**15.15 - Documentation Review:**
- Rust doc comments on all public items
- AGENTS.md accuracy and completeness
- README.md accuracy
- Code examples work

**15.16 - Integration Test Verification:**
- End-to-end training flow
- Progress callbacks fire correctly
- Model save/load roundtrip
- Prediction works after training

### Cargo.toml Dependencies

```toml
[package]
name = "lex-learning"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["lib"]

[dependencies]
# PyO3 for Python interop (use latest 0.27+)
pyo3 = { version = "0.27", features = ["auto-initialize"] }

# Polars for DataFrames
polars = { version = "0.46", features = ["lazy", "dtype-full", "ipc"] }

# Error handling
thiserror = "2.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[build-dependencies]
# For build.rs Python runtime setup
```

---

## 9. Model Registry

### Classification Models

| Name | Library | Notes |
|------|---------|-------|
| `logistic_regression` | sklearn | Good baseline, interpretable |
| `decision_tree` | sklearn | Interpretable, fast |
| `random_forest` | sklearn | Robust, popular |
| `gradient_boosting` | sklearn | Strong performer |
| `extra_trees` | sklearn | Fast training |
| `knn` | sklearn | Simple, non-parametric |
| `svm` | sklearn | Good for small/medium data |
| `xgboost` | xgboost | State-of-the-art boosting |
| `lightgbm` | lightgbm | Fast, memory efficient |
| `neural_network` | tensorflow | Deep learning |

### Regression Models

| Name | Library | Notes |
|------|---------|-------|
| `linear_regression` | sklearn | Simple baseline |
| `ridge` | sklearn | L2 regularization |
| `lasso` | sklearn | L1 regularization |
| `decision_tree` | sklearn | Interpretable |
| `random_forest` | sklearn | Robust |
| `gradient_boosting` | sklearn | Strong performer |
| `extra_trees` | sklearn | Fast |
| `knn` | sklearn | Non-parametric |
| `svr` | sklearn | Support vector regression |
| `xgboost` | xgboost | State-of-the-art |
| `lightgbm` | lightgbm | Fast, efficient |
| `neural_network` | tensorflow | Deep learning |

---

## 10. Testing

### Python Tests (Implemented)

- 112 tests passing
- Located in `python/lex_learning/tests/`
- Run with: `cd python/lex_learning && uv run python -m pytest -v`

### Rust Tests (To Be Implemented)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pipeline_config_builder() { ... }
    
    #[test]
    fn test_train_classification() { ... }
    
    #[test]
    fn test_train_regression() { ... }
    
    #[test]
    fn test_model_save_load() { ... }
    
    #[test]
    fn test_predict_single() { ... }
    
    #[test]
    fn test_predict_batch() { ... }
}
```

---

## 11. Integration with lex-machina

When adding to the lex-machina workspace:

```toml
# lex-machina/Cargo.toml
[workspace]
members = [
    "src-tauri",
    "lex-processing",
    "lex-learning",  # Add this
]

# lex-machina/src-tauri/Cargo.toml
[dependencies]
lex-processing = { path = "../lex-processing" }
lex-learning = { path = "../lex-learning" }
```

**Usage in Tauri:**

```rust
use lex_processing::process_csv;
use lex_learning::{Pipeline, PipelineConfig, ProblemType};

#[tauri::command]
async fn train_model(csv_path: String, config: PipelineConfig) -> Result<TrainingResult, String> {
    // Process data with lex-processing
    let clean_df = process_csv(&csv_path)?;
    
    // Train with lex-learning (run in blocking task)
    let result = tokio::task::spawn_blocking(move || {
        let pipeline = Pipeline::builder()
            .config(config)
            .build()?;
        
        pipeline.train(&clean_df)
    }).await??;
    
    Ok(result)
}
```

---

## 12. Key PyO3 Patterns

### Calling Python from Rust

```rust
use pyo3::prelude::*;

Python::attach(|py| {
    // Import module
    let lex_learning = py.import("lex_learning")?;
    
    // Get class and call method
    let pipeline_class = lex_learning.getattr("Pipeline")?;
    let builder = pipeline_class.call_method0("builder")?;
    
    // Call with arguments
    let result = builder.call_method1("config", (py_config,))?;
    
    Ok(())
})
```

### Holding Python Objects

```rust
pub struct TrainedModel {
    py_model: Py<PyAny>,  // Owned reference, GIL-independent
}

impl TrainedModel {
    pub fn predict(&self, instance: &JsonValue) -> Result<PredictionResult, LexLearningError> {
        Python::attach(|py| {
            let py_dict = json_to_pydict(py, instance)?;
            let result = self.py_model
                .bind(py)
                .call_method1("predict", (py_dict,))?;
            
            extract_prediction_result(py, &result)
        })
    }
}
```

### Progress Callback Bridge

```rust
use pyo3::types::PyCFunction;

pub fn create_progress_callback<'py>(
    py: Python<'py>,
    callback: Arc<dyn Fn(ProgressUpdate) + Send + Sync>,
) -> PyResult<Bound<'py, PyAny>> {
    PyCFunction::new_closure(
        py,
        None,
        None,
        move |args: &Bound<'_, PyTuple>, _kwargs| -> PyResult<()> {
            let py_update = args.get_item(0)?;
            let update = extract_progress_update(&py_update)?;
            callback(update);
            Ok(())
        },
    )
}
```

---

## 13. Notes for AI Agents

### Before Implementing

1. **Get latest docs** using MCP tools:
   - `resolve-library-id` to find library IDs
   - `get-library-docs` with relevant topics

2. **Check existing code** in `python/lex_learning/src/` for Python API patterns

3. **Ask the user** if unsure about:
   - Architecture decisions
   - API naming
   - Trade-offs between approaches

### Key Files to Reference

| Purpose | File |
|---------|------|
| Python public API | `python/lex_learning/src/__init__.py` |
| Python config | `python/lex_learning/src/config.py` |
| Python errors | `python/lex_learning/src/errors.py` |
| Python types | `python/lex_learning/src/core/types.py` |
| Python metrics | `python/lex_learning/src/core/metrics.py` |
| Python pipeline | `python/lex_learning/src/pipeline/orchestrator.py` |
| Python TrainedModel | `python/lex_learning/src/inference/model.py` |
| Python tests | `python/lex_learning/tests/` |

### Common Pitfalls

1. **PyO3 GIL**: Always use `Python::attach()` for Python operations
2. **Arrow conversion**: Use IPC format for zero-copy transfer
3. **Error mapping**: Map Python exceptions to Rust error variants
4. **Memory**: `Py<PyAny>` holds references without GIL, use `.bind(py)` when needed
5. **Python path**: Ensure PYTHONHOME and PYTHONPATH are set before importing

---

*Last Updated: 2026-01-01*
*Rust Edition: 2024 | PyO3: 0.27+ | Python: 3.12*
