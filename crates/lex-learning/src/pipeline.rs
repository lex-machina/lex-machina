//! Training pipeline implementation.
//!
//! This module provides the [`Pipeline`] struct and its builder for running
//! the ML training pipeline. The pipeline orchestrates data preprocessing,
//! algorithm selection, model training, evaluation, and explainability analysis.
//!
//! # Overview
//!
//! The training pipeline executes these stages in order:
//!
//! 1. **Preprocessing** - Encode categorical features, scale numerical features
//! 2. **Algorithm Selection** - Evaluate candidate algorithms on your data
//! 3. **Training** - Train selected models with hyperparameter optimization
//! 4. **Evaluation** - Compute metrics on the test set
//! 5. **Explainability** - Generate SHAP explanations (optional)
//!
//! # Example
//!
//! ```rust,ignore
//! use lex_learning::{Pipeline, PipelineConfig, ProblemType};
//!
//! // Initialize Python runtime (once at startup)
//! lex_learning::initialize()?;
//!
//! // Configure the pipeline
//! let config = PipelineConfig::builder()
//!     .problem_type(ProblemType::Classification)
//!     .target_column("Survived")
//!     .build()?;
//!
//! // Build and run the pipeline
//! let mut pipeline = Pipeline::builder()
//!     .config(config)
//!     .on_progress(|update| {
//!         println!("[{:?}] {:.0}% - {}", update.stage, update.progress * 100.0, update.message);
//!     })
//!     .build()?;
//!
//! let result = pipeline.train(&dataframe)?;
//! println!("Best model: {} (accuracy: {:?})", result.best_model_name, result.metrics.accuracy);
//!
//! // Create a model for inference
//! let model = pipeline.create_trained_model()?;
//! model.save("model.pkl")?;
//! ```
//!
//! # Thread Safety
//!
//! The [`Pipeline`] is not `Send` or `Sync` because it holds Python objects.
//! Training must happen on the thread that initialized the Python runtime.
//! For async applications, use `spawn_blocking` or similar mechanisms.

use pyo3::prelude::*;

use crate::cancellation::CancellationToken;
use crate::config::PipelineConfig;
use crate::error::LexLearningError;
use crate::model::TrainedModel;
use crate::progress::ProgressCallback;
use crate::python::callback::{PyCancellationChecker, PyProgressCallback};
use crate::python::conversion;
use crate::types::TrainingResult;
use polars::prelude::DataFrame;
use pyo3::types::PyTuple;

/// The ML training pipeline.
///
/// Orchestrates the complete ML training workflow including preprocessing,
/// algorithm selection, training, evaluation, and explainability.
///
/// Use [`Pipeline::builder()`] to construct a pipeline with the builder pattern.
///
/// # Lifecycle
///
/// 1. Create a pipeline with [`Pipeline::builder()`]
/// 2. Call [`train()`](Self::train) with your data
/// 3. Optionally call [`create_trained_model()`](Self::create_trained_model) to get a model for inference
///
/// # Example
///
/// ```rust,ignore
/// let mut pipeline = Pipeline::builder()
///     .config(config)
///     .on_progress(|u| println!("{}", u.message))
///     .build()?;
///
/// let result = pipeline.train(&df)?;
/// let model = pipeline.create_trained_model()?;
/// ```
///
/// # Thread Safety
///
/// `Pipeline` is not `Send` or `Sync` due to holding Python objects internally.
pub struct Pipeline {
    config: PipelineConfig,
    progress_callback: Option<ProgressCallback>,
    cancellation_token: Option<CancellationToken>,
    /// Stores the Python TrainingResult for create_trained_model()
    last_py_result: Option<Py<PyAny>>,
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("config", &self.config)
            .field(
                "progress_callback",
                &self.progress_callback.as_ref().map(|_| "<callback>"),
            )
            .field(
                "cancellation_token",
                &self.cancellation_token.as_ref().map(|_| "<token>"),
            )
            .field(
                "last_py_result",
                &self.last_py_result.as_ref().map(|_| "<PyObject>"),
            )
            .finish()
    }
}

impl Pipeline {
    /// Create a new builder for `Pipeline`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let pipeline = Pipeline::builder()
    ///     .config(config)
    ///     .build()?;
    /// ```
    #[must_use]
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::default()
    }

    /// Run the training pipeline on the provided DataFrame.
    ///
    /// This is the main entry point for training. It executes all pipeline stages
    /// and returns a [`TrainingResult`] with metrics and model information.
    ///
    /// # Arguments
    ///
    /// * `df` - The training data as a Polars DataFrame. Must contain:
    ///   - Feature columns (numeric or categorical)
    ///   - Target column (last column, or specified in config)
    ///   - No null values
    ///   - No datetime columns
    ///
    /// # Returns
    ///
    /// Returns [`TrainingResult`] containing:
    /// - Best model name and metrics
    /// - Feature importance scores
    /// - SHAP plots (if enabled)
    /// - Comparison of all evaluated models
    ///
    /// # Errors
    ///
    /// Returns [`LexLearningError`] if:
    /// - [`RuntimeInit`](LexLearningError::RuntimeInit): Python runtime not initialized (call [`initialize()`](crate::initialize) first)
    /// - [`TargetNotFound`](LexLearningError::TargetNotFound): Target column not found in DataFrame
    /// - [`InvalidData`](LexLearningError::InvalidData): Data validation fails (nulls, insufficient rows, etc.)
    /// - [`TrainingFailed`](LexLearningError::TrainingFailed): All models failed to train
    /// - [`Cancelled`](LexLearningError::Cancelled): Training was cancelled via progress callback
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut pipeline = Pipeline::builder()
    ///     .config(config)
    ///     .build()?;
    ///
    /// let result = pipeline.train(&df)?;
    /// println!("Best model: {}", result.best_model_name);
    /// println!("Accuracy: {:?}", result.metrics.accuracy);
    /// ```
    pub fn train(&mut self, df: &DataFrame) -> Result<TrainingResult, LexLearningError> {
        Python::attach(|py| {
            // 1. Convert Polars DataFrame to pandas DataFrame
            let pandas_df = conversion::dataframe_to_python(py, df)?;

            // 2. Convert Rust config to Python PipelineConfig
            let py_config = conversion::config_to_python(py, &self.config)?;

            // 3. Build Python Pipeline with optional progress callback and cancellation check
            let lex_learning = py.import("lex_learning")?;
            let pipeline_class = lex_learning.getattr("Pipeline")?;
            let mut builder = pipeline_class.call_method0("builder")?;
            builder = builder.call_method1("config", (&py_config,))?;

            // Pass progress callback if provided
            let mut py_callback: Option<Bound<'_, PyAny>> = None;
            if let Some(ref callback) = self.progress_callback {
                py_callback =
                    Some(Bound::new(py, PyProgressCallback::new(callback.clone()))?.into_any());
            }

            let mut py_cancellation_check: Option<Bound<'_, PyAny>> = None;
            if let Some(ref token) = self.cancellation_token {
                let check_fn = token.as_check_fn();
                let py_check = PyCancellationChecker::new(check_fn);
                py_cancellation_check = Some(Bound::new(py, py_check)?.into_any());
            }

            if py_callback.is_some() || py_cancellation_check.is_some() {
                let args = PyTuple::new(py, [py_callback.clone(), py_cancellation_check.clone()])?;

                builder = builder.call_method1("on_progress", args)?;
            }

            let py_pipeline = builder.call_method0("build")?;

            // 4. Call train() - map Python exceptions to Rust errors
            let py_result = py_pipeline
                .call_method1("train", (&pandas_df,))
                .map_err(|e| conversion::map_python_error(py, e))?;

            // 5. Store Python result for create_trained_model()
            self.last_py_result = Some(py_result.clone().unbind());

            // 6. Extract serializable TrainingResult
            conversion::extract_training_result(py, &py_result)
        })
    }

    /// Create a [`TrainedModel`] from the last training result.
    ///
    /// This keeps the model in Python memory for efficient inference.
    /// Must be called after [`train()`](Self::train).
    ///
    /// # Returns
    ///
    /// Returns a [`TrainedModel`] that can be used for predictions or saved to disk.
    ///
    /// # Errors
    ///
    /// Returns [`LexLearningError::InvalidConfig`] if [`train()`](Self::train) has not been called yet.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = pipeline.train(&df)?;
    /// let model = pipeline.create_trained_model()?;
    ///
    /// // Use for inference
    /// let prediction = model.predict(&json!({"Age": 25, "Fare": 50.0}))?;
    ///
    /// // Or save to disk
    /// model.save("model.pkl")?;
    /// ```
    pub fn create_trained_model(&self) -> Result<TrainedModel, LexLearningError> {
        let py_result = self.last_py_result.as_ref().ok_or_else(|| {
            LexLearningError::InvalidConfig(
                "No training result available. Call train() first.".to_string(),
            )
        })?;

        Python::attach(|py| {
            // 1. Convert Rust config to Python config (needed for Pipeline)
            let py_config = conversion::config_to_python(py, &self.config)?;

            // 2. Create a Python Pipeline (needed to call create_trained_model)
            let lex_learning = py.import("lex_learning")?;
            let pipeline_class = lex_learning.getattr("Pipeline")?;
            let builder = pipeline_class.call_method0("builder")?;
            let builder = builder.call_method1("config", (&py_config,))?;
            let py_pipeline = builder.call_method0("build")?;

            // 3. Call create_trained_model on Python Pipeline with the stored result
            let py_trained_model = py_pipeline
                .call_method1("create_trained_model", (py_result.bind(py),))
                .map_err(|e| conversion::map_python_error(py, e))?;

            // 4. Wrap the Python TrainedModel in Rust TrainedModel
            Ok(TrainedModel::from_py_object(py_trained_model.unbind()))
        })
    }

    /// Get the pipeline configuration.
    ///
    /// Returns a reference to the [`PipelineConfig`] used by this pipeline.
    #[must_use]
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    /// Returns `true` if training has been completed and a model is available.
    ///
    /// Use this to check if [`create_trained_model()`](Self::create_trained_model) can be called.
    #[must_use]
    pub fn has_training_result(&self) -> bool {
        self.last_py_result.is_some()
    }
}

/// Builder for [`Pipeline`].
///
/// Created via [`Pipeline::builder()`]. Use method chaining to configure
/// the pipeline, then call [`build()`](Self::build) to create the pipeline.
///
/// # Required Configuration
///
/// - [`config()`](Self::config): Pipeline configuration (required)
///
/// # Optional Configuration
///
/// - [`on_progress()`](Self::on_progress): Progress callback for monitoring
/// - [`cancellation_token()`](Self::cancellation_token): Token for cancellation
///
/// # Example
///
/// ```rust,ignore
/// let pipeline = Pipeline::builder()
///     .config(config)
///     .on_progress(|update| println!("{}", update.message))
///     .cancellation_token(token)
///     .build()?;
/// ```
#[derive(Default)]
pub struct PipelineBuilder {
    config: Option<PipelineConfig>,
    progress_callback: Option<ProgressCallback>,
    cancellation_token: Option<CancellationToken>,
}

impl std::fmt::Debug for PipelineBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineBuilder")
            .field("config", &self.config)
            .field(
                "progress_callback",
                &self.progress_callback.as_ref().map(|_| "<callback>"),
            )
            .field(
                "cancellation_token",
                &self.cancellation_token.as_ref().map(|_| "<token>"),
            )
            .finish()
    }
}

impl PipelineBuilder {
    /// Set the pipeline configuration (required).
    ///
    /// The configuration specifies the problem type, target column, and
    /// training parameters.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = PipelineConfig::builder()
    ///     .problem_type(ProblemType::Classification)
    ///     .target_column("target")
    ///     .build()?;
    ///
    /// let pipeline = Pipeline::builder()
    ///     .config(config)
    ///     .build()?;
    /// ```
    #[must_use]
    pub fn config(mut self, config: PipelineConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the progress callback (optional).
    ///
    /// The callback will be invoked with [`ProgressUpdate`](crate::ProgressUpdate)
    /// structs during training to report current status.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let pipeline = Pipeline::builder()
    ///     .config(config)
    ///     .on_progress(|update| {
    ///         println!("[{:?}] {} - {}", update.stage, update.progress, update.message);
    ///     })
    ///     .build()?;
    /// ```
    ///
    /// # Note
    ///
    /// The callback should execute quickly to avoid blocking training.
    /// For expensive operations (like UI updates), consider using channels.
    #[must_use]
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(crate::progress::ProgressUpdate) + Send + Sync + 'static,
    {
        self.progress_callback = Some(std::sync::Arc::new(callback));
        self
    }

    /// Set the cancellation token (optional).
    ///
    /// When a cancellation token is provided, the training pipeline will
    /// periodically check if cancellation has been requested and stop
    /// processing if so.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use lex_learning::CancellationToken;
    ///
    /// let token = CancellationToken::new();
    /// let pipeline = Pipeline::builder()
    ///     .config(config)
    ///     .cancellation_token(token)
    ///     .build()?;
    ///
    /// // Later, from another thread or UI event:
    /// // token.cancel();
    /// ```
    ///
    /// # See Also
    ///
    /// - [`CancellationToken::cancel()`] to signal cancellation
    /// - [`CancellationToken::is_cancelled()`] to check cancellation status
    #[must_use]
    pub fn cancellation_token(mut self, token: CancellationToken) -> Self {
        self.cancellation_token = Some(token);
        self
    }

    /// Build the pipeline.
    ///
    /// # Errors
    ///
    /// Returns [`LexLearningError::InvalidConfig`] if no configuration was provided.
    /// Call [`config()`](Self::config) before calling `build()`.
    pub fn build(self) -> Result<Pipeline, LexLearningError> {
        let config = self.config.ok_or_else(|| {
            LexLearningError::InvalidConfig("Pipeline config is required".to_string())
        })?;

        Ok(Pipeline {
            config,
            progress_callback: self.progress_callback,
            cancellation_token: self.cancellation_token,
            last_py_result: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProblemType;

    #[test]
    fn test_pipeline_builder_requires_config() {
        let result = Pipeline::builder().build();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, LexLearningError::InvalidConfig(_)));
        assert!(err.to_string().contains("config is required"));
    }

    #[test]
    fn test_pipeline_builder_with_config() {
        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Classification)
            .build()
            .unwrap();

        let pipeline = Pipeline::builder().config(config).build().unwrap();

        assert_eq!(pipeline.config().problem_type, ProblemType::Classification);
        assert!(!pipeline.has_training_result());
    }

    #[test]
    fn test_pipeline_builder_debug() {
        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Classification)
            .build()
            .unwrap();

        let builder = Pipeline::builder().config(config);
        let debug_str = format!("{:?}", builder);
        assert!(debug_str.contains("PipelineBuilder"));
    }

    #[test]
    #[ignore = "Requires Python runtime with ML libraries"]
    fn test_train_classification() {
        use polars::prelude::*;

        crate::initialize().expect("Failed to initialize Python");

        let df = df! {
            "Age" => &[22i64, 38, 26, 35, 28, 19, 40, 66, 28, 42],
            "Fare" => &[7.25f64, 71.28, 7.92, 53.10, 8.05, 8.46, 27.72, 10.50, 7.23, 52.00],
            "Pclass" => &[3i64, 1, 3, 1, 3, 3, 1, 3, 3, 1],
            "Survived" => &[0i64, 1, 1, 1, 0, 0, 0, 0, 1, 1],
        }
        .unwrap();

        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Classification)
            .target_column("Survived")
            .optimize_hyperparams(false) // Faster for tests
            .enable_explainability(false) // Faster for tests
            .enable_neural_networks(false) // Faster for tests
            .top_k_algorithms(1)
            .cv_folds(2) // Reduce for small test data (10 rows)
            .build()
            .unwrap();

        let mut pipeline = Pipeline::builder().config(config).build().unwrap();
        let result = pipeline.train(&df).expect("Training should succeed");

        assert!(result.success);
        assert!(!result.best_model_name.is_empty());
        assert!(result.metrics.accuracy.is_some());
        assert!(!result.model_comparison.is_empty());
    }

    #[test]
    #[ignore = "Requires Python runtime with ML libraries"]
    fn test_train_regression() {
        use polars::prelude::*;

        crate::initialize().expect("Failed to initialize Python");

        let df = df! {
            "feature1" => &[1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
            "feature2" => &[2.0f64, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0],
            "target" => &[3.0f64, 6.0, 9.0, 12.0, 15.0, 18.0, 21.0, 24.0, 27.0, 30.0],
        }
        .unwrap();

        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Regression)
            .target_column("target")
            .optimize_hyperparams(false)
            .enable_explainability(false)
            .enable_neural_networks(false)
            .top_k_algorithms(1)
            .cv_folds(2) // Reduce for small test data (10 rows)
            .build()
            .unwrap();

        let mut pipeline = Pipeline::builder().config(config).build().unwrap();
        let result = pipeline.train(&df).expect("Training should succeed");

        assert!(result.success);
        assert!(!result.best_model_name.is_empty());
        assert!(result.metrics.r2.is_some());
        assert!(!result.model_comparison.is_empty());
    }

    #[test]
    #[ignore = "Requires Python runtime with ML libraries"]
    fn test_train_without_target_column_uses_last() {
        use polars::prelude::*;

        crate::initialize().expect("Failed to initialize Python");

        // Target column is last (default behavior)
        let df = df! {
            "feature1" => &[1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
            "feature2" => &[2.0f64, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0],
            "target" => &[3.0f64, 6.0, 9.0, 12.0, 15.0, 18.0, 21.0, 24.0, 27.0, 30.0],
        }
        .unwrap();

        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Regression)
            // Note: no target_column specified, should use last column
            .optimize_hyperparams(false)
            .enable_explainability(false)
            .enable_neural_networks(false)
            .top_k_algorithms(1)
            .cv_folds(2) // Reduce for small test data (10 rows)
            .build()
            .unwrap();

        let mut pipeline = Pipeline::builder().config(config).build().unwrap();
        let result = pipeline.train(&df).expect("Training should succeed");

        assert!(result.success);
    }

    #[test]
    fn test_create_trained_model_requires_train() {
        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Classification)
            .build()
            .unwrap();

        let pipeline = Pipeline::builder().config(config).build().unwrap();

        // Should fail because train() hasn't been called
        let result = pipeline.create_trained_model();

        // Should return InvalidConfig error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, LexLearningError::InvalidConfig(_)),
            "Expected InvalidConfig error, got: {:?}",
            err
        );
    }

    #[test]
    #[ignore = "Requires Python runtime with ML libraries"]
    fn test_train_with_progress_callback() {
        use polars::prelude::*;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};

        crate::initialize().expect("Failed to initialize Python");

        let df = df! {
            "Age" => &[22i64, 38, 26, 35, 28, 19, 40, 66, 28, 42],
            "Fare" => &[7.25f64, 71.28, 7.92, 53.10, 8.05, 8.46, 27.72, 10.50, 7.23, 52.00],
            "Pclass" => &[3i64, 1, 3, 1, 3, 3, 1, 3, 3, 1],
            "Survived" => &[0i64, 1, 1, 1, 0, 0, 0, 0, 1, 1],
        }
        .unwrap();

        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Classification)
            .target_column("Survived")
            .optimize_hyperparams(false)
            .enable_explainability(false)
            .enable_neural_networks(false)
            .top_k_algorithms(1)
            .cv_folds(2)
            .build()
            .unwrap();

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        // Track stages we've seen
        let stages_seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let stages_seen_clone = stages_seen.clone();

        let mut pipeline = Pipeline::builder()
            .config(config)
            .on_progress(move |update| {
                call_count_clone.fetch_add(1, Ordering::SeqCst);

                // Verify progress is in valid range
                assert!(
                    (0.0..=1.0).contains(&update.progress),
                    "Progress {} should be between 0.0 and 1.0",
                    update.progress
                );

                // Verify message is not empty
                assert!(!update.message.is_empty(), "Message should not be empty");

                // Track the stage
                stages_seen_clone
                    .lock()
                    .unwrap()
                    .push(update.stage.as_str().to_string());
            })
            .build()
            .unwrap();

        let result = pipeline.train(&df).expect("Training should succeed");

        // Verify training succeeded
        assert!(result.success);

        // Verify callback was called multiple times
        let count = call_count.load(Ordering::SeqCst);
        assert!(
            count > 0,
            "Progress callback should have been called at least once"
        );

        // Verify we saw some stages
        let stages = stages_seen.lock().unwrap();
        assert!(!stages.is_empty(), "Should have seen at least one stage");

        // Should have seen "initializing" as the first stage
        assert_eq!(
            stages.first(),
            Some(&"initializing".to_string()),
            "First stage should be initializing"
        );

        // Should have seen "complete" as the last stage
        assert_eq!(
            stages.last(),
            Some(&"complete".to_string()),
            "Last stage should be complete"
        );
    }
}
