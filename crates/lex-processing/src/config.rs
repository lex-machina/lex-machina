//! Configuration types for the data preprocessing pipeline.
//!
//! This module provides configuration options using the builder pattern
//! for flexible and ergonomic pipeline setup.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Strategy for handling outliers in numeric columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutlierStrategy {
    /// Cap outliers at IQR bounds (Q1 - 1.5*IQR, Q3 + 1.5*IQR)
    #[default]
    Cap,
    /// Remove rows containing outliers
    Remove,
    /// Replace outliers with the median value
    Median,
    /// Keep outliers as-is (no handling)
    Keep,
}

/// Strategy for imputing missing numeric values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NumericImputation {
    /// Use the mean of non-null values
    Mean,
    /// Use the median of non-null values
    #[default]
    Median,
    /// Use K-Nearest Neighbors imputation
    Knn,
    /// Use a constant value (0.0)
    Zero,
    /// Drop rows with missing values
    Drop,
}

/// Strategy for imputing missing categorical values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CategoricalImputation {
    /// Use the most frequent value (mode)
    #[default]
    Mode,
    /// Use a constant value ("Unknown")
    Constant,
    /// Drop rows with missing values
    Drop,
}

/// Configuration for the preprocessing pipeline.
///
/// Use [`PipelineConfig::builder()`] to create a new configuration
/// with fluent API.
///
/// # Example
///
/// ```rust,ignore
/// use lex_processing::config::{PipelineConfig, OutlierStrategy};
///
/// let config = PipelineConfig::builder()
///     .missing_column_threshold(0.5)
///     .outlier_strategy(OutlierStrategy::Cap)
///     .enable_type_correction(true)
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Threshold for dropping columns with too many missing values (0.0 - 1.0).
    /// Columns with missing percentage above this threshold will be dropped.
    /// Default: 0.7 (70%)
    pub missing_column_threshold: f64,

    /// Threshold for dropping rows with too many missing values (0.0 - 1.0).
    /// Rows with missing percentage above this threshold will be dropped.
    /// Default: 0.8 (80%)
    pub missing_row_threshold: f64,

    /// Strategy for handling outliers in numeric columns.
    /// Default: Cap
    pub outlier_strategy: OutlierStrategy,

    /// Default strategy for imputing missing numeric values.
    /// Default: Median
    pub default_numeric_imputation: NumericImputation,

    /// Default strategy for imputing missing categorical values.
    /// Default: Mode
    pub default_categorical_imputation: CategoricalImputation,

    /// Whether to automatically correct column types.
    /// Default: true
    pub enable_type_correction: bool,

    /// Whether to remove duplicate rows.
    /// Default: true
    pub remove_duplicates: bool,

    /// Number of neighbors for KNN imputation.
    /// Default: 5
    pub knn_neighbors: usize,

    /// Output directory for generated reports and cleaned data.
    /// Default: "output"
    pub output_dir: PathBuf,

    /// Custom output file name (without extension).
    /// If None, uses "processed_dataset_{problem_type}".
    /// Default: None
    pub output_name: Option<String>,

    /// Whether to generate detailed reports.
    /// Default: true
    pub generate_reports: bool,

    /// Whether to use AI for decision making (requires AI client).
    /// If false or no AI client provided, rule-based decisions will be used.
    /// Default: true
    pub use_ai_decisions: bool,

    /// Explicitly specified target column.
    /// If None, the pipeline will auto-detect the target column.
    /// Default: None
    pub target_column: Option<String>,

    /// Whether to save processed data and reports to disk.
    /// When false, results are kept in memory only (useful for GUI apps).
    /// Default: true
    pub save_to_disk: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            missing_column_threshold: 0.7,
            missing_row_threshold: 0.8,
            outlier_strategy: OutlierStrategy::default(),
            default_numeric_imputation: NumericImputation::default(),
            default_categorical_imputation: CategoricalImputation::default(),
            enable_type_correction: true,
            remove_duplicates: true,
            knn_neighbors: 5,
            output_dir: PathBuf::from("output"),
            output_name: None,
            generate_reports: true,
            use_ai_decisions: true,
            target_column: None,
            save_to_disk: true,
        }
    }
}

impl PipelineConfig {
    /// Create a new configuration builder.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = PipelineConfig::builder()
    ///     .missing_column_threshold(0.5)
    ///     .build();
    /// ```
    pub fn builder() -> PipelineConfigBuilder {
        PipelineConfigBuilder::default()
    }

    /// Validate the configuration and return errors if invalid.
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if !(0.0..=1.0).contains(&self.missing_column_threshold) {
            return Err(ConfigValidationError::InvalidThreshold {
                field: "missing_column_threshold".to_string(),
                value: self.missing_column_threshold,
            });
        }

        if !(0.0..=1.0).contains(&self.missing_row_threshold) {
            return Err(ConfigValidationError::InvalidThreshold {
                field: "missing_row_threshold".to_string(),
                value: self.missing_row_threshold,
            });
        }

        if self.knn_neighbors == 0 {
            return Err(ConfigValidationError::InvalidKnnNeighbors(
                self.knn_neighbors,
            ));
        }

        Ok(())
    }
}

/// Errors that can occur during configuration validation.
#[derive(Debug, thiserror::Error)]
pub enum ConfigValidationError {
    #[error("Invalid threshold for '{field}': {value} (must be between 0.0 and 1.0)")]
    InvalidThreshold { field: String, value: f64 },

    #[error("Invalid KNN neighbors: {0} (must be at least 1)")]
    InvalidKnnNeighbors(usize),
}

/// Builder for [`PipelineConfig`] with fluent API.
#[derive(Debug, Default)]
pub struct PipelineConfigBuilder {
    missing_column_threshold: Option<f64>,
    missing_row_threshold: Option<f64>,
    outlier_strategy: Option<OutlierStrategy>,
    default_numeric_imputation: Option<NumericImputation>,
    default_categorical_imputation: Option<CategoricalImputation>,
    enable_type_correction: Option<bool>,
    remove_duplicates: Option<bool>,
    knn_neighbors: Option<usize>,
    output_dir: Option<PathBuf>,
    output_name: Option<String>,
    generate_reports: Option<bool>,
    use_ai_decisions: Option<bool>,
    target_column: Option<String>,
    save_to_disk: Option<bool>,
}

impl PipelineConfigBuilder {
    /// Set the threshold for dropping columns with missing values.
    ///
    /// Columns with a higher percentage of missing values than this threshold
    /// will be dropped from the dataset.
    ///
    /// # Arguments
    /// * `threshold` - Value between 0.0 and 1.0 (e.g., 0.7 = 70%)
    pub fn missing_column_threshold(mut self, threshold: f64) -> Self {
        self.missing_column_threshold = Some(threshold);
        self
    }

    /// Set the threshold for dropping rows with missing values.
    ///
    /// Rows with a higher percentage of missing values than this threshold
    /// will be dropped from the dataset.
    ///
    /// # Arguments
    /// * `threshold` - Value between 0.0 and 1.0 (e.g., 0.8 = 80%)
    pub fn missing_row_threshold(mut self, threshold: f64) -> Self {
        self.missing_row_threshold = Some(threshold);
        self
    }

    /// Set the strategy for handling outliers.
    pub fn outlier_strategy(mut self, strategy: OutlierStrategy) -> Self {
        self.outlier_strategy = Some(strategy);
        self
    }

    /// Set the default numeric imputation strategy.
    pub fn numeric_imputation(mut self, strategy: NumericImputation) -> Self {
        self.default_numeric_imputation = Some(strategy);
        self
    }

    /// Set the default categorical imputation strategy.
    pub fn categorical_imputation(mut self, strategy: CategoricalImputation) -> Self {
        self.default_categorical_imputation = Some(strategy);
        self
    }

    /// Enable or disable automatic type correction.
    pub fn enable_type_correction(mut self, enable: bool) -> Self {
        self.enable_type_correction = Some(enable);
        self
    }

    /// Enable or disable duplicate row removal.
    pub fn remove_duplicates(mut self, remove: bool) -> Self {
        self.remove_duplicates = Some(remove);
        self
    }

    /// Set the number of neighbors for KNN imputation.
    pub fn knn_neighbors(mut self, k: usize) -> Self {
        self.knn_neighbors = Some(k);
        self
    }

    /// Set the output directory for reports and cleaned data.
    pub fn output_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_dir = Some(path.into());
        self
    }

    /// Set a custom output file name (without extension).
    ///
    /// If not set, the default name "processed_dataset_{problem_type}" is used.
    pub fn output_name(mut self, name: impl Into<String>) -> Self {
        self.output_name = Some(name.into());
        self
    }

    /// Enable or disable report generation.
    pub fn generate_reports(mut self, generate: bool) -> Self {
        self.generate_reports = Some(generate);
        self
    }

    /// Enable or disable AI-based decision making.
    ///
    /// If disabled, the pipeline will use rule-based heuristics instead.
    pub fn use_ai_decisions(mut self, use_ai: bool) -> Self {
        self.use_ai_decisions = Some(use_ai);
        self
    }

    /// Set an explicit target column.
    ///
    /// If not set, the pipeline will auto-detect the target column.
    pub fn target_column(mut self, column: impl Into<String>) -> Self {
        self.target_column = Some(column.into());
        self
    }

    /// Enable or disable saving processed data to disk.
    ///
    /// When false, the pipeline keeps results in memory only and skips
    /// all file I/O. Useful for GUI applications that manage their own
    /// export functionality.
    pub fn save_to_disk(mut self, save: bool) -> Self {
        self.save_to_disk = Some(save);
        self
    }

    /// Build the configuration.
    ///
    /// Returns a validated `PipelineConfig` or an error if validation fails.
    pub fn build(self) -> Result<PipelineConfig, ConfigValidationError> {
        let config = PipelineConfig {
            missing_column_threshold: self.missing_column_threshold.unwrap_or(0.7),
            missing_row_threshold: self.missing_row_threshold.unwrap_or(0.8),
            outlier_strategy: self.outlier_strategy.unwrap_or_default(),
            default_numeric_imputation: self.default_numeric_imputation.unwrap_or_default(),
            default_categorical_imputation: self.default_categorical_imputation.unwrap_or_default(),
            enable_type_correction: self.enable_type_correction.unwrap_or(true),
            remove_duplicates: self.remove_duplicates.unwrap_or(true),
            knn_neighbors: self.knn_neighbors.unwrap_or(5),
            output_dir: self.output_dir.unwrap_or_else(|| PathBuf::from("output")),
            output_name: self.output_name,
            generate_reports: self.generate_reports.unwrap_or(true),
            use_ai_decisions: self.use_ai_decisions.unwrap_or(true),
            target_column: self.target_column,
            save_to_disk: self.save_to_disk.unwrap_or(true),
        };

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PipelineConfig::default();
        assert_eq!(config.missing_column_threshold, 0.7);
        assert_eq!(config.missing_row_threshold, 0.8);
        assert_eq!(config.outlier_strategy, OutlierStrategy::Cap);
        assert_eq!(config.knn_neighbors, 5);
        assert!(config.enable_type_correction);
        assert!(config.use_ai_decisions);
    }

    #[test]
    fn test_builder_defaults() {
        let config = PipelineConfig::builder().build().unwrap();
        assert_eq!(config.missing_column_threshold, 0.7);
        assert_eq!(config.missing_row_threshold, 0.8);
    }

    #[test]
    fn test_builder_custom_values() {
        let config = PipelineConfig::builder()
            .missing_column_threshold(0.5)
            .missing_row_threshold(0.6)
            .outlier_strategy(OutlierStrategy::Remove)
            .knn_neighbors(10)
            .use_ai_decisions(false)
            .build()
            .unwrap();

        assert_eq!(config.missing_column_threshold, 0.5);
        assert_eq!(config.missing_row_threshold, 0.6);
        assert_eq!(config.outlier_strategy, OutlierStrategy::Remove);
        assert_eq!(config.knn_neighbors, 10);
        assert!(!config.use_ai_decisions);
    }

    #[test]
    fn test_validation_invalid_column_threshold() {
        let result = PipelineConfig::builder()
            .missing_column_threshold(1.5)
            .build();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigValidationError::InvalidThreshold { .. }
        ));
    }

    #[test]
    fn test_validation_invalid_knn_neighbors() {
        let result = PipelineConfig::builder().knn_neighbors(0).build();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigValidationError::InvalidKnnNeighbors(0)
        ));
    }

    #[test]
    fn test_config_serialization() {
        let config = PipelineConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PipelineConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(
            config.missing_column_threshold,
            deserialized.missing_column_threshold
        );
        assert_eq!(config.outlier_strategy, deserialized.outlier_strategy);
    }

    #[test]
    fn test_pipeline_config_from_json() {
        // Simulate JSON that might come from a frontend
        let json = r#"{
            "missing_column_threshold": 0.5,
            "missing_row_threshold": 0.6,
            "outlier_strategy": "Remove",
            "default_numeric_imputation": "Knn",
            "default_categorical_imputation": "Constant",
            "enable_type_correction": false,
            "remove_duplicates": true,
            "knn_neighbors": 7,
            "output_dir": "custom_output",
            "output_name": "my_dataset",
            "generate_reports": false,
            "use_ai_decisions": false,
            "target_column": "label",
            "save_to_disk": false
        }"#;

        let config: PipelineConfig =
            serde_json::from_str(json).expect("Should deserialize from frontend JSON");

        assert_eq!(config.missing_column_threshold, 0.5);
        assert_eq!(config.missing_row_threshold, 0.6);
        assert_eq!(config.outlier_strategy, OutlierStrategy::Remove);
        assert_eq!(config.default_numeric_imputation, NumericImputation::Knn);
        assert_eq!(
            config.default_categorical_imputation,
            CategoricalImputation::Constant
        );
        assert!(!config.enable_type_correction);
        assert!(config.remove_duplicates);
        assert_eq!(config.knn_neighbors, 7);
        assert_eq!(config.output_dir.to_str().unwrap(), "custom_output");
        assert_eq!(config.output_name, Some("my_dataset".to_string()));
        assert!(!config.generate_reports);
        assert!(!config.use_ai_decisions);
        assert_eq!(config.target_column, Some("label".to_string()));
    }
}
