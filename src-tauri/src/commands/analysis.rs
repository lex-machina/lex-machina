//!
//! Analysis commands for computing comprehensive dataset insights.
//!
//! This module orchestrates profiling, statistical analysis, correlation,
//! and quality assessment. Heavy computation runs in a background thread
//! to keep the UI responsive.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use anofox_statistics::categorical::{chisq_test, cramers_v};
use anofox_statistics::correlation::{pearson, spearman};
use anofox_statistics::nonparametric::kruskal::kruskal_wallis;
use anofox_statistics::nonparametric::wilcoxon::mann_whitney_u;
use anofox_statistics::parametric::anova::{one_way_anova, AnovaKind};
use anofox_statistics::parametric::ttest::{t_test, Alternative, TTestKind};
use chrono::{DateTime, Local, Utc};
use lex_processing::profiler::DataProfiler;
use lex_processing::{DataQualityAnalyzer, DatasetProfile};
use normality::{
    anderson_darling, dagostino_k_squared, jarque_bera, lilliefors, shapiro_wilk,
};
use polars::prelude::*;
use serde::Serialize;
use statrs::distribution::{ContinuousCDF, FisherSnedecor};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::events::AppEventEmitter;
use crate::state::{
    AnalysisCache, AnalysisColumnStats, AnalysisDataset, AnalysisResult, AnalysisSummary,
    AnalysisUIState, AppState, AssociationAnalysis, BoxPlotSummary, CategoryCount,
    CategoricalColumnStats, CorrelationAnalysis, CorrelationPair, DateTimeColumnStats,
    HeatmapMatrix, HistogramBin, MissingnessAnalysis, MissingnessColumn,
    NumericCategoricalAssociation, NumericColumnStats, StatisticalTestResult, TextColumnStats,
    TimeBin, TypeDistributionEntry,
};

// ==========================================================================
// TYPES
// ==========================================================================

/// Result of exporting an analysis report.
#[derive(Debug, Serialize)]
pub struct AnalysisExportResult {
    pub report_path: String,
}

// ==========================================================================
// TAURI COMMANDS
// ==========================================================================

/// Returns the current analysis UI state.
#[tauri::command]
pub fn get_analysis_ui_state(state: State<'_, AppState>) -> AnalysisUIState {
    state.analysis_ui_state.read().clone()
}

/// Updates the analysis UI state.
#[tauri::command]
pub fn set_analysis_ui_state(ui_state: AnalysisUIState, state: State<'_, AppState>) {
    *state.analysis_ui_state.write() = ui_state;
}

/// Returns cached analysis result for the selected dataset.
#[tauri::command]
pub fn get_analysis_result(
    use_processed_data: bool,
    state: State<'_, AppState>,
) -> Option<AnalysisResult> {
    let cache = state.analysis_results.read();
    if use_processed_data {
        cache.processed.clone()
    } else {
        cache.original.clone()
    }
}

/// Clears all cached analysis results.
#[tauri::command]
pub fn clear_analysis_results(state: State<'_, AppState>) {
    *state.analysis_results.write() = AnalysisCache::default();
}

/// Runs the full analysis pipeline for the selected dataset.
#[tauri::command]
pub async fn run_analysis(
    use_processed_data: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    let dataset = if use_processed_data {
        AnalysisDataset::Processed
    } else {
        AnalysisDataset::Original
    };

    let df = {
        let guard = if use_processed_data {
            state.processed_dataframe.read()
        } else {
            state.dataframe.read()
        };

        guard
            .as_ref()
            .map(|loaded| loaded.df.clone())
            .ok_or_else(|| {
                if use_processed_data {
                    "No processed data available".to_string()
                } else {
                    "No data loaded".to_string()
                }
            })?
    };

    app.emit_loading(true, Some("Running analysis..."));

    let analysis_result = match tauri::async_runtime::spawn_blocking(move || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            compute_analysis(dataset, df)
        }))
    })
    .await
    {
        Ok(Ok(Ok(result))) => result,
        Ok(Ok(Err(err))) => {
            app.emit_loading(false, None);
            return Err(err);
        }
        Ok(Err(_panic)) => {
            app.emit_loading(false, None);
            return Err("Analysis task panicked".to_string());
        }
        Err(err) => {
            app.emit_loading(false, None);
            return Err(format!("Analysis task failed: {err}"));
        }
    };

    {
        let mut cache = state.analysis_results.write();
        match analysis_result.dataset {
            AnalysisDataset::Original => {
                cache.original = Some(analysis_result.clone());
            }
            AnalysisDataset::Processed => {
                cache.processed = Some(analysis_result.clone());
            }
        }
    }

    app.emit_loading(false, None);
    Ok(analysis_result)
}

/// Exports the cached analysis report to JSON.
#[tauri::command]
pub async fn export_analysis_report(
    use_processed_data: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AnalysisExportResult, String> {
    let analysis = get_analysis_result(use_processed_data, state)
        .ok_or_else(|| "No analysis results to export".to_string())?;

    let dataset_label = if use_processed_data { "processed" } else { "original" };
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let default_filename = format!("analysis_{dataset_label}_{timestamp}.json");

    let file_path = app
        .dialog()
        .file()
        .add_filter("JSON Files", &["json"])
        .set_file_name(&default_filename)
        .blocking_save_file();

    let report_path = match file_path {
        Some(path) => path.to_string(),
        None => return Err("Export cancelled by user".to_string()),
    };

    let report_json = serde_json::to_string_pretty(&analysis)
        .map_err(|e| format!("Failed to serialize analysis: {e}"))?;

    std::fs::write(&report_path, report_json)
        .map_err(|e| format!("Failed to write report: {e}"))?;

    Ok(AnalysisExportResult { report_path })
}

// ==========================================================================
// ANALYSIS IMPLEMENTATION
// ==========================================================================

fn compute_analysis(dataset: AnalysisDataset, df: DataFrame) -> Result<AnalysisResult, String> {
    let start = Instant::now();
    let dataset_profile = DataProfiler::profile_dataset(&df).map_err(|e| e.to_string())?;
    let quality_issues =
        DataQualityAnalyzer::identify_issues(&dataset_profile, &df).map_err(|e| e.to_string())?;

    let summary = build_summary(&df, &dataset_profile);
    let columns = build_column_stats(&df, &dataset_profile);
    let missingness = build_missingness(&df, &dataset_profile);
    let correlations = build_correlations(&df, &dataset_profile);
    let associations = build_associations(&df, &dataset_profile);

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(AnalysisResult {
        dataset,
        generated_at: Local::now().to_rfc3339(),
        duration_ms,
        summary,
        dataset_profile,
        columns,
        missingness,
        correlations,
        associations,
        quality_issues,
    })
}

fn build_summary(df: &DataFrame, profile: &DatasetProfile) -> AnalysisSummary {
    let total_cells = df.height().saturating_mul(df.width());
    let total_missing: usize = profile
        .column_profiles
        .iter()
        .map(|col| col.null_count)
        .sum();
    let total_missing_percentage = if total_cells > 0 {
        (total_missing as f64 / total_cells as f64) * 100.0
    } else {
        0.0
    };

    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for col in &profile.column_profiles {
        *type_counts.entry(col.inferred_type.clone()).or_insert(0) += 1;
    }

    let mut type_distribution: Vec<TypeDistributionEntry> = type_counts
        .into_iter()
        .map(|(dtype, count)| TypeDistributionEntry {
            percentage: if profile.column_profiles.is_empty() {
                0.0
            } else {
                (count as f64 / profile.column_profiles.len() as f64) * 100.0
            },
            dtype,
            count,
        })
        .collect();
    type_distribution.sort_by(|a, b| b.count.cmp(&a.count));

    AnalysisSummary {
        rows: df.height(),
        columns: df.width(),
        memory_bytes: df.estimated_size() as u64,
        duplicate_count: profile.duplicate_count,
        duplicate_percentage: profile.duplicate_percentage,
        total_missing_cells: total_missing,
        total_missing_percentage,
        type_distribution,
    }
}

fn build_column_stats(df: &DataFrame, profile: &DatasetProfile) -> Vec<AnalysisColumnStats> {
    profile
        .column_profiles
        .iter()
        .filter_map(|column| {
            let series = df.column(&column.name).ok()?;
            let series = series.as_materialized_series();
            let inferred_type = column.inferred_type.as_str();

            let numeric = if inferred_type == "numeric" {
                compute_numeric_stats(series)
            } else {
                None
            };

            let categorical = if inferred_type == "categorical"
                || inferred_type == "binary"
                || inferred_type == "boolean"
            {
                compute_categorical_stats(series)
            } else {
                None
            };

            let text = if inferred_type == "string" {
                compute_text_stats(series)
            } else {
                None
            };

            let datetime = if inferred_type == "datetime" {
                compute_datetime_stats(series)
            } else {
                None
            };

            Some(AnalysisColumnStats {
                profile: column.clone(),
                numeric,
                categorical,
                text,
                datetime,
            })
        })
        .collect()
}

fn compute_numeric_stats(series: &Series) -> Option<NumericColumnStats> {
    let casted = series.cast(&DataType::Float64).ok()?;
    let values: Vec<f64> = casted.f64().ok()?.into_iter().flatten().collect();
    if values.is_empty() {
        return None;
    }

    let mut sorted = values.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    let min = *sorted.first().unwrap_or(&0.0);
    let max = *sorted.last().unwrap_or(&0.0);

    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = if n > 1 {
        values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / (n as f64 - 1.0)
    } else {
        0.0
    };
    let std_dev = variance.sqrt();
    let median = quantile_sorted(&sorted, 0.5);
    let q1 = quantile_sorted(&sorted, 0.25);
    let q3 = quantile_sorted(&sorted, 0.75);
    let iqr = q3 - q1;

    let skewness = if std_dev > 0.0 {
        let m3 = values
            .iter()
            .map(|v| (v - mean).powi(3))
            .sum::<f64>()
            / n as f64;
        m3 / std_dev.powi(3)
    } else {
        0.0
    };

    let kurtosis = if std_dev > 0.0 {
        let m4 = values
            .iter()
            .map(|v| (v - mean).powi(4))
            .sum::<f64>()
            / n as f64;
        m4 / std_dev.powi(4) - 3.0
    } else {
        0.0
    };

    let lower_bound = q1 - 1.5 * iqr;
    let upper_bound = q3 + 1.5 * iqr;
    let outliers_iqr = values
        .iter()
        .filter(|v| **v < lower_bound || **v > upper_bound)
        .count();

    let mad = median_absolute_deviation(&sorted, median);
    let outliers_robust_z = if mad > 0.0 {
        values
            .iter()
            .filter(|v| (0.6745 * (*v - median) / mad).abs() > 3.5)
            .count()
    } else {
        0
    };

    let histogram = build_histogram(&sorted, 24);
    let box_plot = BoxPlotSummary {
        min,
        q1,
        median,
        q3,
        max,
    };

    let mut normality_tests = Vec::new();
    if let Ok(result) = shapiro_wilk(values.clone()) {
        normality_tests.push(to_test_result("Shapiro-Wilk", result.statistic, result.p_value));
    }
    if let Ok(result) = anderson_darling(values.clone()) {
        normality_tests
            .push(to_test_result("Anderson-Darling", result.statistic, result.p_value));
    }
    if let Ok(result) = lilliefors(values.clone()) {
        normality_tests.push(to_test_result("Lilliefors (KS)", result.statistic, result.p_value));
    }
    if let Ok(result) = jarque_bera(values.clone()) {
        normality_tests.push(to_test_result("Jarque-Bera", result.statistic, result.p_value));
    }
    if let Ok(result) = dagostino_k_squared(values.clone()) {
        normality_tests.push(to_test_result("D'Agostino K^2", result.statistic, result.p_value));
    }

    Some(NumericColumnStats {
        min,
        max,
        mean,
        median,
        std_dev,
        variance,
        iqr,
        skewness,
        kurtosis,
        outliers_iqr,
        outliers_robust_z,
        histogram,
        box_plot,
        normality_tests,
    })
}

fn compute_categorical_stats(series: &Series) -> Option<CategoricalColumnStats> {
    let casted = series.cast(&DataType::String).ok()?;
    let values: Vec<String> = casted
        .str()
        .ok()?
        .into_iter()
        .flatten()
        .map(|value| value.to_string())
        .collect();

    if values.is_empty() {
        return None;
    }

    let total = values.len() as f64;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for value in values {
        *counts.entry(value).or_insert(0) += 1;
    }

    let cardinality = counts.len();
    let mut entries: Vec<CategoryCount> = counts
        .into_iter()
        .map(|(value, count)| {
            let percentage = (count as f64 / total) * 100.0;
            CategoryCount {
                value,
                count,
                percentage,
            }
        })
        .collect();
    entries.sort_by(|a, b| b.count.cmp(&a.count));

    let mut entropy = 0.0;
    let mut gini = 1.0;
    for entry in &entries {
        let p = entry.count as f64 / total;
        entropy -= p * p.ln();
        gini -= p.powi(2);
    }

    let imbalance_ratio = if entries.len() > 1 {
        let max_count = entries.first().map(|e| e.count).unwrap_or(0) as f64;
        let min_count = entries.last().map(|e| e.count).unwrap_or(0) as f64;
        if min_count > 0.0 {
            max_count / min_count
        } else {
            0.0
        }
    } else {
        1.0
    };

    Some(CategoricalColumnStats {
        cardinality,
        entropy,
        gini,
        imbalance_ratio,
        top_values: entries.into_iter().take(20).collect(),
    })
}

fn compute_text_stats(series: &Series) -> Option<TextColumnStats> {
    let casted = series.cast(&DataType::String).ok()?;
    let mut lengths = Vec::new();
    let mut empty_count = 0usize;
    let mut whitespace_count = 0usize;
    let mut unique_tokens: HashSet<String> = HashSet::new();

    for value in casted.str().ok()?.into_iter().flatten() {
        let trimmed = value.trim();
        if value.is_empty() {
            empty_count += 1;
        }
        if trimmed.is_empty() {
            whitespace_count += 1;
        }
        lengths.push(value.len() as f64);
        for token in trimmed.split_whitespace() {
            unique_tokens.insert(token.to_string());
        }
    }

    if lengths.is_empty() {
        return None;
    }

    lengths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let total = lengths.len() as f64;
    let min_length = *lengths.first().unwrap_or(&0.0) as usize;
    let max_length = *lengths.last().unwrap_or(&0.0) as usize;
    let mean_length = lengths.iter().sum::<f64>() / total;
    let median_length = quantile_sorted(&lengths, 0.5);

    let empty_percentage = if total > 0.0 {
        (empty_count as f64 / total) * 100.0
    } else {
        0.0
    };
    let whitespace_percentage = if total > 0.0 {
        (whitespace_count as f64 / total) * 100.0
    } else {
        0.0
    };

    let length_histogram = build_histogram(&lengths, 20);

    Some(TextColumnStats {
        min_length,
        max_length,
        mean_length,
        median_length,
        empty_percentage,
        whitespace_percentage,
        unique_token_count: unique_tokens.len(),
        length_histogram,
    })
}

fn compute_datetime_stats(series: &Series) -> Option<DateTimeColumnStats> {
    let casted = series
        .cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
        .ok()?;
    let values: Vec<i64> = casted
        .datetime()
        .ok()?
        .physical()
        .into_iter()
        .flatten()
        .collect();
    if values.is_empty() {
        return None;
    }

    let min = *values.iter().min()?;
    let max = *values.iter().max()?;
    let range_days = (max - min) as f64 / 86_400_000.0;
    let granularity = infer_time_granularity(range_days);
    let time_bins = build_time_bins(&values, &granularity);

    Some(DateTimeColumnStats {
        min: format_timestamp(min),
        max: format_timestamp(max),
        range_days,
        granularity,
        time_bins,
    })
}

fn build_missingness(df: &DataFrame, profile: &DatasetProfile) -> MissingnessAnalysis {
    let total_cells = df.height().saturating_mul(df.width());
    let total_missing: usize = profile
        .column_profiles
        .iter()
        .map(|col| col.null_count)
        .sum();
    let total_missing_percentage = if total_cells > 0 {
        (total_missing as f64 / total_cells as f64) * 100.0
    } else {
        0.0
    };

    let per_column = profile
        .column_profiles
        .iter()
        .map(|col| MissingnessColumn {
            column: col.name.clone(),
            missing_count: col.null_count,
            missing_percentage: col.null_percentage,
        })
        .collect::<Vec<_>>();

    let co_missing_matrix = build_co_missing_matrix(df, profile);

    MissingnessAnalysis {
        total_missing_cells: total_missing,
        total_missing_percentage,
        per_column,
        co_missing_matrix,
    }
}

fn build_correlations(df: &DataFrame, profile: &DatasetProfile) -> CorrelationAnalysis {
    let numeric_columns: Vec<String> = profile
        .column_profiles
        .iter()
        .filter(|col| col.inferred_type == "numeric")
        .map(|col| col.name.clone())
        .collect();

    let mut series_values: Vec<Vec<Option<f64>>> = Vec::new();
    for name in &numeric_columns {
        let values = df
            .column(name)
            .ok()
            .and_then(|column| column.as_materialized_series().cast(&DataType::Float64).ok())
            .and_then(|series| {
                series
                    .f64()
                    .ok()
                    .map(|ca| ca.into_iter().collect::<Vec<Option<f64>>>())
            })
            .unwrap_or_default();
        series_values.push(values);
    }

    let size = numeric_columns.len();
    let mut pearson_values = vec![vec![0.0; size]; size];
    let mut pearson_p_values = vec![vec![1.0; size]; size];
    let mut spearman_values = vec![vec![0.0; size]; size];
    let mut spearman_p_values = vec![vec![1.0; size]; size];
    let mut top_pairs: Vec<CorrelationPair> = Vec::new();

    for i in 0..size {
        for j in i..size {
            if i == j {
                pearson_values[i][j] = 1.0;
                spearman_values[i][j] = 1.0;
                pearson_p_values[i][j] = 0.0;
                spearman_p_values[i][j] = 0.0;
                continue;
            }

            let mut x = Vec::new();
            let mut y = Vec::new();
            for (a, b) in series_values[i]
                .iter()
                .zip(series_values[j].iter())
            {
                if let (Some(a), Some(b)) = (a, b) {
                    x.push(*a);
                    y.push(*b);
                }
            }

            if x.len() < 3 {
                continue;
            }

            if let Ok(result) = pearson(&x, &y, Some(0.95)) {
                pearson_values[i][j] = result.estimate;
                pearson_values[j][i] = result.estimate;
                pearson_p_values[i][j] = result.p_value;
                pearson_p_values[j][i] = result.p_value;
                top_pairs.push(CorrelationPair {
                    column_x: numeric_columns[i].clone(),
                    column_y: numeric_columns[j].clone(),
                    method: "pearson".to_string(),
                    estimate: result.estimate,
                    p_value: result.p_value,
                });
            }

            if let Ok(result) = spearman(&x, &y, Some(0.95)) {
                spearman_values[i][j] = result.estimate;
                spearman_values[j][i] = result.estimate;
                spearman_p_values[i][j] = result.p_value;
                spearman_p_values[j][i] = result.p_value;
            }
        }
    }

    top_pairs.sort_by(|a, b| {
        b.estimate
            .abs()
            .partial_cmp(&a.estimate.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    CorrelationAnalysis {
        numeric_columns: numeric_columns.clone(),
        pearson: HeatmapMatrix {
            x_labels: numeric_columns.clone(),
            y_labels: numeric_columns.clone(),
            values: pearson_values,
            p_values: Some(pearson_p_values),
        },
        spearman: HeatmapMatrix {
            x_labels: numeric_columns.clone(),
            y_labels: numeric_columns,
            values: spearman_values,
            p_values: Some(spearman_p_values),
        },
        top_pairs: top_pairs.into_iter().take(20).collect(),
    }
}

fn build_associations(df: &DataFrame, profile: &DatasetProfile) -> AssociationAnalysis {
    let categorical_columns: Vec<String> = profile
        .column_profiles
        .iter()
        .filter(|col| {
            matches!(
                col.inferred_type.as_str(),
                "categorical" | "binary" | "boolean" | "string"
            )
        })
        .map(|col| col.name.clone())
        .collect();

    let size = categorical_columns.len();
    let mut cramers_values = vec![vec![0.0; size]; size];
    let mut chi_values = vec![vec![0.0; size]; size];
    let mut chi_p_values = vec![vec![1.0; size]; size];

    for i in 0..size {
        for j in i..size {
            if i == j {
                cramers_values[i][j] = 1.0;
                chi_values[i][j] = 0.0;
                chi_p_values[i][j] = 0.0;
                continue;
            }

            let Some(contingency) = build_contingency_table(
                df,
                &categorical_columns[i],
                &categorical_columns[j],
            ) else {
                continue;
            };

            if let Ok(result) = cramers_v(&contingency) {
                cramers_values[i][j] = result.estimate;
                cramers_values[j][i] = result.estimate;
            }
            if let Ok(result) = chisq_test(&contingency, false) {
                chi_values[i][j] = result.statistic;
                chi_values[j][i] = result.statistic;
                chi_p_values[i][j] = result.p_value;
                chi_p_values[j][i] = result.p_value;
            }
        }
    }

    let numeric_columns: Vec<String> = profile
        .column_profiles
        .iter()
        .filter(|col| col.inferred_type == "numeric")
        .map(|col| col.name.clone())
        .collect();

    let mut numeric_categorical = Vec::new();
    for numeric_col in &numeric_columns {
        for categorical_col in &categorical_columns {
            if let Some(association) = build_numeric_categorical_association(
                df,
                numeric_col,
                categorical_col,
            ) {
                numeric_categorical.push(association);
            }
        }
    }

    AssociationAnalysis {
        categorical_columns: categorical_columns.clone(),
        cramers_v: HeatmapMatrix {
            x_labels: categorical_columns.clone(),
            y_labels: categorical_columns.clone(),
            values: cramers_values,
            p_values: None,
        },
        chi_square: HeatmapMatrix {
            x_labels: categorical_columns.clone(),
            y_labels: categorical_columns,
            values: chi_values,
            p_values: Some(chi_p_values),
        },
        numeric_categorical,
    }
}

fn build_numeric_categorical_association(
    df: &DataFrame,
    numeric_column: &str,
    categorical_column: &str,
) -> Option<NumericCategoricalAssociation> {
    let num_series = df.column(numeric_column).ok()?;
    let cat_series = df.column(categorical_column).ok()?;
    let num_series = num_series
        .as_materialized_series()
        .cast(&DataType::Float64)
        .ok()?;
    let cat_series = cat_series
        .as_materialized_series()
        .cast(&DataType::String)
        .ok()?;

    let mut groups: HashMap<String, Vec<f64>> = HashMap::new();
    for (num, cat) in num_series
        .f64()
        .ok()?
        .into_iter()
        .zip(cat_series.str().ok()?.into_iter())
    {
        if let (Some(num), Some(cat)) = (num, cat) {
            groups.entry(cat.to_string()).or_default().push(num);
        }
    }

    if groups.len() < 2 {
        return None;
    }

    let group_values: Vec<Vec<f64>> = groups
        .values()
        .cloned()
        .filter(|values| values.len() >= 2)
        .collect();
    if group_values.len() < 2 {
        return None;
    }

    let total_count: usize = group_values.iter().map(|values| values.len()).sum();
    if total_count <= group_values.len() {
        return None;
    }

    let group_refs: Vec<&[f64]> = group_values.iter().map(|v| v.as_slice()).collect();
    let variance_test = brown_forsythe_test(&group_values);

    let mut anova = None;
    let mut kruskal = None;
    let mut t_test_result = None;
    let mut mann_whitney = None;

    if group_values.len() == 2 {
        let group_a = &group_values[0];
        let group_b = &group_values[1];
        if let Ok(result) = t_test(
            group_a,
            group_b,
            TTestKind::Welch,
            Alternative::TwoSided,
            0.0,
            Some(0.95),
        ) {
            t_test_result = Some(StatisticalTestResult {
                test: "Welch t-test".to_string(),
                statistic: result.statistic,
                p_value: result.p_value,
                df: Some(result.df),
                effect_size: None,
                notes: None,
            });
        }
        if let Ok(result) = mann_whitney_u(
            group_a,
            group_b,
            Alternative::TwoSided,
            true,
            false,
            Some(0.95),
            None,
        ) {
            mann_whitney = Some(StatisticalTestResult {
                test: "Mann-Whitney U".to_string(),
                statistic: result.statistic,
                p_value: result.p_value,
                df: None,
                effect_size: None,
                notes: None,
            });
        }
    } else {
        if let Ok(result) = one_way_anova(&group_refs, AnovaKind::Welch) {
            let effect_size = result
                .ss_between
                .zip(result.ss_total)
                .map(|(between, total)| if total > 0.0 { between / total } else { 0.0 });
            anova = Some(StatisticalTestResult {
                test: "One-way ANOVA".to_string(),
                statistic: result.statistic,
                p_value: result.p_value,
                df: Some(result.df_between),
                effect_size,
                notes: Some(format!("df_within={:.2}", result.df_within)),
            });
        }
        if let Ok(result) = kruskal_wallis(&group_refs) {
            kruskal = Some(StatisticalTestResult {
                test: "Kruskal-Wallis".to_string(),
                statistic: result.statistic,
                p_value: result.p_value,
                df: Some(result.df),
                effect_size: None,
                notes: None,
            });
        }
    }

    Some(NumericCategoricalAssociation {
        numeric_column: numeric_column.to_string(),
        categorical_column: categorical_column.to_string(),
        anova,
        variance_test,
        kruskal,
        t_test: t_test_result,
        mann_whitney,
    })
}

fn build_contingency_table(df: &DataFrame, col_a: &str, col_b: &str) -> Option<Vec<Vec<usize>>> {
    let series_a = df
        .column(col_a)
        .ok()?
        .as_materialized_series()
        .cast(&DataType::String)
        .ok()?;
    let series_b = df
        .column(col_b)
        .ok()?
        .as_materialized_series()
        .cast(&DataType::String)
        .ok()?;

    let mut map_a: HashMap<String, usize> = HashMap::new();
    let mut map_b: HashMap<String, usize> = HashMap::new();
    let mut counts: Vec<Vec<usize>> = Vec::new();

    for (value_a, value_b) in series_a
        .str()
        .ok()?
        .into_iter()
        .zip(series_b.str().ok()?.into_iter())
    {
        let (Some(value_a), Some(value_b)) = (value_a, value_b) else {
            continue;
        };

        let row_index = *map_a.entry(value_a.to_string()).or_insert_with(|| {
            counts.push(vec![0; map_b.len()]);
            counts.len() - 1
        });

        let col_index = if let Some(index) = map_b.get(value_b) {
            *index
        } else {
            let new_index = map_b.len();
            map_b.insert(value_b.to_string(), new_index);
            for row in &mut counts {
                row.push(0);
            }
            new_index
        };

        if let Some(row) = counts.get_mut(row_index) {
            if let Some(cell) = row.get_mut(col_index) {
                *cell += 1;
            }
        }
    }

    if counts.len() < 2 || counts.first().map(|row| row.len()).unwrap_or(0) < 2 {
        None
    } else {
        Some(counts)
    }
}

fn build_co_missing_matrix(df: &DataFrame, profile: &DatasetProfile) -> HeatmapMatrix {
    let labels: Vec<String> = profile
        .column_profiles
        .iter()
        .map(|col| col.name.clone())
        .collect();
    let n = labels.len();
    let rows = df.height().max(1);
    let mut null_masks: Vec<Vec<bool>> = Vec::with_capacity(n);

    for name in &labels {
        let mask = df
            .column(name)
            .map(|series| {
                series
                    .is_null()
                    .into_iter()
                    .map(|value| value.unwrap_or(false))
                    .collect::<Vec<bool>>()
            })
            .unwrap_or_else(|_| vec![false; rows]);
        null_masks.push(mask);
    }

    let mut values = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in i..n {
            let mut count = 0usize;
            for row in 0..rows {
                if null_masks
                    .get(i)
                    .and_then(|mask| mask.get(row))
                    .copied()
                    .unwrap_or(false)
                    && null_masks
                        .get(j)
                        .and_then(|mask| mask.get(row))
                        .copied()
                        .unwrap_or(false)
                {
                    count += 1;
                }
            }

            let percentage = (count as f64 / rows as f64) * 100.0;
            values[i][j] = percentage;
            values[j][i] = percentage;
        }
    }

    HeatmapMatrix {
        x_labels: labels.clone(),
        y_labels: labels,
        values,
        p_values: None,
    }
}

fn build_histogram(values: &[f64], bins: usize) -> Vec<HistogramBin> {
    if values.is_empty() {
        return Vec::new();
    }

    let min = values.first().copied().unwrap_or(0.0);
    let max = values.last().copied().unwrap_or(min);
    if (max - min).abs() < f64::EPSILON {
        return vec![HistogramBin {
            start: min,
            end: max,
            count: values.len(),
        }];
    }

    let bin_count = bins.max(5);
    let width = (max - min) / bin_count as f64;
    let mut counts = vec![0usize; bin_count];

    for value in values {
        let mut index = ((value - min) / width) as usize;
        if index >= bin_count {
            index = bin_count - 1;
        }
        counts[index] += 1;
    }

    counts
        .into_iter()
        .enumerate()
        .map(|(idx, count)| HistogramBin {
            start: min + idx as f64 * width,
            end: min + (idx as f64 + 1.0) * width,
            count,
        })
        .collect()
}

fn quantile_sorted(values: &[f64], quantile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let pos = quantile.clamp(0.0, 1.0) * (values.len() as f64 - 1.0);
    let lower = pos.floor() as usize;
    let upper = pos.ceil() as usize;
    if lower == upper {
        return values[lower];
    }
    let weight = pos - lower as f64;
    values[lower] + (values[upper] - values[lower]) * weight
}

fn median_absolute_deviation(sorted: &[f64], median: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let mut deviations: Vec<f64> = sorted.iter().map(|v| (v - median).abs()).collect();
    deviations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    quantile_sorted(&deviations, 0.5)
}

fn infer_time_granularity(range_days: f64) -> String {
    if range_days >= 730.0 {
        "month".to_string()
    } else if range_days >= 120.0 {
        "week".to_string()
    } else if range_days >= 2.0 {
        "day".to_string()
    } else if range_days >= 0.1 {
        "hour".to_string()
    } else {
        "minute".to_string()
    }
}

fn build_time_bins(values: &[i64], granularity: &str) -> Vec<TimeBin> {
    if values.is_empty() {
        return Vec::new();
    }

    let bucket_ms = match granularity {
        "month" => 30 * 86_400_000,
        "week" => 7 * 86_400_000,
        "day" => 86_400_000,
        "hour" => 3_600_000,
        _ => 60_000,
    };

    let mut buckets: HashMap<i64, usize> = HashMap::new();
    for value in values {
        let bucket = value - (value % bucket_ms);
        *buckets.entry(bucket).or_insert(0) += 1;
    }

    let mut bins: Vec<(i64, usize)> = buckets.into_iter().collect();
    bins.sort_by_key(|(bucket, _)| *bucket);

    bins.into_iter()
        .map(|(bucket, count)| TimeBin {
            label: format_timestamp(bucket),
            count,
        })
        .collect()
}

fn format_timestamp(timestamp_ms: i64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp_millis(timestamp_ms)
        .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap());
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn to_test_result(test: &str, statistic: f64, p_value: f64) -> StatisticalTestResult {
    StatisticalTestResult {
        test: test.to_string(),
        statistic,
        p_value,
        df: None,
        effect_size: None,
        notes: None,
    }
}

fn brown_forsythe_test(groups: &[Vec<f64>]) -> Option<StatisticalTestResult> {
    if groups.len() < 2 {
        return None;
    }

    let mut z_values: Vec<Vec<f64>> = Vec::with_capacity(groups.len());
    let mut group_sizes: Vec<usize> = Vec::with_capacity(groups.len());

    for group in groups {
        if group.len() < 2 {
            return None;
        }
        let mut sorted = group.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = quantile_sorted(&sorted, 0.5);
        let deviations = group.iter().map(|value| (value - median).abs()).collect();
        group_sizes.push(group.len());
        z_values.push(deviations);
    }

    let n_total: usize = group_sizes.iter().sum();
    if n_total <= groups.len() {
        return None;
    }

    let z_means: Vec<f64> = z_values
        .iter()
        .map(|values| values.iter().sum::<f64>() / values.len() as f64)
        .collect();
    let z_grand_mean: f64 = z_values.iter().flatten().sum::<f64>() / n_total as f64;

    let ss_between = group_sizes
        .iter()
        .zip(z_means.iter())
        .map(|(&count, &mean)| count as f64 * (mean - z_grand_mean).powi(2))
        .sum::<f64>();
    let ss_within = z_values
        .iter()
        .zip(z_means.iter())
        .map(|(values, &mean)| {
            values
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
        })
        .sum::<f64>();

    let df1 = (groups.len() - 1) as f64;
    let df2 = (n_total - groups.len()) as f64;
    if df1 <= 0.0 || df2 <= 0.0 {
        return None;
    }

    let ms_between = ss_between / df1;
    let ms_within = ss_within / df2;
    if !ms_between.is_finite() || !ms_within.is_finite() || ms_within <= 0.0 {
        return None;
    }

    let f_stat = ms_between / ms_within;
    if !f_stat.is_finite() || f_stat < 0.0 {
        return None;
    }

    let f_dist = FisherSnedecor::new(df1, df2).ok()?;
    let p_value: f64 = 1.0 - f_dist.cdf(f_stat);
    if !p_value.is_finite() {
        return None;
    }

    Some(StatisticalTestResult {
        test: "Brown-Forsythe".to_string(),
        statistic: f_stat,
        p_value,
        df: Some(df1),
        effect_size: None,
        notes: Some(format!("df2={:.2}", df2)),
    })
}
