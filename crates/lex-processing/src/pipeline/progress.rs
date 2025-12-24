//! Progress reporting and cancellation support for the preprocessing pipeline.
//!
//! This module provides types for tracking pipeline progress and supporting
//! cancellation from external threads (e.g., UI cancel button).
//!
//! # Example
//!
//! ```rust,ignore
//! use data_preprocessing_pipeline::{Pipeline, CancellationToken};
//!
//! let token = CancellationToken::new();
//! let token_clone = token.clone();
//!
//! // In another thread
//! std::thread::spawn(move || {
//!     std::thread::sleep(std::time::Duration::from_secs(5));
//!     token_clone.cancel();
//! });
//!
//! let result = Pipeline::builder()
//!     .cancellation_token(token)
//!     .on_progress(|update| {
//!         println!("[{:?}] {}", update.stage, update.message);
//!     })
//!     .build()?
//!     .process(df);
//! ```

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Stages of the preprocessing pipeline.
///
/// Each stage represents a major phase of the preprocessing workflow.
/// Progress updates include both the current stage and optional sub-stage
/// information for more granular tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreprocessingStage {
    /// Pipeline is initializing and loading data
    Initializing,
    /// Profiling the dataset (type inference, statistics)
    Profiling,
    /// Analyzing data quality and detecting issues
    QualityAnalysis,
    /// Correcting column types (string to numeric, etc.)
    TypeCorrection,
    /// Making preprocessing decisions (AI or rule-based)
    DecisionMaking,
    /// Cleaning the dataset (removing duplicates, pruning)
    Cleaning,
    /// Imputing missing values
    Imputation,
    /// Handling outliers
    OutlierHandling,
    /// Generating reports
    ReportGeneration,
    /// Pipeline completed successfully
    Complete,
    /// Pipeline was cancelled by user
    Cancelled,
    /// Pipeline failed with an error
    Failed,
}

impl PreprocessingStage {
    /// Returns a human-readable name for the stage.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Initializing => "Initializing",
            Self::Profiling => "Profiling Dataset",
            Self::QualityAnalysis => "Analyzing Quality",
            Self::TypeCorrection => "Correcting Types",
            Self::DecisionMaking => "Making Decisions",
            Self::Cleaning => "Cleaning Data",
            Self::Imputation => "Imputing Values",
            Self::OutlierHandling => "Handling Outliers",
            Self::ReportGeneration => "Generating Reports",
            Self::Complete => "Complete",
            Self::Cancelled => "Cancelled",
            Self::Failed => "Failed",
        }
    }

    /// Returns the typical weight of this stage in the overall pipeline (0.0 - 1.0).
    ///
    /// These weights are used to estimate overall progress. They sum to ~1.0
    /// for the main processing stages (excluding terminal states).
    pub fn weight(&self) -> f32 {
        match self {
            Self::Initializing => 0.02,
            Self::Profiling => 0.10,
            Self::QualityAnalysis => 0.08,
            Self::TypeCorrection => 0.10,
            Self::DecisionMaking => 0.10,
            Self::Cleaning => 0.10,
            Self::Imputation => 0.25,
            Self::OutlierHandling => 0.10,
            Self::ReportGeneration => 0.15,
            Self::Complete => 0.0,
            Self::Cancelled => 0.0,
            Self::Failed => 0.0,
        }
    }

    /// Returns the cumulative progress at the start of this stage.
    pub fn base_progress(&self) -> f32 {
        match self {
            Self::Initializing => 0.0,
            Self::Profiling => 0.02,
            Self::QualityAnalysis => 0.12,
            Self::TypeCorrection => 0.20,
            Self::DecisionMaking => 0.30,
            Self::Cleaning => 0.40,
            Self::Imputation => 0.50,
            Self::OutlierHandling => 0.75,
            Self::ReportGeneration => 0.85,
            Self::Complete => 1.0,
            Self::Cancelled => 0.0,
            Self::Failed => 0.0,
        }
    }
}

/// Detailed progress update with sub-stage information.
///
/// This struct provides comprehensive progress information including:
/// - Current pipeline stage
/// - Optional sub-stage for granular tracking (e.g., "Column: Age")
/// - Overall and stage-specific progress percentages
/// - Human-readable message
/// - Item counts for iterative operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    /// Current pipeline stage
    pub stage: PreprocessingStage,

    /// Optional sub-stage description (e.g., "Column: Age", "Row batch 1/10")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_stage: Option<String>,

    /// Overall progress (0.0 - 1.0)
    pub progress: f32,

    /// Progress within current stage (0.0 - 1.0)
    pub stage_progress: f32,

    /// Human-readable message describing current activity
    pub message: String,

    /// Number of items processed in current stage (for iterative operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_processed: Option<usize>,

    /// Total items in current stage (for iterative operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_total: Option<usize>,
}

impl ProgressUpdate {
    /// Creates a new progress update for a stage without sub-stage info.
    pub fn new(stage: PreprocessingStage, stage_progress: f32, message: impl Into<String>) -> Self {
        let progress = stage.base_progress() + (stage.weight() * stage_progress);
        Self {
            stage,
            sub_stage: None,
            progress: progress.clamp(0.0, 1.0),
            stage_progress: stage_progress.clamp(0.0, 1.0),
            message: message.into(),
            items_processed: None,
            items_total: None,
        }
    }

    /// Creates a new progress update with sub-stage information.
    pub fn with_sub_stage(
        stage: PreprocessingStage,
        sub_stage: impl Into<String>,
        stage_progress: f32,
        message: impl Into<String>,
    ) -> Self {
        let progress = stage.base_progress() + (stage.weight() * stage_progress);
        Self {
            stage,
            sub_stage: Some(sub_stage.into()),
            progress: progress.clamp(0.0, 1.0),
            stage_progress: stage_progress.clamp(0.0, 1.0),
            message: message.into(),
            items_processed: None,
            items_total: None,
        }
    }

    /// Creates a new progress update with item counts.
    pub fn with_items(
        stage: PreprocessingStage,
        sub_stage: impl Into<String>,
        current: usize,
        total: usize,
        message: impl Into<String>,
    ) -> Self {
        let stage_progress = if total > 0 {
            current as f32 / total as f32
        } else {
            0.0
        };
        let progress = stage.base_progress() + (stage.weight() * stage_progress);
        Self {
            stage,
            sub_stage: Some(sub_stage.into()),
            progress: progress.clamp(0.0, 1.0),
            stage_progress: stage_progress.clamp(0.0, 1.0),
            message: message.into(),
            items_processed: Some(current),
            items_total: Some(total),
        }
    }

    /// Creates a completion progress update.
    pub fn complete(message: impl Into<String>) -> Self {
        Self {
            stage: PreprocessingStage::Complete,
            sub_stage: None,
            progress: 1.0,
            stage_progress: 1.0,
            message: message.into(),
            items_processed: None,
            items_total: None,
        }
    }

    /// Creates a cancelled progress update.
    pub fn cancelled() -> Self {
        Self {
            stage: PreprocessingStage::Cancelled,
            sub_stage: None,
            progress: 0.0,
            stage_progress: 0.0,
            message: "Pipeline cancelled by user".to_string(),
            items_processed: None,
            items_total: None,
        }
    }

    /// Creates a failed progress update.
    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            stage: PreprocessingStage::Failed,
            sub_stage: None,
            progress: 0.0,
            stage_progress: 0.0,
            message: message.into(),
            items_processed: None,
            items_total: None,
        }
    }
}

/// Trait for receiving progress updates during preprocessing.
///
/// Implement this trait to receive progress updates from the pipeline.
/// Implementations must be `Send + Sync` to allow cross-thread usage,
/// which is essential for Tauri integration where the pipeline runs
/// on a background thread but emits events to the UI.
///
/// # Example
///
/// ```rust,ignore
/// use data_preprocessing_pipeline::{ProgressReporter, ProgressUpdate};
/// use tauri::AppHandle;
///
/// struct TauriProgressReporter {
///     app: AppHandle,
/// }
///
/// impl ProgressReporter for TauriProgressReporter {
///     fn report(&self, update: ProgressUpdate) {
///         self.app.emit("preprocessing:progress", &update).ok();
///     }
/// }
/// ```
pub trait ProgressReporter: Send + Sync {
    /// Called when progress is made during preprocessing.
    ///
    /// This method may be called frequently during processing (e.g., once per column
    /// during imputation). Implementations should be efficient and non-blocking.
    fn report(&self, update: ProgressUpdate);
}

/// Wrapper that implements [`ProgressReporter`] using a closure.
///
/// This provides a convenient way to handle progress updates without
/// implementing the trait manually.
///
/// # Example
///
/// ```rust,ignore
/// use data_preprocessing_pipeline::Pipeline;
///
/// Pipeline::builder()
///     .on_progress(|update| {
///         println!("[{:.0}%] {}", update.progress * 100.0, update.message);
///     })
///     .build()?
///     .process(df);
/// ```
pub struct ClosureProgressReporter<F>
where
    F: Fn(ProgressUpdate) + Send + Sync,
{
    callback: F,
}

impl<F> ClosureProgressReporter<F>
where
    F: Fn(ProgressUpdate) + Send + Sync,
{
    /// Creates a new closure-based progress reporter.
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

impl<F> ProgressReporter for ClosureProgressReporter<F>
where
    F: Fn(ProgressUpdate) + Send + Sync,
{
    fn report(&self, update: ProgressUpdate) {
        (self.callback)(update);
    }
}

/// Token for cancelling a running pipeline.
///
/// This token uses an atomic boolean internally, making it safe to clone
/// and share across threads. Call [`cancel()`](Self::cancel) from any thread
/// to request cancellation of the pipeline.
///
/// The pipeline checks this token at various points during execution and
/// will return [`PreprocessingError::Cancelled`](crate::error::PreprocessingError::Cancelled)
/// if cancellation is requested.
///
/// # Example
///
/// ```rust,ignore
/// use data_preprocessing_pipeline::{Pipeline, CancellationToken};
/// use std::thread;
/// use std::time::Duration;
///
/// let token = CancellationToken::new();
/// let token_for_cancel = token.clone();
///
/// // Spawn a thread that will cancel after 10 seconds
/// thread::spawn(move || {
///     thread::sleep(Duration::from_secs(10));
///     token_for_cancel.cancel();
///     println!("Cancellation requested!");
/// });
///
/// // Run the pipeline with the cancellation token
/// let result = Pipeline::builder()
///     .cancellation_token(token)
///     .build()?
///     .process(df);
///
/// match result {
///     Err(PreprocessingError::Cancelled) => println!("Pipeline was cancelled"),
///     Ok(result) => println!("Pipeline completed"),
///     Err(e) => println!("Pipeline failed: {}", e),
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

// Static assertions for thread safety - required for Tauri integration
// where pipeline runs on background thread but tokens are shared
static_assertions::assert_impl_all!(CancellationToken: Send, Sync);
static_assertions::assert_impl_all!(ProgressUpdate: Send, Sync);

impl CancellationToken {
    /// Creates a new cancellation token.
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Request cancellation of the pipeline.
    ///
    /// This method is thread-safe and can be called from any thread.
    /// The pipeline will check this token periodically and stop processing
    /// if cancellation has been requested.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Check if cancellation has been requested.
    ///
    /// Returns `true` if [`cancel()`](Self::cancel) has been called on this
    /// token or any of its clones.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Reset the token for reuse.
    ///
    /// This clears the cancellation flag, allowing the token to be reused
    /// for another pipeline run.
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_cancellation_token_default_not_cancelled() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_cancel() {
        let token = CancellationToken::new();
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_clone_shares_state() {
        let token1 = CancellationToken::new();
        let token2 = token1.clone();

        assert!(!token1.is_cancelled());
        assert!(!token2.is_cancelled());

        token1.cancel();

        assert!(token1.is_cancelled());
        assert!(token2.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_reset() {
        let token = CancellationToken::new();
        token.cancel();
        assert!(token.is_cancelled());

        token.reset();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_progress_update_new() {
        let update = ProgressUpdate::new(PreprocessingStage::Profiling, 0.5, "Profiling...");
        assert_eq!(update.stage, PreprocessingStage::Profiling);
        assert!(update.sub_stage.is_none());
        assert_eq!(update.stage_progress, 0.5);
        assert_eq!(update.message, "Profiling...");
    }

    #[test]
    fn test_progress_update_with_items() {
        let update = ProgressUpdate::with_items(
            PreprocessingStage::Imputation,
            "Column: Age",
            5,
            10,
            "Imputing column Age",
        );
        assert_eq!(update.stage, PreprocessingStage::Imputation);
        assert_eq!(update.sub_stage, Some("Column: Age".to_string()));
        assert_eq!(update.stage_progress, 0.5);
        assert_eq!(update.items_processed, Some(5));
        assert_eq!(update.items_total, Some(10));
    }

    #[test]
    fn test_progress_update_complete() {
        let update = ProgressUpdate::complete("Done!");
        assert_eq!(update.stage, PreprocessingStage::Complete);
        assert_eq!(update.progress, 1.0);
        assert_eq!(update.stage_progress, 1.0);
    }

    #[test]
    fn test_closure_progress_reporter() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let reporter = ClosureProgressReporter::new(move |_update| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        reporter.report(ProgressUpdate::new(
            PreprocessingStage::Profiling,
            0.5,
            "Test",
        ));
        reporter.report(ProgressUpdate::complete("Done"));

        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_preprocessing_stage_display_name() {
        assert_eq!(PreprocessingStage::Profiling.display_name(), "Profiling Dataset");
        assert_eq!(PreprocessingStage::Imputation.display_name(), "Imputing Values");
        assert_eq!(PreprocessingStage::Complete.display_name(), "Complete");
    }

    #[test]
    fn test_preprocessing_stage_weights_sum() {
        let stages = [
            PreprocessingStage::Initializing,
            PreprocessingStage::Profiling,
            PreprocessingStage::QualityAnalysis,
            PreprocessingStage::TypeCorrection,
            PreprocessingStage::DecisionMaking,
            PreprocessingStage::Cleaning,
            PreprocessingStage::Imputation,
            PreprocessingStage::OutlierHandling,
            PreprocessingStage::ReportGeneration,
        ];

        let total_weight: f32 = stages.iter().map(|s| s.weight()).sum();
        assert!((total_weight - 1.0).abs() < 0.01, "Weights should sum to ~1.0");
    }

    #[test]
    fn test_progress_update_json_serialization() {
        let update = ProgressUpdate::with_items(
            PreprocessingStage::Imputation,
            "Column: Age",
            5,
            10,
            "Imputing missing values in Age column",
        );

        let json = serde_json::to_string(&update).expect("Should serialize");

        // Verify key fields are present in JSON
        assert!(json.contains("\"stage\":\"imputation\""), "Stage should be snake_case");
        assert!(json.contains("\"sub_stage\":\"Column: Age\""));
        assert!(json.contains("\"items_processed\":5"));
        assert!(json.contains("\"items_total\":10"));
        assert!(json.contains("\"message\":\"Imputing missing values in Age column\""));

        // Verify round-trip works
        let deserialized: ProgressUpdate = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.stage, PreprocessingStage::Imputation);
        assert_eq!(deserialized.sub_stage, Some("Column: Age".to_string()));
        assert_eq!(deserialized.items_processed, Some(5));
    }

    #[test]
    fn test_preprocessing_stage_json_values() {
        let stage_expectations = [
            (PreprocessingStage::Initializing, "\"initializing\""),
            (PreprocessingStage::Profiling, "\"profiling\""),
            (PreprocessingStage::QualityAnalysis, "\"quality_analysis\""),
            (PreprocessingStage::TypeCorrection, "\"type_correction\""),
            (PreprocessingStage::DecisionMaking, "\"decision_making\""),
            (PreprocessingStage::Cleaning, "\"cleaning\""),
            (PreprocessingStage::Imputation, "\"imputation\""),
            (PreprocessingStage::OutlierHandling, "\"outlier_handling\""),
            (PreprocessingStage::ReportGeneration, "\"report_generation\""),
            (PreprocessingStage::Complete, "\"complete\""),
            (PreprocessingStage::Cancelled, "\"cancelled\""),
            (PreprocessingStage::Failed, "\"failed\""),
        ];

        for (stage, expected_json) in stage_expectations {
            let json = serde_json::to_string(&stage).expect("Should serialize");
            assert_eq!(json, expected_json, "PreprocessingStage::{:?} should serialize to {}", stage, expected_json);
        }
    }

    #[test]
    fn test_cancellation_across_threads() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        let handle = std::thread::spawn(move || {
            // Simulate pipeline checking cancellation in background thread
            std::thread::sleep(std::time::Duration::from_millis(50));
            token_clone.is_cancelled()
        });

        // Cancel from main thread before the background thread checks
        token.cancel();

        let was_cancelled = handle.join().expect("Thread should not panic");
        assert!(was_cancelled, "Cancellation should be visible across threads");
    }

    #[test]
    fn test_progress_reporter_across_threads() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let reporter = Arc::new(ClosureProgressReporter::new(move |_update| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        }));

        let reporter_clone = reporter.clone();
        let handle = std::thread::spawn(move || {
            reporter_clone.report(ProgressUpdate::new(
                PreprocessingStage::Profiling,
                0.5,
                "Test from background thread",
            ));
        });

        handle.join().expect("Thread should not panic");
        assert_eq!(
            call_count.load(Ordering::SeqCst),
            1,
            "Progress reporter should work across threads"
        );
    }
}
