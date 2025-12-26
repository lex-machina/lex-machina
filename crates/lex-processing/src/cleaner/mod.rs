//! Data cleaning module for preprocessing datasets.
//!
//! This module provides functionality for:
//! - Removing duplicate rows
//! - Dropping columns with high missing rates
//! - Removing rows with excessive missing values
//! - Type correction and conversion
//! - Data sanitization

mod converters;
mod sanitizers;
mod type_corrector;

pub use type_corrector::TypeCorrector;

use crate::types::DatasetProfile;
use anyhow::Result;
use polars::prelude::*;
use tracing::{debug, info};

/// Data cleaner for automatic dataset cleaning operations.
pub struct DataCleaner;

impl DataCleaner {
    /// Perform automatic cleaning operations on a dataset.
    ///
    /// This includes:
    /// 1. Removing duplicate rows
    /// 2. Removing columns with >70% missing values
    /// 3. Removing rows with >80% missing data
    pub fn perform_automatic_cleaning(
        &self,
        df: DataFrame,
        profile: &DatasetProfile,
    ) -> Result<(DataFrame, Vec<String>)> {
        let mut cleaning_actions = Vec::new();
        let mut df = df;

        info!("Performing automatic data cleaning...");

        // 1. Remove duplicate rows
        let before_duplicates = df.height();
        df = df.unique::<&str, &str>(None, UniqueKeepStrategy::First, None)?;
        let after_duplicates = df.height();
        let duplicates_removed = before_duplicates - after_duplicates;

        if duplicates_removed > 0 {
            let pct = (duplicates_removed as f64 / before_duplicates as f64) * 100.0;
            cleaning_actions.push(format!(
                "Removed {} duplicate rows ({:.1}%)",
                duplicates_removed, pct
            ));
            debug!("Removed {} duplicate rows", duplicates_removed);
        } else {
            cleaning_actions.push("No duplicate rows found".to_string());
            debug!("No duplicate rows found");
        }

        // 2. Remove columns with >70% missing values
        let high_missing_cols: Vec<String> = profile
            .column_profiles
            .iter()
            .filter(|col| col.null_percentage > 70.0 && col.inferred_role != "identifier")
            .map(|col| col.name.clone())
            .collect();

        if !high_missing_cols.is_empty() {
            // Convert Vec<String> to PlSmallStr for drop_many
            let cols_ref: Vec<PlSmallStr> = high_missing_cols
                .iter()
                .map(|s| s.as_str().into())
                .collect();
            df = df.drop_many(cols_ref);
            cleaning_actions.push(format!(
                "Automatically removed {} columns with >70% missing values: {:?}",
                high_missing_cols.len(),
                high_missing_cols
            ));
            debug!(
                "Automatically removed {} columns with >70% missing values",
                high_missing_cols.len()
            );
        } else {
            cleaning_actions.push("No columns with >70% missing values found".to_string());
        }

        // 3. Remove rows with >80% missing data
        let missing_threshold = 0.8;
        let before_rows = df.height();

        if df.width() > 0 {
            // Calculate null counts per row - iterate over columns and accumulate
            let mut null_counts = Series::new("nulls".into(), vec![0u32; df.height()]);
            for col in df.get_columns() {
                let series = col.as_materialized_series();
                let null_mask = series.is_null();
                if let Ok(null_int) = null_mask.cast(&DataType::UInt32)
                    && let Ok(sum) = &null_counts + &null_int
                {
                    null_counts = sum;
                }
            }

            // Calculate percentage - cast to Float64 first
            let null_counts_f64 = null_counts.cast(&DataType::Float64)?;
            let total_cols = df.width() as f64;

            // Division: Series / f64 returns Series
            let null_pct = &null_counts_f64 / total_cols;

            // Create mask: Series.lt_eq(f64) returns Result<BooleanChunked>
            let mask = null_pct.lt_eq(missing_threshold)?;

            // Filter dataframe
            df = df.filter(&mask)?;
        }

        let after_rows = df.height();
        let rows_removed = before_rows - after_rows;

        if rows_removed > 0 {
            let pct = (rows_removed as f64 / before_rows as f64) * 100.0;
            cleaning_actions.push(format!(
                "Removed {} rows with >80% missing data ({:.1}%)",
                rows_removed, pct
            ));
            debug!("Removed {} rows with >80% missing data", rows_removed);
        } else {
            cleaning_actions.push("No rows with >80% missing data found".to_string());
        }

        Ok((df, cleaning_actions))
    }
}
