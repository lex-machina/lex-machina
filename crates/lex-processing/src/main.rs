//! CLI entry point for the data preprocessing pipeline.

use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};
use lex_processing::{
    CategoricalImputation, ComprehensiveReport, NumericImputation, OutlierStrategy, Pipeline,
    PipelineConfig, ReportGenerator,
};
use dotenv::dotenv;
use polars::io::csv::read::CsvReadOptions;
use polars::prelude::*;
use std::path::Path;
use tracing::{debug, error, info, warn};

#[cfg(feature = "ai")]
use lex_processing::ai::OpenRouterProvider;
#[cfg(feature = "ai")]
use std::env;
#[cfg(feature = "ai")]
use std::sync::Arc;

/// CLI-compatible outlier strategy enum
#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliOutlierStrategy {
    /// Cap outliers at IQR bounds
    Cap,
    /// Remove rows containing outliers
    Remove,
    /// Replace outliers with median
    Median,
    /// Keep outliers as-is
    Keep,
}

impl From<CliOutlierStrategy> for OutlierStrategy {
    fn from(cli: CliOutlierStrategy) -> Self {
        match cli {
            CliOutlierStrategy::Cap => OutlierStrategy::Cap,
            CliOutlierStrategy::Remove => OutlierStrategy::Remove,
            CliOutlierStrategy::Median => OutlierStrategy::Median,
            CliOutlierStrategy::Keep => OutlierStrategy::Keep,
        }
    }
}

/// CLI-compatible numeric imputation strategy enum
#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliNumericImputation {
    /// Use the mean of non-null values
    Mean,
    /// Use the median of non-null values
    Median,
    /// Use K-Nearest Neighbors imputation
    Knn,
    /// Use zero as the fill value
    Zero,
    /// Drop rows with missing values
    Drop,
}

impl From<CliNumericImputation> for NumericImputation {
    fn from(cli: CliNumericImputation) -> Self {
        match cli {
            CliNumericImputation::Mean => NumericImputation::Mean,
            CliNumericImputation::Median => NumericImputation::Median,
            CliNumericImputation::Knn => NumericImputation::Knn,
            CliNumericImputation::Zero => NumericImputation::Zero,
            CliNumericImputation::Drop => NumericImputation::Drop,
        }
    }
}

/// CLI-compatible categorical imputation strategy enum
#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliCategoricalImputation {
    /// Use the most frequent value (mode)
    Mode,
    /// Use a constant value ("Unknown")
    Constant,
    /// Drop rows with missing values
    Drop,
}

impl From<CliCategoricalImputation> for CategoricalImputation {
    fn from(cli: CliCategoricalImputation) -> Self {
        match cli {
            CliCategoricalImputation::Mode => CategoricalImputation::Mode,
            CliCategoricalImputation::Constant => CategoricalImputation::Constant,
            CliCategoricalImputation::Drop => CategoricalImputation::Drop,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    author = "Lex Machina Team",
    version,
    about = "AI-Driven Data Preprocessing Pipeline",
    long_about = "A high-performance data preprocessing tool for machine learning.\n\n\
                  ENVIRONMENT VARIABLES:\n  \
                  OPENROUTER_API_KEY    API key for OpenRouter (required for AI mode)\n\n\
                  EXAMPLES:\n  \
                  # Basic usage with auto-detection\n  \
                  lex_processing -i data.csv\n\n  \
                  # Specify target column and output\n  \
                  lex_processing -i data.csv --target Survived -o results/\n\n  \
                  # Dry run to preview actions\n  \
                  lex_processing -i data.csv --dry-run\n\n  \
                  # Rule-based mode (no AI)\n  \
                  lex_processing -i data.csv --no-ai"
)]
struct Args {
    /// Path to the CSV file to process
    #[arg(short, long)]
    input: String,

    /// Output directory for results
    #[arg(short, long, default_value = "./outputs")]
    output: String,

    /// Custom output file name (without extension)
    ///
    /// If not specified, uses "processed_dataset_{problem_type}"
    #[arg(long)]
    output_name: Option<String>,

    /// Target column for ML prediction
    ///
    /// If not specified, the pipeline will auto-detect the target
    #[arg(short, long)]
    target: Option<String>,

    /// Preview what the pipeline will do without processing
    ///
    /// Shows dataset profile, detected issues, and proposed actions
    #[arg(long)]
    dry_run: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Disable AI decisions (use rule-based only)
    #[arg(long, default_value = "false")]
    no_ai: bool,

    /// Suppress progress output (only show errors and final result)
    #[arg(short, long)]
    quiet: bool,

    /// Missing column threshold (0.0 - 1.0)
    ///
    /// Columns with missing values above this percentage will be dropped
    #[arg(long, default_value = "0.7")]
    missing_col_threshold: f64,

    /// Missing row threshold (0.0 - 1.0)
    ///
    /// Rows with missing values above this percentage will be dropped
    #[arg(long, default_value = "0.8")]
    missing_row_threshold: f64,

    /// Strategy for handling outliers
    #[arg(long, value_enum, default_value = "cap")]
    outlier_strategy: CliOutlierStrategy,

    /// Strategy for imputing missing numeric values
    #[arg(long, value_enum, default_value = "median")]
    numeric_imputation: CliNumericImputation,

    /// Strategy for imputing missing categorical values
    #[arg(long, value_enum, default_value = "mode")]
    categorical_imputation: CliCategoricalImputation,

    /// Number of neighbors for KNN imputation
    #[arg(long, default_value = "5")]
    knn_neighbors: usize,

    /// Disable type correction
    #[arg(long, default_value = "false")]
    no_type_correction: bool,

    /// Output JSON to stdout instead of human-readable summary
    ///
    /// Disables all progress logs; only outputs the final JSON report.
    /// Useful for piping to other tools: `... --json | jq .problem_type`
    #[arg(long)]
    json: bool,

    /// Write a detailed JSON report to the output directory
    ///
    /// The report will be saved as <input_name>_report.json
    #[arg(short = 'r', long)]
    emit_report: bool,
}

/// Initialize the tracing subscriber for logging.
///
/// When `json_output` is true, logging is completely disabled to ensure
/// only JSON is written to stdout.
fn init_logging(level: &str, quiet: bool, json_output: bool) {
    // If JSON output is requested, don't initialize any logging
    // This ensures stdout only contains the JSON report
    if json_output {
        return;
    }

    use tracing_subscriber::EnvFilter;

    let effective_level = if quiet { "warn" } else { level };

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(effective_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging (disabled if --json is set)
    init_logging(&args.log_level, args.quiet, args.json);

    // Load environment variables from .env file
    dotenv().ok();

    // Validate input file exists
    if !std::path::Path::new(&args.input).exists() {
        return Err(anyhow!("Input file not found: {}", args.input));
    }

    // Create output directory if needed
    if !args.dry_run && !std::path::Path::new(&args.output).exists() {
        std::fs::create_dir_all(&args.output)?;
        info!("Created output directory: {}", args.output);
    }

    // Load dataset first (needed for both dry-run and full processing)
    info!("Loading dataset from: {}", args.input);
    let data = load_csv_with_fallbacks(&args.input)?;
    info!("Dataset loaded successfully: {:?}", data.shape());

    // Handle dry-run mode
    if args.dry_run {
        return run_dry_run(&args, &data);
    }

    // Build configuration
    // Note: generate_reports is set to false - we handle report output via --emit-report flag
    let mut config_builder = PipelineConfig::builder()
        .output_dir(&args.output)
        .use_ai_decisions(!args.no_ai)
        .enable_type_correction(!args.no_type_correction)
        .missing_column_threshold(args.missing_col_threshold)
        .missing_row_threshold(args.missing_row_threshold)
        .outlier_strategy(args.outlier_strategy.into())
        .numeric_imputation(args.numeric_imputation.into())
        .categorical_imputation(args.categorical_imputation.into())
        .knn_neighbors(args.knn_neighbors)
        .generate_reports(false); // Disable internal report generation; handled by CLI

    if let Some(ref name) = args.output_name {
        config_builder = config_builder.output_name(name);
    }

    if let Some(ref target) = args.target {
        config_builder = config_builder.target_column(target);
    }

    let config = config_builder.build()?;

    // Build pipeline
    let pipeline = build_pipeline(&args, config)?;

    run_pipeline(pipeline, &args, data)
}

/// Run dry-run mode - show what would happen without processing
///
/// Note: This function uses `println!` intentionally for user-facing CLI output.
/// Unlike logging (`info!`, `debug!`), this output should always be visible
/// regardless of log level settings since it's the primary purpose of --dry-run.
fn run_dry_run(args: &Args, data: &DataFrame) -> Result<()> {
    use lex_processing::{DataQualityAnalyzer, RuleBasedDecisionEngine, TypeCorrector, PipelineConfig};
    use lex_processing::profiler::DataProfiler;
    use lex_processing::decisions::DecisionEngine;

    println!("\n{}", "=".repeat(80));
    println!("DRY RUN - Preview of preprocessing actions");
    println!("{}\n", "=".repeat(80));

    // 1. Dataset Overview
    println!("DATASET OVERVIEW");
    println!("{}", "-".repeat(40));
    println!("  File: {}", args.input);
    println!("  Rows: {}", data.height());
    println!("  Columns: {}", data.width());
    println!();

    // 2. Profile the dataset
    println!("COLUMN PROFILES");
    println!("{}", "-".repeat(40));

    let profile = DataProfiler::profile_dataset(data)?;

    // Table header
    println!(
        "{:<20} {:<12} {:<12} {:<10} {:<15}",
        "Column", "Type", "Role", "Missing %", "Unique"
    );
    println!("{}", "-".repeat(70));

    for col in &profile.column_profiles {
        println!(
            "{:<20} {:<12} {:<12} {:<10.1} {:<15}",
            truncate_str(&col.name, 19),
            col.inferred_type,
            col.inferred_role,
            col.null_percentage,
            col.unique_count
        );
    }
    println!();

    // 3. Auto cleaning preview
    println!("AUTO-CLEANING PREVIEW");
    println!("{}", "-".repeat(40));

    // Preview high missing columns
    let high_missing_cols: Vec<&String> = profile
        .column_profiles
        .iter()
        .filter(|col| col.null_percentage > args.missing_col_threshold * 100.0)
        .map(|col| &col.name)
        .collect();

    if high_missing_cols.is_empty() {
        println!("  No columns exceed {:.0}% missing threshold", args.missing_col_threshold * 100.0);
    } else {
        println!("  Will drop columns with >{:.0}% missing: {:?}", 
                 args.missing_col_threshold * 100.0, high_missing_cols);
    }

    // Check for duplicates
    let duplicate_count = data.height() - data.unique::<&str, &str>(None, UniqueKeepStrategy::First, None)?.height();
    if duplicate_count > 0 {
        println!("  Will remove {} duplicate rows", duplicate_count);
    } else {
        println!("  No duplicate rows found");
    }
    println!();

    // 4. Type corrections preview
    if !args.no_type_correction {
        println!("TYPE CORRECTIONS PREVIEW");
        println!("{}", "-".repeat(40));

        let type_corrector = TypeCorrector;
        let mismatches = type_corrector.detect_mismatches(data, &profile.column_profiles)?;

        if mismatches.is_empty() {
            println!("  No type corrections needed");
        } else {
            for mismatch in &mismatches {
                println!("  - {}", mismatch);
            }
        }
        println!();
    }

    // 5. Data quality issues
    println!("DATA QUALITY ISSUES");
    println!("{}", "-".repeat(40));

    let issues = DataQualityAnalyzer::identify_issues(&profile, data)?;

    let display_issues: Vec<_> = issues.iter()
        .filter(|i| i.issue_type != "problem_type_selection")  // Skip meta-issue
        .collect();

    if display_issues.is_empty() {
        println!("  No data quality issues detected");
    } else {
        for issue in display_issues {
            let cols = issue.affected_columns.join(", ");
            println!("  - [{}] {}: {}", issue.severity, cols, issue.description);
        }
    }
    println!();

    // 6. Target column and problem type
    println!("ML PROBLEM SETUP");
    println!("{}", "-".repeat(40));

    if let Some(ref target) = args.target {
        // Validate the specified target exists
        if data.column(target).is_ok() {
            println!("  Target column: {} (user-specified)", target);
            
            // Determine problem type based on target
            let target_series = data.column(target)?;
            let unique_count = target_series.n_unique()?;
            let problem_type = if unique_count <= 2 {
                "binary classification"
            } else if unique_count <= 10 {
                "multiclass classification"
            } else {
                "regression"
            };
            println!("  Problem type: {} (estimated)", problem_type);
        } else {
            println!("  WARNING: Specified target '{}' not found in dataset!", target);
            println!("  Available columns: {:?}", data.get_column_names());
        }
    } else {
        // Use rule engine to suggest target
        let config = PipelineConfig::default();
        let rule_engine = RuleBasedDecisionEngine::new(config);
        match rule_engine.finalize_problem_setup(&profile, &std::collections::HashMap::new(), data) {
            Ok((problem_type, target_col)) => {
                println!("  Target column: {} (auto-detected)", target_col);
                println!("  Problem type: {}", problem_type);
            }
            Err(e) => {
                println!("  Could not auto-detect target: {}", e);
            }
        }
    }
    println!();

    // 7. Proposed actions summary
    println!("PROPOSED ACTIONS");
    println!("{}", "-".repeat(40));
    println!("  1. Clean dataset (remove high-null columns/rows)");
    if !args.no_type_correction {
        println!("  2. Correct column types");
    }
    println!("  3. Impute missing values (numeric: {:?}, categorical: {:?})", 
             args.numeric_imputation, args.categorical_imputation);
    println!("  4. Handle outliers (strategy: {:?})", args.outlier_strategy);
    println!("  5. Generate analysis report");
    println!();

    // 8. Output files
    println!("OUTPUT FILES (will be created)");
    println!("{}", "-".repeat(40));
    let output_name = args.output_name.as_deref().unwrap_or("processed_dataset_{problem_type}");
    println!("  - {}/{}.csv", args.output, output_name);
    if args.emit_report {
        let input_stem = extract_file_stem(&args.input);
        println!("  - {}/{}_report.json", args.output, input_stem);
    }
    println!();

    println!("{}", "=".repeat(80));
    println!("To execute this preprocessing, run without --dry-run");
    if !args.emit_report {
        println!("Add --emit-report to save a detailed JSON report");
    }
    println!("{}", "=".repeat(80));

    Ok(())
}

/// Truncate a string to max length with ellipsis
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Build the pipeline with optional AI support
#[cfg(feature = "ai")]
fn build_pipeline(args: &Args, config: PipelineConfig) -> Result<Pipeline> {
    if args.no_ai {
        info!("Running in rule-based mode (AI disabled)");
        return build_pipeline_without_ai(args, config);
    }

    // // Try to get API key
    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        warn!("OPENROUTER_API_KEY not set. Falling back to rule-based decisions.");
        String::new()
    });
    // Try to get API key
    // let api_key = env::var("GEMINI_API_KEY").unwrap_or_else(|_| {
    //     warn!("GEMINI_API_KEY not set. Falling back to rule-based decisions.");
    //     String::new()
    // });
    
    // if api_key.is_empty() {
    //     info!("Running in rule-based mode (no API key)");
    //     return build_pipeline_without_ai(args, config);
    // }

    info!("Running with AI-powered decisions (OpenRouter)");

    // Create AI provider
    let provider = Arc::new(OpenRouterProvider::new(api_key)?);
    // let provider = Arc::new(GeminiProvider::new(api_key)?);

    let mut builder = Pipeline::builder()
        .config(config)
        .ai_provider(provider);

    if !args.quiet {
        builder = builder.on_progress(|update| {
            info!(
                "[{:.0}%] {}: {}",
                update.progress * 100.0,
                update.stage.display_name(),
                update.message
            );
        });
    }

    Ok(builder.build()?)
}

/// Build pipeline without AI (shared logic)
fn build_pipeline_without_ai(args: &Args, config: PipelineConfig) -> Result<Pipeline> {
    // Override config to disable AI, but preserve all other settings
    let mut config_builder = PipelineConfig::builder()
        .output_dir(config.output_dir.to_string_lossy().as_ref())
        .use_ai_decisions(false)
        .enable_type_correction(config.enable_type_correction)
        .missing_column_threshold(config.missing_column_threshold)
        .missing_row_threshold(config.missing_row_threshold)
        .outlier_strategy(config.outlier_strategy)
        .numeric_imputation(config.default_numeric_imputation)
        .categorical_imputation(config.default_categorical_imputation)
        .knn_neighbors(config.knn_neighbors)
        .generate_reports(config.generate_reports); // Preserve generate_reports setting
    
    // Preserve target column if set
    if let Some(ref target) = config.target_column {
        config_builder = config_builder.target_column(target);
    }
    
    // Preserve output name if set
    if let Some(ref name) = config.output_name {
        config_builder = config_builder.output_name(name);
    }
    
    let config = config_builder.build()?;

    let mut builder = Pipeline::builder().config(config);

    if !args.quiet {
        builder = builder.on_progress(|update| {
            info!(
                "[{:.0}%] {}: {}",
                update.progress * 100.0,
                update.stage.display_name(),
                update.message
            );
        });
    }

    Ok(builder.build()?)
}

/// Build the pipeline without AI support (fallback when "ai" feature is disabled)
#[cfg(not(feature = "ai"))]
fn build_pipeline(args: &Args, config: PipelineConfig) -> Result<Pipeline> {
    if !args.no_ai {
        warn!("AI support not compiled in. Using rule-based mode.");
        warn!("Compile with --features ai to enable AI support.");
    }
    info!("Running in rule-based mode");

    build_pipeline_without_ai(args, config)
}

/// Run pipeline and print results
fn run_pipeline(pipeline: Pipeline, args: &Args, data: DataFrame) -> Result<()> {
    info!("{}", "=".repeat(80));
    info!("Starting automated preprocessing pipeline...");
    info!("{}", "=".repeat(80));

    let original_shape = data.shape();
    let original_df = data.clone();
    let result = pipeline.process(data);

    match result {
        Ok(pipeline_result) => {
            handle_pipeline_output(&pipeline_result, &original_df, original_shape, args)
        }
        Err(e) => {
            error!("Pipeline failed: {}", e);
            Err(anyhow!("Pipeline failed: {}", e))
        }
    }
}

/// Handle pipeline output based on CLI flags.
///
/// Output behavior:
/// - Default: Print human-readable summary to stdout
/// - `--json`: Print JSON to stdout only (no logs)
/// - `--emit-report`: Write JSON report to file
fn handle_pipeline_output(
    result: &lex_processing::PipelineResult,
    original_df: &DataFrame,
    original_shape: (usize, usize),
    args: &Args,
) -> Result<()> {
    if !result.success {
        error!(
            "Preprocessing failed: {}",
            result.error.as_ref().unwrap_or(&"Unknown error".to_string())
        );
        return Err(anyhow!("Processing failed"));
    }

    // Read the final DataFrame from the output file
    let final_df = read_output_dataframe(result, &args.output)?;

    // Create the comprehensive report
    let output_file_path = result
        .problem_type
        .as_ref()
        .map(|pt| format!("{}/processed_dataset_{}.csv", args.output, pt));

    // Get profile from the result or create a minimal one
    let profile = lex_processing::profiler::DataProfiler::profile_dataset(&final_df)
        .unwrap_or_else(|_| create_minimal_profile(&final_df));

    let report = ReportGenerator::build_comprehensive_report(
        &args.input,
        output_file_path.as_deref(),
        result,
        original_df,
        &final_df,
        &profile,
    );

    // Handle JSON output to stdout
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    // Handle file report output
    if args.emit_report {
        let input_stem = extract_file_stem(&args.input);
        let generator = ReportGenerator::new(std::path::PathBuf::from(&args.output), None);
        let report_path = generator.write_report_to_file(&report, &input_stem)?;
        info!("Report written to: {}", report_path.display());
    }

    // Print human-readable summary (default behavior)
    print_human_readable_summary(&report, original_shape, args);

    Ok(())
}

/// Extract the file stem (name without extension) from a path.
fn extract_file_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .to_string()
}

/// Read the output DataFrame from the generated CSV file.
fn read_output_dataframe(
    result: &lex_processing::PipelineResult,
    output_dir: &str,
) -> Result<DataFrame> {
    if let Some(ref problem_type) = result.problem_type {
        let output_path = format!("{}/processed_dataset_{}.csv", output_dir, problem_type);
        if Path::new(&output_path).exists() {
            return CsvReadOptions::default()
                .with_has_header(true)
                .try_into_reader_with_file_path(Some(std::path::PathBuf::from(&output_path)))?
                .finish()
                .map_err(|e| anyhow!("Failed to read output file: {}", e));
        }
    }

    // Return an empty DataFrame if we can't read the output
    Ok(DataFrame::empty())
}

/// Create a minimal dataset profile when the full one isn't available.
fn create_minimal_profile(df: &DataFrame) -> lex_processing::DatasetProfile {
    lex_processing::DatasetProfile {
        shape: (df.height(), df.width()),
        column_profiles: Vec::new(),
        target_candidates: Vec::new(),
        problem_type_candidates: Vec::new(),
        complexity_indicators: std::collections::HashMap::new(),
        duplicate_count: 0,
        duplicate_percentage: 0.0,
    }
}

/// Print a human-readable summary of the preprocessing results.
///
/// This is the default output when neither `--json` nor `--quiet` are specified.
fn print_human_readable_summary(
    report: &ComprehensiveReport,
    original_shape: (usize, usize),
    args: &Args,
) {
    let summary = &report.processing_summary;

    println!();
    println!("{}", "=".repeat(80));
    println!("PREPROCESSING COMPLETE");
    println!("{}", "=".repeat(80));
    println!();

    // Input/Output info
    println!(
        "Input:  {} ({} rows x {} columns)",
        report.input_file, original_shape.0, original_shape.1
    );

    if let Some(ref output_file) = report.output_file {
        println!(
            "Output: {} ({} rows x {} columns)",
            output_file, summary.rows_after, summary.columns_after
        );
    } else {
        println!(
            "Output: {}/processed_dataset_{}.csv ({} rows x {} columns)",
            args.output,
            report.problem_type.as_deref().unwrap_or("unknown"),
            summary.rows_after,
            summary.columns_after
        );
    }
    println!();

    // Problem type and target
    if let Some(ref problem_type) = report.problem_type {
        println!("Problem Type: {}", problem_type);
    }
    if let Some(ref target) = report.target_column {
        println!("Target Column: {}", target);
    }
    println!();

    // Processing summary
    println!("Processing Summary:");
    println!("  Duration: {}ms", summary.duration_ms);
    println!(
        "  Rows: {} -> {} ({} removed)",
        summary.rows_before, summary.rows_after, summary.rows_removed
    );
    println!(
        "  Columns: {} -> {} ({} removed)",
        summary.columns_before, summary.columns_after, summary.columns_removed
    );
    println!(
        "  Data Quality: {:.1}% -> {:.1}%",
        summary.data_quality_before * 100.0,
        summary.data_quality_after * 100.0
    );
    println!(
        "  Issues: {} found, {} resolved",
        summary.issues_found, summary.issues_resolved
    );
    println!();

    // Actions taken
    if !report.cleaning_actions.is_empty() || !report.processing_steps.is_empty() {
        println!("Actions Taken:");
        
        // Show key cleaning actions
        for action in report.cleaning_actions.iter().take(5) {
            println!("  - {}", action);
        }
        
        // Show key processing steps (filter out verbose ones)
        let key_steps: Vec<_> = report
            .processing_steps
            .iter()
            .filter(|s| {
                s.contains("impute") || s.contains("Impute") || 
                s.contains("KNN") || s.contains("median") ||
                s.contains("mode") || s.contains("outlier")
            })
            .take(5)
            .collect();
        
        for step in key_steps {
            println!("  - {}", step);
        }
        
        if report.cleaning_actions.len() + report.processing_steps.len() > 10 {
            println!(
                "  ... and {} more actions",
                report.cleaning_actions.len() + report.processing_steps.len() - 10
            );
        }
        println!();
    }

    // Warnings
    if !summary.warnings.is_empty() {
        println!("Warnings:");
        for warning in &summary.warnings {
            println!("  ! {}", warning);
        }
        println!();
    }

    // Hints for more output options
    println!("Use --json for machine-readable output");
    println!("Use --emit-report to save detailed JSON report");
    println!("{}", "=".repeat(80));
}

/// Load CSV with multiple fallback strategies
fn load_csv_with_fallbacks(path: &str) -> Result<DataFrame> {
    use std::path::PathBuf;
    
    // Strategy 1: Standard loading with quote handling
    match CsvReadOptions::default()
        .with_infer_schema_length(Some(100))
        .with_has_header(true)
        .with_parse_options(CsvParseOptions::default().with_quote_char(Some(b'"')))
        .try_into_reader_with_file_path(Some(PathBuf::from(path)))?
        .finish()
    {
        Ok(df) => return Ok(df),
        Err(e) => {
            debug!("Standard loading failed: {}", e);
        }
    }

    // Strategy 2: Without quote handling
    match CsvReadOptions::default()
        .with_infer_schema_length(Some(100))
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(PathBuf::from(path)))?
        .finish()
    {
        Ok(df) => return Ok(df),
        Err(e) => {
            debug!("Loading without quotes failed: {}", e);
        }
    }

    // Strategy 3: Pre-clean content
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let cleaned = clean_csv_content(&content);
            use std::io::Cursor;
            let cursor = Cursor::new(cleaned);

            CsvReadOptions::default()
                .with_infer_schema_length(Some(100))
                .with_has_header(true)
                .into_reader_with_file_handle(cursor)
                .finish()
                .map_err(|e| e.into())
        }
        Err(e) => {
            error!("Could not read file: {}", e);
            Err(e.into())
        }
    }
}

/// Clean CSV content
fn clean_csv_content(content: &str) -> String {
    content
        .replace("\"\"\"", "\"")
        .replace("\"\"", "\"")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
