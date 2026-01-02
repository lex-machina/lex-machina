//! Common types used throughout the lex-learning crate.
//!
//! This module defines result types, metrics, and other data structures
//! returned by the training pipeline and model.
//!
//! # Overview
//!
//! - [`TrainingResult`]: Complete result from [`Pipeline::train()`](crate::Pipeline::train)
//! - [`Metrics`]: Evaluation metrics (classification or regression)
//! - [`ModelComparison`]: Comparison data for evaluated model candidates
//! - [`PredictionResult`]: Result from [`TrainedModel::predict()`](crate::TrainedModel::predict)
//! - [`ModelInfo`]: Metadata about a trained model
//!
//! # Example
//!
//! ```ignore
//! let result = pipeline.train(&df)?;
//!
//! println!("Best model: {}", result.best_model_name);
//! println!("Test score: {:?}", result.metrics.test_score);
//!
//! // Access SHAP plots
//! for (name, png_bytes) in &result.shap_plots {
//!     std::fs::write(format!("{}.png", name), png_bytes)?;
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of a training pipeline run.
///
/// Returned by [`Pipeline::train()`](crate::Pipeline::train). Contains all information
/// about the training process including metrics, feature importance, and explainability plots.
///
/// # Fields
///
/// - `success`: Whether training completed successfully
/// - `best_model_name`: Name of the algorithm that performed best
/// - `metrics`: Evaluation metrics for the best model
/// - `feature_importance`: Ranked list of (feature, importance) pairs
/// - `shap_plots`: SHAP explainability plots as PNG bytes
/// - `model_comparison`: Performance comparison of all evaluated models
/// - `training_time_seconds`: Total wall-clock training time
/// - `warnings`: Non-fatal warnings generated during training
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TrainingResult {
    /// Whether training succeeded.
    ///
    /// If `false`, check `warnings` for error details.
    pub success: bool,

    /// Name of the best performing model.
    ///
    /// This is the algorithm name (e.g., "random_forest", "xgboost", "lightgbm").
    pub best_model_name: String,

    /// Metrics achieved by the best model.
    ///
    /// Contains classification metrics (accuracy, F1, etc.) or regression metrics
    /// (R², RMSE, etc.) depending on the problem type.
    pub metrics: Metrics,

    /// Feature importance scores (feature name, importance).
    ///
    /// Sorted in descending order by importance. Importance values are normalized
    /// to sum to 1.0 for most models.
    pub feature_importance: Vec<(String, f64)>,

    /// SHAP explainability plots as PNG bytes (plot name → bytes).
    ///
    /// Common plot names:
    /// - `"summary"`: SHAP summary plot
    /// - `"beeswarm"`: SHAP beeswarm plot
    /// - `"feature_importance"`: Bar plot of feature importance
    ///
    /// Empty if `enable_explainability` was set to `false` in the config.
    pub shap_plots: HashMap<String, Vec<u8>>,

    /// Comparison of all evaluated models.
    ///
    /// Contains performance metrics for each model that was trained,
    /// useful for understanding why a particular model was selected.
    pub model_comparison: Vec<ModelComparison>,

    /// Total training time in seconds.
    ///
    /// Wall-clock time from start to finish, including preprocessing,
    /// model selection, training, and explainability analysis.
    pub training_time_seconds: f64,

    /// Warnings generated during training.
    ///
    /// Non-fatal issues that occurred during training. Check these even
    /// when `success` is `true`.
    pub warnings: Vec<String>,
}

/// Comparison data for a single model candidate.
///
/// Contains performance metrics and metadata for one of the models
/// evaluated during the algorithm selection phase.
///
/// # Overfitting Risk
///
/// The `overfitting_risk` field is computed by comparing train and test scores:
/// - `"low"`: Train-test gap < 5%
/// - `"medium"`: Train-test gap 5-15%
/// - `"high"`: Train-test gap > 15%
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelComparison {
    /// Model algorithm name (e.g., "random_forest", "xgboost").
    pub name: String,

    /// Score on the held-out test set.
    ///
    /// This is the primary metric for model selection.
    pub test_score: f64,

    /// Score on the training set.
    ///
    /// Used with `test_score` to assess overfitting.
    pub train_score: f64,

    /// Mean cross-validation score.
    ///
    /// Average score across all CV folds, provides a more robust estimate
    /// of model performance than a single train/test split.
    pub cv_score: f64,

    /// Time taken to train this model in seconds.
    pub training_time_seconds: f64,

    /// Hyperparameters used for this model.
    ///
    /// Keys are parameter names, values are the parameter values
    /// (can be numbers, strings, booleans, or nested objects).
    pub hyperparameters: HashMap<String, serde_json::Value>,

    /// Overfitting risk assessment: `"low"`, `"medium"`, or `"high"`.
    ///
    /// Based on the gap between training and test scores.
    pub overfitting_risk: String,
}

/// Metrics from model evaluation.
///
/// Contains optional fields for both classification and regression metrics.
/// Only the relevant fields will be populated based on the problem type.
///
/// # Classification Metrics
///
/// For classification problems, the following fields are populated:
/// - `accuracy`: Overall classification accuracy
/// - `precision`: Weighted precision across classes
/// - `recall`: Weighted recall across classes
/// - `f1_score`: Weighted F1 score
/// - `roc_auc`: ROC AUC (binary classification only)
///
/// # Regression Metrics
///
/// For regression problems, the following fields are populated:
/// - `mse`: Mean Squared Error
/// - `rmse`: Root Mean Squared Error
/// - `mae`: Mean Absolute Error
/// - `r2`: R-squared (coefficient of determination)
///
/// # Common Metrics
///
/// These are populated for both problem types:
/// - `cv_score`: Mean cross-validation score
/// - `test_score`: Score on held-out test set
/// - `train_score`: Score on training set
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Metrics {
    // Common metrics
    /// Cross-validation score (mean across all folds).
    ///
    /// Uses accuracy for classification, R² for regression.
    pub cv_score: Option<f64>,

    /// Score on the held-out test set.
    ///
    /// Uses accuracy for classification, R² for regression.
    pub test_score: Option<f64>,

    /// Score on the training set.
    ///
    /// Used with `test_score` to detect overfitting.
    pub train_score: Option<f64>,

    // Classification metrics
    /// Accuracy score (classification only).
    ///
    /// Fraction of correct predictions. Range: [0.0, 1.0].
    pub accuracy: Option<f64>,

    /// Precision score (classification only).
    ///
    /// Weighted average precision across all classes. Range: [0.0, 1.0].
    pub precision: Option<f64>,

    /// Recall score (classification only).
    ///
    /// Weighted average recall across all classes. Range: [0.0, 1.0].
    pub recall: Option<f64>,

    /// F1 score (classification only).
    ///
    /// Weighted average F1 score across all classes. Range: [0.0, 1.0].
    pub f1_score: Option<f64>,

    /// ROC AUC score (binary classification only).
    ///
    /// Area under the ROC curve. Range: [0.0, 1.0].
    /// `None` for multi-class classification or regression.
    pub roc_auc: Option<f64>,

    // Regression metrics
    /// Mean Squared Error (regression only).
    ///
    /// Average of squared prediction errors. Lower is better.
    pub mse: Option<f64>,

    /// Root Mean Squared Error (regression only).
    ///
    /// Square root of MSE, in the same units as the target. Lower is better.
    pub rmse: Option<f64>,

    /// Mean Absolute Error (regression only).
    ///
    /// Average of absolute prediction errors. Lower is better.
    pub mae: Option<f64>,

    /// R-squared score (regression only).
    ///
    /// Coefficient of determination. Range: (-∞, 1.0], where 1.0 is perfect.
    /// Negative values indicate worse than a constant prediction.
    pub r2: Option<f64>,
}

/// Result of a single prediction.
///
/// Returned by [`TrainedModel::predict()`](crate::TrainedModel::predict) for
/// single-instance predictions.
///
/// # Classification
///
/// For classification, `prediction` contains the predicted class label (string),
/// and `probabilities` contains the probability for each class.
///
/// # Regression
///
/// For regression, `prediction` contains the predicted numeric value,
/// and `probabilities` is `None`.
///
/// # Example
///
/// ```ignore
/// let result = model.predict(&json!({"age": 25, "income": 50000}))?;
///
/// match result.prediction {
///     serde_json::Value::String(class) => println!("Predicted class: {}", class),
///     serde_json::Value::Number(n) => println!("Predicted value: {}", n),
///     _ => {}
/// }
///
/// if let Some(probs) = result.probabilities {
///     for (class, prob) in probs {
///         println!("  {}: {:.2}%", class, prob * 100.0);
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PredictionResult {
    /// The predicted value.
    ///
    /// - Classification: String containing the class label
    /// - Regression: Number containing the predicted value
    pub prediction: serde_json::Value,

    /// Class probabilities (classification only).
    ///
    /// Maps class labels to their predicted probabilities.
    /// Probabilities sum to 1.0. `None` for regression.
    pub probabilities: Option<HashMap<String, f64>>,

    /// Confidence score (if available).
    ///
    /// For classification, this is typically the probability of the predicted class.
    /// May be `None` if the model doesn't provide confidence scores.
    pub confidence: Option<f64>,
}

/// Information about a trained model.
///
/// Returned by [`TrainedModel::get_info()`](crate::TrainedModel::get_info).
/// Contains metadata about the model's configuration and training results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelInfo {
    /// Name of the model algorithm (e.g., "random_forest", "xgboost").
    pub model_name: String,

    /// Type of problem: `"classification"` or `"regression"`.
    pub problem_type: String,

    /// Name of the target column used during training.
    pub target_column: String,

    /// Names of the feature columns in the order expected by the model.
    ///
    /// When making predictions, input data must contain these features.
    pub feature_names: Vec<String>,

    /// Class labels (classification only).
    ///
    /// List of possible class values. `None` for regression models.
    pub class_labels: Option<Vec<String>>,

    /// Metrics achieved during training.
    pub metrics: Metrics,

    /// Hyperparameters used by the model.
    ///
    /// Keys are parameter names, values are the parameter values.
    pub hyperparameters: HashMap<String, serde_json::Value>,
}
