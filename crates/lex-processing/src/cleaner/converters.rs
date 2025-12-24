//! Type conversion functions for data cleaning.

use crate::utils::{clean_numeric_string, is_error_marker};
use anyhow::Result;
use polars::prelude::*;
use std::collections::HashSet;

/// Convert string series to numeric (Float64, Int64, or Int32).
pub(crate) fn string_to_numeric(series: &Series, target_dtype: &DataType) -> Result<Series> {
    let str_series = series.str()?;

    match target_dtype {
        DataType::Float64 => {
            let mut result_vec: Vec<Option<f64>> = Vec::with_capacity(str_series.len());

            for opt_val in str_series.into_iter() {
                match opt_val {
                    Some(val) => {
                        let trimmed = val.trim();

                        if trimmed.is_empty() || is_error_marker(trimmed) {
                            result_vec.push(None);
                            continue;
                        }

                        let cleaned = clean_numeric_string(trimmed);

                        if let Ok(float_val) = cleaned.parse::<f64>() {
                            result_vec.push(Some(float_val));
                        } else {
                            // Try to extract numeric part from mixed strings
                            let numeric_part: String = cleaned
                                .chars()
                                .filter(|c| c.is_numeric() || *c == '.' || *c == '-')
                                .collect();

                            if let Ok(val) = numeric_part.parse::<f64>() {
                                result_vec.push(Some(val));
                            } else {
                                result_vec.push(None);
                            }
                        }
                    }
                    None => result_vec.push(None),
                }
            }

            Ok(Series::new(series.name().clone(), result_vec))
        }
        DataType::Int64 => {
            let mut result_vec: Vec<Option<i64>> = Vec::with_capacity(str_series.len());

            for opt_val in str_series.into_iter() {
                match opt_val {
                    Some(val) => {
                        let trimmed = val.trim();

                        if trimmed.is_empty() || is_error_marker(trimmed) {
                            result_vec.push(None);
                            continue;
                        }

                        let cleaned = clean_numeric_string(trimmed);

                        // Try parsing as float first, then convert to i64
                        if let Ok(float_val) = cleaned.parse::<f64>() {
                            result_vec.push(Some(float_val as i64));
                        } else {
                            result_vec.push(None);
                        }
                    }
                    None => result_vec.push(None),
                }
            }

            Ok(Series::new(series.name().clone(), result_vec))
        }
        DataType::Int32 => {
            let mut result_vec: Vec<Option<i32>> = Vec::with_capacity(str_series.len());

            for opt_val in str_series.into_iter() {
                match opt_val {
                    Some(val) => {
                        let trimmed = val.trim();

                        if trimmed.is_empty() || is_error_marker(trimmed) {
                            result_vec.push(None);
                            continue;
                        }

                        let cleaned = clean_numeric_string(trimmed);

                        // Try parsing as float first, then convert to i32
                        if let Ok(float_val) = cleaned.parse::<f64>() {
                            result_vec.push(Some(float_val as i32));
                        } else {
                            result_vec.push(None);
                        }
                    }
                    None => result_vec.push(None),
                }
            }

            Ok(Series::new(series.name().clone(), result_vec))
        }
        _ => Ok(series.clone()),
    }
}

/// Convert timestamp (in milliseconds) to datetime.
pub(crate) fn timestamp_to_datetime(series: &Series) -> Result<Series> {
    // Try to convert from string first
    if series.dtype() == &DataType::String {
        let str_series = series.str()?;
        let mut timestamps = Vec::with_capacity(str_series.len());

        for opt_val in str_series.into_iter() {
            match opt_val {
                Some(val) => {
                    let cleaned = val.trim();
                    // Try parsing as timestamp (could be seconds or milliseconds)
                    if let Ok(timestamp) = cleaned.parse::<i64>() {
                        // Check if it's seconds (typical range: 1e9 to 2e9 for recent dates)
                        if timestamp > 1_000_000_000 && timestamp < 2_000_000_000 {
                            timestamps.push(Some(timestamp * 1000)); // Convert seconds to milliseconds
                        } else if timestamp > 1_000_000_000_000 && timestamp < 2_000_000_000_000 {
                            timestamps.push(Some(timestamp)); // Already milliseconds
                        } else {
                            timestamps.push(None);
                        }
                    } else {
                        timestamps.push(None);
                    }
                }
                None => timestamps.push(None),
            }
        }

        let timestamp_series = Series::new(series.name().clone(), timestamps);
        return Ok(timestamp_series.cast(&DataType::Datetime(TimeUnit::Milliseconds, None))?);
    }

    // If already numeric, cast directly
    Ok(series.cast(&DataType::Datetime(TimeUnit::Milliseconds, None))?)
}

/// Convert string to boolean.
pub(crate) fn string_to_boolean(series: &Series) -> Result<Series> {
    let str_series = series.str()?;
    let mut result_vec: Vec<Option<bool>> = Vec::with_capacity(str_series.len());

    let true_values: HashSet<&str> = ["true", "t", "yes", "y", "1"].iter().copied().collect();
    let false_values: HashSet<&str> = ["false", "f", "no", "n", "0"].iter().copied().collect();

    for opt_val in str_series.into_iter() {
        match opt_val {
            Some(val) => {
                let cleaned = val.trim().to_lowercase();

                if true_values.contains(cleaned.as_str()) {
                    result_vec.push(Some(true));
                } else if false_values.contains(cleaned.as_str()) {
                    result_vec.push(Some(false));
                } else {
                    result_vec.push(None);
                }
            }
            None => result_vec.push(None),
        }
    }

    Ok(Series::new(series.name().clone(), result_vec))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to check if a value at index is null
    fn is_null_at(series: &Series, idx: usize) -> bool {
        matches!(series.get(idx).unwrap(), AnyValue::Null)
    }

    // Helper function to extract boolean from series
    fn get_bool_at(series: &Series, idx: usize) -> bool {
        match series.get(idx).unwrap() {
            AnyValue::Boolean(b) => b,
            _ => panic!("Expected boolean value"),
        }
    }

    // ========================================================================
    // string_to_numeric() tests - Float64
    // ========================================================================

    #[test]
    fn test_string_to_float64_basic() {
        let series = Series::new("values".into(), &["1.5", "2.5", "3.5"]);
        let result = string_to_numeric(&series, &DataType::Float64).unwrap();
        
        assert_eq!(result.dtype(), &DataType::Float64);
        assert_eq!(result.get(0).unwrap().try_extract::<f64>().unwrap(), 1.5);
        assert_eq!(result.get(1).unwrap().try_extract::<f64>().unwrap(), 2.5);
        assert_eq!(result.get(2).unwrap().try_extract::<f64>().unwrap(), 3.5);
    }

    #[test]
    fn test_string_to_float64_with_currency() {
        let series = Series::new("price".into(), &["$1,234.56", "€100.50", "£999.99"]);
        let result = string_to_numeric(&series, &DataType::Float64).unwrap();
        
        assert_eq!(result.dtype(), &DataType::Float64);
        assert_eq!(result.get(0).unwrap().try_extract::<f64>().unwrap(), 1234.56);
        assert_eq!(result.get(1).unwrap().try_extract::<f64>().unwrap(), 100.50);
        assert_eq!(result.get(2).unwrap().try_extract::<f64>().unwrap(), 999.99);
    }

    #[test]
    fn test_string_to_float64_with_percentage() {
        let series = Series::new("pct".into(), &["75%", "50.5%", "100%"]);
        let result = string_to_numeric(&series, &DataType::Float64).unwrap();
        
        assert_eq!(result.dtype(), &DataType::Float64);
        assert_eq!(result.get(0).unwrap().try_extract::<f64>().unwrap(), 75.0);
        assert_eq!(result.get(1).unwrap().try_extract::<f64>().unwrap(), 50.5);
        assert_eq!(result.get(2).unwrap().try_extract::<f64>().unwrap(), 100.0);
    }

    #[test]
    fn test_string_to_float64_with_whitespace() {
        let series = Series::new("values".into(), &["  42  ", " -3.14 ", "\t10.0\n"]);
        let result = string_to_numeric(&series, &DataType::Float64).unwrap();
        
        assert_eq!(result.get(0).unwrap().try_extract::<f64>().unwrap(), 42.0);
        assert_eq!(result.get(1).unwrap().try_extract::<f64>().unwrap(), -3.14);
        assert_eq!(result.get(2).unwrap().try_extract::<f64>().unwrap(), 10.0);
    }

    #[test]
    fn test_string_to_float64_with_error_markers() {
        let series = Series::new("values".into(), &["ERROR", "N/A", "null", "UNKNOWN", "#N/A"]);
        let result = string_to_numeric(&series, &DataType::Float64).unwrap();
        
        // All error markers should become null
        assert_eq!(result.null_count(), 5);
    }

    #[test]
    fn test_string_to_float64_with_empty_strings() {
        let series = Series::new("values".into(), &["", "  ", "42"]);
        let result = string_to_numeric(&series, &DataType::Float64).unwrap();
        
        assert!(is_null_at(&result, 0));
        assert!(is_null_at(&result, 1));
        assert_eq!(result.get(2).unwrap().try_extract::<f64>().unwrap(), 42.0);
    }

    #[test]
    fn test_string_to_float64_with_nulls() {
        let series = Series::new("values".into(), &[Some("1.0"), None, Some("3.0")]);
        let result = string_to_numeric(&series, &DataType::Float64).unwrap();
        
        assert_eq!(result.get(0).unwrap().try_extract::<f64>().unwrap(), 1.0);
        assert!(is_null_at(&result, 1));
        assert_eq!(result.get(2).unwrap().try_extract::<f64>().unwrap(), 3.0);
    }

    #[test]
    fn test_string_to_float64_negative_numbers() {
        let series = Series::new("values".into(), &["-1.5", "-100", "-.5"]);
        let result = string_to_numeric(&series, &DataType::Float64).unwrap();
        
        assert_eq!(result.get(0).unwrap().try_extract::<f64>().unwrap(), -1.5);
        assert_eq!(result.get(1).unwrap().try_extract::<f64>().unwrap(), -100.0);
        assert_eq!(result.get(2).unwrap().try_extract::<f64>().unwrap(), -0.5);
    }

    #[test]
    fn test_string_to_float64_scientific_notation() {
        let series = Series::new("values".into(), &["1e10", "2.5e-3", "1E6"]);
        let result = string_to_numeric(&series, &DataType::Float64).unwrap();
        
        assert_eq!(result.get(0).unwrap().try_extract::<f64>().unwrap(), 1e10);
        assert_eq!(result.get(1).unwrap().try_extract::<f64>().unwrap(), 2.5e-3);
        assert_eq!(result.get(2).unwrap().try_extract::<f64>().unwrap(), 1e6);
    }

    // ========================================================================
    // string_to_numeric() tests - Int64
    // ========================================================================

    #[test]
    fn test_string_to_int64_basic() {
        let series = Series::new("values".into(), &["1", "2", "3"]);
        let result = string_to_numeric(&series, &DataType::Int64).unwrap();
        
        assert_eq!(result.dtype(), &DataType::Int64);
        assert_eq!(result.get(0).unwrap().try_extract::<i64>().unwrap(), 1);
        assert_eq!(result.get(1).unwrap().try_extract::<i64>().unwrap(), 2);
        assert_eq!(result.get(2).unwrap().try_extract::<i64>().unwrap(), 3);
    }

    #[test]
    fn test_string_to_int64_truncates_floats() {
        let series = Series::new("values".into(), &["1.9", "2.1", "3.5"]);
        let result = string_to_numeric(&series, &DataType::Int64).unwrap();
        
        // Floats should be truncated to integers
        assert_eq!(result.get(0).unwrap().try_extract::<i64>().unwrap(), 1);
        assert_eq!(result.get(1).unwrap().try_extract::<i64>().unwrap(), 2);
        assert_eq!(result.get(2).unwrap().try_extract::<i64>().unwrap(), 3);
    }

    #[test]
    fn test_string_to_int64_with_commas() {
        let series = Series::new("values".into(), &["1,000", "1,000,000", "999"]);
        let result = string_to_numeric(&series, &DataType::Int64).unwrap();
        
        assert_eq!(result.get(0).unwrap().try_extract::<i64>().unwrap(), 1000);
        assert_eq!(result.get(1).unwrap().try_extract::<i64>().unwrap(), 1000000);
        assert_eq!(result.get(2).unwrap().try_extract::<i64>().unwrap(), 999);
    }

    #[test]
    fn test_string_to_int64_with_error_markers() {
        let series = Series::new("values".into(), &["ERROR", "42", "N/A"]);
        let result = string_to_numeric(&series, &DataType::Int64).unwrap();
        
        assert!(is_null_at(&result, 0));
        assert_eq!(result.get(1).unwrap().try_extract::<i64>().unwrap(), 42);
        assert!(is_null_at(&result, 2));
    }

    // ========================================================================
    // string_to_numeric() tests - Int32
    // ========================================================================

    #[test]
    fn test_string_to_int32_basic() {
        let series = Series::new("values".into(), &["100", "200", "300"]);
        let result = string_to_numeric(&series, &DataType::Int32).unwrap();
        
        assert_eq!(result.dtype(), &DataType::Int32);
        assert_eq!(result.get(0).unwrap().try_extract::<i32>().unwrap(), 100);
        assert_eq!(result.get(1).unwrap().try_extract::<i32>().unwrap(), 200);
        assert_eq!(result.get(2).unwrap().try_extract::<i32>().unwrap(), 300);
    }

    #[test]
    fn test_string_to_numeric_unsupported_dtype() {
        let series = Series::new("values".into(), &["1", "2", "3"]);
        let result = string_to_numeric(&series, &DataType::Boolean).unwrap();
        
        // Should return clone of original for unsupported types
        assert_eq!(result.dtype(), &DataType::String);
    }

    // ========================================================================
    // timestamp_to_datetime() tests
    // ========================================================================

    #[test]
    fn test_timestamp_to_datetime_seconds() {
        // Timestamp in seconds (10-digit, 2020-01-01)
        let series = Series::new("ts".into(), &["1577836800"]);
        let result = timestamp_to_datetime(&series).unwrap();
        
        // Should convert seconds to milliseconds
        assert!(matches!(result.dtype(), DataType::Datetime(_, _)));
    }

    #[test]
    fn test_timestamp_to_datetime_milliseconds() {
        // Timestamp in milliseconds (13-digit, 2020-01-01)
        let series = Series::new("ts".into(), &["1577836800000"]);
        let result = timestamp_to_datetime(&series).unwrap();
        
        assert!(matches!(result.dtype(), DataType::Datetime(_, _)));
    }

    #[test]
    fn test_timestamp_to_datetime_invalid_range() {
        // Timestamps outside valid range
        let series = Series::new("ts".into(), &["100", "999999999999999"]);
        let result = timestamp_to_datetime(&series).unwrap();
        
        // Should have nulls for invalid timestamps
        assert_eq!(result.null_count(), 2);
    }

    #[test]
    fn test_timestamp_to_datetime_with_nulls() {
        let series = Series::new("ts".into(), &[Some("1577836800"), None]);
        let result = timestamp_to_datetime(&series).unwrap();
        
        assert!(matches!(result.dtype(), DataType::Datetime(_, _)));
        // Original null should be preserved
        assert!(is_null_at(&result, 1));
    }

    #[test]
    fn test_timestamp_to_datetime_non_numeric_string() {
        let series = Series::new("ts".into(), &["not_a_timestamp", "abc"]);
        let result = timestamp_to_datetime(&series).unwrap();
        
        // Non-numeric strings should become null
        assert_eq!(result.null_count(), 2);
    }

    #[test]
    fn test_timestamp_to_datetime_numeric_series() {
        // If already numeric, should cast directly
        let series = Series::new("ts".into(), &[1577836800000_i64, 1577923200000_i64]);
        let result = timestamp_to_datetime(&series).unwrap();
        
        assert!(matches!(result.dtype(), DataType::Datetime(_, _)));
    }

    // ========================================================================
    // string_to_boolean() tests
    // ========================================================================

    #[test]
    fn test_string_to_boolean_true_values() {
        let series = Series::new("bool".into(), &["true", "TRUE", "True", "t", "T"]);
        let result = string_to_boolean(&series).unwrap();
        
        assert_eq!(result.dtype(), &DataType::Boolean);
        for i in 0..5 {
            assert!(get_bool_at(&result, i));
        }
    }

    #[test]
    fn test_string_to_boolean_false_values() {
        let series = Series::new("bool".into(), &["false", "FALSE", "False", "f", "F"]);
        let result = string_to_boolean(&series).unwrap();
        
        assert_eq!(result.dtype(), &DataType::Boolean);
        for i in 0..5 {
            assert!(!get_bool_at(&result, i));
        }
    }

    #[test]
    fn test_string_to_boolean_yes_no() {
        let series = Series::new("bool".into(), &["yes", "YES", "no", "NO", "y", "n"]);
        let result = string_to_boolean(&series).unwrap();
        
        assert!(get_bool_at(&result, 0));
        assert!(get_bool_at(&result, 1));
        assert!(!get_bool_at(&result, 2));
        assert!(!get_bool_at(&result, 3));
        assert!(get_bool_at(&result, 4));
        assert!(!get_bool_at(&result, 5));
    }

    #[test]
    fn test_string_to_boolean_numeric_strings() {
        let series = Series::new("bool".into(), &["1", "0"]);
        let result = string_to_boolean(&series).unwrap();
        
        assert!(get_bool_at(&result, 0));
        assert!(!get_bool_at(&result, 1));
    }

    #[test]
    fn test_string_to_boolean_with_whitespace() {
        let series = Series::new("bool".into(), &["  true  ", " false ", "\tyes\n"]);
        let result = string_to_boolean(&series).unwrap();
        
        assert!(get_bool_at(&result, 0));
        assert!(!get_bool_at(&result, 1));
        assert!(get_bool_at(&result, 2));
    }

    #[test]
    fn test_string_to_boolean_invalid_values() {
        let series = Series::new("bool".into(), &["maybe", "unknown", "2", "active"]);
        let result = string_to_boolean(&series).unwrap();
        
        // Invalid values should become null
        assert_eq!(result.null_count(), 4);
    }

    #[test]
    fn test_string_to_boolean_with_nulls() {
        let series = Series::new("bool".into(), &[Some("true"), None, Some("false")]);
        let result = string_to_boolean(&series).unwrap();
        
        assert!(get_bool_at(&result, 0));
        assert!(is_null_at(&result, 1));
        assert!(!get_bool_at(&result, 2));
    }

    #[test]
    fn test_string_to_boolean_mixed_valid_invalid() {
        let series = Series::new("bool".into(), &["true", "invalid", "no", "garbage"]);
        let result = string_to_boolean(&series).unwrap();
        
        assert!(get_bool_at(&result, 0));
        assert!(is_null_at(&result, 1));
        assert!(!get_bool_at(&result, 2));
        assert!(is_null_at(&result, 3));
    }
}
