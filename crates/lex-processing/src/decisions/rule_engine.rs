//! Rule-based decision engine using heuristics.

use super::DecisionEngine;
use crate::config::PipelineConfig;
use crate::types::{DataQualityIssue, DatasetProfile};
use anyhow::Result;
use polars::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Rule-based decision maker using heuristics.
///
/// This engine makes decisions based on data characteristics without
/// requiring an AI service. Useful as a fallback or for offline use.
pub struct RuleBasedDecisionEngine {
    config: PipelineConfig,
}

impl RuleBasedDecisionEngine {
    pub fn new(config: PipelineConfig) -> Self {
        Self { config }
    }

    /// Determine imputation strategy for a column based on its characteristics.
    fn determine_imputation_strategy(&self, issue: &DataQualityIssue, df: &DataFrame) -> String {
        let missing_pct = issue
            .detection_details
            .get("missing_percentage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Get the first affected column to analyze
        let col_name = issue.affected_columns.first();

        if let Some(col_name) = col_name
            && let Ok(series) = df.column(col_name) {
                let is_numeric = matches!(
                    series.dtype(),
                    DataType::Int8
                        | DataType::Int16
                        | DataType::Int32
                        | DataType::Int64
                        | DataType::UInt8
                        | DataType::UInt16
                        | DataType::UInt32
                        | DataType::UInt64
                        | DataType::Float32
                        | DataType::Float64
                );

                if is_numeric {
                    // For numeric columns
                    if missing_pct < 5.0 {
                        return "median_imputation".to_string();
                    } else if missing_pct < 20.0 {
                        return "knn_imputation".to_string();
                    } else {
                        return "median_imputation".to_string();
                    }
                } else {
                    // For categorical columns
                    if missing_pct < 10.0 {
                        return "mode_imputation".to_string();
                    } else {
                        return "constant_imputation".to_string();
                    }
                }
            }

        // Default fallback
        "median_imputation".to_string()
    }

    /// Determine problem type based on dataset characteristics.
    pub(crate) fn determine_problem_type(&self, profile: &DatasetProfile) -> String {
        // Look for target candidates
        for col in &profile.column_profiles {
            if col.inferred_role == "target_candidate" {
                // Binary or low cardinality = classification
                if col.unique_count <= 10 || col.inferred_type == "binary" {
                    return "classification".to_string();
                }
                // High cardinality numeric = regression
                if col.inferred_type == "numeric" && col.unique_count > 10 {
                    return "regression".to_string();
                }
            }
        }

        // Default to classification
        "classification".to_string()
    }

    /// Select target column based on heuristics.
    fn select_target_column(&self, profile: &DatasetProfile, problem_type: &str) -> String {
        // Priority 0: If target column is explicitly specified in config, use it
        if let Some(ref target) = self.config.target_column {
            // Validate that the column exists
            let column_exists = profile
                .column_profiles
                .iter()
                .any(|col| &col.name == target);
            
            if column_exists {
                info!("Using explicitly specified target column: {}", target);
                return target.clone();
            } else {
                warn!(
                    "Specified target column '{}' not found in dataset, falling back to auto-detection",
                    target
                );
            }
        }

        // Priority 1: Columns explicitly marked as target candidates
        let candidates: Vec<_> = profile
            .column_profiles
            .iter()
            .filter(|col| {
                col.inferred_role == "target_candidate"
                    && !["identifier", "metadata"].contains(&col.inferred_role.as_str())
            })
            .collect();

        if !candidates.is_empty() {
            // For classification, prefer binary/categorical with low unique count
            if problem_type == "classification" {
                if let Some(col) = candidates.iter().find(|c| c.inferred_type == "binary") {
                    return col.name.clone();
                }
                if let Some(col) = candidates.iter().find(|c| c.unique_count <= 10) {
                    return col.name.clone();
                }
            }
            // For regression, prefer numeric with high unique count
            if problem_type == "regression"
                && let Some(col) = candidates
                    .iter()
                    .find(|c| c.inferred_type == "numeric" && c.unique_count > 10)
                {
                    return col.name.clone();
                }
            return candidates[0].name.clone();
        }

        // Priority 2: Last column (common convention)
        if let Some(last_col) = profile.column_profiles.last()
            && !["identifier", "metadata"].contains(&last_col.inferred_role.as_str()) {
                return last_col.name.clone();
            }

        // Fallback: first non-identifier column
        profile
            .column_profiles
            .iter()
            .find(|c| c.inferred_role != "identifier")
            .map(|c| c.name.clone())
            .unwrap_or_default()
    }
}

impl DecisionEngine for RuleBasedDecisionEngine {
    fn make_decisions(
        &self,
        issues: &[DataQualityIssue],
        df: &DataFrame,
    ) -> Result<HashMap<String, String>> {
        let mut choices = HashMap::new();

        info!("Making rule-based decisions for data quality issues...");

        for (i, issue) in issues.iter().enumerate() {
            let decision_id = format!("{}_{}", issue.issue_type, i);

            let choice = match issue.issue_type.as_str() {
                "missing_values" => self.determine_imputation_strategy(issue, df),
                "problem_type_selection" => self.determine_problem_type(
                    // We don't have profile here, so use a simple heuristic
                    &DatasetProfile {
                        shape: (df.height(), df.width()),
                        column_profiles: vec![],
                        target_candidates: vec![],
                        problem_type_candidates: vec!["classification".to_string()],
                        complexity_indicators: HashMap::new(),
                        duplicate_count: 0,
                        duplicate_percentage: 0.0,
                    },
                ),
                "outliers" => match self.config.outlier_strategy {
                    crate::config::OutlierStrategy::Cap => "cap_outliers".to_string(),
                    crate::config::OutlierStrategy::Remove => "remove_outliers".to_string(),
                    crate::config::OutlierStrategy::Median => "median_replacement".to_string(),
                    crate::config::OutlierStrategy::Keep => "keep_outliers".to_string(),
                },
                _ => {
                    // Use first suggested solution as default
                    issue
                        .suggested_solutions
                        .first()
                        .map(|s| s.option.clone())
                        .unwrap_or_else(|| "default".to_string())
                }
            };

            debug!("Rule-based decision for {}: {}", issue.issue_type, choice);
            choices.insert(decision_id, choice);
        }

        Ok(choices)
    }

    fn finalize_problem_setup(
        &self,
        profile: &DatasetProfile,
        choices: &HashMap<String, String>,
        _df: &DataFrame,
    ) -> Result<(String, String)> {
        // Get problem type from choices or determine from profile
        let problem_type = choices
            .iter()
            .find(|(k, _)| k.contains("problem_type_selection"))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| self.determine_problem_type(profile));

        let target_column = self.select_target_column(profile, &problem_type);

        info!("Problem setup finalized (rule-based):");
        info!("  Problem type: {}", problem_type);
        info!("  Target column: {}", target_column);

        Ok((problem_type, target_column))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OutlierStrategy;
    use crate::types::{ColumnProfile, SolutionOption};

    /// Helper to create a DataQualityIssue for testing
    fn create_issue(issue_type: &str, cols: Vec<&str>, missing_pct: f64) -> DataQualityIssue {
        let mut details = HashMap::new();
        details.insert(
            "missing_percentage".to_string(),
            serde_json::json!(missing_pct),
        );

        DataQualityIssue {
            issue_type: issue_type.to_string(),
            severity: "medium".to_string(),
            affected_columns: cols.iter().map(|s| s.to_string()).collect(),
            description: "Test issue".to_string(),
            business_impact: "None".to_string(),
            detection_details: details,
            suggested_solutions: vec![SolutionOption {
                option: "default_option".to_string(),
                description: "Default".to_string(),
                pros: None,
                cons: None,
                best_for: None,
            }],
        }
    }

    /// Helper to create a ColumnProfile for testing
    fn create_column_profile(
        name: &str,
        inferred_type: &str,
        inferred_role: &str,
        unique_count: usize,
    ) -> ColumnProfile {
        ColumnProfile {
            name: name.to_string(),
            dtype: "String".to_string(),
            inferred_type: inferred_type.to_string(),
            null_count: 0,
            null_percentage: 0.0,
            unique_count,
            sample_values: vec![],
            inferred_role: inferred_role.to_string(),
            characteristics: HashMap::new(),
        }
    }

    /// Helper to create a DatasetProfile for testing
    fn create_profile(columns: Vec<ColumnProfile>) -> DatasetProfile {
        DatasetProfile {
            shape: (100, columns.len()),
            column_profiles: columns,
            target_candidates: vec![],
            problem_type_candidates: vec![],
            complexity_indicators: HashMap::new(),
            duplicate_count: 0,
            duplicate_percentage: 0.0,
        }
    }

    // ==================== determine_imputation_strategy tests ====================

    #[test]
    fn test_imputation_numeric_low_missing_uses_median() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let df = df!["value" => [1.0f64, 2.0, 3.0]].unwrap();
        let issue = create_issue("missing_values", vec!["value"], 3.0);

        let strategy = engine.determine_imputation_strategy(&issue, &df);
        assert_eq!(strategy, "median_imputation");
    }

    #[test]
    fn test_imputation_numeric_moderate_missing_uses_knn() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let df = df!["value" => [1.0f64, 2.0, 3.0]].unwrap();
        let issue = create_issue("missing_values", vec!["value"], 15.0);

        let strategy = engine.determine_imputation_strategy(&issue, &df);
        assert_eq!(strategy, "knn_imputation");
    }

    #[test]
    fn test_imputation_numeric_high_missing_uses_median() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let df = df!["value" => [1.0f64, 2.0, 3.0]].unwrap();
        let issue = create_issue("missing_values", vec!["value"], 25.0);

        let strategy = engine.determine_imputation_strategy(&issue, &df);
        assert_eq!(strategy, "median_imputation");
    }

    #[test]
    fn test_imputation_categorical_low_missing_uses_mode() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let df = df!["category" => ["a", "b", "c"]].unwrap();
        let issue = create_issue("missing_values", vec!["category"], 5.0);

        let strategy = engine.determine_imputation_strategy(&issue, &df);
        assert_eq!(strategy, "mode_imputation");
    }

    #[test]
    fn test_imputation_categorical_high_missing_uses_constant() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let df = df!["category" => ["a", "b", "c"]].unwrap();
        let issue = create_issue("missing_values", vec!["category"], 15.0);

        let strategy = engine.determine_imputation_strategy(&issue, &df);
        assert_eq!(strategy, "constant_imputation");
    }

    #[test]
    fn test_imputation_fallback_missing_column() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let df = df!["other" => [1, 2, 3]].unwrap();
        let issue = create_issue("missing_values", vec!["nonexistent"], 10.0);

        let strategy = engine.determine_imputation_strategy(&issue, &df);
        assert_eq!(strategy, "median_imputation"); // fallback
    }

    // ==================== determine_problem_type tests ====================

    #[test]
    fn test_problem_type_binary_classification() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let profile = create_profile(vec![create_column_profile(
            "survived",
            "binary",
            "target_candidate",
            2,
        )]);

        let problem_type = engine.determine_problem_type(&profile);
        assert_eq!(problem_type, "classification");
    }

    #[test]
    fn test_problem_type_multiclass_classification() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let profile = create_profile(vec![create_column_profile(
            "class",
            "string",
            "target_candidate",
            5,
        )]);

        let problem_type = engine.determine_problem_type(&profile);
        assert_eq!(problem_type, "classification");
    }

    #[test]
    fn test_problem_type_regression() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let profile = create_profile(vec![create_column_profile(
            "price",
            "numeric",
            "target_candidate",
            100,
        )]);

        let problem_type = engine.determine_problem_type(&profile);
        assert_eq!(problem_type, "regression");
    }

    #[test]
    fn test_problem_type_default_classification() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let profile = create_profile(vec![create_column_profile("feature", "numeric", "feature", 50)]);

        let problem_type = engine.determine_problem_type(&profile);
        assert_eq!(problem_type, "classification");
    }

    // ==================== select_target_column tests ====================

    #[test]
    fn test_target_selection_binary_for_classification() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let profile = create_profile(vec![
            create_column_profile("feature1", "numeric", "feature", 50),
            create_column_profile("survived", "binary", "target_candidate", 2),
        ]);

        let target = engine.select_target_column(&profile, "classification");
        assert_eq!(target, "survived");
    }

    #[test]
    fn test_target_selection_numeric_for_regression() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let profile = create_profile(vec![
            create_column_profile("feature1", "numeric", "feature", 50),
            create_column_profile("price", "numeric", "target_candidate", 100),
        ]);

        let target = engine.select_target_column(&profile, "regression");
        assert_eq!(target, "price");
    }

    #[test]
    fn test_target_selection_last_column_fallback() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let profile = create_profile(vec![
            create_column_profile("col1", "numeric", "feature", 50),
            create_column_profile("col2", "numeric", "feature", 50),
        ]);

        let target = engine.select_target_column(&profile, "classification");
        assert_eq!(target, "col2"); // Last column
    }

    #[test]
    fn test_target_selection_skips_identifier() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let profile = create_profile(vec![
            create_column_profile("feature", "numeric", "feature", 50),
            create_column_profile("id", "identifier", "identifier", 100),
        ]);

        let target = engine.select_target_column(&profile, "classification");
        assert_eq!(target, "feature");
    }

    // ==================== make_decisions tests ====================

    #[test]
    fn test_make_decisions_outlier_cap() {
        let config = PipelineConfig::builder()
            .outlier_strategy(OutlierStrategy::Cap)
            .build()
            .unwrap();
        let engine = RuleBasedDecisionEngine::new(config);
        let df = df!["value" => [1.0f64, 2.0, 3.0]].unwrap();
        let issues = vec![create_issue("outliers", vec!["value"], 0.0)];

        let decisions = engine.make_decisions(&issues, &df).unwrap();
        assert!(decisions.values().any(|v| v == "cap_outliers"));
    }

    #[test]
    fn test_make_decisions_outlier_remove() {
        let config = PipelineConfig::builder()
            .outlier_strategy(OutlierStrategy::Remove)
            .build()
            .unwrap();
        let engine = RuleBasedDecisionEngine::new(config);
        let df = df!["value" => [1.0f64, 2.0, 3.0]].unwrap();
        let issues = vec![create_issue("outliers", vec!["value"], 0.0)];

        let decisions = engine.make_decisions(&issues, &df).unwrap();
        assert!(decisions.values().any(|v| v == "remove_outliers"));
    }

    #[test]
    fn test_make_decisions_outlier_keep() {
        let config = PipelineConfig::builder()
            .outlier_strategy(OutlierStrategy::Keep)
            .build()
            .unwrap();
        let engine = RuleBasedDecisionEngine::new(config);
        let df = df!["value" => [1.0f64, 2.0, 3.0]].unwrap();
        let issues = vec![create_issue("outliers", vec!["value"], 0.0)];

        let decisions = engine.make_decisions(&issues, &df).unwrap();
        assert!(decisions.values().any(|v| v == "keep_outliers"));
    }

    #[test]
    fn test_make_decisions_unknown_issue_uses_first_suggestion() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let df = df!["value" => [1, 2, 3]].unwrap();
        let issues = vec![create_issue("unknown_issue_type", vec!["value"], 0.0)];

        let decisions = engine.make_decisions(&issues, &df).unwrap();
        assert!(decisions.values().any(|v| v == "default_option"));
    }

    // ==================== finalize_problem_setup tests ====================

    #[test]
    fn test_finalize_problem_setup_uses_choices() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let profile = create_profile(vec![create_column_profile(
            "target",
            "binary",
            "target_candidate",
            2,
        )]);
        let df = df!["target" => [0, 1, 0, 1]].unwrap();

        let mut choices = HashMap::new();
        choices.insert(
            "problem_type_selection_0".to_string(),
            "regression".to_string(),
        );

        let (problem_type, target) = engine.finalize_problem_setup(&profile, &choices, &df).unwrap();

        // Should use the choice from HashMap
        assert_eq!(problem_type, "regression");
        assert_eq!(target, "target");
    }

    #[test]
    fn test_finalize_problem_setup_defaults_from_profile() {
        let engine = RuleBasedDecisionEngine::new(PipelineConfig::default());
        let profile = create_profile(vec![create_column_profile(
            "survived",
            "binary",
            "target_candidate",
            2,
        )]);
        let df = df!["survived" => [0, 1, 0, 1]].unwrap();
        let choices = HashMap::new();

        let (problem_type, target) = engine.finalize_problem_setup(&profile, &choices, &df).unwrap();

        assert_eq!(problem_type, "classification");
        assert_eq!(target, "survived");
    }
}
