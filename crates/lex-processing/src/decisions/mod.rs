//! Decision-making module for preprocessing strategy selection.
//!
//! This module provides both AI-powered and rule-based decision engines
//! for selecting appropriate preprocessing strategies.

mod ai_engine;
mod rule_engine;

pub use ai_engine::AiDecisionEngine;
pub use rule_engine::RuleBasedDecisionEngine;

use crate::types::{DataQualityIssue, DatasetProfile};
use anyhow::Result;
use polars::prelude::*;
use std::collections::HashMap;

/// Trait for decision-making engines.
///
/// Implementations can use AI, rule-based heuristics, or other strategies
/// to make preprocessing decisions.
pub trait DecisionEngine: Send + Sync {
    /// Make decisions for all data quality issues.
    fn make_decisions(
        &self,
        issues: &[DataQualityIssue],
        df: &DataFrame,
    ) -> Result<HashMap<String, String>>;

    /// Finalize problem setup (problem type and target column).
    fn finalize_problem_setup(
        &self,
        profile: &DatasetProfile,
        choices: &HashMap<String, String>,
        df: &DataFrame,
    ) -> Result<(String, String)>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PipelineConfig;
    use crate::types::ColumnProfile;

    #[test]
    fn test_rule_based_engine_creation() {
        let config = PipelineConfig::default();
        let _engine = RuleBasedDecisionEngine::new(config);
        // Engine created successfully - no panic
    }

    #[test]
    fn test_determine_problem_type_classification() {
        let config = PipelineConfig::default();
        let engine = RuleBasedDecisionEngine::new(config);

        let profile = DatasetProfile {
            shape: (100, 5),
            column_profiles: vec![ColumnProfile {
                name: "target".to_string(),
                dtype: "Int64".to_string(),
                unique_count: 2,
                null_count: 0,
                null_percentage: 0.0,
                sample_values: vec!["0".to_string(), "1".to_string()],
                inferred_type: "binary".to_string(),
                inferred_role: "target_candidate".to_string(),
                characteristics: HashMap::new(),
            }],
            target_candidates: vec!["target".to_string()],
            problem_type_candidates: vec![],
            complexity_indicators: HashMap::new(),
            duplicate_count: 0,
            duplicate_percentage: 0.0,
        };

        let problem_type = engine.determine_problem_type(&profile);
        assert_eq!(problem_type, "classification");
    }
}
