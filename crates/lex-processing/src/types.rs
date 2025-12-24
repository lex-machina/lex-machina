use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnProfile {
    pub name: String,
    pub dtype: String,
    pub unique_count: usize,
    pub null_count: usize,
    pub null_percentage: f64,
    pub sample_values: Vec<String>,
    pub inferred_type: String,
    pub inferred_role: String,
    pub characteristics: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetProfile {
    pub shape: (usize, usize),
    pub column_profiles: Vec<ColumnProfile>,
    pub target_candidates: Vec<String>,
    pub problem_type_candidates: Vec<String>,
    pub complexity_indicators: HashMap<String, serde_json::Value>,
    pub duplicate_count: usize,
    pub duplicate_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityIssue {
    pub issue_type: String,
    pub severity: String,
    pub affected_columns: Vec<String>,
    pub description: String,
    pub business_impact: String,
    pub detection_details: HashMap<String, serde_json::Value>,
    pub suggested_solutions: Vec<SolutionOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionOption {
    pub option: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pros: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cons: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_for: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionQuestion {
    pub id: String,
    pub issue_type: String,
    pub description: String,
    pub business_impact: String,
    pub detection_details: HashMap<String, serde_json::Value>,
    pub affected_columns: Vec<String>,
    pub options: Vec<SolutionOption>,
    pub sample_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub success: bool,
    pub cleaned_data: Option<String>,
    pub target_column: Option<String>,
    pub problem_type: Option<String>,
    pub ai_choices: HashMap<String, String>,
    pub analysis_report: Option<String>,
    pub processing_steps: Vec<String>,
    pub cleaning_actions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Detailed summary of preprocessing actions for UI display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<PreprocessingSummary>,
}

// ============================================================================
// Preprocessing Summary Types (for Tauri UI integration)
// ============================================================================

/// Human-readable summary of what the pipeline did.
///
/// This struct is designed to be serialized and sent to a frontend UI
/// (e.g., via Tauri IPC) to display preprocessing results to users.
///
/// # Example
///
/// ```rust,ignore
/// use data_preprocessing_pipeline::PreprocessingSummary;
///
/// let summary: PreprocessingSummary = result.summary.unwrap();
/// println!("Processed {} rows in {}ms", summary.rows_after, summary.duration_ms);
/// println!("Data quality improved from {:.0}% to {:.0}%",
///     summary.data_quality_score_before * 100.0,
///     summary.data_quality_score_after * 100.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingSummary {
    /// Total execution time in milliseconds.
    pub duration_ms: u64,

    /// Number of rows before preprocessing.
    pub rows_before: usize,
    /// Number of rows after preprocessing.
    pub rows_after: usize,
    /// Number of rows removed during preprocessing.
    pub rows_removed: usize,

    /// Number of columns before preprocessing.
    pub columns_before: usize,
    /// Number of columns after preprocessing.
    pub columns_after: usize,
    /// Number of columns removed during preprocessing.
    pub columns_removed: usize,

    /// Number of data quality issues found.
    pub issues_found: usize,
    /// Number of issues resolved by preprocessing.
    pub issues_resolved: usize,
    /// Data quality score before preprocessing (0.0 - 1.0).
    /// Calculated as percentage of non-null values.
    pub data_quality_score_before: f32,
    /// Data quality score after preprocessing (0.0 - 1.0).
    pub data_quality_score_after: f32,

    /// List of actions taken during preprocessing.
    pub actions: Vec<PreprocessingAction>,

    /// Per-column summaries of changes.
    pub column_summaries: Vec<ColumnSummary>,

    /// Warnings and notes generated during preprocessing.
    pub warnings: Vec<String>,
}

impl Default for PreprocessingSummary {
    fn default() -> Self {
        Self {
            duration_ms: 0,
            rows_before: 0,
            rows_after: 0,
            rows_removed: 0,
            columns_before: 0,
            columns_after: 0,
            columns_removed: 0,
            issues_found: 0,
            issues_resolved: 0,
            data_quality_score_before: 0.0,
            data_quality_score_after: 0.0,
            actions: Vec::new(),
            column_summaries: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl PreprocessingSummary {
    /// Create a new empty summary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an action to the summary.
    pub fn add_action(&mut self, action: PreprocessingAction) {
        self.actions.push(action);
    }

    /// Add a warning to the summary.
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Add a column summary.
    pub fn add_column_summary(&mut self, summary: ColumnSummary) {
        self.column_summaries.push(summary);
    }

    /// Calculate the percentage of rows removed.
    pub fn rows_removed_percentage(&self) -> f32 {
        if self.rows_before == 0 {
            0.0
        } else {
            (self.rows_removed as f32 / self.rows_before as f32) * 100.0
        }
    }

    /// Calculate the percentage of columns removed.
    pub fn columns_removed_percentage(&self) -> f32 {
        if self.columns_before == 0 {
            0.0
        } else {
            (self.columns_removed as f32 / self.columns_before as f32) * 100.0
        }
    }

    /// Calculate data quality improvement as a percentage.
    pub fn quality_improvement(&self) -> f32 {
        (self.data_quality_score_after - self.data_quality_score_before) * 100.0
    }
}

/// A single action taken during preprocessing.
///
/// Actions are logged throughout the pipeline execution to provide
/// a detailed audit trail of what was done to the data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingAction {
    /// Type of action performed.
    pub action_type: ActionType,
    /// Target of the action (column name or "dataset").
    pub target: String,
    /// Human-readable description of the action.
    pub description: String,
    /// Additional details (e.g., values replaced, strategy used).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl PreprocessingAction {
    /// Create a new preprocessing action.
    pub fn new(action_type: ActionType, target: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            action_type,
            target: target.into(),
            description: description.into(),
            details: None,
        }
    }

    /// Add details to the action.
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Types of actions that can be taken during preprocessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// A column was removed from the dataset.
    ColumnRemoved,
    /// One or more rows were removed from the dataset.
    RowsRemoved,
    /// A column's data type was corrected.
    TypeCorrected,
    /// Missing values were imputed.
    ValueImputed,
    /// Outliers were handled (removed, capped, or transformed).
    OutlierHandled,
    /// Duplicate rows were removed.
    DuplicatesRemoved,
    /// A target column was identified.
    TargetIdentified,
    /// The problem type was detected (classification/regression).
    ProblemTypeDetected,
    /// A column was renamed.
    ColumnRenamed,
    /// Invalid values were cleaned/replaced.
    ValueCleaned,
    /// Data was normalized or scaled.
    DataNormalized,
    /// Categories were encoded.
    CategoriesEncoded,
}

impl ActionType {
    /// Get a human-readable display name for the action type.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ColumnRemoved => "Column Removed",
            Self::RowsRemoved => "Rows Removed",
            Self::TypeCorrected => "Type Corrected",
            Self::ValueImputed => "Value Imputed",
            Self::OutlierHandled => "Outlier Handled",
            Self::DuplicatesRemoved => "Duplicates Removed",
            Self::TargetIdentified => "Target Identified",
            Self::ProblemTypeDetected => "Problem Type Detected",
            Self::ColumnRenamed => "Column Renamed",
            Self::ValueCleaned => "Value Cleaned",
            Self::DataNormalized => "Data Normalized",
            Self::CategoriesEncoded => "Categories Encoded",
        }
    }

    /// Get an icon/emoji for the action type (for UI display).
    pub fn icon(&self) -> &'static str {
        match self {
            Self::ColumnRemoved => "🗑️",
            Self::RowsRemoved => "➖",
            Self::TypeCorrected => "🔧",
            Self::ValueImputed => "📝",
            Self::OutlierHandled => "📊",
            Self::DuplicatesRemoved => "🔄",
            Self::TargetIdentified => "🎯",
            Self::ProblemTypeDetected => "🔍",
            Self::ColumnRenamed => "✏️",
            Self::ValueCleaned => "🧹",
            Self::DataNormalized => "📈",
            Self::CategoriesEncoded => "🏷️",
        }
    }
}

/// Summary of changes made to a single column.
///
/// Provides detailed information about what happened to each column
/// during preprocessing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSummary {
    /// Name of the column.
    pub name: String,
    /// Original data type (as string).
    pub original_type: String,
    /// Final data type after preprocessing.
    pub final_type: String,
    /// Number of missing values before preprocessing.
    pub missing_before: usize,
    /// Number of missing values after preprocessing.
    pub missing_after: usize,
    /// Imputation method used, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imputation_method: Option<String>,
    /// Number of outliers handled.
    pub outliers_handled: usize,
    /// Number of type corrections made.
    pub type_corrections: usize,
    /// Number of invalid values cleaned.
    pub values_cleaned: usize,
    /// Whether the column was removed.
    pub was_removed: bool,
    /// Reason for removal, if removed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removal_reason: Option<String>,
}

impl ColumnSummary {
    /// Create a new column summary with default values.
    pub fn new(name: impl Into<String>, original_type: impl Into<String>) -> Self {
        let name = name.into();
        let original_type = original_type.into();
        Self {
            name,
            original_type: original_type.clone(),
            final_type: original_type,
            missing_before: 0,
            missing_after: 0,
            imputation_method: None,
            outliers_handled: 0,
            type_corrections: 0,
            values_cleaned: 0,
            was_removed: false,
            removal_reason: None,
        }
    }

    /// Mark the column as removed with a reason.
    pub fn mark_removed(mut self, reason: impl Into<String>) -> Self {
        self.was_removed = true;
        self.removal_reason = Some(reason.into());
        self
    }

    /// Calculate the percentage of missing values imputed.
    pub fn imputation_percentage(&self) -> f32 {
        if self.missing_before == 0 {
            0.0
        } else {
            let imputed = self.missing_before.saturating_sub(self.missing_after);
            (imputed as f32 / self.missing_before as f32) * 100.0
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessing_summary_default() {
        let summary = PreprocessingSummary::default();
        assert_eq!(summary.duration_ms, 0);
        assert_eq!(summary.rows_before, 0);
        assert!(summary.actions.is_empty());
    }

    #[test]
    fn test_preprocessing_summary_add_action() {
        let mut summary = PreprocessingSummary::new();
        summary.add_action(PreprocessingAction::new(
            ActionType::ColumnRemoved,
            "column_a",
            "Removed due to high null percentage",
        ));
        assert_eq!(summary.actions.len(), 1);
        assert_eq!(summary.actions[0].target, "column_a");
    }

    #[test]
    fn test_preprocessing_summary_percentages() {
        let mut summary = PreprocessingSummary::new();
        summary.rows_before = 100;
        summary.rows_after = 90;
        summary.rows_removed = 10;
        summary.columns_before = 10;
        summary.columns_after = 8;
        summary.columns_removed = 2;

        assert!((summary.rows_removed_percentage() - 10.0).abs() < 0.01);
        assert!((summary.columns_removed_percentage() - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_preprocessing_summary_quality_improvement() {
        let mut summary = PreprocessingSummary::new();
        summary.data_quality_score_before = 0.75;
        summary.data_quality_score_after = 0.95;

        assert!((summary.quality_improvement() - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_preprocessing_action_with_details() {
        let action = PreprocessingAction::new(
            ActionType::ValueImputed,
            "age",
            "Imputed 15 missing values",
        )
        .with_details("Used median imputation (value: 32)");

        assert_eq!(action.action_type, ActionType::ValueImputed);
        assert_eq!(action.target, "age");
        assert!(action.details.is_some());
        assert!(action.details.unwrap().contains("median"));
    }

    #[test]
    fn test_action_type_display_name() {
        assert_eq!(ActionType::ColumnRemoved.display_name(), "Column Removed");
        assert_eq!(ActionType::ValueImputed.display_name(), "Value Imputed");
        assert_eq!(ActionType::DuplicatesRemoved.display_name(), "Duplicates Removed");
    }

    #[test]
    fn test_column_summary_imputation_percentage() {
        let mut summary = ColumnSummary::new("age", "Float64");
        summary.missing_before = 20;
        summary.missing_after = 5;

        assert!((summary.imputation_percentage() - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_column_summary_mark_removed() {
        let summary = ColumnSummary::new("temp_col", "String")
            .mark_removed("Over 90% null values");

        assert!(summary.was_removed);
        assert_eq!(summary.removal_reason.unwrap(), "Over 90% null values");
    }

    #[test]
    fn test_preprocessing_summary_serialization() {
        let mut summary = PreprocessingSummary::new();
        summary.duration_ms = 1500;
        summary.rows_before = 1000;
        summary.rows_after = 950;
        summary.add_action(PreprocessingAction::new(
            ActionType::DuplicatesRemoved,
            "dataset",
            "Removed 50 duplicate rows",
        ));

        let json = serde_json::to_string(&summary).expect("Should serialize");
        assert!(json.contains("1500"));
        assert!(json.contains("duplicates_removed"));
    }

    #[test]
    fn test_pipeline_result_json_roundtrip() {
        let mut result = PipelineResult {
            success: true,
            cleaned_data: Some("path/to/data.csv".to_string()),
            target_column: Some("survived".to_string()),
            problem_type: Some("binary_classification".to_string()),
            ai_choices: HashMap::from([
                ("missing_values".to_string(), "median".to_string()),
                ("outliers".to_string(), "cap".to_string()),
            ]),
            analysis_report: Some("report.json".to_string()),
            processing_steps: vec!["profiling".to_string(), "cleaning".to_string()],
            cleaning_actions: vec!["Removed 10 duplicates".to_string()],
            error: None,
            summary: Some(PreprocessingSummary::default()),
        };
        result.summary.as_mut().unwrap().rows_before = 100;
        result.summary.as_mut().unwrap().rows_after = 95;

        let json = serde_json::to_string(&result).expect("Should serialize");
        let deserialized: PipelineResult = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(result.success, deserialized.success);
        assert_eq!(result.target_column, deserialized.target_column);
        assert_eq!(result.problem_type, deserialized.problem_type);
        assert_eq!(result.ai_choices.len(), deserialized.ai_choices.len());
        assert_eq!(
            result.summary.as_ref().unwrap().rows_before,
            deserialized.summary.as_ref().unwrap().rows_before
        );
    }

    #[test]
    fn test_all_action_types_serialize() {
        let all_types = [
            ActionType::ColumnRemoved,
            ActionType::RowsRemoved,
            ActionType::TypeCorrected,
            ActionType::ValueImputed,
            ActionType::OutlierHandled,
            ActionType::DuplicatesRemoved,
            ActionType::TargetIdentified,
            ActionType::ProblemTypeDetected,
            ActionType::ColumnRenamed,
            ActionType::ValueCleaned,
            ActionType::DataNormalized,
            ActionType::CategoriesEncoded,
        ];

        let expected_json_values = [
            "\"column_removed\"",
            "\"rows_removed\"",
            "\"type_corrected\"",
            "\"value_imputed\"",
            "\"outlier_handled\"",
            "\"duplicates_removed\"",
            "\"target_identified\"",
            "\"problem_type_detected\"",
            "\"column_renamed\"",
            "\"value_cleaned\"",
            "\"data_normalized\"",
            "\"categories_encoded\"",
        ];

        for (action_type, expected) in all_types.iter().zip(expected_json_values.iter()) {
            let json = serde_json::to_string(action_type).expect("Should serialize");
            assert_eq!(&json, *expected, "ActionType::{:?} should serialize to {}", action_type, expected);
        }
    }
}