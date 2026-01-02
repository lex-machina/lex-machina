//! Progress reporting types for the training pipeline.
//!
//! This module defines types for tracking and reporting progress during
//! model training, including [`TrainingStage`], [`ProgressUpdate`], and
//! the [`ProgressCallback`] type alias.
//!
//! # Overview
//!
//! Progress reporting allows you to monitor training in real-time:
//! - Track which stage of the pipeline is currently executing
//! - Get overall progress percentage (0.0 to 1.0)
//! - See which model is currently being trained
//! - Track how many models have been completed
//!
//! # Example
//!
//! ```
//! use lex_learning::{Pipeline, PipelineConfig, ProgressUpdate, TrainingStage};
//!
//! let pipeline = Pipeline::builder()
//!     .config(PipelineConfig::default())
//!     .on_progress(|update: ProgressUpdate| {
//!         println!(
//!             "[{:?}] {:.0}% - {}",
//!             update.stage,
//!             update.progress * 100.0,
//!             update.message
//!         );
//!         if let Some((done, total)) = update.models_completed {
//!             println!("  Models: {}/{}", done, total);
//!         }
//!     })
//!     .build();
//! ```

use std::str::FromStr;
use std::sync::Arc;

/// The current stage of the training pipeline.
///
/// Training progresses through these stages in order (unless cancelled or failed):
///
/// 1. [`Initializing`](Self::Initializing) - Setting up the pipeline
/// 2. [`Preprocessing`](Self::Preprocessing) - Encoding and scaling features
/// 3. [`AlgorithmSelection`](Self::AlgorithmSelection) - Evaluating candidate algorithms
/// 4. [`Training`](Self::Training) - Training the selected model(s)
/// 5. [`Evaluation`](Self::Evaluation) - Evaluating model performance
/// 6. [`Explainability`](Self::Explainability) - Computing SHAP values
/// 7. [`Complete`](Self::Complete) - Training finished successfully
///
/// Terminal states: [`Complete`](Self::Complete), [`Failed`](Self::Failed),
/// [`Cancelled`](Self::Cancelled).
///
/// This enum is marked `#[non_exhaustive]` to allow adding new stages in future versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum TrainingStage {
    /// Pipeline is initializing.
    ///
    /// Setting up the training environment and validating configuration.
    #[default]
    Initializing,

    /// Data preprocessing is in progress.
    ///
    /// Encoding categorical features, scaling numerical features,
    /// and preparing the data for training.
    Preprocessing,

    /// Algorithm selection is in progress.
    ///
    /// Evaluating candidate algorithms with quick training runs
    /// to determine which perform best on this dataset.
    AlgorithmSelection,

    /// Model training is in progress.
    ///
    /// Training the selected algorithm(s) with full hyperparameter
    /// optimization via Optuna.
    Training,

    /// Model evaluation is in progress.
    ///
    /// Computing final metrics on the test set and comparing models.
    Evaluation,

    /// SHAP explainability computation is in progress.
    ///
    /// Computing SHAP values and generating explainability plots.
    /// This stage is skipped if `enable_explainability` is `false`.
    Explainability,

    /// Training completed successfully.
    ///
    /// This is a terminal state. The training result is available.
    Complete,

    /// Training failed.
    ///
    /// This is a terminal state. Check the error message for details.
    Failed,

    /// Training was cancelled.
    ///
    /// This is a terminal state. Training was stopped before completion.
    Cancelled,
}

impl TrainingStage {
    /// Returns the string representation used by the Python library.
    ///
    /// # Examples
    ///
    /// ```
    /// use lex_learning::TrainingStage;
    ///
    /// assert_eq!(TrainingStage::Training.as_str(), "training");
    /// assert_eq!(TrainingStage::AlgorithmSelection.as_str(), "algorithm_selection");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            TrainingStage::Initializing => "initializing",
            TrainingStage::Preprocessing => "preprocessing",
            TrainingStage::AlgorithmSelection => "algorithm_selection",
            TrainingStage::Training => "training",
            TrainingStage::Evaluation => "evaluation",
            TrainingStage::Explainability => "explainability",
            TrainingStage::Complete => "complete",
            TrainingStage::Failed => "failed",
            TrainingStage::Cancelled => "cancelled",
        }
    }

    /// Returns `true` if this is a terminal state.
    ///
    /// Terminal states are [`Complete`](Self::Complete), [`Failed`](Self::Failed),
    /// and [`Cancelled`](Self::Cancelled).
    ///
    /// # Examples
    ///
    /// ```
    /// use lex_learning::TrainingStage;
    ///
    /// assert!(TrainingStage::Complete.is_terminal());
    /// assert!(TrainingStage::Failed.is_terminal());
    /// assert!(!TrainingStage::Training.is_terminal());
    /// ```
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TrainingStage::Complete | TrainingStage::Failed | TrainingStage::Cancelled
        )
    }
}

/// Error type for parsing a [`TrainingStage`] from a string.
///
/// Returned by [`TrainingStage::from_str()`] when the input string doesn't
/// match any known stage.
///
/// # Example
///
/// ```
/// use lex_learning::TrainingStage;
///
/// let result: Result<TrainingStage, _> = "invalid".parse();
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseTrainingStageError {
    /// The invalid input string that couldn't be parsed.
    invalid_value: String,
}

impl ParseTrainingStageError {
    /// Returns the invalid value that caused the parse error.
    #[must_use]
    pub fn invalid_value(&self) -> &str {
        &self.invalid_value
    }
}

impl std::fmt::Display for ParseTrainingStageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid training stage: '{}'. Valid values are: initializing, preprocessing, \
             algorithm_selection, training, evaluation, explainability, complete, failed, cancelled",
            self.invalid_value
        )
    }
}

impl std::error::Error for ParseTrainingStageError {}

impl FromStr for TrainingStage {
    type Err = ParseTrainingStageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "initializing" => Ok(TrainingStage::Initializing),
            "preprocessing" => Ok(TrainingStage::Preprocessing),
            "algorithm_selection" => Ok(TrainingStage::AlgorithmSelection),
            "training" => Ok(TrainingStage::Training),
            "evaluation" => Ok(TrainingStage::Evaluation),
            "explainability" => Ok(TrainingStage::Explainability),
            "complete" => Ok(TrainingStage::Complete),
            "failed" => Ok(TrainingStage::Failed),
            "cancelled" => Ok(TrainingStage::Cancelled),
            _ => Err(ParseTrainingStageError {
                invalid_value: s.to_string(),
            }),
        }
    }
}

/// A progress update from the training pipeline.
///
/// Sent to the progress callback during training to report current status.
///
/// # Fields
///
/// - `stage`: The current pipeline stage
/// - `progress`: Overall progress from 0.0 (just started) to 1.0 (complete)
/// - `message`: Human-readable status message
/// - `current_model`: Name of the model being trained (if in training stage)
/// - `models_completed`: Tuple of (completed, total) models
///
/// # Example
///
/// ```
/// use lex_learning::{ProgressUpdate, TrainingStage};
///
/// let update = ProgressUpdate {
///     stage: TrainingStage::Training,
///     progress: 0.5,
///     message: "Training random_forest".to_string(),
///     current_model: Some("random_forest".to_string()),
///     models_completed: Some((1, 3)),
/// };
///
/// println!("{:.0}% complete", update.progress * 100.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressUpdate {
    /// The current training stage.
    pub stage: TrainingStage,

    /// Overall progress from 0.0 to 1.0.
    ///
    /// - `0.0`: Training just started
    /// - `1.0`: Training complete
    ///
    /// Progress increases monotonically during normal training.
    pub progress: f64,

    /// Human-readable status message.
    ///
    /// Describes what the pipeline is currently doing.
    /// Examples: "Encoding categorical features", "Training xgboost (trial 5/50)".
    pub message: String,

    /// Name of the model currently being trained (if applicable).
    ///
    /// Only populated during the [`Training`](TrainingStage::Training) stage.
    /// Examples: "random_forest", "xgboost", "lightgbm".
    pub current_model: Option<String>,

    /// Number of models completed and total: `(completed, total)`.
    ///
    /// Only populated during stages that train multiple models.
    /// For example, `(2, 5)` means 2 of 5 models have been trained.
    pub models_completed: Option<(u32, u32)>,
}

impl Default for ProgressUpdate {
    fn default() -> Self {
        Self {
            stage: TrainingStage::default(),
            progress: 0.0,
            message: String::new(),
            current_model: None,
            models_completed: None,
        }
    }
}

/// Type alias for a progress callback function.
///
/// The callback receives [`ProgressUpdate`] structs as training progresses.
/// Callbacks must be thread-safe (`Send + Sync`) as they may be called from
/// different threads.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use lex_learning::{ProgressCallback, ProgressUpdate};
///
/// let callback: ProgressCallback = Arc::new(|update: ProgressUpdate| {
///     println!("[{:?}] {} - {}", update.stage, update.progress, update.message);
/// });
/// ```
///
/// # Note
///
/// The callback should execute quickly to avoid blocking training.
/// For expensive operations (like updating a UI), consider sending
/// updates to a channel instead.
pub type ProgressCallback = Arc<dyn Fn(ProgressUpdate) + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_stage_as_str() {
        assert_eq!(TrainingStage::Initializing.as_str(), "initializing");
        assert_eq!(TrainingStage::Preprocessing.as_str(), "preprocessing");
        assert_eq!(TrainingStage::AlgorithmSelection.as_str(), "algorithm_selection");
        assert_eq!(TrainingStage::Training.as_str(), "training");
        assert_eq!(TrainingStage::Evaluation.as_str(), "evaluation");
        assert_eq!(TrainingStage::Explainability.as_str(), "explainability");
        assert_eq!(TrainingStage::Complete.as_str(), "complete");
        assert_eq!(TrainingStage::Failed.as_str(), "failed");
        assert_eq!(TrainingStage::Cancelled.as_str(), "cancelled");
    }

    #[test]
    fn test_training_stage_from_str() {
        assert_eq!(
            "training".parse::<TrainingStage>(),
            Ok(TrainingStage::Training)
        );
        assert_eq!(
            "algorithm_selection".parse::<TrainingStage>(),
            Ok(TrainingStage::AlgorithmSelection)
        );

        let err = "unknown".parse::<TrainingStage>().unwrap_err();
        assert_eq!(err.invalid_value(), "unknown");
        assert!(err.to_string().contains("unknown"));
        assert!(err.to_string().contains("Valid values"));
    }

    #[test]
    fn test_training_stage_roundtrip() {
        // Test that as_str() and from_str() are inverses
        let stages = [
            TrainingStage::Initializing,
            TrainingStage::Preprocessing,
            TrainingStage::AlgorithmSelection,
            TrainingStage::Training,
            TrainingStage::Evaluation,
            TrainingStage::Explainability,
            TrainingStage::Complete,
            TrainingStage::Failed,
            TrainingStage::Cancelled,
        ];

        for stage in stages {
            let s = stage.as_str();
            let parsed: TrainingStage = s.parse().unwrap();
            assert_eq!(parsed, stage);
        }
    }

    #[test]
    fn test_training_stage_is_terminal() {
        assert!(!TrainingStage::Initializing.is_terminal());
        assert!(!TrainingStage::Preprocessing.is_terminal());
        assert!(!TrainingStage::AlgorithmSelection.is_terminal());
        assert!(!TrainingStage::Training.is_terminal());
        assert!(!TrainingStage::Evaluation.is_terminal());
        assert!(!TrainingStage::Explainability.is_terminal());
        assert!(TrainingStage::Complete.is_terminal());
        assert!(TrainingStage::Failed.is_terminal());
        assert!(TrainingStage::Cancelled.is_terminal());
    }

    #[test]
    fn test_training_stage_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(TrainingStage::Training);
        set.insert(TrainingStage::Complete);
        set.insert(TrainingStage::Training); // Duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_progress_update_default() {
        let update = ProgressUpdate::default();
        assert_eq!(update.stage, TrainingStage::Initializing);
        assert_eq!(update.progress, 0.0);
        assert!(update.message.is_empty());
        assert!(update.current_model.is_none());
        assert!(update.models_completed.is_none());
    }

    #[test]
    fn test_progress_update_equality() {
        let update1 = ProgressUpdate {
            stage: TrainingStage::Training,
            progress: 0.5,
            message: "Training".to_string(),
            current_model: Some("xgboost".to_string()),
            models_completed: Some((1, 3)),
        };

        let update2 = ProgressUpdate {
            stage: TrainingStage::Training,
            progress: 0.5,
            message: "Training".to_string(),
            current_model: Some("xgboost".to_string()),
            models_completed: Some((1, 3)),
        };

        assert_eq!(update1, update2);
    }
}
