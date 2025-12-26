//! Shared utilities for the data preprocessing pipeline.
//!
//! This module contains common helper functions used across multiple modules
//! to reduce code duplication and ensure consistency.

use polars::prelude::*;

// =============================================================================
// Data Type Utilities
// =============================================================================

/// Category of a data type for preprocessing purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtypeCategory {
    /// Integer or floating point numbers
    Numeric,
    /// Date or datetime types
    Datetime,
    /// Boolean type
    Boolean,
    /// String/text type
    String,
    /// Other/unknown types
    Other,
}

/// Check if a DataType is numeric (integer or float).
#[inline]
pub fn is_numeric_dtype(dtype: &DataType) -> bool {
    matches!(
        dtype,
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
    )
}

/// Check if a DataType is a datetime type.
#[inline]
pub fn is_datetime_dtype(dtype: &DataType) -> bool {
    matches!(
        dtype,
        DataType::Datetime(_, _) | DataType::Date | DataType::Time
    )
}

/// Check if a DataType is boolean.
#[inline]
pub fn is_boolean_dtype(dtype: &DataType) -> bool {
    matches!(dtype, DataType::Boolean)
}

/// Get the category of a DataType.
pub fn get_dtype_category(dtype: &DataType) -> DtypeCategory {
    if is_numeric_dtype(dtype) {
        DtypeCategory::Numeric
    } else if is_datetime_dtype(dtype) {
        DtypeCategory::Datetime
    } else if is_boolean_dtype(dtype) {
        DtypeCategory::Boolean
    } else if matches!(dtype, DataType::String | DataType::Categorical(_, _)) {
        DtypeCategory::String
    } else {
        DtypeCategory::Other
    }
}

/// Get the dtype category of a Series.
pub fn series_dtype_category(series: &Series) -> DtypeCategory {
    get_dtype_category(series.dtype())
}

/// Get the dtype category as a string (for backward compatibility).
pub fn dtype_category_str(series: &Series) -> &'static str {
    match series_dtype_category(series) {
        DtypeCategory::Numeric => "numeric",
        DtypeCategory::Datetime => "datetime",
        DtypeCategory::Boolean => "binary",
        DtypeCategory::String => "string",
        DtypeCategory::Other => "other",
    }
}

// =============================================================================
// String Parsing Utilities
// =============================================================================

/// Characters commonly used in numeric formatting that should be stripped.
pub const NUMERIC_FORMAT_CHARS: [char; 6] = [',', '$', '%', '€', '£', ' '];

/// Common error/missing value markers in data.
pub const ERROR_MARKERS: [&str; 8] = [
    "error", "unknown", "n/a", "na", "null", "missing", "none", "#n/a",
];

/// Clean a string for numeric parsing by removing formatting characters.
///
/// # Example
///
/// ```rust,ignore
/// use data_preprocessing_pipeline::utils::clean_numeric_string;
///
/// assert_eq!(clean_numeric_string("$1,234.56"), "1234.56");
/// assert_eq!(clean_numeric_string("  42%  "), "42");
/// ```
pub fn clean_numeric_string(s: &str) -> String {
    let mut result = s.trim().to_string();
    for c in NUMERIC_FORMAT_CHARS {
        result = result.replace(c, "");
    }
    result
}

/// Check if a string is an error/missing value marker.
///
/// # Example
///
/// ```rust,ignore
/// use data_preprocessing_pipeline::utils::is_error_marker;
///
/// assert!(is_error_marker("ERROR"));
/// assert!(is_error_marker("N/A"));
/// assert!(!is_error_marker("42"));
/// ```
pub fn is_error_marker(s: &str) -> bool {
    let lower = s.trim().to_ascii_lowercase();
    ERROR_MARKERS.iter().any(|&marker| lower == marker)
}

/// Try to parse a string as a numeric value (f64).
///
/// Handles common formatting like currency symbols, percentages, and thousands separators.
pub fn parse_numeric_string(s: &str) -> Option<f64> {
    let cleaned = clean_numeric_string(s);
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse::<f64>().ok()
}

/// Check if a string can be parsed as a numeric value.
pub fn is_numeric_string(s: &str) -> bool {
    parse_numeric_string(s).is_some()
}

/// Check if a string value looks like a float (has decimal point or fractional part).
pub fn looks_like_float(s: &str) -> bool {
    let cleaned = clean_numeric_string(s);
    if let Ok(num) = cleaned.parse::<f64>() {
        cleaned.contains('.') || num.fract() != 0.0
    } else {
        false
    }
}

// =============================================================================
// Series Statistics Utilities
// =============================================================================

/// Calculate the mode (most frequent value) of a string Series.
pub fn string_mode(series: &Series) -> Option<String> {
    let non_null = series.drop_nulls();
    if non_null.is_empty() {
        return None;
    }

    let str_series = match non_null.cast(&DataType::String) {
        Ok(s) => s,
        Err(_) => return None,
    };

    let str_chunked = match str_series.str() {
        Ok(s) => s,
        Err(_) => return None,
    };

    let mut value_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for val in str_chunked.into_iter().flatten() {
        *value_counts.entry(val.to_string()).or_insert(0) += 1;
    }

    value_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(val, _)| val)
}

/// Count values in a Series that can be parsed as numeric.
pub fn count_numeric_values(series: &Series) -> (usize, usize) {
    let mut numeric_count = 0;
    let mut total_count = 0;

    if let Ok(str_series) = series.str() {
        for val in str_series.into_iter().flatten() {
            let trimmed = val.trim();
            if trimmed.is_empty() || is_error_marker(trimmed) {
                continue;
            }
            total_count += 1;
            if is_numeric_string(trimmed) {
                numeric_count += 1;
            }
        }
    }

    (numeric_count, total_count)
}

/// Get the ratio of numeric-parseable values in a string Series.
pub fn numeric_ratio(series: &Series) -> f64 {
    let (numeric_count, total_count) = count_numeric_values(series);
    if total_count == 0 {
        0.0
    } else {
        numeric_count as f64 / total_count as f64
    }
}

// =============================================================================
// Series Transformation Utilities
// =============================================================================

/// Fill null values in a numeric Series with a specific value.
pub fn fill_numeric_nulls(series: &Series, fill_value: f64) -> PolarsResult<Series> {
    let mask = series.is_null();
    let len = series.len();
    let mut result_vec = Vec::with_capacity(len);

    for i in 0..len {
        if mask.get(i).unwrap_or(false) {
            result_vec.push(Some(fill_value));
        } else {
            let val = series.get(i)?;
            result_vec.push(Some(val.try_extract::<f64>()?));
        }
    }

    Ok(Series::new(series.name().clone(), result_vec))
}

/// Fill null values in a string Series with a specific value.
pub fn fill_string_nulls(series: &Series, fill_value: &str) -> PolarsResult<Series> {
    let mask = series.is_null();
    let len = series.len();
    let mut result_vec = Vec::with_capacity(len);

    for i in 0..len {
        if mask.get(i).unwrap_or(false) {
            result_vec.push(Some(fill_value.to_string()));
        } else {
            let val = series.get(i)?;
            result_vec.push(Some(format!("{}", val)));
        }
    }

    Ok(Series::new(series.name().clone(), result_vec))
}

/// Collect sample values from a Series (non-null values only).
pub fn collect_sample_values(series: &Series, max_samples: usize) -> Vec<String> {
    let non_null = series.drop_nulls();
    if non_null.is_empty() {
        return Vec::new();
    }

    let sample_size = std::cmp::min(max_samples, non_null.len());
    let mut samples = Vec::with_capacity(sample_size);

    for i in 0..sample_size {
        if let Ok(val) = non_null.get(i) {
            samples.push(format!("{}", val));
        }
    }

    samples
}

// =============================================================================
// Boolean Detection Utilities
// =============================================================================

/// Common boolean true representations.
pub const BOOLEAN_TRUE_VALUES: [&str; 8] =
    ["true", "yes", "1", "t", "y", "on", "enabled", "active"];

/// Common boolean false representations.
pub const BOOLEAN_FALSE_VALUES: [&str; 8] =
    ["false", "no", "0", "f", "n", "off", "disabled", "inactive"];

/// Check if a string represents a boolean true value.
pub fn is_boolean_true(s: &str) -> bool {
    let lower = s.trim().to_ascii_lowercase();
    BOOLEAN_TRUE_VALUES.iter().any(|&v| v == lower)
}

/// Check if a string represents a boolean false value.
pub fn is_boolean_false(s: &str) -> bool {
    let lower = s.trim().to_ascii_lowercase();
    BOOLEAN_FALSE_VALUES.iter().any(|&v| v == lower)
}

/// Check if a string represents a boolean value (true or false).
pub fn is_boolean_string(s: &str) -> bool {
    is_boolean_true(s) || is_boolean_false(s)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_numeric_dtype() {
        assert!(is_numeric_dtype(&DataType::Int64));
        assert!(is_numeric_dtype(&DataType::Float64));
        assert!(!is_numeric_dtype(&DataType::String));
        assert!(!is_numeric_dtype(&DataType::Boolean));
    }

    #[test]
    fn test_is_datetime_dtype() {
        assert!(is_datetime_dtype(&DataType::Date));
        assert!(is_datetime_dtype(&DataType::Datetime(
            TimeUnit::Milliseconds,
            None
        )));
        assert!(!is_datetime_dtype(&DataType::String));
    }

    #[test]
    fn test_clean_numeric_string() {
        assert_eq!(clean_numeric_string("$1,234.56"), "1234.56");
        assert_eq!(clean_numeric_string("  42%  "), "42");
        assert_eq!(clean_numeric_string("€100"), "100");
        assert_eq!(clean_numeric_string("1 000"), "1000");
    }

    #[test]
    fn test_is_error_marker() {
        assert!(is_error_marker("ERROR"));
        assert!(is_error_marker("error"));
        assert!(is_error_marker("N/A"));
        assert!(is_error_marker("unknown"));
        assert!(is_error_marker("  MISSING  "));
        assert!(!is_error_marker("42"));
        assert!(!is_error_marker("hello"));
    }

    #[test]
    fn test_parse_numeric_string() {
        assert_eq!(parse_numeric_string("42"), Some(42.0));
        assert_eq!(parse_numeric_string("$1,234.56"), Some(1234.56));
        assert_eq!(parse_numeric_string("-100"), Some(-100.0));
        assert_eq!(parse_numeric_string(""), None);
        assert_eq!(parse_numeric_string("hello"), None);
    }

    #[test]
    fn test_looks_like_float() {
        assert!(looks_like_float("3.14"));
        assert!(looks_like_float("1.0"));
        assert!(!looks_like_float("42"));
        assert!(!looks_like_float("100"));
    }

    #[test]
    fn test_is_boolean_string() {
        assert!(is_boolean_string("true"));
        assert!(is_boolean_string("FALSE"));
        assert!(is_boolean_string("yes"));
        assert!(is_boolean_string("0"));
        assert!(!is_boolean_string("maybe"));
        assert!(!is_boolean_string("42"));
    }

    #[test]
    fn test_dtype_category() {
        assert_eq!(get_dtype_category(&DataType::Int64), DtypeCategory::Numeric);
        assert_eq!(
            get_dtype_category(&DataType::Float64),
            DtypeCategory::Numeric
        );
        assert_eq!(get_dtype_category(&DataType::Date), DtypeCategory::Datetime);
        assert_eq!(
            get_dtype_category(&DataType::Boolean),
            DtypeCategory::Boolean
        );
        assert_eq!(get_dtype_category(&DataType::String), DtypeCategory::String);
    }

    #[test]
    fn test_fill_numeric_nulls() {
        let series = Series::new("test".into(), &[Some(1.0), None, Some(3.0)]);
        let filled = fill_numeric_nulls(&series, 0.0).unwrap();

        assert_eq!(filled.get(0).unwrap().try_extract::<f64>().unwrap(), 1.0);
        assert_eq!(filled.get(1).unwrap().try_extract::<f64>().unwrap(), 0.0);
        assert_eq!(filled.get(2).unwrap().try_extract::<f64>().unwrap(), 3.0);
    }

    #[test]
    fn test_string_mode() {
        let series = Series::new("test".into(), &["a", "b", "a", "c", "a"]);
        assert_eq!(string_mode(&series), Some("a".to_string()));
    }

    #[test]
    fn test_collect_sample_values() {
        let series = Series::new("test".into(), &[Some("a"), None, Some("b"), Some("c")]);
        let samples = collect_sample_values(&series, 5);
        assert_eq!(samples.len(), 3); // Only non-null values
    }
}
