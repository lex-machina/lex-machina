//! Data profiling module for dataset analysis.
//!
//! This module provides functionality for profiling datasets, including:
//! - Type inference for columns
//! - Role detection (identifier, target, feature)
//! - Statistical analysis
//! - Complexity assessment

mod role_inference;
mod statistics;
mod type_inference;

use crate::types::{ColumnProfile, DatasetProfile};
use anyhow::Result;
use polars::prelude::*;
use rand::prelude::*;
use std::collections::HashMap;

// Re-export for internal use
pub(crate) use role_inference::infer_column_role;
pub(crate) use statistics::extract_column_characteristics;
pub(crate) use type_inference::infer_column_type_advanced;

/// Data profiler for analyzing dataset structure and characteristics.
pub struct DataProfiler;

impl DataProfiler {
    /// Profile an entire dataset to understand its structure.
    ///
    /// This function analyzes each column, detects duplicates, infers problem types,
    /// and assesses overall complexity.
    pub fn profile_dataset(df: &DataFrame) -> Result<DatasetProfile> {
        let mut column_profiles = Vec::new();
        let mut target_candidates = Vec::new();

        // Profile each column
        for col_name in df.get_column_names() {
            let profile = Self::profile_column(df, col_name)?;
            if profile.inferred_role == "target_candidate" {
                target_candidates.push(col_name.to_string());
            }
            column_profiles.push(profile);
        }

        // Detect duplicates
        let duplicate_count = df.height()
            - df.unique::<&str, &str>(None, UniqueKeepStrategy::First, None)?
                .height();
        let duplicate_percentage = if df.height() > 0 {
            (duplicate_count as f64 / df.height() as f64) * 100.0
        } else {
            0.0
        };

        // Infer problem types
        let problem_type_candidates = Self::infer_problem_types(&column_profiles);

        // Complexity analysis
        let complexity_indicators = Self::analyze_complexity(df, &column_profiles);

        Ok(DatasetProfile {
            shape: (df.height(), df.width()),
            column_profiles,
            target_candidates,
            problem_type_candidates,
            complexity_indicators,
            duplicate_count,
            duplicate_percentage,
        })
    }

    fn profile_column(df: &DataFrame, col_name: &str) -> Result<ColumnProfile> {
        let col = df.column(col_name)?;
        let series = col.as_materialized_series();
        let dtype = format!("{:?}", series.dtype());
        let unique_count = series.n_unique()?;
        let null_count = series.null_count();
        let null_percentage = (null_count as f64 / df.height() as f64) * 100.0;

        // Get sample values - IMPORTANT: Get fresh samples after any transformations
        let mut sample_values = Vec::new();
        let non_null_series = series.drop_nulls();
        if !non_null_series.is_empty() {
            let sample_size = std::cmp::min(10, non_null_series.len());
            let mut rng = StdRng::seed_from_u64(42);
            let indices: Vec<usize> = (0..non_null_series.len()).collect();
            let sampled_indices: Vec<usize> = indices
                .choose_multiple(&mut rng, sample_size)
                .copied()
                .collect();

            for idx in sampled_indices {
                if let Ok(val) = non_null_series.get(idx) {
                    // Convert to string for analysis
                    sample_values.push(format!("{}", val));
                }
            }
        }

        // Infer column type using advanced detection
        let inferred_type = infer_column_type_advanced(series, &sample_values, col_name)?;

        // Infer column role - pass total_rows parameter
        let inferred_role = infer_column_role(
            col_name,
            &inferred_type,
            unique_count,
            df.height(), // Add total rows
        );

        // Extract characteristics
        let characteristics = extract_column_characteristics(series, &inferred_type, unique_count)?;

        Ok(ColumnProfile {
            name: col_name.to_string(),
            dtype,
            unique_count,
            null_count,
            null_percentage,
            sample_values,
            inferred_type,
            inferred_role,
            characteristics,
        })
    }

    fn infer_problem_types(column_profiles: &[ColumnProfile]) -> Vec<String> {
        let mut problem_types = Vec::new();

        let classification_candidates: Vec<_> = column_profiles
            .iter()
            .filter(|col| {
                col.inferred_role == "target_candidate"
                    && (col.inferred_type == "binary"
                        || col.inferred_type == "string"
                        || col.inferred_type == "categorical"
                        || col.unique_count <= 10)
            })
            .collect();

        if !classification_candidates.is_empty() {
            problem_types.push("classification".to_string());
        }

        let regression_candidates: Vec<_> = column_profiles
            .iter()
            .filter(|col| {
                col.inferred_role == "target_candidate"
                    && col.inferred_type == "numeric"
                    && col.unique_count > 10
            })
            .collect();

        if !regression_candidates.is_empty() {
            problem_types.push("regression".to_string());
        }

        let datetime_cols: Vec<_> = column_profiles
            .iter()
            .filter(|col| col.inferred_type == "datetime")
            .collect();

        if !datetime_cols.is_empty() {
            problem_types.push("time_series".to_string());
        }

        if problem_types.is_empty() {
            problem_types.push("clustering".to_string());
        }

        problem_types
    }

    fn analyze_complexity(
        df: &DataFrame,
        column_profiles: &[ColumnProfile],
    ) -> HashMap<String, serde_json::Value> {
        let mut complexity = HashMap::new();

        let n_rows = df.height();
        let n_features = column_profiles
            .iter()
            .filter(|col| col.inferred_role == "feature")
            .count();

        let size_category = match n_rows {
            0..=9_999 => "small",
            10_000..=99_999 => "medium",
            100_000..=999_999 => "large",
            _ => "very_large",
        };

        let feature_complexity = match n_features {
            0..=20 => "low",
            21..=200 => "medium",
            201..=1000 => "high",
            _ => "very_high",
        };

        let type_set: std::collections::HashSet<_> = column_profiles
            .iter()
            .map(|col| &col.inferred_type)
            .collect();
        let mixed_types = type_set.len() > 3;

        let high_cardinality_features = column_profiles
            .iter()
            .filter(|col| {
                col.characteristics
                    .get("cardinality")
                    .and_then(|v| v.as_str())
                    == Some("high")
            })
            .count();

        let avg_null_pct: f64 = column_profiles
            .iter()
            .map(|col| col.null_percentage)
            .sum::<f64>()
            / column_profiles.len() as f64;

        let missing_data_complexity = if avg_null_pct > 20.0 { "high" } else { "low" };

        complexity.insert(
            "size_category".to_string(),
            serde_json::json!(size_category),
        );
        complexity.insert("feature_count".to_string(), serde_json::json!(n_features));
        complexity.insert(
            "feature_complexity".to_string(),
            serde_json::json!(feature_complexity),
        );
        complexity.insert("mixed_types".to_string(), serde_json::json!(mixed_types));
        complexity.insert(
            "high_cardinality_features".to_string(),
            serde_json::json!(high_cardinality_features),
        );
        complexity.insert(
            "missing_data_complexity".to_string(),
            serde_json::json!(missing_data_complexity),
        );

        complexity
    }
}
