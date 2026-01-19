//! lex-learning: Automated ML training library with embedded Python.
//!
//! This crate provides a Rust API for training machine learning models using
//! a bundled Python 3.12 runtime and the `lex_learning` Python library. It
//! enables automated model selection, hyperparameter optimization, and
//! SHAP-based explainability without requiring a system Python installation.
//!
//! # Features
//!
//! - **Automated ML Pipeline**: Train models with minimal configuration
//! - **Model Selection**: Automatic algorithm selection from sklearn, XGBoost, LightGBM
//! - **Hyperparameter Optimization**: Optuna-based tuning with cross-validation
//! - **Explainability**: SHAP feature importance and visualization
//! - **Bundled Runtime**: Self-contained Python 3.12 with all dependencies
//! - **Progress Reporting**: Real-time training progress callbacks
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use lex_learning::{Pipeline, PipelineConfig, ProblemType, TrainedModel};
//! use polars::prelude::*;
//!
//! // Initialize Python runtime (call once at startup)
//! lex_learning::initialize()?;
//!
//! // Configure the pipeline
//! let config = PipelineConfig::builder()
//!     .problem_type(ProblemType::Classification)
//!     .target_column("Survived")
//!     .build()?;
//!
//! // Build and run the pipeline
//! let pipeline = Pipeline::builder()
//!     .config(config)
//!     .on_progress(|u| println!("{:.0}% - {}", u.progress * 100.0, u.message))
//!     .build()?;
//!
//! let result = pipeline.train(&dataframe)?;
//!
//! // Keep model in memory for inference
//! let model = pipeline.create_trained_model(&result)?;
//!
//! // Single prediction
//! let prediction = model.predict(&serde_json::json!({"Age": 25, "Sex": "male"}))?;
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        Rust Application                         │
//! │                                                                 │
//! │  PipelineConfig ──► Pipeline ──► TrainingResult ──► TrainedModel│
//! │                                                                 │
//! └───────────────────────────┬─────────────────────────────────────┘
//!                             │ PyO3
//!                             ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                   Bundled Python Runtime                        │
//! │                                                                 │
//! │  lex_learning package (embedded at compile time)                │
//! │  ├── sklearn, XGBoost, LightGBM, TensorFlow                    │
//! │  ├── Optuna (hyperparameter optimization)                       │
//! │  └── SHAP (explainability)                                      │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Initialization
//!
//! The Python runtime must be initialized before any ML operations:
//!
//! ```rust,ignore
//! // At application startup
//! lex_learning::initialize()?;
//!
//! // Check if already initialized
//! if lex_learning::is_initialized() {
//!     println!("Python runtime ready");
//! }
//! ```
//!
//! Initialization is idempotent - calling [`initialize()`] multiple times is safe.
//!
//! # Error Handling
//!
//! All fallible operations return [`Result<T, LexLearningError>`]. The error
//! type provides specific variants for different failure modes:
//!
//! - [`LexLearningError::RuntimeInit`] - Python runtime initialization failed
//! - [`LexLearningError::InvalidConfig`] - Invalid pipeline configuration
//! - [`LexLearningError::InvalidData`] - Invalid input data
//! - [`LexLearningError::TrainingFailed`] - Model training failed
//! - [`LexLearningError::InferenceError`] - Prediction failed
//!
//! See [`LexLearningError`] for the complete list.
//!
//! # Thread Safety
//!
//! The Python GIL (Global Interpreter Lock) is managed automatically by PyO3.
//! Multiple threads can call into Python, but only one will execute at a time.
//! For CPU-bound training, this is typically not a bottleneck since the heavy
//! computation happens in native code (numpy, sklearn, etc.).
//!
//! # Model Persistence
//!
//! Trained models can be saved and loaded:
//!
//! ```rust,ignore
//! // Save model
//! model.save("model.pkl")?;
//!
//! // Load model (in a new session)
//! lex_learning::initialize()?;
//! let model = TrainedModel::load("model.pkl")?;
//! let prediction = model.predict(&input)?;
//! ```
//!
//! Models can also be serialized to bytes for custom storage:
//!
//! ```rust,ignore
//! let bytes = model.to_bytes()?;
//! let model = TrainedModel::from_bytes(&bytes)?;
//! ```
//!
//! # Modules
//!
//! - [`python`] - Python runtime management and interop utilities

mod cancellation;
mod config;
mod error;
mod model;
mod pipeline;
mod progress;
pub mod python;
mod types;

// Re-export public API
//
// Configuration types
pub use config::{PipelineConfig, PipelineConfigBuilder, ProblemType};
// Cancellation token
pub use cancellation::CancellationToken;
// Error types
pub use error::LexLearningError;
// Model types
pub use model::TrainedModel;
// Pipeline types
pub use pipeline::{Pipeline, PipelineBuilder};
// Progress reporting types
pub use progress::{ProgressCallback, ProgressUpdate, TrainingStage};
// Result and metrics types
pub use types::{Metrics, ModelComparison, ModelInfo, PredictionResult, TrainingResult};

/// Initialize the Python runtime.
///
/// This function must be called once before any ML operations. It:
///
/// 1. Locates the bundled Python runtime directory
/// 2. Sets `PYTHONHOME` and `PYTHONPATH` environment variables
/// 3. Initializes the Python interpreter via PyO3
/// 4. Fixes `sys.executable` for joblib multiprocessing compatibility
/// 5. Extracts embedded Python source files if needed
/// 6. Verifies the `lex_learning` package can be imported
///
/// # Idempotence
///
/// This function is safe to call multiple times. Subsequent calls after the
/// first successful initialization return immediately with `Ok(())`.
///
/// # Example
///
/// ```rust,ignore
/// // At application startup
/// lex_learning::initialize()?;
///
/// // Safe to call again
/// lex_learning::initialize()?; // Returns Ok(()) immediately
/// ```
///
/// # Errors
///
/// Returns [`LexLearningError::RuntimeInit`] if:
/// - The bundled Python runtime directory cannot be found
/// - Environment variable setup fails
/// - Python interpreter initialization fails
/// - The `lex_learning` package cannot be imported
///
/// # Panics
///
/// This function does not panic. All errors are returned as `Result`.
///
/// [`LexLearningError::RuntimeInit`]: crate::LexLearningError::RuntimeInit
#[must_use = "initialization errors should be handled"]
pub fn initialize() -> Result<(), LexLearningError> {
    python::runtime::initialize()
}

/// Check if the Python runtime has been successfully initialized.
///
/// Returns `true` if [`initialize()`] has been called and completed successfully,
/// `false` otherwise.
///
/// # Example
///
/// ```rust,ignore
/// if !lex_learning::is_initialized() {
///     lex_learning::initialize()?;
/// }
///
/// // Now safe to use ML operations
/// let pipeline = Pipeline::builder()
///     .config(config)
///     .build()?;
/// ```
///
/// # Thread Safety
///
/// This function is thread-safe and can be called from any thread at any time.
#[must_use = "the initialization status should be checked"]
pub fn is_initialized() -> bool {
    python::runtime::is_initialized()
}
