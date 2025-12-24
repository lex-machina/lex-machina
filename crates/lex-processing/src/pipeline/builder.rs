//! Main preprocessing pipeline module.
//!
//! This module provides the core `Pipeline` struct and builder for
//! orchestrating the data preprocessing workflow.

use crate::ai::AIProvider;
use crate::cleaner::{DataCleaner, TypeCorrector};
use crate::config::PipelineConfig;
use crate::decisions::{AiDecisionEngine, DecisionEngine, RuleBasedDecisionEngine};
use crate::error::{PreprocessingError, Result};
use crate::pipeline::progress::{
    CancellationToken, ClosureProgressReporter, PreprocessingStage, ProgressReporter,
    ProgressUpdate,
};
use crate::pipeline::PreprocessingExecutor;
use crate::profiler::DataProfiler;
use crate::quality::DataQualityAnalyzer;
use crate::reporting::ReportGenerator;
use crate::types::{
    ActionType, ColumnSummary, PipelineResult, PreprocessingAction, PreprocessingSummary,
};
use polars::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info};

/// The main preprocessing pipeline.
///
/// Use [`Pipeline::builder()`] to create a new pipeline with custom configuration.
///
/// # Example
///
/// ```rust,ignore
/// use lex_processing::{Pipeline, PipelineConfig, CancellationToken};
/// use lex_processing::ai::OpenRouterProvider;
/// use std::sync::Arc;
///
/// // With AI provider and progress reporting
/// let provider = Arc::new(OpenRouterProvider::new(api_key)?);
/// let token = CancellationToken::new();
///
/// let result = Pipeline::builder()
///     .ai_provider(provider)
///     .cancellation_token(token.clone())
///     .on_progress(|update| {
///         println!("[{:.0}%] {}", update.progress * 100.0, update.message);
///     })
///     .config(PipelineConfig::default())
///     .build()?
///     .process(dataframe)?;
///
/// // Without AI (rule-based)
/// let result = Pipeline::builder()
///     .config(PipelineConfig::builder().use_ai_decisions(false).build()?)
///     .build()?
///     .process(dataframe)?;
/// ```
pub struct Pipeline {
    config: PipelineConfig,
    ai_provider: Option<Arc<dyn AIProvider>>,
    progress_reporter: Option<Arc<dyn ProgressReporter>>,
    cancellation_token: CancellationToken,
    cleaner: DataCleaner,
    executor: PreprocessingExecutor,
    reporter: ReportGenerator,
    type_corrector: TypeCorrector,
}

// Ensure Pipeline is Send (can be moved to another thread)
// This is important for Tauri integration where pipeline runs in a background task
static_assertions::assert_impl_all!(Pipeline: Send);

impl Pipeline {
    /// Create a new pipeline builder.
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::default()
    }

    /// Process a DataFrame through the preprocessing pipeline.
    ///
    /// Returns a `PipelineResult` containing the cleaned data and metadata.
    ///
    /// # Errors
    ///
    /// Returns `Err(PreprocessingError::Cancelled)` if the pipeline was cancelled
    /// via the cancellation token. Other errors may occur during processing.
    pub fn process(&self, df: DataFrame) -> Result<PipelineResult> {
        match self.process_internal(df) {
            Ok(result) => {
                self.report_progress(ProgressUpdate::complete("Pipeline completed successfully"));
                Ok(result)
            }
            Err(e) => {
                if e.is_cancelled() {
                    self.report_progress(ProgressUpdate::cancelled());
                } else {
                    self.report_progress(ProgressUpdate::failed(e.to_string()));
                }
                error!("Pipeline error: {}", e);
                Err(e)
            }
        }
    }

    /// Check if cancellation has been requested.
    fn check_cancelled(&self) -> Result<()> {
        if self.cancellation_token.is_cancelled() {
            return Err(PreprocessingError::Cancelled);
        }
        Ok(())
    }

    /// Report progress if a reporter is configured.
    fn report_progress(&self, update: ProgressUpdate) {
        if let Some(reporter) = &self.progress_reporter {
            reporter.report(update);
        }
    }

    fn process_internal(&self, df: DataFrame) -> Result<PipelineResult> {
        let start_time = Instant::now();
        
        info!("Starting preprocessing pipeline...");
        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::Initializing,
            0.0,
            "Starting preprocessing pipeline...",
        ));

        // Initialize summary tracking
        let mut summary = PreprocessingSummary::new();
        summary.rows_before = df.height();
        summary.columns_before = df.width();
        
        // Calculate initial data quality score (percentage of non-null values)
        summary.data_quality_score_before = self.calculate_data_quality_score(&df);

        let mut processing_steps: Vec<String> = Vec::new();
        let mut cleaning_actions: Vec<String> = Vec::new();

        self.check_cancelled()?;

        // Step 1: Type correction (if enabled)
        let df = if self.config.enable_type_correction {
            self.report_progress(ProgressUpdate::new(
                PreprocessingStage::TypeCorrection,
                0.0,
                "Performing type correction...",
            ));
            info!("Step 1: Performing type correction...");
            
            let temp_profile = DataProfiler::profile_dataset(&df)
                .map_err(|e| PreprocessingError::ProfilingFailed(e.to_string()))?;
            
            let (corrected_df, type_steps) = self
                .type_corrector
                .correct_column_types(df, &temp_profile.column_profiles)
                .map_err(|e| PreprocessingError::CleaningFailed(e.to_string()))?;
            
            // Track type corrections in summary
            let corrections_count = type_steps.len();
            if corrections_count > 0 {
                summary.add_action(
                    PreprocessingAction::new(
                        ActionType::TypeCorrected,
                        "dataset",
                        format!("Corrected types for {} columns", corrections_count),
                    )
                    .with_details(type_steps.join("; ")),
                );
            }
            
            processing_steps.extend(type_steps);
            
            self.report_progress(ProgressUpdate::new(
                PreprocessingStage::TypeCorrection,
                1.0,
                "Type correction complete",
            ));
            
            corrected_df
        } else {
            info!("Step 1: Skipping type correction (disabled)");
            df
        };

        self.check_cancelled()?;

        // Step 2: Profile the dataset
        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::Profiling,
            0.0,
            "Profiling dataset...",
        ));
        info!("Step 2: Profiling dataset...");
        
        let profile = DataProfiler::profile_dataset(&df)
            .map_err(|e| PreprocessingError::ProfilingFailed(e.to_string()))?;
        
        debug!("Shape: {:?}", profile.shape);
        for col in &profile.column_profiles {
            debug!(
                "  {}: {} (inferred: {})",
                col.name, col.dtype, col.inferred_type
            );
        }
        
        // Initialize column summaries from profile
        for col_profile in &profile.column_profiles {
            let col_summary = ColumnSummary::new(&col_profile.name, &col_profile.dtype);
            summary.add_column_summary(col_summary);
        }
        
        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::Profiling,
            1.0,
            "Profiling complete",
        ));

        self.check_cancelled()?;

        // Step 3: Automatic cleaning
        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::Cleaning,
            0.0,
            "Performing automatic cleaning...",
        ));
        info!("Step 3: Performing automatic cleaning...");
        
        let rows_before_cleaning = df.height();
        let cols_before_cleaning = df.width();
        
        let (df, mut new_cleaning_actions) = self
            .cleaner
            .perform_automatic_cleaning(df, &profile)
            .map_err(|e| PreprocessingError::CleaningFailed(e.to_string()))?;
        
        // Track rows/columns removed during cleaning
        let rows_removed = rows_before_cleaning.saturating_sub(df.height());
        let cols_removed = cols_before_cleaning.saturating_sub(df.width());
        
        if rows_removed > 0 {
            summary.add_action(PreprocessingAction::new(
                ActionType::RowsRemoved,
                "dataset",
                format!("Removed {} rows during cleaning", rows_removed),
            ));
        }
        
        if cols_removed > 0 {
            summary.add_action(PreprocessingAction::new(
                ActionType::ColumnRemoved,
                "dataset",
                format!("Removed {} columns during cleaning", cols_removed),
            ));
        }
        
        // Track duplicate removal from cleaning actions
        for action in &new_cleaning_actions {
            if action.to_lowercase().contains("duplicate") {
                summary.add_action(PreprocessingAction::new(
                    ActionType::DuplicatesRemoved,
                    "dataset",
                    action.clone(),
                ));
            }
        }
        
        cleaning_actions.append(&mut new_cleaning_actions);
        
        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::Cleaning,
            1.0,
            "Automatic cleaning complete",
        ));

        self.check_cancelled()?;

        // Step 4: Quality analysis
        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::QualityAnalysis,
            0.0,
            "Analyzing data quality...",
        ));
        info!("Step 4: Final profiling after cleaning...");
        
        let final_profile = DataProfiler::profile_dataset(&df)
            .map_err(|e| PreprocessingError::ProfilingFailed(e.to_string()))?;
        
        debug!("Final shape: {:?}", (df.height(), df.width()));

        // Step 5: Identify issues and make decisions
        info!("Step 5: Identifying data quality issues...");
        let issues = DataQualityAnalyzer::identify_issues(&final_profile, &df)
            .map_err(|e| PreprocessingError::ProfilingFailed(e.to_string()))?;
        
        summary.issues_found = issues.len();
        
        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::QualityAnalysis,
            1.0,
            format!("Found {} data quality issues", issues.len()),
        ));

        self.check_cancelled()?;

        // Step 6: Make decisions
        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::DecisionMaking,
            0.0,
            "Making preprocessing decisions...",
        ));
        
        let (user_choices, problem_type, target_column) =
            self.make_decisions(&final_profile, &issues, &df)?;
        
        // Track problem type and target detection
        summary.add_action(PreprocessingAction::new(
            ActionType::ProblemTypeDetected,
            "dataset",
            format!("Detected problem type: {}", problem_type),
        ));
        
        summary.add_action(PreprocessingAction::new(
            ActionType::TargetIdentified,
            &target_column,
            format!("Identified target column: {}", target_column),
        ));
        
        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::DecisionMaking,
            1.0,
            format!(
                "Detected {} problem, target: {}",
                problem_type, target_column
            ),
        ));

        self.check_cancelled()?;

        // Step 7: Remove date columns if needed
        let cols_before_date_removal = df.width();
        let df = self.remove_date_columns_if_needed(&df, &problem_type, &final_profile, &target_column)?;
        
        let date_cols_removed = cols_before_date_removal.saturating_sub(df.width());
        if date_cols_removed > 0 {
            summary.add_action(PreprocessingAction::new(
                ActionType::ColumnRemoved,
                "dataset",
                format!("Removed {} date columns (not suitable for {})", date_cols_removed, problem_type),
            ));
        }

        // Step 8: Execute preprocessing (imputation, outlier handling)
        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::Imputation,
            0.0,
            "Executing preprocessing...",
        ));
        info!("Step 6: Executing preprocessing...");
        
        let (mut df_for_training, df_with_ids, execution_steps) = self
            .executor
            .execute_comprehensive_preprocessing(df, &final_profile, &user_choices, &target_column)
            .map_err(|e| PreprocessingError::CleaningFailed(e.to_string()))?;
        
        // Track imputation actions from execution steps
        for step in &execution_steps {
            let step_lower = step.to_lowercase();
            if step_lower.contains("impute") || step_lower.contains("fill") {
                summary.add_action(
                    PreprocessingAction::new(ActionType::ValueImputed, "dataset", step.clone())
                );
            } else if step_lower.contains("outlier") {
                summary.add_action(
                    PreprocessingAction::new(ActionType::OutlierHandled, "dataset", step.clone())
                );
            }
        }
        
        processing_steps.extend(execution_steps);
        
        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::Imputation,
            1.0,
            "Preprocessing execution complete",
        ));

        self.check_cancelled()?;

        // Step 9: Save output files
        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::ReportGeneration,
            0.0,
            "Saving output files...",
        ));
        info!("Step 7: Saving output files...");

        // Always save the processed CSV (this is the main output)
        self.reporter
            .generate_files(
                &problem_type,
                &df_with_ids,
                &mut df_for_training,
                &target_column,
            )
            .map_err(|e| PreprocessingError::ReportGenerationFailed(e.to_string()))?;

        // Generate JSON analysis report only if enabled
        let analysis_report = if self.config.generate_reports {
            let report = self
                .reporter
                .generate_comprehensive_analysis_report(
                    &df_with_ids,
                    &df_for_training,
                    &final_profile,
                    &processing_steps,
                    &cleaning_actions,
                    &problem_type,
                    &target_column,
                )
                .map_err(|e| PreprocessingError::ReportGenerationFailed(e.to_string()))?;
            Some(report)
        } else {
            None
        };

        self.report_progress(ProgressUpdate::new(
            PreprocessingStage::ReportGeneration,
            1.0,
            "Output files saved",
        ));

        // Finalize summary
        summary.duration_ms = start_time.elapsed().as_millis() as u64;
        summary.rows_after = df_for_training.height();
        summary.columns_after = df_for_training.width();
        summary.rows_removed = summary.rows_before.saturating_sub(summary.rows_after);
        summary.columns_removed = summary.columns_before.saturating_sub(summary.columns_after);
        summary.data_quality_score_after = self.calculate_data_quality_score(&df_for_training);
        
        // Estimate issues resolved (simple heuristic: resolved = found - remaining nulls)
        summary.issues_resolved = summary.issues_found.saturating_sub(
            self.count_remaining_issues(&df_for_training)
        );
        
        // Update column summaries with final types
        self.update_column_summaries(&mut summary, &df_for_training);
        
        // Add any warnings
        if summary.rows_removed_percentage() > 30.0 {
            summary.add_warning(format!(
                "High data loss: {:.1}% of rows were removed",
                summary.rows_removed_percentage()
            ));
        }
        if summary.columns_removed_percentage() > 30.0 {
            summary.add_warning(format!(
                "High feature loss: {:.1}% of columns were removed",
                summary.columns_removed_percentage()
            ));
        }

        Ok(PipelineResult {
            success: true,
            cleaned_data: Some(format!("{:?}", df_for_training)),
            target_column: Some(target_column),
            problem_type: Some(problem_type),
            ai_choices: user_choices,
            analysis_report,
            processing_steps,
            cleaning_actions,
            error: None,
            summary: Some(summary),
        })
    }

    /// Calculate data quality score as percentage of non-null values.
    fn calculate_data_quality_score(&self, df: &DataFrame) -> f32 {
        if df.height() == 0 || df.width() == 0 {
            return 0.0;
        }
        
        let total_cells = df.height() * df.width();
        let mut null_count = 0usize;
        
        for col in df.get_columns() {
            null_count += col.null_count();
        }
        
        let non_null = total_cells.saturating_sub(null_count);
        non_null as f32 / total_cells as f32
    }
    
    /// Count remaining data quality issues in the dataframe.
    fn count_remaining_issues(&self, df: &DataFrame) -> usize {
        let mut issue_count = 0;
        
        for col in df.get_columns() {
            // Count columns with remaining nulls as issues
            if col.null_count() > 0 {
                issue_count += 1;
            }
        }
        
        issue_count
    }
    
    /// Update column summaries with final data types after processing.
    fn update_column_summaries(&self, summary: &mut PreprocessingSummary, df: &DataFrame) {
        let final_columns: HashMap<String, String> = df
            .get_columns()
            .iter()
            .map(|col| (col.name().to_string(), col.dtype().to_string()))
            .collect();
        
        for col_summary in &mut summary.column_summaries {
            if let Some(final_type) = final_columns.get(&col_summary.name) {
                col_summary.final_type = final_type.clone();
            } else {
                // Column was removed
                col_summary.was_removed = true;
                col_summary.removal_reason = Some("Removed during preprocessing".to_string());
            }
        }
    }

    fn make_decisions(
        &self,
        profile: &crate::types::DatasetProfile,
        issues: &[crate::types::DataQualityIssue],
        df: &DataFrame,
    ) -> Result<(HashMap<String, String>, String, String)> {
        // Decide which engine to use
        let use_ai = self.config.use_ai_decisions && self.ai_provider.is_some();

        if use_ai {
            info!("Using AI-powered decision engine...");
            let ai_provider = self.ai_provider.as_ref().unwrap();
            let engine = AiDecisionEngine::new(ai_provider.as_ref(), self.config.clone());
            let choices = engine
                .make_decisions(issues, df)
                .map_err(|e| PreprocessingError::AiClientError(e.to_string()))?;
            let (problem_type, target_column) = engine
                .finalize_problem_setup(profile, &choices, df)
                .map_err(|e| PreprocessingError::AiClientError(e.to_string()))?;
            Ok((choices, problem_type, target_column))
        } else {
            info!("Using rule-based decision engine...");
            let engine = RuleBasedDecisionEngine::new(self.config.clone());
            let choices = engine
                .make_decisions(issues, df)
                .map_err(|e| PreprocessingError::CleaningFailed(e.to_string()))?;
            let (problem_type, target_column) = engine
                .finalize_problem_setup(profile, &choices, df)
                .map_err(|e| PreprocessingError::CleaningFailed(e.to_string()))?;
            Ok((choices, problem_type, target_column))
        }
    }

    fn remove_date_columns_if_needed(
        &self,
        df: &DataFrame,
        problem_type: &str,
        profile: &crate::types::DatasetProfile,
        target_column: &str,
    ) -> Result<DataFrame> {
        if !["classification", "regression"].contains(&problem_type) {
            return Ok(df.clone());
        }

        let date_columns: Vec<String> = profile
            .column_profiles
            .iter()
            .filter(|col| {
                (col.inferred_type == "datetime" || col.inferred_type == "date")
                    && col.name != target_column
            })
            .map(|col| col.name.clone())
            .collect();

        if date_columns.is_empty() {
            debug!("No date columns to remove");
            return Ok(df.clone());
        }

        debug!(
            "Removing {} date columns for {} problem: {:?}",
            date_columns.len(),
            problem_type,
            date_columns
        );

        let columns_to_keep: Vec<PlSmallStr> = df
            .get_column_names()
            .into_iter()
            .filter(|col| !date_columns.contains(&col.to_string()))
            .cloned()
            .collect();

        df.select(columns_to_keep)
            .map_err(PreprocessingError::Polars)
    }
}

/// Builder for creating a [`Pipeline`] instance.
///
/// Use [`Pipeline::builder()`] to get started.
///
/// # Example
///
/// ```rust,ignore
/// use lex_processing::{Pipeline, PipelineConfig, CancellationToken};
/// use std::sync::Arc;
///
/// let token = CancellationToken::new();
///
/// let pipeline = Pipeline::builder()
///     .config(PipelineConfig::default())
///     .cancellation_token(token)
///     .on_progress(|update| {
///         println!("[{:.0}%] {}", update.progress * 100.0, update.message);
///     })
///     .build()?;
/// ```
#[derive(Default)]
pub struct PipelineBuilder {
    config: Option<PipelineConfig>,
    ai_provider: Option<Arc<dyn AIProvider>>,
    progress_reporter: Option<Arc<dyn ProgressReporter>>,
    cancellation_token: Option<CancellationToken>,
}

// Ensure PipelineBuilder is Send (can be moved to another thread during construction)
static_assertions::assert_impl_all!(PipelineBuilder: Send);

impl PipelineBuilder {
    /// Set the pipeline configuration.
    pub fn config(mut self, config: PipelineConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the AI provider for AI-powered decisions.
    ///
    /// The provider must implement the [`AIProvider`] trait.
    /// Use `Arc` to allow the provider to be shared and reused across
    /// multiple pipeline runs.
    ///
    /// If not provided and `use_ai_decisions` is true in config,
    /// the pipeline will fall back to rule-based decisions.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use data_preprocessing_pipeline::ai::OpenRouterProvider;
    /// use std::sync::Arc;
    ///
    /// let provider = Arc::new(OpenRouterProvider::new("api-key")?);
    ///
    /// // Provider can be cloned and reused
    /// let pipeline1 = Pipeline::builder()
    ///     .ai_provider(provider.clone())
    ///     .build()?;
    ///
    /// let pipeline2 = Pipeline::builder()
    ///     .ai_provider(provider)
    ///     .build()?;
    /// ```
    pub fn ai_provider(mut self, provider: Arc<dyn AIProvider>) -> Self {
        self.ai_provider = Some(provider);
        self
    }

    /// Set a progress reporter for receiving updates during processing.
    ///
    /// Use this when you need a custom progress reporter implementation,
    /// such as for Tauri event emission.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use data_preprocessing_pipeline::{ProgressReporter, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// struct MyReporter;
    ///
    /// impl ProgressReporter for MyReporter {
    ///     fn report(&self, update: ProgressUpdate) {
    ///         println!("{}: {}", update.stage.display_name(), update.message);
    ///     }
    /// }
    ///
    /// let pipeline = Pipeline::builder()
    ///     .progress_reporter(Arc::new(MyReporter))
    ///     .build()?;
    /// ```
    pub fn progress_reporter(mut self, reporter: Arc<dyn ProgressReporter>) -> Self {
        self.progress_reporter = Some(reporter);
        self
    }

    /// Set a progress callback closure.
    ///
    /// This is a convenience method for simple progress handling.
    /// For more complex scenarios, use [`progress_reporter`](Self::progress_reporter).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let pipeline = Pipeline::builder()
    ///     .on_progress(|update| {
    ///         println!("[{:.0}%] {:?}: {}",
    ///             update.progress * 100.0,
    ///             update.stage,
    ///             update.message
    ///         );
    ///     })
    ///     .build()?;
    /// ```
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(ProgressUpdate) + Send + Sync + 'static,
    {
        self.progress_reporter = Some(Arc::new(ClosureProgressReporter::new(callback)));
        self
    }

    /// Set a cancellation token for stopping the pipeline.
    ///
    /// Clone the token and call [`CancellationToken::cancel()`] from
    /// any thread to request cancellation. The pipeline will check
    /// this token at various points and return
    /// [`PreprocessingError::Cancelled`] if cancellation is requested.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use data_preprocessing_pipeline::{Pipeline, CancellationToken};
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let token = CancellationToken::new();
    /// let token_for_cancel = token.clone();
    ///
    /// // Cancel after 5 seconds
    /// thread::spawn(move || {
    ///     thread::sleep(Duration::from_secs(5));
    ///     token_for_cancel.cancel();
    /// });
    ///
    /// let result = Pipeline::builder()
    ///     .cancellation_token(token)
    ///     .build()?
    ///     .process(df);
    ///
    /// if let Err(PreprocessingError::Cancelled) = result {
    ///     println!("Pipeline was cancelled");
    /// }
    /// ```
    pub fn cancellation_token(mut self, token: CancellationToken) -> Self {
        self.cancellation_token = Some(token);
        self
    }

    /// Build the pipeline.
    ///
    /// Returns an error if the configuration is invalid.
    pub fn build(self) -> std::result::Result<Pipeline, crate::config::ConfigValidationError> {
        let config = self.config.unwrap_or_default();
        config.validate()?;

        // Create report generator with config's output settings
        let reporter = ReportGenerator::new(
            config.output_dir.clone(),
            config.output_name.clone(),
        );

        Ok(Pipeline {
            config,
            ai_provider: self.ai_provider,
            progress_reporter: self.progress_reporter,
            cancellation_token: self.cancellation_token.unwrap_or_default(),
            cleaner: DataCleaner,
            executor: PreprocessingExecutor,
            reporter,
            type_corrector: TypeCorrector,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_pipeline_builder_default() {
        let pipeline = Pipeline::builder().build().unwrap();
        assert!(pipeline.ai_provider.is_none());
        assert!(pipeline.config.enable_type_correction);
    }

    #[test]
    fn test_pipeline_builder_with_config() {
        let config = PipelineConfig::builder()
            .enable_type_correction(false)
            .use_ai_decisions(false)
            .build()
            .unwrap();

        let pipeline = Pipeline::builder().config(config).build().unwrap();

        assert!(!pipeline.config.enable_type_correction);
        assert!(!pipeline.config.use_ai_decisions);
    }

    #[test]
    fn test_pipeline_builder_with_cancellation_token() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        let pipeline = Pipeline::builder()
            .cancellation_token(token)
            .build()
            .unwrap();

        assert!(!pipeline.cancellation_token.is_cancelled());

        token_clone.cancel();

        assert!(pipeline.cancellation_token.is_cancelled());
    }

    #[test]
    fn test_pipeline_builder_with_progress_callback() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let pipeline = Pipeline::builder()
            .on_progress(move |_update| {
                call_count_clone.fetch_add(1, Ordering::SeqCst);
            })
            .build()
            .unwrap();

        // Manually trigger a progress report
        pipeline.report_progress(ProgressUpdate::new(
            PreprocessingStage::Profiling,
            0.5,
            "Test",
        ));

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_check_cancelled() {
        let token = CancellationToken::new();

        let pipeline = Pipeline::builder()
            .cancellation_token(token.clone())
            .build()
            .unwrap();

        // Should succeed when not cancelled
        assert!(pipeline.check_cancelled().is_ok());

        // Cancel and check again
        token.cancel();
        let result = pipeline.check_cancelled();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PreprocessingError::Cancelled
        ));
    }
}
