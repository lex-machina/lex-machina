use crate::types::{DataQualityIssue, DatasetProfile, SolutionOption};
use anyhow::Result;
use polars::prelude::*;
use std::collections::HashMap;

pub struct DataQualityAnalyzer;

impl DataQualityAnalyzer {
    pub fn identify_issues(
        dataset_profile: &DatasetProfile,
        df: &DataFrame,
    ) -> Result<Vec<DataQualityIssue>> {
        let mut issues = Vec::new();

        // Outlier issues
        issues.extend(Self::analyze_outliers(dataset_profile, df)?);

        // Problem type selection issue
        issues.push(Self::create_problem_type_selection_issue());

        // Missing values issues
        issues.extend(Self::analyze_missing_values(dataset_profile, df)?);

        Ok(issues)
    }

    fn analyze_outliers(profile: &DatasetProfile, df: &DataFrame) -> Result<Vec<DataQualityIssue>> {
        let mut issues = Vec::new();
        let mut numeric_cols_with_outliers = Vec::new();
        let mut outlier_details = HashMap::new();

        for col in &profile.column_profiles {
            if col.inferred_type == "numeric"
                && col
                    .characteristics
                    .get("has_outliers")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            {
                numeric_cols_with_outliers.push(col.name.clone());

                // Get detailed outlier information
                let col = df.column(&col.name)?;
                let series = col.as_materialized_series().drop_nulls();
                if !series.is_empty() {
                    let float_series = series.cast(&DataType::Float64)?;

                    // Calculate quartiles manually
                    let sorted = float_series.sort(SortOptions::default())?;
                    let n = sorted.len();
                    let q1_idx = (n as f64 * 0.25) as usize;
                    let q3_idx = (n as f64 * 0.75) as usize;

                    let q1_val = sorted.get(q1_idx)?.try_extract::<f64>().unwrap_or(0.0);
                    let q3_val = sorted.get(q3_idx)?.try_extract::<f64>().unwrap_or(0.0);
                    let iqr = q3_val - q1_val;

                    let lower_bound = q1_val - 1.5 * iqr;
                    let upper_bound = q3_val + 1.5 * iqr;

                    let f64_series = float_series.f64()?;
                    let outliers_low: Vec<f64> = f64_series
                        .into_iter()
                        .filter_map(|v| v.filter(|&val| val < lower_bound))
                        .take(3)
                        .collect();

                    let outliers_high: Vec<f64> = f64_series
                        .into_iter()
                        .filter_map(|v| v.filter(|&val| val > upper_bound))
                        .take(3)
                        .collect();

                    let normal_min = f64_series
                        .into_iter()
                        .filter_map(|v| v.filter(|&val| val >= lower_bound))
                        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                        .unwrap_or(0.0);

                    let normal_max = f64_series
                        .into_iter()
                        .filter_map(|v| v.filter(|&val| val <= upper_bound))
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                        .unwrap_or(0.0);

                    let mut outlier_examples = Vec::new();
                    outlier_examples.extend(outliers_low.iter().map(|v| format!("{:.2}", v)));
                    outlier_examples.extend(outliers_high.iter().map(|v| format!("{:.2}", v)));
                    outlier_examples.truncate(5);

                    let outlier_count = f64_series
                        .into_iter()
                        .filter(|v| {
                            if let Some(val) = v {
                                *val < lower_bound || *val > upper_bound
                            } else {
                                false
                            }
                        })
                        .count();

                    let mut details = HashMap::new();
                    details.insert(
                        "normal_range".to_string(),
                        serde_json::json!(format!("{:.2} to {:.2}", normal_min, normal_max)),
                    );
                    details.insert(
                        "outlier_examples".to_string(),
                        serde_json::json!(outlier_examples),
                    );
                    details.insert(
                        "outlier_count".to_string(),
                        serde_json::json!(outlier_count),
                    );

                    outlier_details.insert(col.name().clone().to_string(), details);
                }
            }
        }

        if !numeric_cols_with_outliers.is_empty() {
            let mut description_parts = Vec::new();
            for col_name in &numeric_cols_with_outliers {
                if let Some(details) = outlier_details.get(col_name) {
                    let normal_range: Option<&str> = details
                        .get("normal_range")
                        .and_then(|v: &serde_json::Value| v.as_str());
                    let outlier_count: u64 = details
                        .get("outlier_count")
                        .and_then(|v: &serde_json::Value| v.as_u64())
                        .unwrap_or(0);
                    let examples: String = details
                        .get("outlier_examples")
                        .and_then(|v: &serde_json::Value| v.as_array())
                        .map(|arr: &Vec<serde_json::Value>| {
                            arr.iter()
                                .filter_map(|v: &serde_json::Value| v.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();

                    description_parts.push(format!(
                        "In column '{}', values normally range from {}, but there are {} outliers with values like: {}",
                        col_name, normal_range.unwrap_or("unknown"), outlier_count, examples
                    ));
                }
            }

            let full_description = format!(
                "Outliers detected in {} numeric columns:\n{}",
                numeric_cols_with_outliers.len(),
                description_parts.join("\n")
            );

            let mut detection_details = HashMap::new();
            detection_details.insert(
                "outlier_details".to_string(),
                serde_json::json!(outlier_details),
            );

            issues.push(DataQualityIssue {
                issue_type: "outliers_detected".to_string(),
                severity: "medium".to_string(),
                affected_columns: numeric_cols_with_outliers,
                description: full_description,
                business_impact: "Outliers may represent errors or important edge cases"
                    .to_string(),
                detection_details,
                suggested_solutions: vec![
                    SolutionOption {
                        option: "keep_outliers".to_string(),
                        description: "Keep outliers - they might represent important patterns"
                            .to_string(),
                        pros: Some(
                            "No information loss, captures full business reality".to_string(),
                        ),
                        cons: Some("May hurt model performance on typical cases".to_string()),
                        best_for: None,
                    },
                    SolutionOption {
                        option: "cap_outliers".to_string(),
                        description: "Cap extreme values at 5th/95th percentiles".to_string(),
                        pros: Some(
                            "Reduces extreme influence while preserving patterns".to_string(),
                        ),
                        cons: Some("Some information loss at the extremes".to_string()),
                        best_for: None,
                    },
                    SolutionOption {
                        option: "remove_outliers".to_string(),
                        description: "Remove rows containing extreme outliers".to_string(),
                        pros: Some("Clean dataset optimized for typical patterns".to_string()),
                        cons: Some("Data loss, may miss important edge cases".to_string()),
                        best_for: None,
                    },
                ],
            });
        }

        Ok(issues)
    }

    fn analyze_missing_values(
        profile: &DatasetProfile,
        df: &DataFrame,
    ) -> Result<Vec<DataQualityIssue>> {
        let mut issues = Vec::new();

        let columns_with_missing: Vec<_> = profile
            .column_profiles
            .iter()
            .filter(|col| {
                col.null_percentage > 10.0
                    && col.null_percentage <= 50.0
                    && col.inferred_role != "identifier"
            })
            .collect();

        if !columns_with_missing.is_empty() {
            for col_profile in columns_with_missing {
                let total_rows = df.height();
                let missing_count = col_profile.null_count;
                let missing_pct = col_profile.null_percentage;

                // *** KEY FIX: Get ACTUAL dtype from DataFrame ***
                let actual_dtype = if let Ok(col) = df.column(&col_profile.name) {
                    match col.dtype() {
                        DataType::Int8
                        | DataType::Int16
                        | DataType::Int32
                        | DataType::Int64
                        | DataType::UInt8
                        | DataType::UInt16
                        | DataType::UInt32
                        | DataType::UInt64
                        | DataType::Float32
                        | DataType::Float64 => "numeric",
                        DataType::Datetime(_, _) | DataType::Date => "datetime",
                        DataType::Boolean => "binary",
                        _ => "string",
                    }
                } else {
                    "string"
                };

                // Get sample values
                let sample_values = if let Ok(col) = df.column(&col_profile.name) {
                    let series = col.as_materialized_series();
                    let non_null = series.drop_nulls();
                    if !non_null.is_empty() {
                        let sample_size = std::cmp::min(5, non_null.len());
                        (0..sample_size)
                            .map(|i| {
                                non_null
                                    .get(i)
                                    .map(|v| format!("{}", v))
                                    .unwrap_or_default()
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    } else {
                        "No non-null values".to_string()
                    }
                } else {
                    "Unable to read values".to_string()
                };

                // Build detection details
                let mut detection_details = HashMap::new();
                detection_details.insert(
                    "column_name".to_string(),
                    serde_json::json!(col_profile.name),
                );
                detection_details
                    .insert("actual_dtype".to_string(), serde_json::json!(actual_dtype));
                detection_details.insert(
                    "inferred_type".to_string(),
                    serde_json::json!(col_profile.inferred_type),
                );
                detection_details.insert(
                    "missing_count".to_string(),
                    serde_json::json!(missing_count),
                );
                detection_details.insert(
                    "missing_percentage".to_string(),
                    serde_json::json!(missing_pct),
                );
                detection_details.insert("total_rows".to_string(), serde_json::json!(total_rows));
                detection_details.insert(
                    "unique_values".to_string(),
                    serde_json::json!(col_profile.unique_count),
                );
                detection_details.insert(
                    "sample_values".to_string(),
                    serde_json::json!(sample_values),
                );

                // *** Use ACTUAL dtype to determine strategies ***
                let suggested_solutions = Self::create_column_specific_strategies(
                    actual_dtype, // Use actual dtype, not inferred_type
                    col_profile.unique_count,
                    total_rows,
                    missing_pct,
                );

                let description = format!(
                    "Column '{}' has {:.1}% missing values ({} out of {} rows).\n\
                    Data type: {} ({})\n\
                    Unique values: {}\n\
                    Sample values: {}",
                    col_profile.name,
                    missing_pct,
                    missing_count,
                    total_rows,
                    actual_dtype,
                    if actual_dtype == "numeric" {
                        "Integer or Float"
                    } else if actual_dtype == "string" {
                        "Text/Categorical"
                    } else if actual_dtype == "datetime" {
                        "Date/Time"
                    } else {
                        "Other"
                    },
                    col_profile.unique_count,
                    sample_values
                );

                issues.push(DataQualityIssue {
                    issue_type: format!("missing_values_{}", col_profile.name),
                    severity: if missing_pct > 30.0 { "high" } else { "medium" }.to_string(),
                    affected_columns: vec![col_profile.name.clone()],
                    description,
                    business_impact: format!(
                        "Missing {:.1}% of values in '{}' ({}). \
                        The chosen imputation method will affect model performance.",
                        missing_pct, col_profile.name, actual_dtype
                    ),
                    detection_details,
                    suggested_solutions,
                });
            }
        }

        Ok(issues)
    }

    /// Create strategies based on ACTUAL dtype from DataFrame
    fn create_column_specific_strategies(
        actual_dtype: &str,
        unique_count: usize,
        total_rows: usize,
        missing_pct: f64,
    ) -> Vec<SolutionOption> {
        let mut strategies = Vec::new();

        match actual_dtype {
            "numeric" => {
                // For NUMERIC columns - offer statistical methods
                strategies.push(SolutionOption {
                    option: "median_imputation".to_string(),
                    description: "Fill with median value - Robust to outliers".to_string(),
                    pros: Some("Fast, robust to outliers, preserves central tendency".to_string()),
                    cons: Some("Reduces variance, ignores column relationships".to_string()),
                    best_for: Some(format!(
                        "Numeric column with {} unique values",
                        unique_count
                    )),
                });

                strategies.push(SolutionOption {
                    option: "mean_imputation".to_string(),
                    description: "Fill with mean value - Best for normal distributions".to_string(),
                    pros: Some("Fast, preserves overall mean, simple to understand".to_string()),
                    cons: Some("Sensitive to outliers, reduces variance".to_string()),
                    best_for: Some("Normally distributed numeric data".to_string()),
                });

                if total_rows >= 100 && missing_pct < 40.0 {
                    strategies.push(SolutionOption {
                        option: "knn_imputation".to_string(),
                        description: format!(
                            "K-Nearest Neighbors (K={}) - Predict from similar rows",
                            ((total_rows as f64).sqrt().floor() as usize).clamp(3, 10)
                        ),
                        pros: Some("Most accurate for numeric data, captures relationships".to_string()),
                        cons: Some("Computationally expensive, slower and can be biased if features aren't scaled or if irrelevant features dominate distance calculations.".to_string()),
                        best_for: Some("Numeric data with correlated columns and small datasets".to_string()),
                    });
                }
            }
            "string" | "categorical" => {
                // For STRING/CATEGORICAL columns
                strategies.push(SolutionOption {
                    option: "mode_imputation".to_string(),
                    description: "Fill with most frequent value".to_string(),
                    pros: Some(format!(
                        "Fast, preserves distribution ({} categories)",
                        unique_count
                    )),
                    cons: Some("May bias towards majority class".to_string()),
                    best_for: Some("Categorical text data with clear mode".to_string()),
                });

                if unique_count <= 10 {
                    strategies.push(SolutionOption {
                        option: "category_indicator".to_string(),
                        description: format!(
                            "Add 'Missing' as new category ({} existing)",
                            unique_count
                        ),
                        pros: Some("Preserves missingness information, no data loss".to_string()),
                        cons: Some("Adds complexity, may not work with all models".to_string()),
                        best_for: Some("When missingness itself is informative".to_string()),
                    });
                }

                strategies.push(SolutionOption {
                    option: "constant_imputation".to_string(),
                    description: "Fill with constant value like 'Unknown'".to_string(),
                    pros: Some("Simple, clearly marks imputed values".to_string()),
                    cons: Some("May introduce artificial patterns".to_string()),
                    best_for: Some("Text data where missingness is not informative".to_string()),
                });
            }
            "datetime" | "date" => {
                strategies.push(SolutionOption {
                    option: "forward_fill".to_string(),
                    description: "Fill with previous non-null value".to_string(),
                    pros: Some("Preserves temporal continuity, good for time series".to_string()),
                    cons: Some("May propagate stale values, assumes ordering".to_string()),
                    best_for: Some("Time-ordered datetime data".to_string()),
                });

                strategies.push(SolutionOption {
                    option: "interpolation".to_string(),
                    description: "Interpolate between surrounding values".to_string(),
                    pros: Some("Smooth transitions, respects temporal trends".to_string()),
                    cons: Some("May not reflect actual patterns".to_string()),
                    best_for: Some("Datetime with regular intervals".to_string()),
                });
            }
            _ => {
                strategies.push(SolutionOption {
                    option: "constant_imputation".to_string(),
                    description: "Fill with a constant value".to_string(),
                    pros: Some("Simple, safe fallback".to_string()),
                    cons: Some("Generic approach".to_string()),
                    best_for: Some("Unknown or mixed types".to_string()),
                });
            }
        }

        strategies
    }

    fn create_problem_type_selection_issue() -> DataQualityIssue {
        DataQualityIssue {
            issue_type: "problem_type_selection".to_string(),
            severity: "high".to_string(),
            affected_columns: Vec::new(),
            description: "Please select the type of machine learning problem you want to solve".to_string(),
            business_impact: "Determines the type of analysis and algorithms available".to_string(),
            detection_details: HashMap::new(),
            suggested_solutions: vec![
                SolutionOption {
                    option: "classification".to_string(),
                    description: "Predict categories/classes (e.g., spam/not spam, survived/not survived, disease/healthy)".to_string(),
                    pros: Some("Clear categorical predictions, well-established algorithms, easy to interpret results".to_string()),
                    cons: Some("Requires labeled target variable with discrete categories".to_string()),
                    best_for: None,
                },
                SolutionOption {
                    option: "regression".to_string(),
                    description: "Predict continuous numerical values (e.g., price, age, temperature, sales amount)".to_string(),
                    pros: Some("Precise numerical predictions, captures relationships with continuous outcomes".to_string()),
                    cons: Some("Requires numerical target variable, predictions may need rounding for practical use".to_string()),
                    best_for: None,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ColumnProfile;
    use serde_json::json;

    /// Helper to create a ColumnProfile for testing
    fn create_column_profile(
        name: &str,
        dtype: &str,
        inferred_type: &str,
        null_count: usize,
        null_percentage: f64,
        unique_count: usize,
        inferred_role: &str,
        has_outliers: bool,
    ) -> ColumnProfile {
        let mut characteristics = HashMap::new();
        characteristics.insert("has_outliers".to_string(), json!(has_outliers));

        ColumnProfile {
            name: name.to_string(),
            dtype: dtype.to_string(),
            inferred_type: inferred_type.to_string(),
            null_count,
            null_percentage,
            unique_count,
            sample_values: vec![],
            inferred_role: inferred_role.to_string(),
            characteristics,
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

    // ==================== identify_issues tests ====================

    #[test]
    fn test_identify_issues_always_has_problem_type_selection() {
        let profile = create_profile(vec![create_column_profile(
            "col", "Int64", "numeric", 0, 0.0, 10, "feature", false,
        )]);
        let df = df!["col" => [1, 2, 3, 4, 5]].unwrap();

        let issues = DataQualityAnalyzer::identify_issues(&profile, &df).unwrap();

        assert!(
            issues
                .iter()
                .any(|i| i.issue_type == "problem_type_selection")
        );
    }

    #[test]
    fn test_identify_issues_detects_missing_values() {
        let profile = create_profile(vec![create_column_profile(
            "col", "Float64", "numeric", 20, 20.0, 5, "feature", false,
        )]);
        let df = df!["col" => [Some(1.0), Some(2.0), None, None, Some(5.0)]].unwrap();

        let issues = DataQualityAnalyzer::identify_issues(&profile, &df).unwrap();

        assert!(
            issues
                .iter()
                .any(|i| i.issue_type.contains("missing_values"))
        );
    }

    #[test]
    fn test_identify_issues_skips_low_missing_percentage() {
        let profile = create_profile(vec![create_column_profile(
            "col", "Int64", "numeric", 5, 5.0, 10, "feature", false,
        )]);
        let df = df!["col" => [1, 2, 3, 4, 5]].unwrap();

        let issues = DataQualityAnalyzer::identify_issues(&profile, &df).unwrap();

        // Should not have missing_values issue for <10%
        assert!(
            !issues
                .iter()
                .any(|i| i.issue_type.contains("missing_values"))
        );
    }

    #[test]
    fn test_identify_issues_skips_identifier_columns() {
        let profile = create_profile(vec![create_column_profile(
            "id",
            "Int64",
            "numeric",
            20,
            20.0,
            100,
            "identifier",
            false,
        )]);
        let df = df!["id" => [1, 2, 3, 4, 5]].unwrap();

        let issues = DataQualityAnalyzer::identify_issues(&profile, &df).unwrap();

        // Should not have missing_values issue for identifier columns
        assert!(
            !issues
                .iter()
                .any(|i| i.issue_type.contains("missing_values_id"))
        );
    }

    // ==================== analyze_outliers tests ====================

    #[test]
    fn test_analyze_outliers_detects_outliers() {
        let profile = create_profile(vec![create_column_profile(
            "value", "Float64", "numeric", 0, 0.0, 10, "feature", true,
        )]);
        // Data with clear outlier
        let df = df!["value" => [1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0]].unwrap();

        let issues = DataQualityAnalyzer::analyze_outliers(&profile, &df).unwrap();

        assert!(!issues.is_empty());
        assert_eq!(issues[0].issue_type, "outliers_detected");
        assert!(issues[0].affected_columns.contains(&"value".to_string()));
    }

    #[test]
    fn test_analyze_outliers_no_outliers() {
        let profile = create_profile(vec![create_column_profile(
            "value", "Float64", "numeric", 0, 0.0, 10, "feature",
            false, // has_outliers = false
        )]);
        let df = df!["value" => [1.0f64, 2.0, 3.0, 4.0, 5.0]].unwrap();

        let issues = DataQualityAnalyzer::analyze_outliers(&profile, &df).unwrap();

        assert!(issues.is_empty());
    }

    #[test]
    fn test_analyze_outliers_has_three_solutions() {
        let profile = create_profile(vec![create_column_profile(
            "value", "Float64", "numeric", 0, 0.0, 10, "feature", true,
        )]);
        let df = df!["value" => [1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0]].unwrap();

        let issues = DataQualityAnalyzer::analyze_outliers(&profile, &df).unwrap();

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggested_solutions.len(), 3);
        // Check that all three strategies are present
        let options: Vec<_> = issues[0]
            .suggested_solutions
            .iter()
            .map(|s| s.option.as_str())
            .collect();
        assert!(options.contains(&"keep_outliers"));
        assert!(options.contains(&"cap_outliers"));
        assert!(options.contains(&"remove_outliers"));
    }

    // ==================== analyze_missing_values tests ====================

    #[test]
    fn test_analyze_missing_values_numeric() {
        let profile = create_profile(vec![create_column_profile(
            "price", "Float64", "numeric", 20, 20.0, 50, "feature", false,
        )]);
        let df = df!["price" => [Some(10.0), Some(20.0), None, None, Some(50.0)]].unwrap();

        let issues = DataQualityAnalyzer::analyze_missing_values(&profile, &df).unwrap();

        assert_eq!(issues.len(), 1);
        assert!(issues[0].issue_type.contains("missing_values"));
        // Should suggest median/mean for numeric
        let options: Vec<_> = issues[0]
            .suggested_solutions
            .iter()
            .map(|s| s.option.as_str())
            .collect();
        assert!(options.contains(&"median_imputation"));
    }

    #[test]
    fn test_analyze_missing_values_string() {
        let profile = create_profile(vec![create_column_profile(
            "category", "String", "string", 20, 20.0, 5, "feature", false,
        )]);
        let df = df!["category" => [Some("a"), Some("b"), None, None, Some("c")]].unwrap();

        let issues = DataQualityAnalyzer::analyze_missing_values(&profile, &df).unwrap();

        assert_eq!(issues.len(), 1);
        // Should suggest mode for categorical
        let options: Vec<_> = issues[0]
            .suggested_solutions
            .iter()
            .map(|s| s.option.as_str())
            .collect();
        assert!(options.contains(&"mode_imputation"));
    }

    #[test]
    fn test_analyze_missing_values_severity_high() {
        let profile = create_profile(vec![create_column_profile(
            "col", "Float64", "numeric", 35, 35.0, 50, "feature", false,
        )]);
        let df = df!["col" => [Some(1.0), Some(2.0), None, None, None]].unwrap();

        let issues = DataQualityAnalyzer::analyze_missing_values(&profile, &df).unwrap();

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, "high"); // >30% missing
    }

    #[test]
    fn test_analyze_missing_values_severity_medium() {
        let profile = create_profile(vec![create_column_profile(
            "col", "Float64", "numeric", 15, 15.0, 50, "feature", false,
        )]);
        let df = df!["col" => [Some(1.0), Some(2.0), None, Some(4.0), Some(5.0)]].unwrap();

        let issues = DataQualityAnalyzer::analyze_missing_values(&profile, &df).unwrap();

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, "medium"); // 10-30% missing
    }

    // ==================== create_column_specific_strategies tests ====================

    #[test]
    fn test_strategies_numeric_basic() {
        let strategies =
            DataQualityAnalyzer::create_column_specific_strategies("numeric", 50, 100, 15.0);

        assert!(strategies.len() >= 2);
        let options: Vec<_> = strategies.iter().map(|s| s.option.as_str()).collect();
        assert!(options.contains(&"median_imputation"));
        assert!(options.contains(&"mean_imputation"));
    }

    #[test]
    fn test_strategies_numeric_knn_available() {
        // KNN requires >= 100 rows and < 40% missing
        let strategies =
            DataQualityAnalyzer::create_column_specific_strategies("numeric", 50, 150, 20.0);

        let options: Vec<_> = strategies.iter().map(|s| s.option.as_str()).collect();
        assert!(options.contains(&"knn_imputation"));
    }

    #[test]
    fn test_strategies_numeric_knn_not_available_high_missing() {
        // KNN not available when >40% missing
        let strategies =
            DataQualityAnalyzer::create_column_specific_strategies("numeric", 50, 150, 45.0);

        let options: Vec<_> = strategies.iter().map(|s| s.option.as_str()).collect();
        assert!(!options.contains(&"knn_imputation"));
    }

    #[test]
    fn test_strategies_string() {
        let strategies =
            DataQualityAnalyzer::create_column_specific_strategies("string", 5, 100, 15.0);

        let options: Vec<_> = strategies.iter().map(|s| s.option.as_str()).collect();
        assert!(options.contains(&"mode_imputation"));
        assert!(options.contains(&"constant_imputation"));
    }

    #[test]
    fn test_strategies_string_category_indicator() {
        // category_indicator available when unique_count <= 10
        let strategies =
            DataQualityAnalyzer::create_column_specific_strategies("string", 5, 100, 15.0);

        let options: Vec<_> = strategies.iter().map(|s| s.option.as_str()).collect();
        assert!(options.contains(&"category_indicator"));
    }

    #[test]
    fn test_strategies_datetime() {
        let strategies =
            DataQualityAnalyzer::create_column_specific_strategies("datetime", 50, 100, 15.0);

        let options: Vec<_> = strategies.iter().map(|s| s.option.as_str()).collect();
        assert!(options.contains(&"forward_fill"));
        assert!(options.contains(&"interpolation"));
    }

    // ==================== create_problem_type_selection_issue tests ====================

    #[test]
    fn test_problem_type_selection_has_classification_and_regression() {
        let issue = DataQualityAnalyzer::create_problem_type_selection_issue();

        assert_eq!(issue.issue_type, "problem_type_selection");
        assert_eq!(issue.severity, "high");
        assert_eq!(issue.suggested_solutions.len(), 2);

        let options: Vec<_> = issue
            .suggested_solutions
            .iter()
            .map(|s| s.option.as_str())
            .collect();
        assert!(options.contains(&"classification"));
        assert!(options.contains(&"regression"));
    }
}
