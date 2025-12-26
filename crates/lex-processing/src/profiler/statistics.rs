//! Statistical analysis functions for column profiling.

use anyhow::Result;
use polars::prelude::*;
use std::collections::HashMap;

/// Extract statistical characteristics from a column.
pub(crate) fn extract_column_characteristics(
    series: &Series,
    inferred_type: &str,
    unique_count: usize,
) -> Result<HashMap<String, serde_json::Value>> {
    let mut characteristics = HashMap::new();

    let cardinality = if unique_count < 10 {
        "low"
    } else if unique_count < 50 {
        "medium"
    } else {
        "high"
    };
    characteristics.insert("cardinality".to_string(), serde_json::json!(cardinality));

    if inferred_type == "numeric" {
        let non_null = series.drop_nulls();
        if !non_null.is_empty() {
            let float_series = non_null.cast(&DataType::Float64)?;
            let mean = float_series.mean().unwrap_or(0.0);

            let std = calculate_std(&float_series)?;
            let skewness = calculate_skewness(&float_series)?;

            characteristics.insert("mean".to_string(), serde_json::json!(mean));
            characteristics.insert("std".to_string(), serde_json::json!(std));
            characteristics.insert("skewness".to_string(), serde_json::json!(skewness));
            characteristics.insert(
                "has_outliers".to_string(),
                serde_json::json!(detect_outliers(&float_series)?),
            );

            let distribution = if skewness.abs() < 1.0 {
                "normal"
            } else {
                "skewed"
            };
            characteristics.insert("distribution".to_string(), serde_json::json!(distribution));
        }
    } else if inferred_type == "string" || inferred_type == "categorical" {
        // Handle string/categorical columns
        let non_null = series.drop_nulls();
        if !non_null.is_empty()
            && let Ok(value_counts_df) = non_null.value_counts(true, false, "count".into(), false)
            && value_counts_df.height() > 0
            && let (Ok(counts_col), Ok(values_col)) = (
                value_counts_df.column("count"),
                value_counts_df.column(non_null.name()),
            )
            && !counts_col.is_empty()
        {
            let most_freq_value = values_col.get(0)?;
            let most_freq = format!("{}", most_freq_value);

            characteristics.insert("most_frequent".to_string(), serde_json::json!(most_freq));

            let counts_col_series = counts_col.as_materialized_series();
            let counts: Vec<f64> = counts_col_series
                .f64()
                .map(|ca| ca.into_iter().flatten().collect())
                .unwrap_or_default();

            if counts.len() > 1 {
                let counts_mean: f64 = counts.iter().sum::<f64>() / counts.len() as f64;
                let variance: f64 = counts
                    .iter()
                    .map(|c| (c - counts_mean).powi(2))
                    .sum::<f64>()
                    / counts.len() as f64;
                let counts_std = variance.sqrt();

                let freq_dist = if counts_std < counts_mean {
                    "balanced"
                } else {
                    "imbalanced"
                };
                characteristics.insert(
                    "frequency_distribution".to_string(),
                    serde_json::json!(freq_dist),
                );
            }
        }
    }

    Ok(characteristics)
}

/// Calculate standard deviation of a series.
pub(crate) fn calculate_std(series: &Series) -> Result<f64> {
    let mean = series.mean().unwrap_or(0.0);
    let n = series.len() as f64;

    if n <= 1.0 {
        return Ok(0.0);
    }

    let float_series = series.f64()?;
    let variance: f64 = float_series
        .into_iter()
        .filter_map(|v| v.map(|val| (val - mean).powi(2)))
        .sum::<f64>()
        / (n - 1.0);

    Ok(variance.sqrt())
}

/// Calculate skewness of a series.
pub(crate) fn calculate_skewness(series: &Series) -> Result<f64> {
    let mean = series.mean().unwrap_or(0.0);
    let std = calculate_std(series)?;

    if std == 0.0 {
        return Ok(0.0);
    }

    let n = series.len() as f64;
    let float_series = series.f64()?;

    let skew_sum: f64 = float_series
        .into_iter()
        .filter_map(|v| v.map(|val| ((val - mean) / std).powi(3)))
        .sum();

    Ok(skew_sum / n)
}

/// Detect if a series has outliers using IQR method.
pub(crate) fn detect_outliers(series: &Series) -> Result<bool> {
    // Sort and calculate quartiles manually
    let sorted = series.sort(SortOptions::default())?;
    let n = sorted.len();

    if n < 4 {
        return Ok(false);
    }

    let q1_idx = (n as f64 * 0.25) as usize;
    let q3_idx = (n as f64 * 0.75) as usize;

    let q1_val = sorted.get(q1_idx)?.try_extract::<f64>().unwrap_or(0.0);
    let q3_val = sorted.get(q3_idx)?.try_extract::<f64>().unwrap_or(0.0);
    let iqr = q3_val - q1_val;

    let lower_bound = q1_val - 1.5 * iqr;
    let upper_bound = q3_val + 1.5 * iqr;

    let float_series = series.f64()?;
    let outlier_count: usize = float_series
        .into_iter()
        .filter(|v| {
            if let Some(val) = v {
                *val < lower_bound || *val > upper_bound
            } else {
                false
            }
        })
        .count();

    Ok(outlier_count > series.len() / 20)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== calculate_std tests ====================

    #[test]
    fn test_calculate_std_basic() {
        // Values: 1, 2, 3, 4, 5
        // Mean = 3, Variance = ((1-3)^2 + (2-3)^2 + (3-3)^2 + (4-3)^2 + (5-3)^2) / 4 = 10/4 = 2.5
        // Std = sqrt(2.5) ≈ 1.58
        let series = Series::new("val".into(), &[1.0f64, 2.0, 3.0, 4.0, 5.0]);
        let std = calculate_std(&series).unwrap();
        assert!((std - 1.58).abs() < 0.1);
    }

    #[test]
    fn test_calculate_std_single_value() {
        let series = Series::new("val".into(), &[5.0f64]);
        let std = calculate_std(&series).unwrap();
        assert_eq!(std, 0.0);
    }

    #[test]
    fn test_calculate_std_identical_values() {
        let series = Series::new("val".into(), &[5.0f64, 5.0, 5.0, 5.0]);
        let std = calculate_std(&series).unwrap();
        assert_eq!(std, 0.0);
    }

    #[test]
    fn test_calculate_std_empty_returns_zero() {
        let series: Series = Series::new("val".into(), Vec::<f64>::new());
        let std = calculate_std(&series).unwrap();
        assert_eq!(std, 0.0);
    }

    // ==================== calculate_skewness tests ====================

    #[test]
    fn test_calculate_skewness_symmetric() {
        // Symmetric distribution should have skewness close to 0
        let series = Series::new("val".into(), &[1.0f64, 2.0, 3.0, 4.0, 5.0]);
        let skew = calculate_skewness(&series).unwrap();
        assert!(skew.abs() < 0.1);
    }

    #[test]
    fn test_calculate_skewness_positive() {
        // Right-skewed data (long tail on the right)
        let series = Series::new("val".into(), &[1.0f64, 1.0, 1.0, 1.0, 10.0]);
        let skew = calculate_skewness(&series).unwrap();
        assert!(skew > 0.0);
    }

    #[test]
    fn test_calculate_skewness_zero_std() {
        // All same values - std is 0, skewness should be 0
        let series = Series::new("val".into(), &[5.0f64, 5.0, 5.0, 5.0]);
        let skew = calculate_skewness(&series).unwrap();
        assert_eq!(skew, 0.0);
    }

    // ==================== detect_outliers tests ====================

    #[test]
    fn test_detect_outliers_with_outlier() {
        // Data with clear outlier
        let series = Series::new(
            "val".into(),
            &[1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0],
        );
        let has_outliers = detect_outliers(&series).unwrap();
        assert!(has_outliers);
    }

    #[test]
    fn test_detect_outliers_no_outlier() {
        // Data with no outliers (values within 1.5*IQR)
        let series = Series::new(
            "val".into(),
            &[1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0],
        );
        let has_outliers = detect_outliers(&series).unwrap();
        assert!(!has_outliers);
    }

    #[test]
    fn test_detect_outliers_small_sample() {
        // Less than 4 elements - can't calculate quartiles meaningfully
        let series = Series::new("val".into(), &[1.0f64, 2.0, 100.0]);
        let has_outliers = detect_outliers(&series).unwrap();
        assert!(!has_outliers);
    }

    #[test]
    fn test_detect_outliers_iqr_zero() {
        // All same values - IQR = 0, any different value is technically an outlier
        // But with threshold of 5%, might not trigger
        let series = Series::new("val".into(), &[5.0f64, 5.0, 5.0, 5.0, 5.0]);
        let has_outliers = detect_outliers(&series).unwrap();
        // With all same values, no outliers
        assert!(!has_outliers);
    }

    // ==================== extract_column_characteristics tests ====================

    #[test]
    fn test_characteristics_numeric_column() {
        let series = Series::new("price".into(), &[10.0f64, 20.0, 30.0, 40.0, 50.0]);
        let chars = extract_column_characteristics(&series, "numeric", 5).unwrap();

        assert!(chars.contains_key("cardinality"));
        assert!(chars.contains_key("mean"));
        assert!(chars.contains_key("std"));
        assert!(chars.contains_key("skewness"));
        assert!(chars.contains_key("has_outliers"));
        assert!(chars.contains_key("distribution"));

        // Mean should be 30
        let mean = chars["mean"].as_f64().unwrap();
        assert!((mean - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_characteristics_string_column() {
        let series = Series::new("category".into(), &["a", "b", "a", "b", "a"]);
        let chars = extract_column_characteristics(&series, "string", 2).unwrap();

        assert!(chars.contains_key("cardinality"));
        assert!(chars.contains_key("most_frequent"));
        // "a" appears 3 times, "b" appears 2 times
        let most_freq = chars["most_frequent"].as_str().unwrap();
        assert!(most_freq.contains("a")); // "a" is most frequent
    }

    #[test]
    fn test_characteristics_low_cardinality() {
        let series = Series::new("val".into(), &[1.0f64, 2.0, 3.0]);
        let chars = extract_column_characteristics(&series, "numeric", 3).unwrap();

        let cardinality = chars["cardinality"].as_str().unwrap();
        assert_eq!(cardinality, "low"); // < 10 unique values
    }

    #[test]
    fn test_characteristics_medium_cardinality() {
        let series = Series::new("val".into(), &[1.0f64, 2.0, 3.0]);
        let chars = extract_column_characteristics(&series, "numeric", 25).unwrap();

        let cardinality = chars["cardinality"].as_str().unwrap();
        assert_eq!(cardinality, "medium"); // 10-50 unique values
    }

    #[test]
    fn test_characteristics_high_cardinality() {
        let series = Series::new("val".into(), &[1.0f64, 2.0, 3.0]);
        let chars = extract_column_characteristics(&series, "numeric", 100).unwrap();

        let cardinality = chars["cardinality"].as_str().unwrap();
        assert_eq!(cardinality, "high"); // > 50 unique values
    }

    #[test]
    fn test_characteristics_empty_series() {
        let series: Series = Series::new("val".into(), Vec::<f64>::new());
        let chars = extract_column_characteristics(&series, "numeric", 0).unwrap();

        // Should have cardinality but numeric stats might be missing
        assert!(chars.contains_key("cardinality"));
    }

    #[test]
    fn test_characteristics_distribution_normal() {
        // Symmetric data - skewness close to 0 -> normal distribution
        let series = Series::new("val".into(), &[1.0f64, 2.0, 3.0, 4.0, 5.0]);
        let chars = extract_column_characteristics(&series, "numeric", 5).unwrap();

        let dist = chars["distribution"].as_str().unwrap();
        assert_eq!(dist, "normal");
    }

    #[test]
    fn test_characteristics_distribution_skewed() {
        // Heavily skewed data
        let series = Series::new("val".into(), &[1.0f64, 1.0, 1.0, 1.0, 100.0]);
        let chars = extract_column_characteristics(&series, "numeric", 2).unwrap();

        let dist = chars["distribution"].as_str().unwrap();
        assert_eq!(dist, "skewed");
    }
}
