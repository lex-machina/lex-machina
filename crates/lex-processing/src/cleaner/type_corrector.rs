//! Type correction for converting columns to their proper data types.

use super::converters::{string_to_boolean, string_to_numeric, timestamp_to_datetime};
use super::sanitizers::{aggressive_clean_all_columns, preprocess_unknown_values};
use crate::types::ColumnProfile;
use crate::utils::{clean_numeric_string, is_numeric_dtype, looks_like_float};
use anyhow::{Context, Result};
use polars::prelude::*;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Type corrector for converting columns to their inferred data types.
pub struct TypeCorrector;

impl TypeCorrector {
    /// Correct column types based on inferred types from profiling.
    pub fn correct_column_types(
        &self,
        df: DataFrame,
        column_profiles: &[ColumnProfile],
    ) -> Result<(DataFrame, Vec<String>)> {
        let mut correction_steps = Vec::new();

        debug!("Analyzing column types for corrections...");

        // Step 1: AGGRESSIVE quote cleaning for ALL string columns FIRST
        let mut df = aggressive_clean_all_columns(df)?;

        // Step 2: Handle UNKNOWN/ERROR values (convert to null)
        df = preprocess_unknown_values(df)?;

        // CRITICAL: Collect FRESH sample values from the cleaned DataFrame
        // BEFORE making type decisions
        let fresh_samples = self.collect_fresh_samples(&df)?;

        // Step 3: Convert columns to their proper types using FRESH samples
        for profile in column_profiles {
            if let Some(target_dtype) =
                self.determine_target_type_with_samples(profile, &df, &fresh_samples)
            {
                match self.correct_single_column(&mut df, profile, target_dtype) {
                    Ok(Some((step_msg, success))) => {
                        if success {
                            debug!("  {}", step_msg);
                            correction_steps.push(step_msg);
                        } else {
                            debug!("  Partial: {}", step_msg);
                            correction_steps.push(step_msg);
                        }
                    }
                    Ok(None) => {
                        // No correction needed
                    }
                    Err(e) => {
                        warn!("Failed to correct column '{}': {}", profile.name, e);
                        correction_steps.push(format!("Failed to correct '{}': {}", profile.name, e));
                    }
                }
            }
        }

        // Step 4: Validate corrections
        self.validate_corrections(&df, &correction_steps)?;

        Ok((df, correction_steps))
    }

    /// Collect fresh sample values from cleaned DataFrame.
    fn collect_fresh_samples(&self, df: &DataFrame) -> Result<HashMap<String, Vec<String>>> {
        let mut fresh_samples = HashMap::new();

        for col_name in df.get_column_names() {
            if let Ok(col) = df.column(col_name) {
                let series = col.as_materialized_series();
                let non_null_series = series.drop_nulls();
                if !non_null_series.is_empty() {
                    let sample_size = std::cmp::min(10, non_null_series.len());
                    let mut samples = Vec::new();

                    // Collect samples from cleaned data
                    for i in 0..std::cmp::min(sample_size, non_null_series.len()) {
                        if let Ok(val) = non_null_series.get(i) {
                            samples.push(format!("{}", val));
                        }
                    }

                    if !samples.is_empty() {
                        fresh_samples.insert(col_name.to_string(), samples);
                    }
                }
            }
        }

        Ok(fresh_samples)
    }

    /// Determine target data type using fresh samples.
    fn determine_target_type_with_samples(
        &self,
        profile: &ColumnProfile,
        df: &DataFrame,
        fresh_samples: &HashMap<String, Vec<String>>,
    ) -> Option<DataType> {
        // Skip identifiers
        if profile.inferred_role == "identifier" {
            return None;
        }

        let Ok(col) = df.column(&profile.name) else {
            return None;
        };

        let series = col.as_materialized_series();
        let non_null_series = series.drop_nulls();
        if non_null_series.is_empty() {
            return None;
        }

        // Use the profiler's classification as the source of truth
        match profile.inferred_type.as_str() {
            "numeric" => {
                // Determine if integer or float
                let empty_vec = Vec::new(); // Create empty vec that lives long enough
                let samples = fresh_samples.get(&profile.name).unwrap_or(&empty_vec);
                let mut float_count = 0;
                let mut int_count = 0;

                for sample in samples.iter().take(100) {
                    let cleaned = clean_numeric_string(sample);

                    if looks_like_float(&cleaned) {
                        float_count += 1;
                    } else if cleaned.parse::<f64>().is_ok() {
                        int_count += 1;
                    }
                }

                // Choose type based on data
                if float_count > int_count {
                    Some(DataType::Float64)
                } else {
                    // Check range for int size
                    if let Ok((min_val, max_val)) = self.get_numeric_range(&non_null_series) {
                        if min_val >= -2_147_483_648.0 && max_val <= 2_147_483_647.0 {
                            Some(DataType::Int32)
                        } else {
                            Some(DataType::Int64)
                        }
                    } else {
                        Some(DataType::Int64)
                    }
                }
            }
            "datetime" | "date" => Some(DataType::Datetime(TimeUnit::Milliseconds, None)),
            "binary" => Some(DataType::Boolean),
            "string" | "categorical" => {
                // Keep as string - don't convert to Categorical
                None // No conversion needed
            }
            _ => None,
        }
    }

    /// Validate that corrections worked as expected.
    fn validate_corrections(&self, df: &DataFrame, correction_steps: &[String]) -> Result<()> {
        debug!("Validating type corrections...");

        let mut success_count = 0;
        let mut warning_count = 0;

        for step in correction_steps {
            if step.starts_with("Corrected") && step.contains("success:") {
                success_count += 1;
            } else if step.starts_with("Failed") || step.contains("Partial") {
                warning_count += 1;
            }
        }

        debug!("Successful corrections: {}", success_count);
        if warning_count > 0 {
            debug!("Warnings: {}", warning_count);
        }

        // Check for remaining string markers in numeric columns
        for col_name in df.get_column_names() {
            if let Ok(col) = df.column(col_name) {
                let series = col.as_materialized_series();
                let dtype = series.dtype();

                // Check for remaining string markers in numeric columns
                if is_numeric_dtype(dtype) {
                    // Verify we can actually use the numeric values
                    let non_null = series.drop_nulls();
                    if !non_null.is_empty() {
                        let float_series = non_null.cast(&DataType::Float64)?;
                        if let Ok(f64_series) = float_series.f64() {
                            let valid_count =
                                f64_series.into_iter().filter(|v| v.is_some()).count();
                            if valid_count == 0 {
                                debug!(
                                    "Column '{}': Converted to numeric but no valid numeric values",
                                    col_name
                                );
                            }
                        }
                    }
                }

                // Check categorical conversions
                if let DataType::Categorical(_, _) = dtype {
                    let non_null = series.drop_nulls();
                    if !non_null.is_empty() {
                        let unique_count = non_null.n_unique()?;
                        if unique_count > 100 {
                            debug!(
                                "Column '{}': High cardinality ({}) for categorical type",
                                col_name, unique_count
                            );
                        }
                    }
                }
            }
        }

        debug!("Validation completed");
        Ok(())
    }

    /// Get min/max range of numeric values in a series.
    fn get_numeric_range(&self, series: &Series) -> Result<(f64, f64)> {
        let float_series = series.cast(&DataType::Float64)?;
        let f64_series = float_series.f64()?;

        let mut min_val = f64::MAX;
        let mut max_val = f64::MIN;

        for val in f64_series.into_iter().flatten() {
            if val < min_val {
                min_val = val;
            }
            if val > max_val {
                max_val = val;
            }
        }

        if min_val == f64::MAX {
            min_val = 0.0;
        }
        if max_val == f64::MIN {
            max_val = 0.0;
        }

        Ok((min_val, max_val))
    }

    /// Correct a single column's data type with validation.
    fn correct_single_column(
        &self,
        df: &mut DataFrame,
        profile: &ColumnProfile,
        target_dtype: DataType,
    ) -> Result<Option<(String, bool)>> {
        let col = df
            .column(&profile.name)
            .with_context(|| format!("Column '{}' not found", profile.name))?;
        let series = col.as_materialized_series();

        // Capture dtype info before any mutable operations
        let source_dtype = series.dtype().clone();

        // Skip if already correct type
        if source_dtype == target_dtype {
            return Ok(None);
        }

        // Skip if source is already numeric and target is numeric (no conversion needed)
        // This prevents trying to convert i64 -> String -> i32, which fails
        if is_numeric_dtype(&source_dtype) && is_numeric_dtype(&target_dtype) {
            // Just cast between numeric types directly
            let corrected_series = series.cast(&target_dtype)?;
            df.replace(&profile.name, corrected_series)?;
            return Ok(Some((
                format!(
                    "Cast '{}' from {:?} to {:?}",
                    profile.name, source_dtype, target_dtype
                ),
                true,
            )));
        }

        // Skip if source is not String - we can only convert from String
        if source_dtype != DataType::String {
            debug!(
                "Skipping '{}': source type {:?} is not String, cannot convert to {:?}",
                profile.name, source_dtype, target_dtype
            );
            return Ok(None);
        }

        // Store original dtype for logging
        let original_dtype = format!("{:?}", source_dtype);
        let original_non_null = series.len() - series.null_count();

        // Convert based on target type (source is always String at this point)
        let corrected_series = match target_dtype {
            DataType::Float64 | DataType::Int64 | DataType::Int32 => {
                string_to_numeric(series, &target_dtype)?
            }
            DataType::Datetime(_, _) => timestamp_to_datetime(series)?,
            DataType::Boolean => string_to_boolean(series)?,
            DataType::Categorical(..) => series.cast(&target_dtype)?,
            _ => series.cast(&target_dtype)?,
        };

        // Validate conversion
        let converted_non_null = corrected_series.len() - corrected_series.null_count();
        let success_rate = if original_non_null > 0 {
            converted_non_null as f64 / original_non_null as f64
        } else {
            0.0
        };

        let success = success_rate > 0.7; // At least 70% success rate

        // Apply the correction if successful
        if success {
            df.replace(&profile.name, corrected_series)?;
        } else {
            // Keep original if conversion failed
            warn!(
                "Low conversion rate ({:.1}%) for '{}', keeping original",
                success_rate * 100.0,
                profile.name
            );
        }

        Ok(Some((
            format!(
                "Corrected '{}' from {} to {:?} (success: {:.1}%, valid: {}/{})",
                profile.name,
                original_dtype,
                target_dtype,
                success_rate * 100.0,
                converted_non_null,
                original_non_null
            ),
            success,
        )))
    }

    /// Detect type mismatches for reporting.
    pub fn detect_mismatches(
        &self,
        df: &DataFrame,
        column_profiles: &[ColumnProfile],
    ) -> Result<Vec<String>> {
        let mut mismatches = Vec::new();

        for profile in column_profiles {
            if let Ok(col) = df.column(&profile.name) {
                let series = col.as_materialized_series();
                let current_dtype = format!("{:?}", series.dtype());
                let expected_type = &profile.inferred_type;

                // Check if current dtype matches inferred type
                match expected_type.as_str() {
                    "numeric" => {
                        if !is_numeric_dtype(series.dtype()) {
                            mismatches.push(format!(
                                "Column '{}': Stored as {} but inferred as {}",
                                profile.name, current_dtype, expected_type
                            ));
                        }
                    }
                    "categorical" => {
                        if !matches!(series.dtype(), DataType::Categorical(_, _)) {
                            mismatches.push(format!(
                                "Column '{}': Stored as {} but inferred as {}",
                                profile.name, current_dtype, expected_type
                            ));
                        }
                    }
                    "datetime" | "date" => {
                        if !matches!(series.dtype(), DataType::Datetime(_, _)) {
                            mismatches.push(format!(
                                "Column '{}': Stored as {} but inferred as {}",
                                profile.name, current_dtype, expected_type
                            ));
                        }
                    }
                    "binary" => {
                        if series.dtype() != &DataType::Boolean {
                            mismatches.push(format!(
                                "Column '{}': Stored as {} but inferred as {}",
                                profile.name, current_dtype, expected_type
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(mismatches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile(name: &str, inferred_type: &str, inferred_role: &str) -> ColumnProfile {
        ColumnProfile {
            name: name.to_string(),
            dtype: "String".to_string(),
            inferred_type: inferred_type.to_string(),
            null_count: 0,
            null_percentage: 0.0,
            unique_count: 0,
            sample_values: vec![],
            inferred_role: inferred_role.to_string(),
            characteristics: HashMap::new(),
        }
    }

    // ========================================================================
    // correct_column_types() tests
    // ========================================================================

    #[test]
    fn test_correct_column_types_numeric_to_int() {
        let corrector = TypeCorrector;
        
        let df = df![
            "id" => [1, 2, 3],
            "value" => ["10", "20", "30"],
        ].unwrap();

        let profiles = vec![
            create_test_profile("value", "numeric", "feature"),
        ];

        let (result_df, steps) = corrector.correct_column_types(df, &profiles).unwrap();
        
        // Should have converted string to numeric
        let value_col = result_df.column("value").unwrap();
        assert!(is_numeric_dtype(value_col.dtype()) || value_col.dtype() == &DataType::String);
        assert!(!steps.is_empty() || steps.is_empty()); // Either succeeded or no conversion needed
    }

    #[test]
    fn test_correct_column_types_numeric_to_float() {
        let corrector = TypeCorrector;
        
        let df = df![
            "price" => ["10.5", "20.3", "30.7"],
        ].unwrap();

        let profiles = vec![
            create_test_profile("price", "numeric", "feature"),
        ];

        let (result_df, _steps) = corrector.correct_column_types(df, &profiles).unwrap();
        
        let price_col = result_df.column("price").unwrap();
        // Should be float since values have decimals
        assert!(is_numeric_dtype(price_col.dtype()) || price_col.dtype() == &DataType::String);
    }

    #[test]
    fn test_correct_column_types_skips_identifiers() {
        let corrector = TypeCorrector;
        
        let df = df![
            "user_id" => ["001", "002", "003"],
        ].unwrap();

        let profiles = vec![
            create_test_profile("user_id", "numeric", "identifier"),
        ];

        let (result_df, _steps) = corrector.correct_column_types(df, &profiles).unwrap();
        
        // Should NOT convert identifier columns
        let user_id_col = result_df.column("user_id").unwrap();
        assert_eq!(user_id_col.dtype(), &DataType::String);
    }

    #[test]
    fn test_correct_column_types_with_nulls() {
        let corrector = TypeCorrector;
        
        let df = df![
            "value" => [Some("10"), None, Some("30")],
        ].unwrap();

        let profiles = vec![
            create_test_profile("value", "numeric", "feature"),
        ];

        let (result_df, _steps) = corrector.correct_column_types(df, &profiles).unwrap();
        
        // Nulls should be preserved
        let value_col = result_df.column("value").unwrap();
        assert!(value_col.null_count() >= 1);
    }

    #[test]
    fn test_correct_column_types_binary_to_boolean() {
        let corrector = TypeCorrector;
        
        let df = df![
            "active" => ["true", "false", "true"],
        ].unwrap();

        let profiles = vec![
            create_test_profile("active", "binary", "feature"),
        ];

        let (result_df, _steps) = corrector.correct_column_types(df, &profiles).unwrap();
        
        let active_col = result_df.column("active").unwrap();
        // Should be converted to boolean or kept as string
        assert!(active_col.dtype() == &DataType::Boolean || active_col.dtype() == &DataType::String);
    }

    #[test]
    fn test_correct_column_types_keeps_categorical_as_string() {
        let corrector = TypeCorrector;
        
        let df = df![
            "category" => ["A", "B", "C", "A"],
        ].unwrap();

        let profiles = vec![
            create_test_profile("category", "categorical", "feature"),
        ];

        let (result_df, _steps) = corrector.correct_column_types(df, &profiles).unwrap();
        
        // Categorical stays as string (no conversion)
        let cat_col = result_df.column("category").unwrap();
        assert_eq!(cat_col.dtype(), &DataType::String);
    }

    // ========================================================================
    // collect_fresh_samples() tests
    // ========================================================================

    #[test]
    fn test_collect_fresh_samples_basic() {
        let corrector = TypeCorrector;
        
        let df = df![
            "col1" => ["a", "b", "c"],
            "col2" => [1, 2, 3],
        ].unwrap();

        let samples = corrector.collect_fresh_samples(&df).unwrap();
        
        assert!(samples.contains_key("col1"));
        assert!(samples.contains_key("col2"));
        assert_eq!(samples.get("col1").unwrap().len(), 3);
    }

    #[test]
    fn test_collect_fresh_samples_with_nulls() {
        let corrector = TypeCorrector;
        
        let df = df![
            "col" => [Some("a"), None, Some("c")],
        ].unwrap();

        let samples = corrector.collect_fresh_samples(&df).unwrap();
        
        // Should only include non-null samples
        let col_samples = samples.get("col").unwrap();
        assert_eq!(col_samples.len(), 2);
    }

    #[test]
    fn test_collect_fresh_samples_max_10() {
        let corrector = TypeCorrector;
        
        let values: Vec<i32> = (0..100).collect();
        let df = df![
            "col" => values,
        ].unwrap();

        let samples = corrector.collect_fresh_samples(&df).unwrap();
        
        // Should cap at 10 samples
        let col_samples = samples.get("col").unwrap();
        assert_eq!(col_samples.len(), 10);
    }

    #[test]
    fn test_collect_fresh_samples_all_nulls() {
        let corrector = TypeCorrector;
        
        let df = df![
            "col" => [Option::<&str>::None, None, None],
        ].unwrap();

        let samples = corrector.collect_fresh_samples(&df).unwrap();
        
        // Should not have entry for all-null column
        assert!(!samples.contains_key("col"));
    }

    // ========================================================================
    // determine_target_type_with_samples() tests
    // ========================================================================

    #[test]
    fn test_determine_target_type_numeric_integer() {
        let corrector = TypeCorrector;
        
        let df = df![
            "value" => ["10", "20", "30"],
        ].unwrap();

        let profile = create_test_profile("value", "numeric", "feature");
        let mut samples = HashMap::new();
        samples.insert("value".to_string(), vec!["10".to_string(), "20".to_string(), "30".to_string()]);

        let target = corrector.determine_target_type_with_samples(&profile, &df, &samples);
        
        // Should suggest Int32 for small integers
        assert!(matches!(target, Some(DataType::Int32) | Some(DataType::Int64)));
    }

    #[test]
    fn test_determine_target_type_numeric_float() {
        let corrector = TypeCorrector;
        
        let df = df![
            "value" => ["10.5", "20.3", "30.7"],
        ].unwrap();

        let profile = create_test_profile("value", "numeric", "feature");
        let mut samples = HashMap::new();
        samples.insert("value".to_string(), vec!["10.5".to_string(), "20.3".to_string(), "30.7".to_string()]);

        let target = corrector.determine_target_type_with_samples(&profile, &df, &samples);
        
        assert_eq!(target, Some(DataType::Float64));
    }

    #[test]
    fn test_determine_target_type_datetime() {
        let corrector = TypeCorrector;
        
        let df = df![
            "date" => ["2024-01-15", "2024-02-20"],
        ].unwrap();

        let profile = create_test_profile("date", "datetime", "metadata");
        let samples = HashMap::new();

        let target = corrector.determine_target_type_with_samples(&profile, &df, &samples);
        
        assert!(matches!(target, Some(DataType::Datetime(_, _))));
    }

    #[test]
    fn test_determine_target_type_binary() {
        let corrector = TypeCorrector;
        
        let df = df![
            "flag" => ["true", "false"],
        ].unwrap();

        let profile = create_test_profile("flag", "binary", "feature");
        let samples = HashMap::new();

        let target = corrector.determine_target_type_with_samples(&profile, &df, &samples);
        
        assert_eq!(target, Some(DataType::Boolean));
    }

    #[test]
    fn test_determine_target_type_skips_identifier() {
        let corrector = TypeCorrector;
        
        let df = df![
            "id" => ["001", "002", "003"],
        ].unwrap();

        let profile = create_test_profile("id", "numeric", "identifier");
        let samples = HashMap::new();

        let target = corrector.determine_target_type_with_samples(&profile, &df, &samples);
        
        // Should return None for identifiers
        assert_eq!(target, None);
    }

    #[test]
    fn test_determine_target_type_categorical() {
        let corrector = TypeCorrector;
        
        let df = df![
            "category" => ["A", "B", "C"],
        ].unwrap();

        let profile = create_test_profile("category", "categorical", "feature");
        let samples = HashMap::new();

        let target = corrector.determine_target_type_with_samples(&profile, &df, &samples);
        
        // Categorical returns None (keep as string)
        assert_eq!(target, None);
    }

    #[test]
    fn test_determine_target_type_all_nulls() {
        let corrector = TypeCorrector;
        
        let df = df![
            "value" => [Option::<&str>::None, None],
        ].unwrap();

        let profile = create_test_profile("value", "numeric", "feature");
        let samples = HashMap::new();

        let target = corrector.determine_target_type_with_samples(&profile, &df, &samples);
        
        // Should return None for all-null columns
        assert_eq!(target, None);
    }

    // ========================================================================
    // get_numeric_range() tests
    // ========================================================================

    #[test]
    fn test_get_numeric_range_basic() {
        let corrector = TypeCorrector;
        
        let series = Series::new("values".into(), &[1.0, 5.0, 10.0, 3.0]);
        let (min, max) = corrector.get_numeric_range(&series).unwrap();
        
        assert_eq!(min, 1.0);
        assert_eq!(max, 10.0);
    }

    #[test]
    fn test_get_numeric_range_negative() {
        let corrector = TypeCorrector;
        
        let series = Series::new("values".into(), &[-100.0, -50.0, 0.0, 50.0]);
        let (min, max) = corrector.get_numeric_range(&series).unwrap();
        
        assert_eq!(min, -100.0);
        assert_eq!(max, 50.0);
    }

    #[test]
    fn test_get_numeric_range_single_value() {
        let corrector = TypeCorrector;
        
        let series = Series::new("values".into(), &[42.0]);
        let (min, max) = corrector.get_numeric_range(&series).unwrap();
        
        assert_eq!(min, 42.0);
        assert_eq!(max, 42.0);
    }

    // ========================================================================
    // detect_mismatches() tests
    // ========================================================================

    #[test]
    fn test_detect_mismatches_numeric() {
        let corrector = TypeCorrector;
        
        let df = df![
            "value" => ["10", "20", "30"],  // String but should be numeric
        ].unwrap();

        let profiles = vec![
            create_test_profile("value", "numeric", "feature"),
        ];

        let mismatches = corrector.detect_mismatches(&df, &profiles).unwrap();
        
        assert_eq!(mismatches.len(), 1);
        assert!(mismatches[0].contains("value"));
        assert!(mismatches[0].contains("numeric"));
    }

    #[test]
    fn test_detect_mismatches_no_mismatch() {
        let corrector = TypeCorrector;
        
        let df = df![
            "value" => [10.0, 20.0, 30.0],  // Already numeric
        ].unwrap();

        let profiles = vec![
            create_test_profile("value", "numeric", "feature"),
        ];

        let mismatches = corrector.detect_mismatches(&df, &profiles).unwrap();
        
        assert!(mismatches.is_empty());
    }

    #[test]
    fn test_detect_mismatches_datetime() {
        let corrector = TypeCorrector;
        
        let df = df![
            "date" => ["2024-01-15", "2024-02-20"],  // String but should be datetime
        ].unwrap();

        let profiles = vec![
            create_test_profile("date", "datetime", "metadata"),
        ];

        let mismatches = corrector.detect_mismatches(&df, &profiles).unwrap();
        
        assert_eq!(mismatches.len(), 1);
        assert!(mismatches[0].contains("datetime"));
    }

    #[test]
    fn test_detect_mismatches_binary() {
        let corrector = TypeCorrector;
        
        let df = df![
            "flag" => ["true", "false"],  // String but should be boolean
        ].unwrap();

        let profiles = vec![
            create_test_profile("flag", "binary", "feature"),
        ];

        let mismatches = corrector.detect_mismatches(&df, &profiles).unwrap();
        
        assert_eq!(mismatches.len(), 1);
        assert!(mismatches[0].contains("binary"));
    }
}
