//! Pipeline module.
//!
//! This module provides the main preprocessing pipeline and related components.

mod builder;
mod executor;
pub mod outliers;
pub mod progress;

pub use builder::{Pipeline, PipelineBuilder};
pub use executor::PreprocessingExecutor;
pub use outliers::OutlierHandler;
pub use progress::{
    CancellationToken, ClosureProgressReporter, PreprocessingStage, ProgressReporter,
    ProgressUpdate,
};
