//! Progress callback bridge from Python to Rust.
//!
//! This module provides [`PyProgressCallback`], a Python-callable wrapper that
//! bridges progress updates from Python's training pipeline to Rust callbacks.
//! It enables real-time training progress reporting across the language boundary.
//!
//! # Architecture
//!
//! The callback bridge works as follows:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                         Rust Application                            │
//! │                                                                     │
//! │   Pipeline::builder()                                               │
//! │       .on_progress(|update| println!("{}", update.message))  ──┐    │
//! │       .build()                                                 │    │
//! │                                                                │    │
//! │                                    wraps in PyProgressCallback │    │
//! │                                                                ▼    │
//! └────────────────────────────────────────────────────────────────┬────┘
//!                                                                  │
//! ┌────────────────────────────────────────────────────────────────┼────┐
//! │                      Python Training Pipeline                  │    │
//! │                                                                ▼    │
//! │   pipeline.train(df)                                                │
//! │       │                                                             │
//! │       ▼                                                             │
//! │   progress_callback(ProgressUpdate(...))  ───► PyProgressCallback   │
//! │                                                    .__call__(...)   │
//! │                                                         │           │
//! │                                                         ▼           │
//! │                                            extract_progress_update  │
//! │                                                         │           │
//! └─────────────────────────────────────────────────────────┼───────────┘
//!                                                           │
//! ┌─────────────────────────────────────────────────────────┼───────────┐
//! │                         Rust Callback                   │           │
//! │                                                         ▼           │
//! │   |update: ProgressUpdate| {                                        │
//! │       println!("Stage: {:?}, Progress: {:.0}%",                     │
//! │           update.stage, update.progress * 100.0);                   │
//! │   }                                                                 │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Thread Safety
//!
//! [`PyProgressCallback`] wraps a [`ProgressCallback`], which is defined as
//! `Arc<dyn Fn(ProgressUpdate) + Send + Sync>`. This means:
//!
//! - The callback can be safely shared across threads
//! - Multiple Python threads can invoke the callback concurrently
//! - The Rust callback implementation must be thread-safe
//!
//! # Type Conversion
//!
//! The Python `ProgressUpdate` dataclass is converted to Rust as follows:
//!
//! | Python Field | Python Type | Rust Type |
//! |--------------|-------------|-----------|
//! | `stage` | `TrainingStage` enum | [`TrainingStage`] (via `.value` string) |
//! | `progress` | `float` | `f64` |
//! | `message` | `str` | `String` |
//! | `current_model` | `str \| None` | `Option<String>` |
//! | `models_completed` | `tuple[int, int] \| None` | `Option<(u32, u32)>` |
//!
//! [`ProgressCallback`]: crate::progress::ProgressCallback
//! [`TrainingStage`]: crate::progress::TrainingStage

use pyo3::prelude::*;

use crate::progress::{ProgressCallback, ProgressUpdate, TrainingStage};

/// Type alias for the cancellation check function.
///
/// This is a closure that returns `true` if the operation should be cancelled.
type CancellationCheckFn = Box<dyn Fn() -> bool + Send + Sync>;

/// Python-callable wrapper for Rust progress callback.
///
/// This struct wraps a Rust [`ProgressCallback`] closure and exposes it as a
/// Python callable via the `__call__` method. When Python calls this object
/// with a `ProgressUpdate`, it extracts the fields and invokes the Rust callback.
///
/// # PyO3 Integration
///
/// This type is marked with `#[pyclass]`, making it usable from Python. It is
/// typically created in Rust and passed to Python's `Pipeline.builder().on_progress()`.
///
/// # Thread Safety
///
/// The wrapped [`ProgressCallback`] is `Send + Sync`, allowing safe concurrent
/// invocation from multiple Python threads if needed.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use lex_learning::progress::{ProgressCallback, ProgressUpdate};
///
/// let callback: ProgressCallback = Arc::new(|update: ProgressUpdate| {
///     println!("Progress: {:.0}% - {}", update.progress * 100.0, update.message);
/// });
///
/// Python::attach(|py| {
///     let py_callback = Bound::new(py, PyProgressCallback::new(callback))?;
///     // Pass to Python pipeline builder
///     builder.call_method1("on_progress", (py_callback,))?;
///     Ok(())
/// });
/// ```
///
/// [`ProgressCallback`]: crate::progress::ProgressCallback
#[pyclass]
pub struct PyProgressCallback {
    callback: ProgressCallback,
}

impl PyProgressCallback {
    /// Create a new [`PyProgressCallback`] wrapping the given Rust callback.
    ///
    /// # Arguments
    ///
    /// * `callback` - The Rust progress callback to wrap
    ///
    /// # Returns
    ///
    /// A new [`PyProgressCallback`] instance that can be passed to Python.
    #[must_use]
    pub fn new(callback: ProgressCallback) -> Self {
        Self { callback }
    }
}

#[pymethods]
impl PyProgressCallback {
    /// Called by Python with a `ProgressUpdate` object.
    ///
    /// This method is invoked automatically when Python calls the callback
    /// object. It extracts the progress update fields from the Python object
    /// and invokes the wrapped Rust callback.
    ///
    /// # Arguments
    ///
    /// * `py_update` - The Python `ProgressUpdate` dataclass instance
    ///
    /// # Errors
    ///
    /// Returns a [`PyErr`] if:
    /// - The `stage` attribute is missing or not a valid enum
    /// - The `progress` attribute is missing or not a float
    /// - The `message` attribute is missing or not a string
    ///
    /// Note: Optional fields (`current_model`, `models_completed`) do not
    /// cause errors if missing or invalid; they default to `None`.
    ///
    /// # Panics
    ///
    /// This method does not panic. If the Rust callback panics, the panic
    /// will propagate but this is considered a bug in the callback implementation.
    ///
    /// [`PyErr`]: pyo3::PyErr
    fn __call__(&self, py_update: &Bound<'_, PyAny>) -> PyResult<()> {
        let update = extract_progress_update(py_update)?;
        (self.callback)(update);
        Ok(())
    }
}

/// Extract Python `ProgressUpdate` to Rust [`ProgressUpdate`].
///
/// Converts a Python `ProgressUpdate` dataclass instance to a Rust struct
/// by extracting each field individually.
///
/// # Field Conversion
///
/// | Python Field | Conversion Method |
/// |--------------|-------------------|
/// | `stage` | `.value` string → [`TrainingStage::from_str`] |
/// | `progress` | Direct `f64` extraction |
/// | `message` | Direct `String` extraction |
/// | `current_model` | [`extract_optional_string`] |
/// | `models_completed` | [`extract_optional_tuple`] |
///
/// # Arguments
///
/// * `py_update` - The Python `ProgressUpdate` object
///
/// # Returns
///
/// A Rust [`ProgressUpdate`] struct with all fields populated.
///
/// # Errors
///
/// Returns a [`PyErr`] if:
/// - The `stage` attribute is missing or `.value` extraction fails
/// - The `progress` attribute is missing or is not a float
/// - The `message` attribute is missing or is not a string
///
/// # Fallback Behavior
///
/// - If `stage` string parsing fails, defaults to [`TrainingStage::Initializing`]
/// - Optional fields (`current_model`, `models_completed`) default to `None` on failure
///
/// [`ProgressUpdate`]: crate::progress::ProgressUpdate
/// [`TrainingStage::from_str`]: crate::progress::TrainingStage
/// [`TrainingStage::Initializing`]: crate::progress::TrainingStage::Initializing
/// [`PyErr`]: pyo3::PyErr
fn extract_progress_update(py_update: &Bound<'_, PyAny>) -> PyResult<ProgressUpdate> {
    // Extract stage: Python enum -> .value string -> Rust enum
    let stage_obj = py_update.getattr("stage")?;
    let stage_str: String = stage_obj.getattr("value")?.extract()?;
    let stage = stage_str.parse().unwrap_or(TrainingStage::Initializing);

    // Extract required fields
    let progress: f64 = py_update.getattr("progress")?.extract()?;
    let message: String = py_update.getattr("message")?.extract()?;

    // Extract optional fields (None in Python → None in Rust)
    let current_model = extract_optional_string(py_update, "current_model");
    let models_completed = extract_optional_tuple(py_update, "models_completed");

    Ok(ProgressUpdate {
        stage,
        progress,
        message,
        current_model,
        models_completed,
    })
}

/// Extract an optional string attribute from a Python object.
///
/// Safely extracts a string attribute that may be `None` in Python.
/// This is used for optional fields in `ProgressUpdate` like `current_model`.
///
/// # Arguments
///
/// * `obj` - The Python object containing the attribute
/// * `attr` - The attribute name to extract
///
/// # Returns
///
/// - `Some(String)` if the attribute exists, is not `None`, and is a valid string
/// - `None` if the attribute is missing, is Python `None`, or extraction fails
///
/// # Note
///
/// This function never returns an error; failures are silently converted to `None`.
/// This is intentional for optional fields where missing/invalid values should
/// not abort the entire extraction.
fn extract_optional_string(obj: &Bound<'_, PyAny>, attr: &str) -> Option<String> {
    obj.getattr(attr)
        .ok()
        .and_then(|v| if v.is_none() { None } else { v.extract().ok() })
}

/// Extract an optional `(u32, u32)` tuple attribute from a Python object.
///
/// Safely extracts a 2-tuple of unsigned integers that may be `None` in Python.
/// This is used for the `models_completed` field in `ProgressUpdate`, which
/// represents `(completed_count, total_count)`.
///
/// # Arguments
///
/// * `obj` - The Python object containing the attribute
/// * `attr` - The attribute name to extract
///
/// # Returns
///
/// - `Some((u32, u32))` if the attribute exists, is not `None`, and is a valid tuple
/// - `None` if the attribute is missing, is Python `None`, or extraction fails
///
/// # Note
///
/// This function never returns an error; failures are silently converted to `None`.
/// This is intentional for optional fields where missing/invalid values should
/// not abort the entire extraction.
fn extract_optional_tuple(obj: &Bound<'_, PyAny>, attr: &str) -> Option<(u32, u32)> {
    obj.getattr(attr)
        .ok()
        .and_then(|v| if v.is_none() { None } else { v.extract().ok() })
}

/// Python-callable wrapper for Rust cancellation check function.
///
/// This struct wraps a closure that checks if cancellation has been requested
/// and exposes it as a Python callable. When Python calls the object
/// (via `__call__`), it invokes the wrapped closure.
///
/// # PyO3 Integration
///
/// This type is marked with `#[pyclass]`, making it usable from Python.
/// It is typically created in Rust and passed to Python's `CallbackProgressReporter`
/// via the `cancellation_check` parameter.
///
/// # Thread Safety
///
/// The wrapped check function is `Send + Sync`, allowing safe concurrent
/// invocation from multiple Python threads.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use lex_learning::CancellationToken;
///
/// let token = CancellationToken::new();
/// let check_fn = token.as_check_fn();
///
/// Python::attach(|py| {
///     let py_check = Bound::new(py, PyCancellationChecker::new(check_fn))?;
///     // Pass to Python pipeline builder
///     builder.call_method1("on_progress", (py_callback, py_check))?;
///     Ok(())
/// });
/// ```
#[pyclass]
pub struct PyCancellationChecker {
    check_fn: CancellationCheckFn,
}

impl PyCancellationChecker {
    /// Create a new [`PyCancellationChecker`] wrapping the given check function.
    ///
    /// # Arguments
    ///
    /// * `check_fn` - The closure that returns `true` if cancelled
    ///
    /// # Returns
    ///
    /// A new [`PyCancellationChecker`] instance that can be passed to Python.
    #[must_use]
    pub fn new(check_fn: impl Fn() -> bool + Send + Sync + 'static) -> Self {
        Self {
            check_fn: Box::new(check_fn),
        }
    }
}

#[pymethods]
impl PyCancellationChecker {
    /// Check if cancellation has been requested.
    ///
    /// This method is invoked by Python's `CallbackProgressReporter` to check
    /// if the operation should be cancelled.
    ///
    /// # Returns
    ///
    /// `true` if cancellation has been requested, `false` otherwise.
    fn __call__(&self) -> bool {
        (self.check_fn)()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_py_progress_callback_creation() {
        let count = Arc::new(AtomicU32::new(0));
        let count_clone = count.clone();

        let callback: ProgressCallback = Arc::new(move |_update| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let py_callback = PyProgressCallback::new(callback);

        // Verify the callback is stored (we can't easily test __call__ without Python)
        assert_eq!(count.load(Ordering::SeqCst), 0);

        // The actual callback invocation is tested in integration tests
        drop(py_callback);
    }
}
