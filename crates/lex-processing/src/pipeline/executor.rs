//! Preprocessing executor module.
//!
//! Contains the main execution logic for data preprocessing operations.

use crate::imputers::{KNNImputer, StatisticalImputer};
use crate::pipeline::outliers::OutlierHandler;
use crate::types::{ColumnProfile, DatasetProfile};
use crate::utils::{dtype_category_str, fill_numeric_nulls, fill_string_nulls, is_numeric_dtype};
use anyhow::Result;
use polars::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Executes preprocessing operations on a DataFrame.
pub struct PreprocessingExecutor;

impl PreprocessingExecutor {
    /// Execute comprehensive preprocessing based on AI/rule-based decisions.
    pub fn execute_comprehensive_preprocessing(
        &self,
        mut df: DataFrame,
        profile: &DatasetProfile,
        ai_choices: &HashMap<String, String>,
        _target_column: &str,
    ) -> Result<(DataFrame, DataFrame, Vec<String>)> {
        let mut processing_steps = Vec::new();

        info!("Executing comprehensive preprocessing based on AI decisions...");

        // 1. Handle data type conversions FIRST
        info!("Step 4.1: Handling data type conversions...");
        let col_names: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();
        for col_profile in &profile.column_profiles {
            if col_profile.dtype == "String"
                && col_profile.inferred_type == "numeric"
                && col_names.contains(&col_profile.name)
            {
                match df.column(&col_profile.name)?.cast(&DataType::Float64) {
                    Ok(converted) => {
                        let converted_series = converted.take_materialized_series();
                        if let Err(e) = df.replace(&col_profile.name, converted_series) {
                            processing_steps.push(format!(
                                "Failed to replace {} with converted values: {}",
                                col_profile.name, e
                            ));
                            warn!("Failed to replace {}: {}", col_profile.name, e);
                        } else {
                            processing_steps.push(format!(
                                "Converted {} from String to numeric",
                                col_profile.name
                            ));
                            debug!("Converted {} to numeric", col_profile.name);
                        }
                    }
                    Err(e) => {
                        processing_steps.push(format!(
                            "Failed to convert {} to numeric: {}",
                            col_profile.name, e
                        ));
                        warn!("Failed to convert {} to numeric: {}", col_profile.name, e);
                    }
                }
            }
        }

        // 2. COMPREHENSIVE missing value handling with COLUMN-SPECIFIC strategies
        info!("Step 4.2: Column-specific missing value handling...");
        self.handle_missing_values(&mut df, profile, ai_choices, &mut processing_steps)?;

        // 3. Handle outliers based on AI decision
        info!("Step 4.3: Handling outliers...");
        OutlierHandler::handle_outliers(&mut df, profile, ai_choices, &mut processing_steps)?;

        // 4. Prepare final datasets
        info!("Step 4.4: Preparing final datasets...");
        let (df_for_training, df_with_identifiers) = self.prepare_final_datasets(df, profile, &mut processing_steps)?;

        info!("Comprehensive preprocessing completed successfully");
        info!("Training dataset shape: {:?}", (df_for_training.height(), df_for_training.width()));
        info!("Dataset with IDs shape: {:?}", (df_with_identifiers.height(), df_with_identifiers.width()));
        
        let remaining_nulls: usize = df_for_training
            .get_columns()
            .iter()
            .map(|col| col.null_count())
            .sum();
        debug!("Missing values remaining: {}", remaining_nulls);

        Ok((df_for_training, df_with_identifiers, processing_steps))
    }

    /// Handle missing values for all columns with null values.
    fn handle_missing_values(
        &self,
        df: &mut DataFrame,
        profile: &DatasetProfile,
        ai_choices: &HashMap<String, String>,
        processing_steps: &mut Vec<String>,
    ) -> Result<()> {
        let col_names_for_missing: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();
        let columns_with_missing: Vec<_> = profile
            .column_profiles
            .iter()
            .filter(|col| col.null_count > 0 && col_names_for_missing.contains(&col.name))
            .collect();

        if columns_with_missing.is_empty() {
            return Ok(());
        }

        debug!(
            "Processing {} columns with missing values",
            columns_with_missing.len()
        );

        for col_profile in &columns_with_missing {
            let col_name = &col_profile.name;
            let missing_count = col_profile.null_count;
            let missing_pct = col_profile.null_percentage;
            
            // Get ACTUAL dtype from DataFrame
            let actual_dtype = if let Ok(column) = df.column(col_name) {
                let series = column.as_materialized_series();
                dtype_category_str(series)
            } else {
                "string"
            };
            
            debug!("Processing '{}' ({} - {:.1}% missing)...", 
                    col_name, actual_dtype, missing_pct);

            let strategy = Self::get_column_strategy(col_name, ai_choices);
            debug!("Strategy for '{}': {}", col_name, strategy);

            match actual_dtype {
                "numeric" => {
                    self.handle_numeric_missing(df, col_name, missing_count, &strategy, processing_steps)?;
                }
                "string" => {
                    self.handle_string_missing(df, col_profile, &strategy, processing_steps)?;
                }
                "datetime" => {
                    self.handle_datetime_missing(df, col_name, missing_count, processing_steps)?;
                }
                _ => {
                    debug!("Using automatic fallback strategy for '{}'", col_name);
                    StatisticalImputer::apply_fallback_imputation(df, col_profile, processing_steps)?;
                }
            }
        }

        // Verify no missing values remain
        let remaining_missing: usize = df
            .get_columns()
            .iter()
            .map(|col| col.null_count())
            .sum();
        
        if remaining_missing > 0 {
            warn!(
                "{} missing values still remain, applying final cleanup...",
                remaining_missing
            );
            
            Self::final_missing_value_cleanup(df, processing_steps)?;
        } else {
            info!("All missing values handled successfully");
        }

        Ok(())
    }

    /// Handle missing values in numeric columns.
    fn handle_numeric_missing(
        &self,
        df: &mut DataFrame,
        col_name: &str,
        missing_count: usize,
        strategy: &str,
        processing_steps: &mut Vec<String>,
    ) -> Result<()> {
        match strategy {
            "knn_imputation" => {
                let k_neighbors = (df.height() as f64).sqrt().floor() as usize;
                let k = k_neighbors.clamp(1, 10).min(df.height() / 2);
                
                let imputer = KNNImputer::new(k);
                let col_names = vec![col_name.to_string()];
                
                match imputer.fit_transform(df, &col_names) {
                    Ok(imputed_df) => {
                        if let Ok(imputed_col) = imputed_df.column(col_name) {
                            let imputed_series = imputed_col.as_materialized_series().clone();
                            if let Err(e) = df.replace(col_name, imputed_series) {
                                warn!("Failed to replace: {}", e);
                            } else {
                                processing_steps.push(format!(
                                    "KNN imputed '{}': {} values (K={})",
                                    col_name, missing_count, k
                                ));
                                debug!("KNN imputed '{}': {} values", col_name, missing_count);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("KNN failed: {}, using median fallback", e);
                        StatisticalImputer::apply_numeric_median(df, col_name, processing_steps)?;
                    }
                }
            }
            "mean_imputation" => {
                StatisticalImputer::apply_numeric_mean(df, col_name, processing_steps)?;
                debug!("Mean imputed '{}'", col_name);
            }
            _ => {
                // Default: median for numeric
                StatisticalImputer::apply_numeric_median(df, col_name, processing_steps)?;
                debug!("Median imputed '{}'", col_name);
            }
        }
        Ok(())
    }

    /// Handle missing values in string columns.
    fn handle_string_missing(
        &self,
        df: &mut DataFrame,
        col_profile: &ColumnProfile,
        strategy: &str,
        processing_steps: &mut Vec<String>,
    ) -> Result<()> {
        let col_name = &col_profile.name;
        
        match strategy {
            "category_indicator" => {
                StatisticalImputer::apply_category_indicator(df, col_profile, processing_steps)?;
                debug!("Category indicator added for '{}'", col_name);
            }
            "constant_imputation" => {
                StatisticalImputer::apply_constant_imputation(df, col_profile, processing_steps)?;
                debug!("Constant value imputed for '{}'", col_name);
            }
            _ => {
                // Default: mode for strings
                StatisticalImputer::apply_mode_imputation(df, col_profile, processing_steps)?;
                debug!("Mode imputed for '{}'", col_name);
            }
        }
        Ok(())
    }

    /// Handle missing values in datetime columns.
    fn handle_datetime_missing(
        &self,
        df: &mut DataFrame,
        col_name: &str,
        missing_count: usize,
        processing_steps: &mut Vec<String>,
    ) -> Result<()> {
        if let Ok(column) = df.column(col_name) {
            let series = column.as_materialized_series();
            let filled = series.fill_null(FillNullStrategy::Forward(None))?;
            let filled = filled.fill_null(FillNullStrategy::Backward(None))?;
            
            if let Err(e) = df.replace(col_name, filled) {
                warn!("Failed to fill datetime '{}': {}", col_name, e);
            } else {
                processing_steps.push(format!(
                    "Forward fill '{}': {} values",
                    col_name, missing_count
                ));
                debug!("Forward fill applied for '{}'", col_name);
            }
        }
        Ok(())
    }

    /// Get the AI-selected strategy for a specific column.
    fn get_column_strategy(col_name: &str, ai_choices: &HashMap<String, String>) -> String {
        // Look for column-specific decision
        for (choice_id, choice) in ai_choices {
            if choice_id.contains(&format!("missing_values_{}", col_name)) {
                return choice.clone();
            }
        }
        
        // Default to median for numeric (will be overridden by actual dtype check)
        "median_imputation".to_string()
    }

    /// Final cleanup for any remaining missing values.
    fn final_missing_value_cleanup(
        df: &mut DataFrame,
        processing_steps: &mut Vec<String>,
    ) -> Result<()> {
        let column_names: Vec<String> = df.get_column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        
        for col_name in column_names {
            if let Ok(column) = df.column(&col_name)
                && column.null_count() > 0 {
                    let series = column.as_materialized_series();
                    let filled = if is_numeric_dtype(series.dtype()) {
                        fill_numeric_nulls(series, 0.0)?
                    } else {
                        fill_string_nulls(series, "Unknown")?
                    };
                    
                    df.replace(&col_name, filled)?;
                }
        }
        
        processing_steps.push("Final cleanup: filled all remaining missing values".to_string());
        Ok(())
    }

    /// Prepare final datasets (training and with identifiers).
    fn prepare_final_datasets(
        &self,
        df: DataFrame,
        profile: &DatasetProfile,
        processing_steps: &mut Vec<String>,
    ) -> Result<(DataFrame, DataFrame)> {
        let identifier_cols: Vec<String> = profile
            .column_profiles
            .iter()
            .filter(|col| col.inferred_type == "identifier")
            .map(|col| col.name.clone())
            .collect();

        let df_with_identifiers = df.clone();
        let df_for_training = if !identifier_cols.is_empty() {
            let cols_to_keep: Vec<PlSmallStr> = df
                .get_column_names()
                .into_iter()
                .filter(|col| !identifier_cols.contains(&col.to_string()))
                .cloned()
                .collect();
            df.select(cols_to_keep)?
        } else {
            df
        };

        if !identifier_cols.is_empty() {
            processing_steps.push(format!(
                "Excluded identifier columns from training dataset: {:?}",
                identifier_cols
            ));
            debug!("Excluded identifier columns: {:?}", identifier_cols);
        } else {
            processing_steps.push("No identifier columns found to exclude".to_string());
        }

        Ok((df_for_training, df_with_identifiers))
    }
}
