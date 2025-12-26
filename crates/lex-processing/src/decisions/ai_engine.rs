//! AI-powered decision engine using an LLM via API.

use super::DecisionEngine;
use crate::ai::AIProvider;
use crate::config::PipelineConfig;
use crate::types::{DataQualityIssue, DatasetProfile, DecisionQuestion, SolutionOption};
use anyhow::Result;
use polars::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// AI-powered decision maker using an LLM via API.
///
/// This decision engine uses any [`AIProvider`] implementation to make
/// intelligent preprocessing decisions based on the data characteristics.
///
/// # Example
///
/// ```rust,ignore
/// use lex_processing::ai::OpenRouterProvider;
/// use lex_processing::decisions::AiDecisionEngine;
/// use lex_processing::config::PipelineConfig;
///
/// let provider = OpenRouterProvider::new("api-key")?;
/// let engine = AiDecisionEngine::new(&provider, PipelineConfig::default());
/// ```
pub struct AiDecisionEngine<'a> {
    ai_provider: &'a dyn AIProvider,
    config: PipelineConfig,
}

impl<'a> AiDecisionEngine<'a> {
    /// Create a new AI decision engine with the given provider and config.
    pub fn new(ai_provider: &'a dyn AIProvider, config: PipelineConfig) -> Self {
        Self {
            ai_provider,
            config,
        }
    }

    fn get_sample_data_for_issue(
        &self,
        df: &DataFrame,
        issue: &DataQualityIssue,
    ) -> Result<String> {
        let sample_size = std::cmp::min(5, df.height());

        let sample_df = if !issue.affected_columns.is_empty() {
            let sample_cols: Vec<&str> = issue
                .affected_columns
                .iter()
                .take(5)
                .map(|s| s.as_str())
                .collect();
            df.select(sample_cols)?.head(Some(sample_size))
        } else {
            df.head(Some(sample_size))
        };

        Ok(format!(
            "Sample data (first {} rows):\n{}",
            sample_size, sample_df
        ))
    }

    fn get_validated_problem_type(&self, ai_choices: &HashMap<String, String>) -> String {
        let mut problem_type = None;

        for (choice_id, choice) in ai_choices {
            if choice_id.contains("problem_type_selection") {
                problem_type = Some(choice.clone());
                break;
            }
        }

        let valid_problem_types = ["classification", "regression"];
        if let Some(pt) = problem_type
            && valid_problem_types.contains(&pt.as_str())
        {
            return pt;
        }

        warn!("AI chose invalid problem type, defaulting to classification");
        "classification".to_string()
    }

    fn get_validated_target_column(
        &self,
        profile: &DatasetProfile,
        problem_type: &str,
        df: &DataFrame,
    ) -> Result<String> {
        if !["classification", "regression"].contains(&problem_type) {
            return Ok(String::new());
        }

        // Priority 0: If target column is explicitly specified in config, use it
        if let Some(ref target) = self.config.target_column {
            // Validate that the column exists
            let column_exists = profile
                .column_profiles
                .iter()
                .any(|col| &col.name == target);

            if column_exists {
                info!("Using explicitly specified target column: {}", target);
                return Ok(target.clone());
            } else {
                warn!(
                    "Specified target column '{}' not found in dataset, falling back to AI selection",
                    target
                );
            }
        }

        // Get ALL available columns (excluding identifiers and metadata)
        let all_available_cols: Vec<String> = profile
            .column_profiles
            .iter()
            .filter(|col| !["identifier", "metadata"].contains(&col.inferred_role.as_str()))
            .map(|col| col.name.clone())
            .collect();

        if all_available_cols.is_empty() {
            warn!("No available columns found for target selection");
            return Ok(String::new());
        }

        // Create sample data preview
        let sample_size = std::cmp::min(8, df.height());
        let sample_cols: Vec<&str> = all_available_cols.iter().map(|s| s.as_str()).collect();
        let sample_df = df.select(sample_cols)?.head(Some(sample_size));
        let sample_data_str = format!(
            "Dataset sample (first {} rows):\n{}",
            sample_size, sample_df
        );

        // Create simple, unbiased options
        let mut options = Vec::new();

        for col_name in &all_available_cols {
            let Some(col_profile) = profile.column_profiles.iter().find(|c| &c.name == col_name)
            else {
                continue;
            };

            // Get column-specific sample values
            let col_sample: Vec<String> = df
                .column(col_name)?
                .as_materialized_series()
                .drop_nulls()
                .head(Some(3))
                .iter()
                .map(|v| format!("{}", v))
                .collect();

            let sample_values = if !col_sample.is_empty() {
                format!("Sample: {:?}", col_sample)
            } else {
                "No sample values".to_string()
            };

            let description = format!(
                "Column: {} | Type: {} | Unique: {} | Missing: {:.1}% | {}",
                col_name,
                col_profile.inferred_type,
                col_profile.unique_count,
                col_profile.null_percentage,
                sample_values
            );

            options.push(SolutionOption {
                option: col_name.clone(),
                description,
                pros: None,
                cons: None,
                best_for: None,
            });
        }

        let mut detection_details = HashMap::new();
        detection_details.insert("problem_type".to_string(), serde_json::json!(problem_type));
        detection_details.insert(
            "all_columns".to_string(),
            serde_json::json!(all_available_cols),
        );
        detection_details.insert(
            "dataset_sample".to_string(),
            serde_json::json!(sample_data_str),
        );

        let target_question = DecisionQuestion {
            id: "target_column_selection".to_string(),
            issue_type: "target_column_selection".to_string(),
            description: format!(
                "Select the MOST APPROPRIATE target column for {} prediction. \
                Analyze the sample data below and choose the column that represents \
                what you want to predict based on the other features.\n\n{}",
                problem_type, sample_data_str
            ),
            business_impact: "This choice determines the prediction task and model performance"
                .to_string(),
            detection_details,
            affected_columns: all_available_cols.clone(),
            options,
            sample_data: sample_data_str,
        };

        // Get AI choice
        let target_column = self
            .ai_provider
            .make_preprocessing_decision(&target_question)?;

        // Validate AI choice
        if !all_available_cols.contains(&target_column) {
            warn!(
                "AI chose invalid target '{}', selecting first available column",
                target_column
            );
            return Ok(all_available_cols[0].clone());
        }

        info!("AI selected target: {}", target_column);
        Ok(target_column)
    }
}

impl<'a> DecisionEngine for AiDecisionEngine<'a> {
    fn make_decisions(
        &self,
        issues: &[DataQualityIssue],
        df: &DataFrame,
    ) -> Result<HashMap<String, String>> {
        let mut ai_choices = HashMap::new();

        info!("Making AI-driven decisions for data quality issues...");

        for (i, issue) in issues.iter().enumerate() {
            let question = DecisionQuestion {
                id: format!("{}_{}", issue.issue_type, i),
                issue_type: issue.issue_type.clone(),
                description: issue.description.clone(),
                business_impact: issue.business_impact.clone(),
                detection_details: issue.detection_details.clone(),
                affected_columns: issue.affected_columns.clone(),
                options: issue.suggested_solutions.clone(),
                sample_data: self.get_sample_data_for_issue(df, issue)?,
            };

            let ai_choice = self.ai_provider.make_preprocessing_decision(&question)?;
            debug!("AI Decision for {}: {}", issue.issue_type, ai_choice);
            ai_choices.insert(question.id, ai_choice);
        }

        Ok(ai_choices)
    }

    fn finalize_problem_setup(
        &self,
        profile: &DatasetProfile,
        ai_choices: &HashMap<String, String>,
        df: &DataFrame,
    ) -> Result<(String, String)> {
        let problem_type = self.get_validated_problem_type(ai_choices);
        let target_column = self.get_validated_target_column(profile, &problem_type, df)?;

        info!("Problem setup finalized:");
        info!("  Problem type: {}", problem_type);
        info!("  Target column: {}", target_column);

        Ok((problem_type, target_column))
    }
}
