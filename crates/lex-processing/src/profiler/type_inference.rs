//! Type inference logic for column analysis.

use crate::utils::{is_boolean_string, is_error_marker, is_numeric_dtype, is_numeric_string};
use anyhow::Result;
use once_cell::sync::Lazy;
use polars::prelude::*;
use regex::Regex;

use super::role_inference::is_identifier_column_advanced;

// Date pattern regexes - compiled once at startup
static DATE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"^\d{4}[-/]\d{1,2}[-/]\d{1,2}$").expect("Invalid regex: YYYY-MM-DD"),
        Regex::new(r"^\d{1,2}[-/]\d{1,2}[-/]\d{4}$").expect("Invalid regex: MM-DD-YYYY"),
        Regex::new(r"^\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2}").expect("Invalid regex: datetime"),
        Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}").expect("Invalid regex: ISO"),
    ]
});

/// Advanced type inference that handles post-correction data.
pub(crate) fn infer_column_type_advanced(
    series: &Series,
    sample_values: &[String],
    col_name: &str,
) -> Result<String> {
    // Skip if all null
    if series.null_count() == series.len() {
        return Ok("unknown".to_string());
    }

    let non_null_series = series.drop_nulls();
    if non_null_series.is_empty() {
        return Ok("unknown".to_string());
    }

    // Check 1: Boolean types (native and string representations)
    if is_boolean_column(series, sample_values) {
        return Ok("binary".to_string());
    }

    // Check 2: Datetime - only if it looks like actual dates, not numeric timestamps
    if is_actual_datetime_column(series, sample_values) {
        return Ok("datetime".to_string());
    }

    // Check 3: Numeric - including string columns with numeric content
    // KEY FIX: If it's numeric, return "numeric" - NEVER convert to categorical
    if is_numeric_column_advanced(series, sample_values) {
        return Ok("numeric".to_string());
    }

    // Check 4: Identifier patterns (with context awareness)
    if is_identifier_column_advanced(&non_null_series, sample_values, col_name) {
        return Ok("identifier".to_string());
    }

    // Check 5: Text vs string (only for non-numeric strings)
    if series.dtype() == &DataType::String {
        let unique_ratio = non_null_series.n_unique()? as f64 / non_null_series.len() as f64;
        let str_series = non_null_series.str()?;
        let avg_length: f64 = str_series
            .into_iter()
            .filter_map(|v| v.map(|s| s.len()))
            .sum::<usize>() as f64
            / non_null_series.len() as f64;

        // Text if high uniqueness and long values
        if unique_ratio > 0.7 && avg_length > 30.0 {
            return Ok("text".to_string());
        } else {
            // Everything else is "string" (for categorical text data)
            return Ok("string".to_string());
        }
    }

    // Default to string for other types
    Ok("string".to_string())
}

/// More aggressive numeric detection that handles "ERROR", "UNKNOWN", etc.
pub(crate) fn is_numeric_column_advanced(series: &Series, sample_values: &[String]) -> bool {
    let name_suggests_numeric = {
        let indicators = [
            "quantity", "price", "amount", "cost", "total", "value", "sum", "count", "number",
            "int", "float", "score", "rating", "percent", "percentage",
        ];
        let lower = series.name().to_lowercase();
        indicators.iter().any(|ind| lower.contains(ind))
    };

    // --- 1. Native numeric types ---
    if is_numeric_dtype(series.dtype()) {
        return true;
    }

    // --- 2. String-based numeric detection ---
    if series.dtype() == &DataType::String {
        let non_null_count = series.len() - series.null_count();
        if non_null_count == 0 {
            return false;
        }

        // Thresholds
        let threshold_samples = if name_suggests_numeric { 0.3 } else { 0.6 };
        let threshold_fallback = if name_suggests_numeric { 0.4 } else { 0.7 };

        // 2a. Sample values provided externally
        if !sample_values.is_empty() {
            let mut numeric_samples = 0;
            let mut total_samples = 0;

            for sample in sample_values.iter().take(20) {
                let trimmed = sample.trim();
                if trimmed.is_empty() || is_error_marker(trimmed) {
                    continue;
                }
                total_samples += 1;
                if is_numeric_string(trimmed) {
                    numeric_samples += 1;
                }
            }

            if total_samples > 0 {
                let ratio = numeric_samples as f64 / total_samples as f64;
                if ratio >= threshold_samples {
                    return true;
                }
            }
        }

        // 2b. Fallback: sample directly from series
        if let Ok(str_series) = series.str() {
            let sample_size = std::cmp::min(100, non_null_count);
            let mut numeric_count = 0;
            let mut total_checked = 0;

            for i in 0..str_series.len() {
                if total_checked >= sample_size {
                    break;
                }
                if let Some(val) = str_series.get(i) {
                    let trimmed = val.trim();
                    if trimmed.is_empty() || is_error_marker(trimmed) {
                        continue;
                    }
                    total_checked += 1;
                    if is_numeric_string(trimmed) {
                        numeric_count += 1;
                    }
                }
            }

            return total_checked > 0
                && (numeric_count as f64 / total_checked as f64) >= threshold_fallback;
        }
    }

    false
}

/// Check if column is boolean (including string representations).
pub(crate) fn is_boolean_column(series: &Series, sample_values: &[String]) -> bool {
    // Native boolean type
    if series.dtype() == &DataType::Boolean {
        return true;
    }

    // Check string representations using the utility function
    if series.dtype() == &DataType::String && !sample_values.is_empty() {
        let mut boolean_count = 0;

        for sample in sample_values.iter().take(5) {
            if is_boolean_string(sample) {
                boolean_count += 1;
            }
        }

        // If most samples look boolean
        if boolean_count >= 3 && sample_values.len() >= 3 {
            return true;
        }
    }

    false
}

/// Check if column contains actual datetime values (not numeric timestamps).
pub(crate) fn is_actual_datetime_column(_series: &Series, sample_values: &[String]) -> bool {
    if sample_values.is_empty() {
        return false;
    }

    // More specific datetime patterns that require proper date formatting
    let patterns = &*DATE_PATTERNS;

    let mut date_like_count = 0;
    let mut total_checked = 0;

    for sample in sample_values.iter().take(10) {
        total_checked += 1;

        // Skip empty or numeric-looking samples (could be timestamps)
        let trimmed = sample.trim();
        if trimmed.is_empty() || trimmed.parse::<f64>().is_ok() {
            continue;
        }

        // Check if it matches any date pattern
        for pattern in patterns.iter() {
            if pattern.is_match(trimmed) {
                date_like_count += 1;
                break;
            }
        }
    }

    // Need at least 70% of checked samples to be date-like
    if total_checked > 0 {
        (date_like_count as f64 / total_checked as f64) > 0.7
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== infer_column_type_advanced tests ====================

    #[test]
    fn test_infer_type_all_null_returns_unknown() {
        let series = Series::new("col".into(), &[None::<i64>, None, None]);
        let samples = vec![];
        
        let result = infer_column_type_advanced(&series, &samples, "col").unwrap();
        assert_eq!(result, "unknown");
    }

    #[test]
    fn test_infer_type_boolean_native() {
        let series = Series::new("is_active".into(), &[true, false, true, false]);
        let samples = vec!["true".to_string(), "false".to_string()];
        
        let result = infer_column_type_advanced(&series, &samples, "is_active").unwrap();
        assert_eq!(result, "binary");
    }

    #[test]
    fn test_infer_type_boolean_string_representation() {
        let series = Series::new("flag".into(), &["yes", "no", "yes", "no", "yes"]);
        let samples = vec![
            "yes".to_string(),
            "no".to_string(),
            "yes".to_string(),
            "no".to_string(),
            "yes".to_string(),
        ];
        
        let result = infer_column_type_advanced(&series, &samples, "flag").unwrap();
        assert_eq!(result, "binary");
    }

    #[test]
    fn test_infer_type_datetime_iso_format() {
        let series = Series::new("date".into(), &["2024-01-15", "2024-02-20", "2024-03-25"]);
        let samples = vec![
            "2024-01-15".to_string(),
            "2024-02-20".to_string(),
            "2024-03-25".to_string(),
        ];
        
        let result = infer_column_type_advanced(&series, &samples, "date").unwrap();
        assert_eq!(result, "datetime");
    }

    #[test]
    fn test_infer_type_datetime_with_time() {
        let series = Series::new(
            "timestamp".into(),
            &["2024-01-15T10:30:00", "2024-02-20T14:45:00"],
        );
        let samples = vec![
            "2024-01-15T10:30:00".to_string(),
            "2024-02-20T14:45:00".to_string(),
        ];
        
        let result = infer_column_type_advanced(&series, &samples, "timestamp").unwrap();
        assert_eq!(result, "datetime");
    }

    #[test]
    fn test_infer_type_numeric_native_int() {
        let series = Series::new("count".into(), &[1i64, 2, 3, 4, 5]);
        let samples = vec![];
        
        let result = infer_column_type_advanced(&series, &samples, "count").unwrap();
        assert_eq!(result, "numeric");
    }

    #[test]
    fn test_infer_type_numeric_native_float() {
        let series = Series::new("price".into(), &[1.5f64, 2.5, 3.5, 4.5, 5.5]);
        let samples = vec![];
        
        let result = infer_column_type_advanced(&series, &samples, "price").unwrap();
        assert_eq!(result, "numeric");
    }

    #[test]
    fn test_infer_type_numeric_string_representation() {
        let series = Series::new("amount".into(), &["100", "200", "300", "400", "500"]);
        let samples = vec![
            "100".to_string(),
            "200".to_string(),
            "300".to_string(),
            "400".to_string(),
            "500".to_string(),
        ];
        
        let result = infer_column_type_advanced(&series, &samples, "amount").unwrap();
        assert_eq!(result, "numeric");
    }

    #[test]
    fn test_infer_type_text_high_uniqueness_long_values() {
        let series = Series::new(
            "description".into(),
            &[
                "This is a very long description that exceeds thirty characters easily",
                "Another long unique description for testing text detection properly",
                "Third unique and lengthy description to test the text classification logic",
            ],
        );
        let samples = vec![];
        
        let result = infer_column_type_advanced(&series, &samples, "description").unwrap();
        assert_eq!(result, "text");
    }

    #[test]
    fn test_infer_type_string_categorical() {
        let series = Series::new("category".into(), &["red", "blue", "green", "red", "blue"]);
        let samples = vec![];
        
        let result = infer_column_type_advanced(&series, &samples, "category").unwrap();
        assert_eq!(result, "string");
    }

    // ==================== is_numeric_column_advanced tests ====================

    #[test]
    fn test_numeric_column_native_float64() {
        let series = Series::new("value".into(), &[1.0f64, 2.0, 3.0]);
        assert!(is_numeric_column_advanced(&series, &[]));
    }

    #[test]
    fn test_numeric_column_native_int64() {
        let series = Series::new("value".into(), &[1i64, 2, 3]);
        assert!(is_numeric_column_advanced(&series, &[]));
    }

    #[test]
    fn test_numeric_column_string_with_numbers() {
        let series = Series::new("value".into(), &["1.5", "2.5", "3.5", "4.5", "5.5"]);
        let samples = vec![
            "1.5".to_string(),
            "2.5".to_string(),
            "3.5".to_string(),
            "4.5".to_string(),
            "5.5".to_string(),
        ];
        assert!(is_numeric_column_advanced(&series, &samples));
    }

    #[test]
    fn test_numeric_column_name_hint_lower_threshold() {
        // Column named "price" should have lower threshold for numeric detection
        let series = Series::new("price".into(), &["100", "ERROR", "300"]);
        let samples = vec!["100".to_string(), "ERROR".to_string(), "300".to_string()];
        // With name hint, threshold is 0.3, so 2/2 valid (67%) should pass
        assert!(is_numeric_column_advanced(&series, &samples));
    }

    #[test]
    fn test_numeric_column_with_error_markers() {
        let series = Series::new("value".into(), &["100", "ERROR", "N/A", "200", "300"]);
        let samples = vec![
            "100".to_string(),
            "ERROR".to_string(),
            "N/A".to_string(),
            "200".to_string(),
            "300".to_string(),
        ];
        // ERROR and N/A are skipped, so 3/3 = 100% numeric
        assert!(is_numeric_column_advanced(&series, &samples));
    }

    #[test]
    fn test_numeric_column_string_all_nulls_returns_false() {
        // String series with no non-null values can't be determined as numeric
        let series: Series = Series::new("value".into(), &[None::<&str>, None, None]);
        assert!(!is_numeric_column_advanced(&series, &[]));
    }

    #[test]
    fn test_numeric_column_string_non_numeric() {
        let series = Series::new("name".into(), &["Alice", "Bob", "Charlie", "Diana", "Eve"]);
        let samples = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ];
        assert!(!is_numeric_column_advanced(&series, &samples));
    }

    // ==================== is_boolean_column tests ====================

    #[test]
    fn test_boolean_column_native_type() {
        let series = Series::new("flag".into(), &[true, false, true]);
        assert!(is_boolean_column(&series, &[]));
    }

    #[test]
    fn test_boolean_column_string_true_false() {
        let series = Series::new("flag".into(), &["true", "false", "true", "false", "true"]);
        let samples = vec![
            "true".to_string(),
            "false".to_string(),
            "true".to_string(),
            "false".to_string(),
            "true".to_string(),
        ];
        assert!(is_boolean_column(&series, &samples));
    }

    #[test]
    fn test_boolean_column_string_yes_no() {
        let series = Series::new("active".into(), &["yes", "no", "yes", "no", "yes"]);
        let samples = vec![
            "yes".to_string(),
            "no".to_string(),
            "yes".to_string(),
            "no".to_string(),
            "yes".to_string(),
        ];
        assert!(is_boolean_column(&series, &samples));
    }

    #[test]
    fn test_boolean_column_string_01() {
        let series = Series::new("binary".into(), &["0", "1", "1", "0", "1"]);
        let samples = vec![
            "0".to_string(),
            "1".to_string(),
            "1".to_string(),
            "0".to_string(),
            "1".to_string(),
        ];
        assert!(is_boolean_column(&series, &samples));
    }

    #[test]
    fn test_boolean_column_not_boolean() {
        let series = Series::new("category".into(), &["red", "blue", "green"]);
        let samples = vec!["red".to_string(), "blue".to_string(), "green".to_string()];
        assert!(!is_boolean_column(&series, &samples));
    }

    #[test]
    fn test_boolean_column_empty_samples() {
        let series = Series::new("flag".into(), &["true", "false"]);
        // With empty samples and non-native type, should return false
        assert!(!is_boolean_column(&series, &[]));
    }

    // ==================== is_actual_datetime_column tests ====================

    #[test]
    fn test_datetime_column_yyyy_mm_dd() {
        let samples = vec![
            "2024-01-15".to_string(),
            "2024-02-20".to_string(),
            "2024-03-25".to_string(),
        ];
        let series = Series::new("date".into(), &["2024-01-15", "2024-02-20", "2024-03-25"]);
        assert!(is_actual_datetime_column(&series, &samples));
    }

    #[test]
    fn test_datetime_column_mm_dd_yyyy() {
        let samples = vec![
            "01/15/2024".to_string(),
            "02/20/2024".to_string(),
            "03/25/2024".to_string(),
        ];
        let series = Series::new("date".into(), &["01/15/2024", "02/20/2024", "03/25/2024"]);
        assert!(is_actual_datetime_column(&series, &samples));
    }

    #[test]
    fn test_datetime_column_iso_with_time() {
        let samples = vec![
            "2024-01-15T10:30:00".to_string(),
            "2024-02-20T14:45:00".to_string(),
        ];
        let series = Series::new(
            "timestamp".into(),
            &["2024-01-15T10:30:00", "2024-02-20T14:45:00"],
        );
        assert!(is_actual_datetime_column(&series, &samples));
    }

    #[test]
    fn test_datetime_column_space_separator() {
        let samples = vec![
            "2024-01-15 10:30:00".to_string(),
            "2024-02-20 14:45:00".to_string(),
        ];
        let series = Series::new(
            "timestamp".into(),
            &["2024-01-15 10:30:00", "2024-02-20 14:45:00"],
        );
        assert!(is_actual_datetime_column(&series, &samples));
    }

    #[test]
    fn test_datetime_column_not_datetime_numeric() {
        // Numeric strings that could be timestamps but don't match date patterns
        let samples = vec![
            "1705312200".to_string(),
            "1705398600".to_string(),
            "1705485000".to_string(),
        ];
        let series = Series::new(
            "timestamp".into(),
            &["1705312200", "1705398600", "1705485000"],
        );
        assert!(!is_actual_datetime_column(&series, &samples));
    }

    #[test]
    fn test_datetime_column_empty_samples() {
        let series = Series::new("date".into(), &["2024-01-15"]);
        assert!(!is_actual_datetime_column(&series, &[]));
    }

    #[test]
    fn test_datetime_column_mixed_content_below_threshold() {
        // Only 1 out of 3 is a date pattern (33% < 70% threshold)
        let samples = vec![
            "2024-01-15".to_string(),
            "not a date".to_string(),
            "also not".to_string(),
        ];
        let series = Series::new("mixed".into(), &["2024-01-15", "not a date", "also not"]);
        assert!(!is_actual_datetime_column(&series, &samples));
    }
}
