//! Configuration types for the ML training pipeline.
//!
//! This module provides [`PipelineConfig`] and its builder for configuring
//! the training pipeline, as well as the [`ProblemType`] enum.
//!
//! # Example
//!
//! ```
//! use lex_learning::{PipelineConfig, ProblemType};
//!
//! let config = PipelineConfig::builder()
//!     .problem_type(ProblemType::Classification)
//!     .target_column("Survived")
//!     .cv_folds(5)
//!     .test_size(0.2)
//!     .build()
//!     .expect("valid config");
//! ```

use crate::error::LexLearningError;

/// The type of machine learning problem to solve.
///
/// This determines which models and metrics are used during training:
/// - [`Classification`](Self::Classification): Uses accuracy, F1, precision, recall
/// - [`Regression`](Self::Regression): Uses R², RMSE, MAE
///
/// This enum is marked `#[non_exhaustive]` to allow adding new problem types
/// (e.g., multi-label classification) in future versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum ProblemType {
    /// Classification problem (predicting discrete classes).
    ///
    /// Use this for problems where the target is a categorical variable,
    /// such as spam detection, image classification, or customer churn prediction.
    #[default]
    Classification,

    /// Regression problem (predicting continuous values).
    ///
    /// Use this for problems where the target is a numerical variable,
    /// such as price prediction, temperature forecasting, or age estimation.
    Regression,
}

impl ProblemType {
    /// Returns the string representation used by the Python library.
    ///
    /// # Examples
    ///
    /// ```
    /// use lex_learning::ProblemType;
    ///
    /// assert_eq!(ProblemType::Classification.as_str(), "classification");
    /// assert_eq!(ProblemType::Regression.as_str(), "regression");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            ProblemType::Classification => "classification",
            ProblemType::Regression => "regression",
        }
    }
}

/// Configuration for the ML training pipeline.
///
/// Use [`PipelineConfig::builder()`] to construct a configuration with the builder pattern.
/// All fields have sensible defaults except `problem_type` which defaults to
/// [`Classification`](ProblemType::Classification).
///
/// # Validation
///
/// The builder validates the following constraints on [`build()`](PipelineConfigBuilder::build):
/// - `test_size` must be in range `(0.0, 1.0)` (exclusive)
/// - `cv_folds` must be at least 2
/// - `top_k_algorithms` must be at least 1
/// - `n_trials` must be at least 1
/// - `shap_max_samples` must be at least 1
///
/// # Example
///
/// ```
/// use lex_learning::{PipelineConfig, ProblemType};
///
/// let config = PipelineConfig::builder()
///     .problem_type(ProblemType::Regression)
///     .target_column("price")
///     .optimize_hyperparams(true)
///     .n_trials(100)
///     .build()
///     .expect("valid config");
/// ```
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// The type of problem (classification or regression).
    pub problem_type: ProblemType,

    /// Name of the target column in the DataFrame.
    ///
    /// If `None`, the last column is used as the target.
    pub target_column: Option<String>,

    /// Specific algorithm to use (if `None`, auto-selection is used).
    ///
    /// When specified, only this algorithm is trained (skips algorithm selection).
    /// Must be a valid algorithm name from the model registry.
    pub algorithm: Option<String>,

    /// Number of top algorithms to evaluate during selection (default: 3).
    ///
    /// During algorithm selection, this many top-performing algorithms are
    /// selected for full training with hyperparameter optimization.
    /// Must be at least 1.
    pub top_k_algorithms: u32,

    /// Whether to optimize hyperparameters with Optuna (default: true).
    ///
    /// When enabled, Optuna runs `n_trials` optimization trials for each model.
    /// Disable for faster training with default hyperparameters.
    pub optimize_hyperparams: bool,

    /// Number of Optuna trials for hyperparameter optimization (default: 30).
    ///
    /// More trials generally produce better hyperparameters but take longer.
    /// Must be at least 1.
    pub n_trials: u32,

    /// Number of cross-validation folds (default: 5).
    ///
    /// Used during training and hyperparameter optimization.
    /// Must be at least 2.
    pub cv_folds: u32,

    /// Fraction of data to use for testing (default: 0.2).
    ///
    /// Must be between 0.0 and 1.0 (exclusive). Common values are 0.2 (20%) or 0.3 (30%).
    pub test_size: f64,

    /// Whether to include neural network models in selection (default: false).
    ///
    /// Neural networks (Keras) can be slower to train and require more data.
    /// Enable only if you have sufficient data and compute resources.
    pub enable_neural_networks: bool,

    /// Whether to generate SHAP explainability plots (default: true).
    ///
    /// SHAP analysis adds overhead but provides valuable feature importance insights.
    pub enable_explainability: bool,

    /// Maximum samples for SHAP computation (default: 100).
    ///
    /// Limits the number of samples used for SHAP analysis to control memory
    /// usage and computation time. Must be at least 1.
    pub shap_max_samples: u32,

    /// Random seed for reproducibility (default: 42).
    ///
    /// Set to the same value for reproducible training results.
    pub random_seed: u64,

    /// Number of parallel jobs (default: -1 for all cores).
    ///
    /// - `-1`: Use all available CPU cores
    /// - `1`: Single-threaded (useful for debugging)
    /// - `n > 1`: Use exactly `n` cores
    pub n_jobs: i32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            problem_type: ProblemType::default(),
            target_column: None,
            algorithm: None,
            top_k_algorithms: 3,
            optimize_hyperparams: true,
            n_trials: 30, // Match Python default
            cv_folds: 5,
            test_size: 0.2,
            enable_neural_networks: false, // Keep false for faster default training
            enable_explainability: true,
            shap_max_samples: 100,
            random_seed: 42,
            n_jobs: -1,
        }
    }
}

impl PipelineConfig {
    /// Create a new builder for `PipelineConfig`.
    ///
    /// # Example
    ///
    /// ```
    /// use lex_learning::{PipelineConfig, ProblemType};
    ///
    /// let config = PipelineConfig::builder()
    ///     .problem_type(ProblemType::Classification)
    ///     .target_column("target")
    ///     .build()
    ///     .expect("valid config");
    /// ```
    #[must_use]
    pub fn builder() -> PipelineConfigBuilder {
        PipelineConfigBuilder::default()
    }
}

/// Builder for [`PipelineConfig`].
///
/// Created via [`PipelineConfig::builder()`]. All setters return `self` to allow
/// method chaining.
///
/// # Example
///
/// ```
/// use lex_learning::{PipelineConfig, ProblemType};
///
/// let config = PipelineConfig::builder()
///     .problem_type(ProblemType::Regression)
///     .target_column("price")
///     .cv_folds(10)
///     .test_size(0.3)
///     .build()
///     .expect("valid config");
/// ```
#[derive(Debug, Clone, Default)]
pub struct PipelineConfigBuilder {
    config: PipelineConfig,
}

impl PipelineConfigBuilder {
    /// Set the problem type (classification or regression).
    #[must_use]
    pub fn problem_type(mut self, problem_type: ProblemType) -> Self {
        self.config.problem_type = problem_type;
        self
    }

    /// Set the target column name.
    ///
    /// If not set, the last column in the DataFrame is used as the target.
    #[must_use]
    pub fn target_column(mut self, column: impl Into<String>) -> Self {
        self.config.target_column = Some(column.into());
        self
    }

    /// Set a specific algorithm to use (skip auto-selection).
    ///
    /// When set, only this algorithm is trained. Must be a valid algorithm name
    /// from the model registry (e.g., "random_forest", "xgboost", "lightgbm").
    #[must_use]
    pub fn algorithm(mut self, algorithm: impl Into<String>) -> Self {
        self.config.algorithm = Some(algorithm.into());
        self
    }

    /// Set the number of top algorithms to evaluate (default: 3).
    ///
    /// # Panics
    ///
    /// Does not panic, but [`build()`](Self::build) will return an error if `k < 1`.
    #[must_use]
    pub fn top_k_algorithms(mut self, k: u32) -> Self {
        self.config.top_k_algorithms = k;
        self
    }

    /// Enable or disable hyperparameter optimization (default: true).
    #[must_use]
    pub fn optimize_hyperparams(mut self, optimize: bool) -> Self {
        self.config.optimize_hyperparams = optimize;
        self
    }

    /// Set the number of Optuna trials (default: 30).
    ///
    /// # Panics
    ///
    /// Does not panic, but [`build()`](Self::build) will return an error if `n < 1`.
    #[must_use]
    pub fn n_trials(mut self, n: u32) -> Self {
        self.config.n_trials = n;
        self
    }

    /// Set the number of cross-validation folds (default: 5).
    ///
    /// # Panics
    ///
    /// Does not panic, but [`build()`](Self::build) will return an error if `folds < 2`.
    #[must_use]
    pub fn cv_folds(mut self, folds: u32) -> Self {
        self.config.cv_folds = folds;
        self
    }

    /// Set the test size fraction (default: 0.2).
    ///
    /// # Panics
    ///
    /// Does not panic, but [`build()`](Self::build) will return an error if
    /// `size <= 0.0` or `size >= 1.0`.
    #[must_use]
    pub fn test_size(mut self, size: f64) -> Self {
        self.config.test_size = size;
        self
    }

    /// Enable or disable neural network models (default: false).
    ///
    /// Neural networks require TensorFlow/Keras and may be slower to train.
    #[must_use]
    pub fn enable_neural_networks(mut self, enable: bool) -> Self {
        self.config.enable_neural_networks = enable;
        self
    }

    /// Enable or disable SHAP explainability (default: true).
    #[must_use]
    pub fn enable_explainability(mut self, enable: bool) -> Self {
        self.config.enable_explainability = enable;
        self
    }

    /// Set the maximum samples for SHAP computation (default: 100).
    ///
    /// # Panics
    ///
    /// Does not panic, but [`build()`](Self::build) will return an error if `samples < 1`.
    #[must_use]
    pub fn shap_max_samples(mut self, samples: u32) -> Self {
        self.config.shap_max_samples = samples;
        self
    }

    /// Set the random seed for reproducibility (default: 42).
    #[must_use]
    pub fn random_seed(mut self, seed: u64) -> Self {
        self.config.random_seed = seed;
        self
    }

    /// Set the number of parallel jobs (default: -1 for all cores).
    ///
    /// - `-1`: Use all available CPU cores
    /// - `1`: Single-threaded
    /// - `n > 1`: Use exactly `n` cores
    #[must_use]
    pub fn n_jobs(mut self, jobs: i32) -> Self {
        self.config.n_jobs = jobs;
        self
    }

    /// Build the configuration, validating all settings.
    ///
    /// # Errors
    ///
    /// Returns [`LexLearningError::InvalidConfig`] if:
    /// - `test_size` is not in range `(0.0, 1.0)`
    /// - `cv_folds` is less than 2
    /// - `top_k_algorithms` is less than 1
    /// - `n_trials` is less than 1
    /// - `shap_max_samples` is less than 1
    pub fn build(self) -> Result<PipelineConfig, LexLearningError> {
        // Validate test_size
        if self.config.test_size <= 0.0 || self.config.test_size >= 1.0 {
            return Err(LexLearningError::InvalidConfig(
                "test_size must be between 0.0 and 1.0 (exclusive)".to_string(),
            ));
        }

        // Validate cv_folds
        if self.config.cv_folds < 2 {
            return Err(LexLearningError::InvalidConfig(
                "cv_folds must be at least 2".to_string(),
            ));
        }

        // Validate top_k_algorithms
        if self.config.top_k_algorithms == 0 {
            return Err(LexLearningError::InvalidConfig(
                "top_k_algorithms must be at least 1".to_string(),
            ));
        }

        // Validate n_trials
        if self.config.n_trials == 0 {
            return Err(LexLearningError::InvalidConfig(
                "n_trials must be at least 1".to_string(),
            ));
        }

        // Validate shap_max_samples
        if self.config.shap_max_samples == 0 {
            return Err(LexLearningError::InvalidConfig(
                "shap_max_samples must be at least 1".to_string(),
            ));
        }

        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PipelineConfig::default();
        assert_eq!(config.problem_type, ProblemType::Classification);
        assert_eq!(config.cv_folds, 5);
        assert_eq!(config.test_size, 0.2);
        assert_eq!(config.n_trials, 30); // Updated default
        assert!(!config.enable_neural_networks);
        assert!(config.enable_explainability);
    }

    #[test]
    fn test_builder() {
        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Regression)
            .target_column("price")
            .n_trials(100)
            .build()
            .unwrap();

        assert_eq!(config.problem_type, ProblemType::Regression);
        assert_eq!(config.target_column, Some("price".to_string()));
        assert_eq!(config.n_trials, 100);
    }

    #[test]
    fn test_invalid_test_size() {
        let result = PipelineConfig::builder().test_size(0.0).build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("test_size"));

        let result = PipelineConfig::builder().test_size(1.0).build();
        assert!(result.is_err());

        let result = PipelineConfig::builder().test_size(-0.1).build();
        assert!(result.is_err());

        let result = PipelineConfig::builder().test_size(1.5).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_cv_folds() {
        let result = PipelineConfig::builder().cv_folds(1).build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cv_folds"));

        let result = PipelineConfig::builder().cv_folds(0).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_top_k_algorithms() {
        let result = PipelineConfig::builder().top_k_algorithms(0).build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("top_k_algorithms"));
    }

    #[test]
    fn test_invalid_n_trials() {
        let result = PipelineConfig::builder().n_trials(0).build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("n_trials"));
    }

    #[test]
    fn test_invalid_shap_max_samples() {
        let result = PipelineConfig::builder().shap_max_samples(0).build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("shap_max_samples"));
    }

    #[test]
    fn test_problem_type_as_str() {
        assert_eq!(ProblemType::Classification.as_str(), "classification");
        assert_eq!(ProblemType::Regression.as_str(), "regression");
    }

    #[test]
    fn test_problem_type_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ProblemType::Classification);
        set.insert(ProblemType::Regression);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_builder_chaining() {
        // Test that all builder methods can be chained
        let config = PipelineConfig::builder()
            .problem_type(ProblemType::Classification)
            .target_column("target")
            .algorithm("random_forest")
            .top_k_algorithms(5)
            .optimize_hyperparams(false)
            .n_trials(20)
            .cv_folds(10)
            .test_size(0.3)
            .enable_neural_networks(true)
            .enable_explainability(false)
            .shap_max_samples(50)
            .random_seed(123)
            .n_jobs(4)
            .build()
            .unwrap();

        assert_eq!(config.algorithm, Some("random_forest".to_string()));
        assert_eq!(config.top_k_algorithms, 5);
        assert!(!config.optimize_hyperparams);
        assert_eq!(config.cv_folds, 10);
        assert!((config.test_size - 0.3).abs() < f64::EPSILON);
        assert!(config.enable_neural_networks);
        assert!(!config.enable_explainability);
        assert_eq!(config.shap_max_samples, 50);
        assert_eq!(config.random_seed, 123);
        assert_eq!(config.n_jobs, 4);
    }
}
