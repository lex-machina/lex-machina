//! Imputation module for handling missing values.
//!
//! This module provides various imputation strategies including:
//! - KNN imputation
//! - Statistical imputation (mean, median, mode)

mod knn;
mod statistical;

pub use knn::KNNImputer;
pub use statistical::StatisticalImputer;
