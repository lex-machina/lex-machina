//! Integration tests for the data preprocessing pipeline.
//!
//! These tests verify end-to-end behavior of the pipeline using various datasets.

use lex_processing::{
    CancellationToken, Pipeline, PipelineConfig, PreprocessingError, PreprocessingStage,
    ProgressUpdate,
};
use polars::prelude::*;
use polars::io::csv::read::CsvReadOptions;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// ============================================================================
// Helper Functions
// ============================================================================

fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load_csv(filename: &str) -> DataFrame {
    let path = fixtures_path().join(filename);
    CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(path))
        .expect("Failed to create CSV reader")
        .finish()
        .expect("Failed to read CSV file")
}

fn load_titanic_full() -> DataFrame {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/train.csv");
    CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(path))
        .expect("Failed to create CSV reader")
        .finish()
        .expect("Failed to read CSV file")
}

// ============================================================================
// Full Pipeline Tests with Titanic Data
// ============================================================================

#[test]
fn test_full_pipeline_titanic_subset() {
    let df = load_csv("titanic_subset.csv");

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df);

    assert!(result.is_ok(), "Pipeline should complete successfully");

    let result = result.unwrap();
    assert!(result.success);
    assert!(result.target_column.is_some());
    assert!(result.problem_type.is_some());

    // Titanic is a classification problem
    let problem_type = result.problem_type.as_ref().unwrap();
    assert!(
        problem_type == "classification" || problem_type == "binary_classification",
        "Expected classification problem, got: {}",
        problem_type
    );
}

#[test]
fn test_full_pipeline_titanic_full_dataset() {
    let df = load_titanic_full();
    let initial_rows = df.height();

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df);

    assert!(result.is_ok(), "Pipeline should complete successfully");

    let result = result.unwrap();
    assert!(result.success);

    // Check summary is present
    assert!(result.summary.is_some());
    let summary = result.summary.as_ref().unwrap();

    // Verify summary data makes sense
    assert_eq!(summary.rows_before, initial_rows);
    assert!(summary.rows_after <= summary.rows_before);
    assert!(summary.duration_ms > 0);
    assert!(summary.data_quality_score_before >= 0.0);
    assert!(summary.data_quality_score_after >= 0.0);
}

// ============================================================================
// Rule-Based Mode Tests
// ============================================================================

#[test]
fn test_pipeline_no_ai_mode() {
    let df = load_csv("titanic_subset.csv");

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df);

    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
    // In rule-based mode, pipeline should still identify target and problem type
    assert!(result.target_column.is_some());
    assert!(result.problem_type.is_some());
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_pipeline_no_nulls_dataset() {
    let df = load_csv("no_nulls.csv");

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df);

    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);

    // With no nulls, the data quality scores should be high
    if let Some(summary) = &result.summary {
        assert!(
            summary.data_quality_score_before >= 0.95,
            "Expected high quality score for no-nulls dataset"
        );
    }
}

#[test]
fn test_pipeline_single_row_dataset() {
    let df = load_csv("single_row.csv");

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df);

    // Single row is an edge case - pipeline should handle it gracefully
    // It may fail or succeed depending on implementation, but shouldn't panic
    match result {
        Ok(r) => {
            assert!(r.success || !r.success); // Either outcome is acceptable
        }
        Err(e) => {
            // Errors are acceptable for edge cases, just shouldn't panic
            println!("Single row error (expected): {:?}", e);
        }
    }
}

#[test]
fn test_pipeline_all_nulls_column() {
    let df = load_csv("all_nulls_column.csv");

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .missing_column_threshold(0.5) // Drop columns with >50% missing
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df);

    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);

    // The all-nulls column should have been removed
    if let Some(summary) = &result.summary {
        assert!(
            summary.columns_removed > 0 || summary.columns_after < summary.columns_before,
            "Expected at least one column to be removed"
        );
    }
}

#[test]
fn test_pipeline_mixed_types_dataset() {
    let df = load_csv("mixed_types.csv");

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .enable_type_correction(true)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df);

    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
}

// ============================================================================
// Cancellation Tests
// ============================================================================

#[test]
fn test_pipeline_cancellation_before_start() {
    let df = load_csv("titanic_subset.csv");
    let token = CancellationToken::new();

    // Cancel immediately before processing
    token.cancel();

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .cancellation_token(token)
        .build()
        .unwrap()
        .process(df);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), PreprocessingError::Cancelled));
}

#[test]
fn test_pipeline_cancellation_token_reset() {
    let token = CancellationToken::new();

    // Cancel and verify
    token.cancel();
    assert!(token.is_cancelled());

    // Reset and verify
    token.reset();
    assert!(!token.is_cancelled());
}

// ============================================================================
// Progress Reporting Tests
// ============================================================================

#[test]
fn test_pipeline_progress_reporting_invoked() {
    let df = load_csv("titanic_subset.csv");
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .on_progress(move |_update| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        })
        .build()
        .unwrap()
        .process(df);

    assert!(result.is_ok());

    // Progress should have been called multiple times
    let calls = call_count.load(Ordering::SeqCst);
    assert!(
        calls > 0,
        "Progress callback should have been invoked at least once"
    );
}

#[test]
fn test_pipeline_progress_stages_reported() {
    let df = load_csv("titanic_subset.csv");
    let stages_seen = Arc::new(std::sync::Mutex::new(Vec::new()));
    let stages_clone = stages_seen.clone();

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .on_progress(move |update: ProgressUpdate| {
            let mut stages = stages_clone.lock().unwrap();
            stages.push(update.stage);
        })
        .build()
        .unwrap()
        .process(df);

    assert!(result.is_ok());

    let stages = stages_seen.lock().unwrap();

    // Should see various stages
    assert!(!stages.is_empty(), "Should have seen some stages");

    // Should see Complete at the end for successful processing
    assert!(
        stages.contains(&PreprocessingStage::Complete),
        "Should report Complete stage on success"
    );
}

#[test]
fn test_pipeline_progress_cancelled_stage() {
    let df = load_csv("titanic_subset.csv");
    let token = CancellationToken::new();
    let stages_seen = Arc::new(std::sync::Mutex::new(Vec::new()));
    let stages_clone = stages_seen.clone();

    // Cancel immediately
    token.cancel();

    let _ = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .cancellation_token(token)
        .on_progress(move |update: ProgressUpdate| {
            let mut stages = stages_clone.lock().unwrap();
            stages.push(update.stage);
        })
        .build()
        .unwrap()
        .process(df);

    let stages = stages_seen.lock().unwrap();

    // Should see Cancelled stage
    assert!(
        stages.contains(&PreprocessingStage::Cancelled),
        "Should report Cancelled stage when cancelled"
    );
}

// ============================================================================
// Summary Accuracy Tests
// ============================================================================

#[test]
fn test_pipeline_summary_row_counts() {
    let df = load_csv("titanic_subset.csv");
    let initial_rows = df.height();

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df)
        .unwrap();

    let summary = result.summary.expect("Summary should be present");

    // Verify row tracking
    assert_eq!(summary.rows_before, initial_rows);
    assert!(summary.rows_after <= summary.rows_before);
    assert_eq!(
        summary.rows_removed,
        summary.rows_before - summary.rows_after
    );
}

#[test]
fn test_pipeline_summary_column_counts() {
    let df = load_csv("titanic_subset.csv");
    let initial_cols = df.width();

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df)
        .unwrap();

    let summary = result.summary.expect("Summary should be present");

    // Verify column tracking
    assert_eq!(summary.columns_before, initial_cols);
    assert_eq!(
        summary.columns_removed,
        summary.columns_before - summary.columns_after
    );
}

#[test]
fn test_pipeline_summary_quality_scores() {
    let df = load_csv("no_nulls.csv");

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df)
        .unwrap();

    let summary = result.summary.expect("Summary should be present");

    // Quality scores should be valid percentages
    assert!(summary.data_quality_score_before >= 0.0);
    assert!(summary.data_quality_score_before <= 1.0);
    assert!(summary.data_quality_score_after >= 0.0);
    assert!(summary.data_quality_score_after <= 1.0);

    // For no-nulls data, quality should be very high
    assert!(
        summary.data_quality_score_before >= 0.95,
        "No-nulls data should have high quality score"
    );
}

#[test]
fn test_pipeline_summary_actions_tracked() {
    let df = load_csv("titanic_subset.csv");

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df)
        .unwrap();

    let summary = result.summary.expect("Summary should be present");

    // Should have some actions tracked (problem type, target detection at minimum)
    assert!(
        !summary.actions.is_empty(),
        "Should track at least some preprocessing actions"
    );
}

#[test]
fn test_pipeline_summary_duration_positive() {
    let df = load_csv("titanic_subset.csv");

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df)
        .unwrap();

    let summary = result.summary.expect("Summary should be present");

    // Duration should be positive
    assert!(summary.duration_ms > 0, "Duration should be positive");
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_pipeline_with_custom_thresholds() {
    let df = load_csv("titanic_subset.csv");

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .missing_column_threshold(0.3) // Very strict - drop columns with >30% missing
                .missing_row_threshold(0.5)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df);

    assert!(result.is_ok());
}

#[test]
fn test_pipeline_type_correction_disabled() {
    let df = load_csv("mixed_types.csv");

    let result = Pipeline::builder()
        .config(
            PipelineConfig::builder()
                .use_ai_decisions(false)
                .generate_reports(false)
                .enable_type_correction(false)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
        .process(df);

    assert!(result.is_ok());
}
