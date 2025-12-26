//! Data Preprocessing Pipeline Library
//!
//! A high-performance, AI-optional data preprocessing library built with Rust and Polars.
//!
//! # Overview
//!
//! This library provides automated data preprocessing capabilities including:
//!
//! - **Data Profiling**: Automatic type inference, role detection, and statistical analysis
//! - **Data Cleaning**: Duplicate removal, missing value handling, outlier detection
//! - **Type Correction**: Intelligent type conversion and correction
//! - **AI-Powered Decisions**: Optional AI integration for preprocessing strategy selection
//! - **Rule-Based Fallback**: Works without AI using heuristic-based decisions
//! - **Progress Reporting**: Real-time progress updates with cancellation support
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use lex_processing::{Pipeline, PipelineConfig, CancellationToken};
//! use lex_processing::ai::OpenRouterProvider;
//! use polars::prelude::*;
//! use std::sync::Arc;
//!
//! // Load data
//! let df = CsvReader::from_path("data.csv")?.finish()?;
//!
//! // Option 1: With AI-powered decisions and progress reporting
//! let provider = Arc::new(OpenRouterProvider::new(api_key)?);
//! let token = CancellationToken::new();
//!
//! let result = Pipeline::builder()
//!     .ai_provider(provider)
//!     .cancellation_token(token.clone())
//!     .on_progress(|update| {
//!         println!("[{:.0}%] {}", update.progress * 100.0, update.message);
//!     })
//!     .build()?
//!     .process(df)?;
//!
//! // Option 2: Rule-based only (no AI required)
//! let config = PipelineConfig::builder()
//!     .use_ai_decisions(false)
//!     .missing_column_threshold(0.5)
//!     .build()?;
//!
//! let result = Pipeline::builder()
//!     .config(config)
//!     .build()?
//!     .process(df)?;
//!
//! println!("Preprocessing complete!");
//! println!("Problem type: {:?}", result.problem_type);
//! println!("Target column: {:?}", result.target_column);
//! ```
//!
//! # AI Providers
//!
//! The library supports multiple AI providers through the [`ai::AIProvider`] trait.
//! Currently implemented providers:
//!
//! - [`ai::OpenRouterProvider`] - OpenRouter API (supports multiple LLM models)
//! - [`ai::GeminiProvider`] - Google Gemini API
//!
//! To implement your own provider, see the [`ai`] module documentation.
//!
//! # Configuration
//!
//! Use [`PipelineConfig`] to customize preprocessing behavior:
//!
//! ```rust,ignore
//! use lex_processing::config::*;
//!
//! let config = PipelineConfig::builder()
//!     .missing_column_threshold(0.7)      // Drop columns with >70% missing
//!     .missing_row_threshold(0.8)         // Drop rows with >80% missing
//!     .outlier_strategy(OutlierStrategy::Cap)
//!     .numeric_imputation(NumericImputation::Median)
//!     .categorical_imputation(CategoricalImputation::Mode)
//!     .knn_neighbors(5)
//!     .enable_type_correction(true)
//!     .use_ai_decisions(true)
//!     .build()?;
//! ```
//!
//! # Progress Reporting
//!
//! The pipeline supports real-time progress reporting and cancellation:
//!
//! ```rust,ignore
//! use lex_processing::{Pipeline, CancellationToken, ProgressReporter, ProgressUpdate};
//! use std::sync::Arc;
//!
//! // Create a cancellation token
//! let token = CancellationToken::new();
//! let token_for_cancel = token.clone();
//!
//! // Cancel from another thread after 10 seconds
//! std::thread::spawn(move || {
//!     std::thread::sleep(std::time::Duration::from_secs(10));
//!     token_for_cancel.cancel();
//! });
//!
//! // Run with progress reporting
//! let result = Pipeline::builder()
//!     .cancellation_token(token)
//!     .on_progress(|update| {
//!         println!("[{:?}] {}", update.stage, update.message);
//!     })
//!     .build()?
//!     .process(df);
//!
//! match result {
//!     Ok(result) => println!("Success!"),
//!     Err(PreprocessingError::Cancelled) => println!("Cancelled by user"),
//!     Err(e) => println!("Error: {}", e),
//! }
//! ```

// Core modules (new subdirectory structure)
pub mod ai;
pub mod cleaner;
pub mod config;
pub mod decisions;
pub mod error;
pub mod imputers;
pub mod pipeline;
pub mod profiler;
pub mod quality;
pub mod reporting;
pub mod types;
pub mod utils;

// Re-exports for convenient access
pub use cleaner::TypeCorrector;
pub use config::{
    CategoricalImputation, ConfigValidationError, NumericImputation, OutlierStrategy,
    PipelineConfig, PipelineConfigBuilder,
};
pub use decisions::{AiDecisionEngine, DecisionEngine, RuleBasedDecisionEngine};
pub use error::{PreprocessingError, Result as PreprocessingResult, ResultExt};
pub use imputers::{KNNImputer, StatisticalImputer};
pub use pipeline::{
    CancellationToken, ClosureProgressReporter, Pipeline, PipelineBuilder, PreprocessingExecutor,
    PreprocessingStage, ProgressReporter, ProgressUpdate,
};
pub use quality::DataQualityAnalyzer;
pub use reporting::{
    AlgorithmRationale, ComprehensiveReport, DatasetProfileSummary, ProcessingSummaryReport,
    QualityAssessment, ReportGenerator, ReportParams,
};
pub use types::{
    ActionType, ColumnProfile, ColumnSummary, DataQualityIssue, DatasetProfile, DecisionQuestion,
    PipelineResult, PreprocessingAction, PreprocessingSummary, SolutionOption,
};
pub use utils::{
    DtypeCategory, clean_numeric_string, dtype_category_str, fill_numeric_nulls, fill_string_nulls,
    get_dtype_category, is_boolean_string, is_error_marker, is_numeric_dtype, parse_numeric_string,
};
