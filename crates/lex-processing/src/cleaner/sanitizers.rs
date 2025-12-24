//! Data sanitization functions for cleaning values.

use anyhow::Result;
use polars::prelude::*;
use std::collections::HashSet;
use tracing::debug;

/// Aggressively clean quotes and whitespace from all string columns.
pub(crate) fn aggressive_clean_all_columns(df: DataFrame) -> Result<DataFrame> {
    let mut df = df;
    let column_names: Vec<String> = df
        .get_column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    debug!("Aggressively cleaning quotes and whitespace from all columns...");

    for col_name in &column_names {
        if let Ok(col) = df.column(col_name) {
            let series = col.as_materialized_series();
            if series.dtype() == &DataType::String {
                let str_series = series.str()?;
                let mut cleaned_values = Vec::with_capacity(str_series.len());

                for opt_val in str_series.into_iter() {
                    match opt_val {
                        Some(val) => {
                            // Apply deep quote cleaning
                            let cleaned = deep_clean_quotes(val);

                            // Only keep non-empty values
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
    }

    debug!("Quote cleaning completed");
    Ok(df)
}

/// Deep cleaning that removes all forms of quotes through multiple passes.
pub(crate) fn deep_clean_quotes(value: &str) -> String {
    let mut cleaned = value.trim().to_string();

    // Keep cleaning until no more quotes are removed
    let mut iterations = 0;
    let max_iterations = 10; // Prevent infinite loops

    loop {
        if iterations >= max_iterations {
            break;
        }
        iterations += 1;

        let before_len = cleaned.len();

        // Remove triple quotes ("""value""" -> value)
        if cleaned.starts_with("\"\"\"") && cleaned.ends_with("\"\"\"") && cleaned.len() > 6 {
            cleaned = cleaned[3..cleaned.len() - 3].to_string();
            cleaned = cleaned.trim().to_string();
            continue;
        }

        // Remove double quotes ("value" -> value)
        if cleaned.starts_with("\"\"") && cleaned.ends_with("\"\"") && cleaned.len() > 4 {
            cleaned = cleaned[2..cleaned.len() - 2].to_string();
            cleaned = cleaned.trim().to_string();
            continue;
        }

        // Remove single quotes on both sides ("value" -> value)
        if cleaned.starts_with('\"') && cleaned.ends_with('\"') && cleaned.len() > 2 {
            cleaned = cleaned[1..cleaned.len() - 1].to_string();
            cleaned = cleaned.trim().to_string();
            continue;
        }

        // Remove single quotes ('value' -> value)
        if cleaned.starts_with('\'') && cleaned.ends_with('\'') && cleaned.len() > 2 {
            cleaned = cleaned[1..cleaned.len() - 1].to_string();
            cleaned = cleaned.trim().to_string();
            continue;
        }

        // If nothing changed, we're done
        if cleaned.len() == before_len {
            break;
        }
    }

    // Final cleanup - remove any remaining internal quote artifacts
    cleaned = cleaned
        .replace("\"\"\"", "") // Remove any triple quotes
        .replace("\"\"", "") // Remove any double quotes
        .trim()
        .to_string();

    cleaned
}

/// Preprocess UNKNOWN/ERROR values in all their forms before type conversion.
pub(crate) fn preprocess_unknown_values(df: DataFrame) -> Result<DataFrame> {
    let mut df = df;
    let column_names: Vec<String> = df
        .get_column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    debug!("Converting UNKNOWN/ERROR values to null...");

    // Define all possible forms of UNKNOWN/ERROR (case-insensitive)
    let unknown_patterns: HashSet<String> = [
        "unknown", "error", "n/a", "na", "null", "none", "missing", "nan", "#n/a", "#error",
        "", " ", "  ", // Empty and whitespace
    ]
    .iter()
    .map(|s| s.to_lowercase())
    .collect();

    let mut total_replacements = 0;

    for col_name in &column_names {
        if let Ok(col) = df.column(col_name) {
            let series = col.as_materialized_series();
            if series.dtype() == &DataType::String {
                let (cleaned_series, count) =
                    replace_unknown_with_null(series, &unknown_patterns)?;
                if count > 0 {
                    total_replacements += count;
                    df.replace(col_name, cleaned_series)?;
                }
            }
        }
    }

    if total_replacements > 0 {
        debug!(
            "Replaced {} UNKNOWN/ERROR values with null",
            total_replacements
        );
    }

    Ok(df)
}

/// Replace all forms of UNKNOWN/ERROR with null values.
pub(crate) fn replace_unknown_with_null(
    series: &Series,
    unknown_patterns: &HashSet<String>,
) -> Result<(Series, usize)> {
    let str_series = series.str()?;
    let mut cleaned_values = Vec::with_capacity(str_series.len());
    let mut replacement_count = 0;

    for opt_val in str_series.into_iter() {
        match opt_val {
            Some(val) => {
                let trimmed_val = val.trim().to_lowercase();

                // Check if this value matches any UNKNOWN pattern
                if unknown_patterns.contains(&trimmed_val) || trimmed_val.is_empty() {
                    cleaned_values.push(None);
                    replacement_count += 1;
                } else {
                    // Keep original value (preserve case)
                    cleaned_values.push(Some(val.trim().to_string()));
                }
            }
            None => cleaned_values.push(None),
        }
    }

    Ok((
        Series::new(series.name().clone(), cleaned_values),
        replacement_count,
    ))
}
