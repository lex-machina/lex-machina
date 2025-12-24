//! Statistical imputation methods.
//!
//! Provides mean, median, mode, and other statistical imputation strategies.

use crate::types::ColumnProfile;
use crate::utils::{fill_string_nulls, string_mode};
use anyhow::Result;
use polars::prelude::*;

/// Statistical imputation methods for filling missing values.
pub struct StatisticalImputer;

impl StatisticalImputer {
    /// Apply median imputation for numeric columns.
    pub fn apply_numeric_median(
        df: &mut DataFrame,
        col_name: &str,
        processing_steps: &mut Vec<String>,
    ) -> Result<()> {
        if let Ok(col) = df.column(col_name) {
            let series = col.as_materialized_series();
            if let Some(median_val) = series.median() {
                let series_clone = series.clone();
                Self::fill_with_value(
                    df, 
                    col_name, 
                    median_val, 
                    &series_clone, 
                    processing_steps, 
                    "median"
                )?;
            }
        }
        Ok(())
    }

    /// Apply mean imputation for numeric columns.
    pub fn apply_numeric_mean(
        df: &mut DataFrame,
        col_name: &str,
        processing_steps: &mut Vec<String>,
    ) -> Result<()> {
        if let Ok(col) = df.column(col_name) {
            let series = col.as_materialized_series();
            if let Some(mean_val) = series.mean() {
                let series_clone = series.clone();
                Self::fill_with_value(
                    df, 
                    col_name, 
                    mean_val, 
                    &series_clone, 
                    processing_steps, 
                    "mean"
                )?;
            }
        }
        Ok(())
    }

    /// Apply mode imputation for categorical columns.
    pub fn apply_mode_imputation(
        df: &mut DataFrame,
        col_profile: &ColumnProfile,
        processing_steps: &mut Vec<String>,
    ) -> Result<()> {
        let col = &col_profile.name;
        
        if let Ok(column) = df.column(col) {
            let series = column.as_materialized_series();
            if let Some(mode_val) = string_mode(series) {
                let filled = fill_string_nulls(series, &mode_val)?;
                df.replace(col, filled)?;
                
                processing_steps.push(format!(
                    "Filled '{}' with mode: '{}'",
                    col, mode_val
                ));
            }
        }
        
        Ok(())
    }

    /// Apply category indicator (add "Missing" as new category).
    pub fn apply_category_indicator(
        df: &mut DataFrame,
        col_profile: &ColumnProfile,
        processing_steps: &mut Vec<String>,
    ) -> Result<()> {
        let col = &col_profile.name;
        
        if let Ok(column) = df.column(col) {
            let series = column.as_materialized_series();
            let filled = fill_string_nulls(series, "Missing")?;
            df.replace(col, filled)?;
            
            processing_steps.push(format!(
                "Added 'Missing' category indicator to '{}'",
                col
            ));
        }
        
        Ok(())
    }

    /// Apply constant imputation.
    pub fn apply_constant_imputation(
        df: &mut DataFrame,
        col_profile: &ColumnProfile,
        processing_steps: &mut Vec<String>,
    ) -> Result<()> {
        let col = &col_profile.name;
        
        if let Ok(column) = df.column(col) {
            let series = column.as_materialized_series();
            let filled = fill_string_nulls(series, "Unknown")?;
            df.replace(col, filled)?;
            
            processing_steps.push(format!(
                "Filled '{}' with constant value: 'Unknown'",
                col
            ));
        }
        
        Ok(())
    }

    /// Apply fallback imputation based on data type.
    pub fn apply_fallback_imputation(
        df: &mut DataFrame,
        col_profile: &ColumnProfile,
        processing_steps: &mut Vec<String>,
    ) -> Result<()> {
        let col = &col_profile.name;
        
        match col_profile.inferred_type.as_str() {
            "numeric" => {
                // Get median value and clone series to break borrow
                let (median_val, series_clone) = if let Ok(column) = df.column(col) {
                    let series = column.as_materialized_series();
                    (series.median(), series.clone())
                } else {
                    (None, Series::new_empty(PlSmallStr::EMPTY, &DataType::Float64))
                };
                
                if let Some(median_val) = median_val {
                    Self::fill_with_value(df, col, median_val, &series_clone, 
                                         processing_steps, "median (fallback)")?;
                }
            }
            "categorical" | "binary" => {
                Self::apply_mode_imputation(df, col_profile, processing_steps)?;
            }
            "datetime" | "date" => {
                if let Ok(column) = df.column(col) {
                    let series = column.as_materialized_series();
                    let filled = series.fill_null(FillNullStrategy::Forward(None))?;
                    let filled = filled.fill_null(FillNullStrategy::Backward(None))?;
                    df.replace(col, filled)?;
                    
                    processing_steps.push(format!(
                        "Forward fill '{}' (fallback)",
                        col
                    ));
                }
            }
            _ => {
                if let Ok(column) = df.column(col) {
                    let series = column.as_materialized_series();
                    let filled = fill_string_nulls(series, "Unknown")?;
                    df.replace(col, filled)?;
                    
                    processing_steps.push(format!(
                        "Filled '{}' with 'Unknown' (fallback)",
                        col
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Fill numeric column with a specific value.
    fn fill_with_value(
        df: &mut DataFrame,
        col_name: &str,
        fill_value: f64,
        series: &Series,
        processing_steps: &mut Vec<String>,
        method: &str,
    ) -> Result<()> {
        let mask = series.is_null();
        let mut result_vec = Vec::with_capacity(series.len());
        
        for i in 0..series.len() {
            if mask.get(i).unwrap_or(false) {
                result_vec.push(Some(fill_value));
            } else {
                let val = series.get(i)?;
                result_vec.push(Some(val.try_extract::<f64>()?));
            }
        }
        
        let result = Series::new(col_name.into(), result_vec);
        df.replace(col_name, result)?;
        
        processing_steps.push(format!(
            "Filled '{}' with {}: {:.2}",
            col_name, method, fill_value
        ));
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_column_profile(name: &str, inferred_type: &str) -> ColumnProfile {
        ColumnProfile {
            name: name.to_string(),
            dtype: "Unknown".to_string(),
            inferred_type: inferred_type.to_string(),
            null_count: 0,
            null_percentage: 0.0,
            unique_count: 0,
            sample_values: vec![],
            inferred_role: "feature".to_string(),
            characteristics: std::collections::HashMap::new(),
        }
    }

    // ========================================================================
    // apply_numeric_median() tests
    // ========================================================================

    #[test]
    fn test_apply_numeric_median_basic() {
        let mut df = df![
            "values" => [Some(1.0), None, Some(3.0), None, Some(5.0)],
        ].unwrap();
        let mut steps = Vec::new();

        StatisticalImputer::apply_numeric_median(&mut df, "values", &mut steps).unwrap();

        let values = df.column("values").unwrap();
        assert_eq!(values.null_count(), 0);
        
        // Median of [1, 3, 5] = 3
        let imputed_1 = values.get(1).unwrap().try_extract::<f64>().unwrap();
        let imputed_3 = values.get(3).unwrap().try_extract::<f64>().unwrap();
        assert_eq!(imputed_1, 3.0);
        assert_eq!(imputed_3, 3.0);
        
        assert!(steps[0].contains("median"));
    }

    #[test]
    fn test_apply_numeric_median_no_nulls() {
        let mut df = df![
            "values" => [1.0, 2.0, 3.0],
        ].unwrap();
        let mut steps = Vec::new();

        StatisticalImputer::apply_numeric_median(&mut df, "values", &mut steps).unwrap();

        // Values unchanged, but a step is still logged
        let values = df.column("values").unwrap();
        assert_eq!(values.get(0).unwrap().try_extract::<f64>().unwrap(), 1.0);
        assert_eq!(values.get(1).unwrap().try_extract::<f64>().unwrap(), 2.0);
        assert_eq!(values.get(2).unwrap().try_extract::<f64>().unwrap(), 3.0);
    }

    #[test]
    fn test_apply_numeric_median_single_value() {
        let mut df = df![
            "values" => [Some(42.0), None, None],
        ].unwrap();
        let mut steps = Vec::new();

        StatisticalImputer::apply_numeric_median(&mut df, "values", &mut steps).unwrap();

        // Median of single value [42] = 42
        let values = df.column("values").unwrap();
        assert_eq!(values.null_count(), 0);
        assert_eq!(values.get(1).unwrap().try_extract::<f64>().unwrap(), 42.0);
    }

    #[test]
    fn test_apply_numeric_median_all_nulls() {
        let mut df = df![
            "values" => [Option::<f64>::None, None, None],
        ].unwrap();
        let mut steps = Vec::new();

        // Should not panic, but no imputation happens since median is None
        StatisticalImputer::apply_numeric_median(&mut df, "values", &mut steps).unwrap();
        
        // Steps should be empty since median couldn't be calculated
        assert!(steps.is_empty());
    }

    #[test]
    fn test_apply_numeric_median_nonexistent_column() {
        let mut df = df![
            "other" => [1.0, 2.0, 3.0],
        ].unwrap();
        let mut steps = Vec::new();

        // Should not panic for non-existent column
        StatisticalImputer::apply_numeric_median(&mut df, "values", &mut steps).unwrap();
        assert!(steps.is_empty());
    }

    // ========================================================================
    // apply_numeric_mean() tests
    // ========================================================================

    #[test]
    fn test_apply_numeric_mean_basic() {
        let mut df = df![
            "values" => [Some(1.0), None, Some(5.0)],
        ].unwrap();
        let mut steps = Vec::new();

        StatisticalImputer::apply_numeric_mean(&mut df, "values", &mut steps).unwrap();

        // Mean of [1, 5] = 3
        let values = df.column("values").unwrap();
        assert_eq!(values.null_count(), 0);
        assert_eq!(values.get(1).unwrap().try_extract::<f64>().unwrap(), 3.0);
        
        assert!(steps[0].contains("mean"));
    }

    #[test]
    fn test_apply_numeric_mean_preserves_original_values() {
        let mut df = df![
            "values" => [Some(10.0), None, Some(20.0)],
        ].unwrap();
        let mut steps = Vec::new();

        StatisticalImputer::apply_numeric_mean(&mut df, "values", &mut steps).unwrap();

        let values = df.column("values").unwrap();
        // Original values preserved
        assert_eq!(values.get(0).unwrap().try_extract::<f64>().unwrap(), 10.0);
        assert_eq!(values.get(2).unwrap().try_extract::<f64>().unwrap(), 20.0);
        // Mean = 15
        assert_eq!(values.get(1).unwrap().try_extract::<f64>().unwrap(), 15.0);
    }

    // ========================================================================
    // apply_mode_imputation() tests
    // ========================================================================

    #[test]
    fn test_apply_mode_imputation_basic() {
        let mut df = df![
            "category" => [Some("A"), Some("B"), Some("A"), None, Some("A")],
        ].unwrap();
        let col_profile = create_test_column_profile("category", "categorical");
        let mut steps = Vec::new();

        StatisticalImputer::apply_mode_imputation(&mut df, &col_profile, &mut steps).unwrap();

        let category = df.column("category").unwrap();
        assert_eq!(category.null_count(), 0);
        // Mode is "A" (appears 3 times)
        assert_eq!(category.get(3).unwrap().to_string(), "\"A\"");
        
        assert!(steps[0].contains("mode"));
    }

    #[test]
    fn test_apply_mode_imputation_tie_breaking() {
        let mut df = df![
            "category" => [Some("A"), Some("B"), None],
        ].unwrap();
        let col_profile = create_test_column_profile("category", "categorical");
        let mut steps = Vec::new();

        StatisticalImputer::apply_mode_imputation(&mut df, &col_profile, &mut steps).unwrap();

        // When there's a tie, string_mode returns the first encountered
        let category = df.column("category").unwrap();
        assert_eq!(category.null_count(), 0);
    }

    #[test]
    fn test_apply_mode_imputation_all_unique() {
        let mut df = df![
            "category" => [Some("A"), Some("B"), Some("C"), None],
        ].unwrap();
        let col_profile = create_test_column_profile("category", "categorical");
        let mut steps = Vec::new();

        StatisticalImputer::apply_mode_imputation(&mut df, &col_profile, &mut steps).unwrap();

        // When all unique, mode is still the first one
        let category = df.column("category").unwrap();
        assert_eq!(category.null_count(), 0);
    }

    // ========================================================================
    // apply_category_indicator() tests
    // ========================================================================

    #[test]
    fn test_apply_category_indicator_basic() {
        let mut df = df![
            "category" => [Some("A"), None, Some("B")],
        ].unwrap();
        let col_profile = create_test_column_profile("category", "categorical");
        let mut steps = Vec::new();

        StatisticalImputer::apply_category_indicator(&mut df, &col_profile, &mut steps).unwrap();

        let category = df.column("category").unwrap();
        assert_eq!(category.null_count(), 0);
        assert_eq!(category.get(1).unwrap().to_string(), "\"Missing\"");
        
        assert!(steps[0].contains("Missing"));
    }

    #[test]
    fn test_apply_category_indicator_multiple_nulls() {
        let mut df = df![
            "category" => [None, Some("A"), None, None],
        ].unwrap();
        let col_profile = create_test_column_profile("category", "categorical");
        let mut steps = Vec::new();

        StatisticalImputer::apply_category_indicator(&mut df, &col_profile, &mut steps).unwrap();

        let category = df.column("category").unwrap();
        assert_eq!(category.null_count(), 0);
        
        // Check that nulls are filled with "Missing" and non-nulls preserved
        // Use contains to avoid quoting issues
        assert!(category.get(0).unwrap().to_string().contains("Missing"));
        assert!(category.get(1).unwrap().to_string().contains("A"));
        assert!(category.get(2).unwrap().to_string().contains("Missing"));
        assert!(category.get(3).unwrap().to_string().contains("Missing"));
    }

    // ========================================================================
    // apply_constant_imputation() tests
    // ========================================================================

    #[test]
    fn test_apply_constant_imputation_basic() {
        let mut df = df![
            "text" => [Some("Hello"), None, Some("World")],
        ].unwrap();
        let col_profile = create_test_column_profile("text", "text");
        let mut steps = Vec::new();

        StatisticalImputer::apply_constant_imputation(&mut df, &col_profile, &mut steps).unwrap();

        let text = df.column("text").unwrap();
        assert_eq!(text.null_count(), 0);
        assert_eq!(text.get(1).unwrap().to_string(), "\"Unknown\"");
        
        assert!(steps[0].contains("Unknown"));
    }

    // ========================================================================
    // apply_fallback_imputation() tests
    // ========================================================================

    #[test]
    fn test_apply_fallback_imputation_numeric() {
        let mut df = df![
            "values" => [Some(1.0), None, Some(5.0)],
        ].unwrap();
        let col_profile = create_test_column_profile("values", "numeric");
        let mut steps = Vec::new();

        StatisticalImputer::apply_fallback_imputation(&mut df, &col_profile, &mut steps).unwrap();

        let values = df.column("values").unwrap();
        assert_eq!(values.null_count(), 0);
        // Median of [1, 5] = 3
        assert_eq!(values.get(1).unwrap().try_extract::<f64>().unwrap(), 3.0);
        
        assert!(steps[0].contains("fallback"));
    }

    #[test]
    fn test_apply_fallback_imputation_categorical() {
        let mut df = df![
            "category" => [Some("A"), Some("A"), None],
        ].unwrap();
        let col_profile = create_test_column_profile("category", "categorical");
        let mut steps = Vec::new();

        StatisticalImputer::apply_fallback_imputation(&mut df, &col_profile, &mut steps).unwrap();

        let category = df.column("category").unwrap();
        assert_eq!(category.null_count(), 0);
        // Mode is "A"
        assert_eq!(category.get(2).unwrap().to_string(), "\"A\"");
    }

    #[test]
    fn test_apply_fallback_imputation_binary() {
        let mut df = df![
            "flag" => [Some("yes"), Some("yes"), Some("no"), None],
        ].unwrap();
        let col_profile = create_test_column_profile("flag", "binary");
        let mut steps = Vec::new();

        StatisticalImputer::apply_fallback_imputation(&mut df, &col_profile, &mut steps).unwrap();

        let flag = df.column("flag").unwrap();
        assert_eq!(flag.null_count(), 0);
        // Mode is "yes"
        assert_eq!(flag.get(3).unwrap().to_string(), "\"yes\"");
    }

    #[test]
    fn test_apply_fallback_imputation_unknown_type() {
        let mut df = df![
            "unknown" => [Some("data"), None, Some("more")],
        ].unwrap();
        let col_profile = create_test_column_profile("unknown", "weird_type");
        let mut steps = Vec::new();

        StatisticalImputer::apply_fallback_imputation(&mut df, &col_profile, &mut steps).unwrap();

        let unknown = df.column("unknown").unwrap();
        assert_eq!(unknown.null_count(), 0);
        // Falls back to "Unknown"
        assert_eq!(unknown.get(1).unwrap().to_string(), "\"Unknown\"");
        
        assert!(steps[0].contains("fallback"));
    }

    // ========================================================================
    // fill_with_value() tests (indirectly tested via above)
    // ========================================================================

    #[test]
    fn test_fill_with_value_logs_correct_step() {
        let mut df = df![
            "values" => [Some(1.0), None, Some(3.0)],
        ].unwrap();
        let mut steps = Vec::new();

        StatisticalImputer::apply_numeric_median(&mut df, "values", &mut steps).unwrap();

        assert_eq!(steps.len(), 1);
        assert!(steps[0].contains("values"));
        assert!(steps[0].contains("median"));
        assert!(steps[0].contains("2.00")); // median of [1, 3] = 2
    }

    #[test]
    fn test_fill_with_value_preserves_type() {
        let mut df = df![
            "values" => [Some(10.0), None, Some(20.0)],
        ].unwrap();
        let mut steps = Vec::new();

        StatisticalImputer::apply_numeric_mean(&mut df, "values", &mut steps).unwrap();

        let values = df.column("values").unwrap();
        // Result should still be Float64
        assert!(matches!(values.dtype(), DataType::Float64));
    }
}
