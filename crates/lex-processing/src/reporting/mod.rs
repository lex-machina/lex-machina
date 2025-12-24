//! Report generation module.
//!
//! This module provides functionality for generating analysis reports
//! and saving processed datasets.
//!
//! # Comprehensive Reports
//!
//! Use [`ComprehensiveReport`] to generate unified reports suitable for:
//! - JSON output to stdout (`--json` CLI flag)
//! - JSON file output (`--emit-report` CLI flag)
//! - Programmatic access in library mode
//!
//! # Example
//!
//! ```rust,ignore
//! use data_preprocessing_pipeline::reporting::{ReportGenerator, ComprehensiveReport};
//!
//! // Build a comprehensive report from pipeline results
//! let report = ReportGenerator::build_comprehensive_report(
//!     "data/train.csv",
//!     Some("output/processed.csv"),
//!     &pipeline_result,
//!     &original_df,
//!     &final_df,
//!     &profile,
//! );
//!
//! // Print as JSON
//! println!("{}", serde_json::to_string_pretty(&report)?);
//!
//! // Or write to file
//! let generator = ReportGenerator::new(PathBuf::from("output"), None);
//! generator.write_report_to_file(&report, "train")?;
//! ```

mod generator;

pub use generator::{
    AlgorithmRationale, ComprehensiveReport, DatasetProfileSummary, ProcessingSummaryReport,
    QualityAssessment, ReportGenerator, ReportParams,
};
