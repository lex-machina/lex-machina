use anyhow::Result;
use polars::prelude::*;
use tracing::debug;

pub struct KNNImputer {
    n_neighbors: usize,
}

impl KNNImputer {
    /// Create a new KNN imputer with specified number of neighbors
    pub fn new(n_neighbors: usize) -> Self {
        Self {
            n_neighbors: n_neighbors.max(1), // Ensure at least 1 neighbor
        }
    }

    /// Fit and transform the dataframe, imputing missing values
    pub fn fit_transform(&self, df: &DataFrame, columns: &[String]) -> Result<DataFrame> {
        let mut result_df = df.clone();

        // Only process numeric columns that have missing values
        let numeric_cols_to_impute: Vec<String> = columns
            .iter()
            .filter(|col| {
                if let Ok(series) = df.column(col) {
                    series.null_count() > 0
                        && matches!(
                            series.dtype(),
                            DataType::Int8
                                | DataType::Int16
                                | DataType::Int32
                                | DataType::Int64
                                | DataType::UInt8
                                | DataType::UInt16
                                | DataType::UInt32
                                | DataType::UInt64
                                | DataType::Float32
                                | DataType::Float64
                        )
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        if numeric_cols_to_impute.is_empty() {
            return Ok(result_df);
        }

        debug!("KNN imputing {} columns", numeric_cols_to_impute.len());

        // Get all numeric columns for distance calculation (context)
        let all_numeric_cols: Vec<String> = df
            .get_columns()
            .iter()
            .filter(|col| {
                matches!(
                    col.dtype(),
                    DataType::Int8
                        | DataType::Int16
                        | DataType::Int32
                        | DataType::Int64
                        | DataType::UInt8
                        | DataType::UInt16
                        | DataType::UInt32
                        | DataType::UInt64
                        | DataType::Float32
                        | DataType::Float64
                )
            })
            .map(|col| col.name().to_string())
            .collect();

        // Convert numeric columns to f64 matrix for computation
        let data_matrix = self.create_data_matrix(df, &all_numeric_cols)?;
        let n_rows = df.height();
        let n_cols = all_numeric_cols.len();

        // For each column that needs imputation
        for col_name in &numeric_cols_to_impute {
            let col_idx = all_numeric_cols
                .iter()
                .position(|c| c == col_name)
                .ok_or_else(|| anyhow::anyhow!("Column not found"))?;

            let series = df.column(col_name)?;
            let mut imputed_values = Vec::with_capacity(n_rows);

            // Get null mask as a boolean chunked array
            let null_mask = series.is_null();

            // For each row
            for row_idx in 0..n_rows {
                // Check if this row has a null value using the mask
                let is_null = null_mask.get(row_idx).unwrap_or(false);

                if is_null {
                    // Find K nearest neighbors with non-null values in this column
                    let imputed_value = self.impute_value(
                        &data_matrix,
                        row_idx,
                        col_idx,
                        n_rows,
                        n_cols,
                        &null_mask,
                    )?;
                    imputed_values.push(Some(imputed_value));
                } else {
                    // Keep original value
                    let val = series.get(row_idx)?;
                    imputed_values.push(Some(val.try_extract::<f64>()?));
                }
            }

            // Create new series with imputed values
            let imputed_series = Series::new(col_name.as_str().into(), imputed_values);
            result_df.replace(col_name, imputed_series)?;
        }

        Ok(result_df)
    }

    /// Create a data matrix from the dataframe for distance calculations
    fn create_data_matrix(
        &self,
        df: &DataFrame,
        columns: &[String],
    ) -> Result<Vec<Vec<Option<f64>>>> {
        let n_rows = df.height();
        let n_cols = columns.len();
        let mut matrix = vec![vec![None; n_cols]; n_rows];

        for (col_idx, col_name) in columns.iter().enumerate() {
            let series = df.column(col_name)?;
            let float_series = series.cast(&DataType::Float64)?;
            let f64_series = float_series.f64()?;

            for (row_idx, row) in matrix.iter_mut().enumerate().take(n_rows) {
                row[col_idx] = f64_series.get(row_idx);
            }
        }

        Ok(matrix)
    }

    /// Impute a single missing value using KNN
    fn impute_value(
        &self,
        data_matrix: &[Vec<Option<f64>>],
        target_row: usize,
        target_col: usize,
        n_rows: usize,
        n_cols: usize,
        null_mask: &BooleanChunked,
    ) -> Result<f64> {
        // Find all rows that have a non-null value in the target column
        let candidate_rows: Vec<usize> = (0..n_rows)
            .filter(|&row| row != target_row && !null_mask.get(row).unwrap_or(true))
            .collect();

        if candidate_rows.is_empty() {
            // Fallback to average of non-null values
            let sum: f64 = data_matrix.iter().filter_map(|row| row[target_col]).sum();
            let count = data_matrix
                .iter()
                .filter(|row| row[target_col].is_some())
                .count();
            return Ok(if count > 0 { sum / count as f64 } else { 0.0 });
        }

        // Calculate distances to all candidate rows
        let mut distances: Vec<(usize, f64)> = candidate_rows
            .iter()
            .map(|&candidate_row| {
                let distance = self.calculate_distance(
                    &data_matrix[target_row],
                    &data_matrix[candidate_row],
                    target_col,
                    n_cols,
                );
                (candidate_row, distance)
            })
            .collect();

        // Sort by distance (ascending)
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take K nearest neighbors
        let k = self.n_neighbors.min(distances.len());
        let nearest_neighbors: Vec<usize> = distances.iter().take(k).map(|(idx, _)| *idx).collect();

        // Calculate weighted average based on distances
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;

        for &neighbor_row in &nearest_neighbors {
            if let Some(value) = data_matrix[neighbor_row][target_col] {
                let distance = distances
                    .iter()
                    .find(|(idx, _)| *idx == neighbor_row)
                    .map(|(_, d)| *d)
                    .unwrap_or(1.0);

                // Use inverse distance as weight (avoiding division by zero)
                let weight = if distance < 1e-10 {
                    1e10 // Very close neighbor gets very high weight
                } else {
                    1.0 / distance
                };

                weighted_sum += value * weight;
                weight_sum += weight;
            }
        }

        if weight_sum > 0.0 {
            Ok(weighted_sum / weight_sum)
        } else {
            // Fallback to simple average
            let sum: f64 = data_matrix.iter().filter_map(|row| row[target_col]).sum();
            let count = data_matrix
                .iter()
                .filter(|row| row[target_col].is_some())
                .count();
            Ok(if count > 0 { sum / count as f64 } else { 0.0 })
        }
    }

    /// Calculate Euclidean distance between two rows, ignoring the target column and null values
    fn calculate_distance(
        &self,
        row1: &[Option<f64>],
        row2: &[Option<f64>],
        skip_col: usize,
        n_cols: usize,
    ) -> f64 {
        let mut sum_squared_diff = 0.0;
        let mut count = 0;

        for col_idx in 0..n_cols {
            if col_idx == skip_col {
                continue; // Skip the column we're imputing
            }

            if let (Some(val1), Some(val2)) = (row1[col_idx], row2[col_idx]) {
                let diff = val1 - val2;
                sum_squared_diff += diff * diff;
                count += 1;
            }
        }

        if count > 0 {
            (sum_squared_diff / count as f64).sqrt() // Normalized Euclidean distance
        } else {
            f64::INFINITY // No common non-null features
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // KNNImputer::new() tests
    // ========================================================================

    #[test]
    fn test_knn_imputer_new_with_valid_neighbors() {
        let imputer = KNNImputer::new(5);
        assert_eq!(imputer.n_neighbors, 5);
    }

    #[test]
    fn test_knn_imputer_new_with_zero_neighbors_defaults_to_one() {
        let imputer = KNNImputer::new(0);
        assert_eq!(imputer.n_neighbors, 1);
    }

    #[test]
    fn test_knn_imputer_new_with_one_neighbor() {
        let imputer = KNNImputer::new(1);
        assert_eq!(imputer.n_neighbors, 1);
    }

    // ========================================================================
    // fit_transform() tests - basic functionality
    // ========================================================================

    #[test]
    fn test_fit_transform_basic_imputation() {
        let imputer = KNNImputer::new(2);

        // Create a DataFrame with some missing values
        let df = df![
            "feature1" => [1.0, 2.0, 3.0, 4.0, 5.0],
            "feature2" => [Some(10.0), Some(20.0), None, Some(40.0), Some(50.0)],
        ]
        .unwrap();

        let columns = vec!["feature2".to_string()];
        let result = imputer.fit_transform(&df, &columns).unwrap();

        // Check that null was imputed
        let feature2 = result.column("feature2").unwrap();
        assert_eq!(feature2.null_count(), 0);

        // The imputed value should be reasonable (between neighbors)
        let imputed_value = feature2.get(2).unwrap().try_extract::<f64>().unwrap();
        assert!(imputed_value > 15.0 && imputed_value < 45.0);
    }

    #[test]
    fn test_fit_transform_empty_dataframe() {
        let imputer = KNNImputer::new(3);

        let df = DataFrame::empty();
        let columns: Vec<String> = vec![];

        let result = imputer.fit_transform(&df, &columns).unwrap();
        assert_eq!(result.height(), 0);
    }

    #[test]
    fn test_fit_transform_no_missing_values() {
        let imputer = KNNImputer::new(3);

        let df = df![
            "feature1" => [1.0, 2.0, 3.0],
            "feature2" => [10.0, 20.0, 30.0],
        ]
        .unwrap();

        let columns = vec!["feature2".to_string()];
        let result = imputer.fit_transform(&df, &columns).unwrap();

        // Should return unchanged
        assert_eq!(
            result
                .column("feature2")
                .unwrap()
                .get(0)
                .unwrap()
                .try_extract::<f64>()
                .unwrap(),
            10.0
        );
        assert_eq!(
            result
                .column("feature2")
                .unwrap()
                .get(1)
                .unwrap()
                .try_extract::<f64>()
                .unwrap(),
            20.0
        );
        assert_eq!(
            result
                .column("feature2")
                .unwrap()
                .get(2)
                .unwrap()
                .try_extract::<f64>()
                .unwrap(),
            30.0
        );
    }

    #[test]
    fn test_fit_transform_single_row() {
        let imputer = KNNImputer::new(3);

        // Single row with a missing value - no neighbors available
        let df = df![
            "feature1" => [1.0],
            "feature2" => [Option::<f64>::None],
        ]
        .unwrap();

        let columns = vec!["feature2".to_string()];
        let result = imputer.fit_transform(&df, &columns).unwrap();

        // Should fallback to 0.0 since no valid values exist
        let feature2 = result.column("feature2").unwrap();
        let imputed = feature2.get(0).unwrap().try_extract::<f64>().unwrap();
        assert_eq!(imputed, 0.0);
    }

    #[test]
    fn test_fit_transform_all_nulls_in_column() {
        let imputer = KNNImputer::new(3);

        let df = df![
            "feature1" => [1.0, 2.0, 3.0],
            "feature2" => [Option::<f64>::None, None, None],
        ]
        .unwrap();

        let columns = vec!["feature2".to_string()];
        let result = imputer.fit_transform(&df, &columns).unwrap();

        // All values should be 0.0 (fallback when no valid values)
        let feature2 = result.column("feature2").unwrap();
        for i in 0..3 {
            let val = feature2.get(i).unwrap().try_extract::<f64>().unwrap();
            assert_eq!(val, 0.0);
        }
    }

    #[test]
    fn test_fit_transform_n_neighbors_greater_than_rows() {
        let imputer = KNNImputer::new(10); // More than 3 rows

        let df = df![
            "feature1" => [1.0, 2.0, 3.0],
            "feature2" => [Some(10.0), None, Some(30.0)],
        ]
        .unwrap();

        let columns = vec!["feature2".to_string()];
        let result = imputer.fit_transform(&df, &columns).unwrap();

        // Should still work, using all available neighbors
        let feature2 = result.column("feature2").unwrap();
        assert_eq!(feature2.null_count(), 0);
    }

    #[test]
    fn test_fit_transform_skips_non_numeric_columns() {
        let imputer = KNNImputer::new(3);

        let df = df![
            "name" => ["Alice", "Bob", "Charlie"],
            "age" => [Some(25.0), None, Some(35.0)],
        ]
        .unwrap();

        // Try to impute both - only "age" should be processed
        let columns = vec!["name".to_string(), "age".to_string()];
        let result = imputer.fit_transform(&df, &columns).unwrap();

        // Age should be imputed
        let age = result.column("age").unwrap();
        assert_eq!(age.null_count(), 0);

        // Name should be unchanged (still strings)
        let name = result.column("name").unwrap();
        assert_eq!(name.get(0).unwrap().to_string(), "\"Alice\"");
    }

    #[test]
    fn test_fit_transform_multiple_columns() {
        let imputer = KNNImputer::new(2);

        let df = df![
            "a" => [1.0, 2.0, 3.0, 4.0],
            "b" => [Some(10.0), None, Some(30.0), Some(40.0)],
            "c" => [Some(100.0), Some(200.0), None, Some(400.0)],
        ]
        .unwrap();

        let columns = vec!["b".to_string(), "c".to_string()];
        let result = imputer.fit_transform(&df, &columns).unwrap();

        assert_eq!(result.column("b").unwrap().null_count(), 0);
        assert_eq!(result.column("c").unwrap().null_count(), 0);
    }

    #[test]
    fn test_fit_transform_mixed_null_patterns() {
        let imputer = KNNImputer::new(2);

        // Complex pattern where rows have nulls in different columns
        let df = df![
            "a" => [Some(1.0), None, Some(3.0), Some(4.0)],
            "b" => [Some(10.0), Some(20.0), None, Some(40.0)],
        ]
        .unwrap();

        let columns = vec!["a".to_string(), "b".to_string()];
        let result = imputer.fit_transform(&df, &columns).unwrap();

        assert_eq!(result.column("a").unwrap().null_count(), 0);
        assert_eq!(result.column("b").unwrap().null_count(), 0);
    }

    #[test]
    fn test_fit_transform_integer_columns() {
        let imputer = KNNImputer::new(2);

        let df = df![
            "feature" => [1i64, 2i64, 3i64],
            "target" => [Some(10i64), None, Some(30i64)],
        ]
        .unwrap();

        let columns = vec!["target".to_string()];
        let result = imputer.fit_transform(&df, &columns).unwrap();

        // Integer column should be imputed (converted to float internally)
        let target = result.column("target").unwrap();
        assert_eq!(target.null_count(), 0);
    }

    // ========================================================================
    // create_data_matrix() tests
    // ========================================================================

    #[test]
    fn test_create_data_matrix_basic() {
        let imputer = KNNImputer::new(3);

        let df = df![
            "a" => [1.0, 2.0, 3.0],
            "b" => [10.0, 20.0, 30.0],
        ]
        .unwrap();

        let columns = vec!["a".to_string(), "b".to_string()];
        let matrix = imputer.create_data_matrix(&df, &columns).unwrap();

        assert_eq!(matrix.len(), 3); // 3 rows
        assert_eq!(matrix[0].len(), 2); // 2 columns
        assert_eq!(matrix[0][0], Some(1.0));
        assert_eq!(matrix[0][1], Some(10.0));
        assert_eq!(matrix[2][0], Some(3.0));
        assert_eq!(matrix[2][1], Some(30.0));
    }

    #[test]
    fn test_create_data_matrix_with_nulls() {
        let imputer = KNNImputer::new(3);

        let df = df![
            "a" => [Some(1.0), None, Some(3.0)],
            "b" => [Some(10.0), Some(20.0), None],
        ]
        .unwrap();

        let columns = vec!["a".to_string(), "b".to_string()];
        let matrix = imputer.create_data_matrix(&df, &columns).unwrap();

        assert_eq!(matrix[0][0], Some(1.0));
        assert_eq!(matrix[1][0], None); // Null preserved
        assert_eq!(matrix[2][1], None); // Null preserved
    }

    // ========================================================================
    // calculate_distance() tests
    // ========================================================================

    #[test]
    fn test_calculate_distance_identical_rows() {
        let imputer = KNNImputer::new(3);

        let row1 = vec![Some(1.0), Some(2.0), Some(3.0)];
        let row2 = vec![Some(1.0), Some(2.0), Some(3.0)];

        let distance = imputer.calculate_distance(&row1, &row2, 0, 3);
        assert_eq!(distance, 0.0);
    }

    #[test]
    fn test_calculate_distance_simple() {
        let imputer = KNNImputer::new(3);

        let row1 = vec![Some(0.0), Some(0.0), Some(0.0)];
        let row2 = vec![Some(0.0), Some(3.0), Some(4.0)];

        // Skip column 0, so distance is sqrt(((3-0)^2 + (4-0)^2) / 2) = sqrt(12.5)
        let distance = imputer.calculate_distance(&row1, &row2, 0, 3);
        let expected = (12.5_f64).sqrt();
        assert!((distance - expected).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_distance_with_nulls() {
        let imputer = KNNImputer::new(3);

        let row1 = vec![Some(0.0), None, Some(0.0)];
        let row2 = vec![Some(0.0), Some(3.0), Some(4.0)];

        // Column 1 has null in row1, so only column 2 is used
        // Skip column 0, distance = sqrt(16/1) = 4.0
        let distance = imputer.calculate_distance(&row1, &row2, 0, 3);
        assert_eq!(distance, 4.0);
    }

    #[test]
    fn test_calculate_distance_no_common_features() {
        let imputer = KNNImputer::new(3);

        let row1 = vec![Some(1.0), None, None];
        let row2 = vec![Some(2.0), None, None];

        // Skip column 0, columns 1 and 2 both have nulls
        let distance = imputer.calculate_distance(&row1, &row2, 0, 3);
        assert_eq!(distance, f64::INFINITY);
    }

    #[test]
    fn test_calculate_distance_skips_target_column() {
        let imputer = KNNImputer::new(3);

        let row1 = vec![Some(100.0), Some(0.0), Some(0.0)];
        let row2 = vec![Some(0.0), Some(3.0), Some(4.0)];

        // Skip column 0 (the large difference), use columns 1 and 2
        let distance = imputer.calculate_distance(&row1, &row2, 0, 3);
        let expected = (12.5_f64).sqrt(); // sqrt((9 + 16) / 2)
        assert!((distance - expected).abs() < 1e-10);
    }

    // ========================================================================
    // impute_value() tests - indirectly tested via fit_transform
    // ========================================================================

    #[test]
    fn test_impute_uses_weighted_average() {
        let imputer = KNNImputer::new(2);

        // Row 0 and Row 2 are neighbors of Row 1
        // Row 0: feature1=1, feature2=10
        // Row 1: feature1=2, feature2=null (to impute)
        // Row 2: feature1=3, feature2=30
        //
        // Distance from Row 1 to Row 0: |2-1| = 1
        // Distance from Row 1 to Row 2: |2-3| = 1
        // Equal distances, so weighted average = (10 + 30) / 2 = 20

        let df = df![
            "feature1" => [1.0, 2.0, 3.0],
            "feature2" => [Some(10.0), None, Some(30.0)],
        ]
        .unwrap();

        let columns = vec!["feature2".to_string()];
        let result = imputer.fit_transform(&df, &columns).unwrap();

        let imputed = result
            .column("feature2")
            .unwrap()
            .get(1)
            .unwrap()
            .try_extract::<f64>()
            .unwrap();
        assert!((imputed - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_impute_closer_neighbor_has_more_weight() {
        let imputer = KNNImputer::new(2);

        // Row 0: feature1=1, feature2=10
        // Row 1: feature1=1.1, feature2=null (to impute) - very close to Row 0
        // Row 2: feature1=10, feature2=100
        //
        // Row 1 is much closer to Row 0, so imputed value should be closer to 10

        let df = df![
            "feature1" => [1.0, 1.1, 10.0],
            "feature2" => [Some(10.0), None, Some(100.0)],
        ]
        .unwrap();

        let columns = vec!["feature2".to_string()];
        let result = imputer.fit_transform(&df, &columns).unwrap();

        let imputed = result
            .column("feature2")
            .unwrap()
            .get(1)
            .unwrap()
            .try_extract::<f64>()
            .unwrap();
        // Should be much closer to 10 than to 100
        assert!(imputed < 30.0);
    }

    #[test]
    fn test_impute_zero_distance_gives_high_weight() {
        let imputer = KNNImputer::new(2);

        // Row 0 and Row 1 are identical in feature1 (zero distance)
        let df = df![
            "feature1" => [5.0, 5.0, 100.0],
            "feature2" => [Some(10.0), None, Some(1000.0)],
        ]
        .unwrap();

        let columns = vec!["feature2".to_string()];
        let result = imputer.fit_transform(&df, &columns).unwrap();

        let imputed = result
            .column("feature2")
            .unwrap()
            .get(1)
            .unwrap()
            .try_extract::<f64>()
            .unwrap();
        // Should be very close to 10 due to zero distance = very high weight
        assert!((imputed - 10.0).abs() < 1.0);
    }
}
