use crate::types::{ColumnSummary, DatasetProfile, PipelineResult};
use anyhow::Result;
use chrono::Local;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, info, warn};

// ============================================================================
// Comprehensive Report Types
// ============================================================================

/// Comprehensive report merging all report data for CLI and library output.
///
/// This struct combines data from multiple sources:
/// - Pipeline result summary
/// - Dataset profile  
/// - Processing steps and cleaning actions
/// - Algorithm rationale and quality assessment
///
/// Use this for both JSON output (`--json`) and file writing (`--emit-report`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveReport {
    // Metadata
    /// Timestamp when the report was generated
    pub generated_at: String,
    /// Path to the input file
    pub input_file: String,
    /// Path to the output file (if written)
    pub output_file: Option<String>,

    // Processing summary (from PipelineResult.summary)
    /// Summary of processing actions and results
    pub processing_summary: ProcessingSummaryReport,

    // Algorithm rationale
    /// Rationale for algorithm/strategy choices
    pub algorithm_rationale: AlgorithmRationale,

    // Actions taken
    /// List of cleaning actions performed
    pub cleaning_actions: Vec<String>,
    /// List of processing steps executed
    pub processing_steps: Vec<String>,

    // Quality assessment
    /// Quality metrics before and after processing
    pub quality_assessment: QualityAssessment,

    // ML problem setup
    /// Detected problem type (classification/regression)
    pub problem_type: Option<String>,
    /// Target column for prediction
    pub target_column: Option<String>,
    /// AI/rule-based decisions made
    pub decisions_made: HashMap<String, String>,

    // Dataset profile summary
    /// Summary of dataset characteristics
    pub dataset_profile: DatasetProfileSummary,

    // Column details
    /// Per-column summaries of changes
    pub column_summaries: Vec<ColumnSummary>,
}

/// Summary of processing for the comprehensive report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingSummaryReport {
    /// Total execution time in milliseconds
    pub duration_ms: u64,
    /// Number of rows before preprocessing
    pub rows_before: usize,
    /// Number of rows after preprocessing
    pub rows_after: usize,
    /// Number of rows removed
    pub rows_removed: usize,
    /// Percentage of rows removed
    pub rows_removed_percent: f32,
    /// Number of columns before preprocessing
    pub columns_before: usize,
    /// Number of columns after preprocessing
    pub columns_after: usize,
    /// Number of columns removed
    pub columns_removed: usize,
    /// Percentage of columns removed
    pub columns_removed_percent: f32,
    /// Number of issues found
    pub issues_found: usize,
    /// Number of issues resolved
    pub issues_resolved: usize,
    /// Data quality score before (0.0-1.0)
    pub data_quality_before: f32,
    /// Data quality score after (0.0-1.0)
    pub data_quality_after: f32,
    /// Quality improvement percentage
    pub quality_improvement: f32,
    /// Warnings generated during processing
    pub warnings: Vec<String>,
}

/// Algorithm rationale for the comprehensive report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmRationale {
    /// Detected problem type
    pub problem_type: String,
    /// Dataset size category (small/medium/large)
    pub size_category: Option<String>,
    /// Feature complexity assessment
    pub feature_complexity: Option<String>,
}

/// Quality assessment metrics for the comprehensive report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    /// Number of duplicate rows found
    pub duplicate_count: usize,
    /// Percentage of duplicates
    pub duplicate_percentage: String,
    /// Columns identified as having outliers
    pub outlier_columns: Vec<String>,
    /// Columns with high null percentage (>50%)
    pub high_null_columns: Vec<String>,
}

/// Dataset profile summary for the comprehensive report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetProfileSummary {
    /// Original shape (rows, columns)
    pub original_shape: (usize, usize),
    /// Final shape after processing
    pub final_shape: (usize, usize),
    /// Count of columns by inferred type
    pub type_counts: HashMap<String, usize>,
    /// Percentage breakdown by type
    pub type_percentages: HashMap<String, String>,
    /// Target candidates identified
    pub target_candidates: Vec<String>,
}

pub struct ReportGenerator {
    output_dir: PathBuf,
    output_name: Option<String>,
}

/// Parameters for generating comprehensive analysis reports
pub struct ReportParams<'a> {
    pub original_df: &'a DataFrame,
    pub final_df: &'a DataFrame,
    pub profile: &'a DatasetProfile,
    pub processing_steps: &'a [String],
    pub cleaning_actions: &'a [String],
    pub problem_type: &'a str,
    pub target_column: &'a str,
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./outputs"),
            output_name: None,
        }
    }
}

impl ReportGenerator {
    /// Create a new ReportGenerator with custom output settings.
    pub fn new(output_dir: PathBuf, output_name: Option<String>) -> Self {
        Self { output_dir, output_name }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn generate_comprehensive_analysis_report(
        &self,
        original_df: &DataFrame,
        final_df: &DataFrame,
        profile: &DatasetProfile,
        processing_steps: &[String],
        cleaning_actions: &[String],
        problem_type: &str,
        target_column: &str,
    ) -> Result<String> {
        self.generate_report(ReportParams {
            original_df,
            final_df,
            profile,
            processing_steps,
            cleaning_actions,
            problem_type,
            target_column,
        })
    }

    /// Generate comprehensive analysis report using structured parameters
    pub fn generate_report(&self, params: ReportParams<'_>) -> Result<String> {
        let ReportParams {
            original_df,
            final_df,
            profile,
            processing_steps,
            cleaning_actions,
            problem_type,
            target_column,
        } = params;
        
        // Column type analysis
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for col in &profile.column_profiles {
            *type_counts.entry(col.inferred_type.clone()).or_insert(0) += 1;
        }

        let type_percentages: HashMap<String, f64> = type_counts
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    (*v as f64 / profile.column_profiles.len() as f64) * 100.0,
                )
            })
            .collect();

        // Outlier detection
        let outlier_columns: Vec<String> = profile
            .column_profiles
            .iter()
            .filter(|col| {
                col.characteristics
                    .get("has_outliers")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
            .map(|col| col.name.clone())
            .collect();

        // Build JSON structure
        let report = json!({
            "summary": {
                "rows": original_df.height(),
                "columns": original_df.width(),
                "processed_shape": [final_df.height(), final_df.width()],
                "problem_type": problem_type,
                "target_column": target_column,
                "processing_success": true,
            },
            "composition": {
                "type_counts": type_counts,
                "type_percentages": type_percentages.iter().map(|(k, v)| (k, format!("{:.1}", v))).collect::<HashMap<_, _>>(),
            },
            "quality_assessment": {
                "duplicate_count": profile.duplicate_count,
                "duplicate_percentage": format!("{:.1}", profile.duplicate_percentage),
                "outlier_columns": outlier_columns,
            },
            "cleaning_actions": cleaning_actions,
            "processing_steps": processing_steps,
            "algorithm_rationale": {
                "problem_type": problem_type,
                "size_category": profile.complexity_indicators.get("size_category"),
                "feature_complexity": profile.complexity_indicators.get("feature_complexity"),
            },
            "generated_at": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });

        // Save JSON report
        fs::create_dir_all(&self.output_dir)?;
        let report_path = self.output_dir.join("data_analysis_report.json");
        let mut file = File::create(&report_path)?;
        file.write_all(serde_json::to_string_pretty(&report)?.as_bytes())?;

        info!("Report saved: {}", report_path.display());

        Ok(serde_json::to_string_pretty(&report)?)
    }

    pub fn generate_files(
        &self,
        problem_type: &str,
        _df_with_ids: &DataFrame,
        df_for_training: &mut DataFrame,
        target_column: &str,
    ) -> Result<()> {
        // Use custom output name or default
        let file_name = self.output_name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| format!("processed_dataset_{}", problem_type));

        // Move the target column to the end
        let other_cols: Vec<PlSmallStr> = df_for_training
            .get_column_names()
            .into_iter()
            .filter(|col| col.as_str() != target_column)
            .cloned()
            .collect();

        let mut final_cols = other_cols;
        final_cols.push(target_column.into());

        *df_for_training = df_for_training.select(final_cols)?;

        // CRITICAL: Clean the dataframe one more time before writing
        // to ensure NO quotes remain in the data
        *df_for_training = self.final_cleaning_pass(df_for_training.clone())?;

        // Save to CSV with proper settings to avoid quote issues
        fs::create_dir_all(&self.output_dir)?;
        let output_path = self.output_dir.join(format!("{}.csv", file_name));
        let mut file = File::create(&output_path)?;
        
        // Configure CsvWriter with minimal quoting
        CsvWriter::new(&mut file)
            .include_header(true)
            .with_separator(b',')
            .with_quote_char(b'"')  // Use standard double quote
            .finish(df_for_training)?;

        info!("Dataset saved: {}", output_path.display());
        
        // Verify the output doesn't have triple quotes
        self.verify_output(&output_path.to_string_lossy())?;

        Ok(())
    }
    
    /// Final cleaning pass to remove any remaining quotes from all string columns
    fn final_cleaning_pass(&self, df: DataFrame) -> Result<DataFrame> {
        let mut df = df;
        let column_names: Vec<String> = df.get_column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        
        for col_name in &column_names {
            if let Ok(series) = df.column(col_name)
                && series.dtype() == &DataType::String {
                    let str_series = series.str()?;
                    let mut cleaned_values = Vec::with_capacity(str_series.len());
                    
                    for opt_val in str_series.into_iter() {
                        match opt_val {
                            Some(val) => {
                                // Remove any remaining quotes
                                let cleaned = val
                                    .trim()
                                    .replace("\"\"\"", "")
                                    .replace("\"\"", "")
                                    .replace('\"', "")
                                    .trim()
                                    .to_string();
                                
                                if cleaned.is_empty() {
                                    cleaned_values.push(None);
                                } else {
                                    cleaned_values.push(Some(cleaned));
                                }
                            }
                            None => cleaned_values.push(None),
                        }
                    }
                    
                    let cleaned_series = Series::new(col_name.as_str().into(), cleaned_values);
                    df.replace(col_name, cleaned_series)?;
                }
        }
        
        Ok(df)
    }
    
    /// Verify the output file doesn't have triple quotes
    fn verify_output(&self, path: &str) -> Result<()> {
        use std::fs;
        
        let content = fs::read_to_string(path)?;
        
        // Check for triple quotes
        if content.contains("\"\"\"") {
            warn!("Output file contains triple quotes!");
            warn!("This may indicate a quote escaping issue.");
        } else {
            debug!("Output verification passed - no triple quotes detected");
        }
        
        Ok(())
    }
    
    /// Build a comprehensive report from pipeline results.
    ///
    /// This method creates a single, unified report structure that can be:
    /// - Serialized to JSON and printed to stdout (`--json`)
    /// - Written to a file (`--emit-report`)
    /// - Used programmatically in library mode
    pub fn build_comprehensive_report(
        input_file: &str,
        output_file: Option<&str>,
        result: &PipelineResult,
        original_df: &DataFrame,
        final_df: &DataFrame,
        profile: &DatasetProfile,
    ) -> ComprehensiveReport {
        // Extract summary or use defaults
        let summary = result.summary.as_ref();
        
        // Build processing summary
        let processing_summary = ProcessingSummaryReport {
            duration_ms: summary.map(|s| s.duration_ms).unwrap_or(0),
            rows_before: summary.map(|s| s.rows_before).unwrap_or(original_df.height()),
            rows_after: summary.map(|s| s.rows_after).unwrap_or(final_df.height()),
            rows_removed: summary.map(|s| s.rows_removed).unwrap_or(0),
            rows_removed_percent: summary.map(|s| s.rows_removed_percentage()).unwrap_or(0.0),
            columns_before: summary.map(|s| s.columns_before).unwrap_or(original_df.width()),
            columns_after: summary.map(|s| s.columns_after).unwrap_or(final_df.width()),
            columns_removed: summary.map(|s| s.columns_removed).unwrap_or(0),
            columns_removed_percent: summary.map(|s| s.columns_removed_percentage()).unwrap_or(0.0),
            issues_found: summary.map(|s| s.issues_found).unwrap_or(0),
            issues_resolved: summary.map(|s| s.issues_resolved).unwrap_or(0),
            data_quality_before: summary.map(|s| s.data_quality_score_before).unwrap_or(0.0),
            data_quality_after: summary.map(|s| s.data_quality_score_after).unwrap_or(0.0),
            quality_improvement: summary.map(|s| s.quality_improvement()).unwrap_or(0.0),
            warnings: summary.map(|s| s.warnings.clone()).unwrap_or_default(),
        };
        
        // Build algorithm rationale
        let algorithm_rationale = AlgorithmRationale {
            problem_type: result.problem_type.clone().unwrap_or_else(|| "unknown".to_string()),
            size_category: profile.complexity_indicators.get("size_category")
                .and_then(|v| v.as_str())
                .map(String::from),
            feature_complexity: profile.complexity_indicators.get("feature_complexity")
                .and_then(|v| v.as_str())
                .map(String::from),
        };
        
        // Build quality assessment
        let outlier_columns: Vec<String> = profile
            .column_profiles
            .iter()
            .filter(|col| {
                col.characteristics
                    .get("has_outliers")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
            .map(|col| col.name.clone())
            .collect();
            
        let high_null_columns: Vec<String> = profile
            .column_profiles
            .iter()
            .filter(|col| col.null_percentage > 50.0)
            .map(|col| col.name.clone())
            .collect();
        
        let quality_assessment = QualityAssessment {
            duplicate_count: profile.duplicate_count,
            duplicate_percentage: format!("{:.1}", profile.duplicate_percentage),
            outlier_columns,
            high_null_columns,
        };
        
        // Build dataset profile summary
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for col in &profile.column_profiles {
            *type_counts.entry(col.inferred_type.clone()).or_insert(0) += 1;
        }
        
        let type_percentages: HashMap<String, String> = type_counts
            .iter()
            .map(|(k, v)| {
                let pct = (*v as f64 / profile.column_profiles.len().max(1) as f64) * 100.0;
                (k.clone(), format!("{:.1}", pct))
            })
            .collect();
        
        let dataset_profile = DatasetProfileSummary {
            original_shape: (original_df.height(), original_df.width()),
            final_shape: (final_df.height(), final_df.width()),
            type_counts,
            type_percentages,
            target_candidates: profile.target_candidates.clone(),
        };
        
        // Extract column summaries from result or create from profile
        let column_summaries = summary
            .map(|s| s.column_summaries.clone())
            .unwrap_or_else(|| {
                profile
                    .column_profiles
                    .iter()
                    .map(|col| ColumnSummary::new(&col.name, &col.dtype))
                    .collect()
            });
        
        ComprehensiveReport {
            generated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            input_file: input_file.to_string(),
            output_file: output_file.map(String::from),
            processing_summary,
            algorithm_rationale,
            cleaning_actions: result.cleaning_actions.clone(),
            processing_steps: result.processing_steps.clone(),
            quality_assessment,
            problem_type: result.problem_type.clone(),
            target_column: result.target_column.clone(),
            decisions_made: result.ai_choices.clone(),
            dataset_profile,
            column_summaries,
        }
    }
    
    /// Write a comprehensive report to a JSON file.
    ///
    /// The report is written to the output directory with the specified base name.
    /// For example, if `report_base_name` is "train", the file will be "train_report.json".
    pub fn write_report_to_file(
        &self,
        report: &ComprehensiveReport,
        report_base_name: &str,
    ) -> Result<PathBuf> {
        fs::create_dir_all(&self.output_dir)?;
        
        let report_path = self.output_dir.join(format!("{}_report.json", report_base_name));
        let mut file = File::create(&report_path)?;
        file.write_all(serde_json::to_string_pretty(report)?.as_bytes())?;
        
        info!("Report saved: {}", report_path.display());
        
        Ok(report_path)
    }
}