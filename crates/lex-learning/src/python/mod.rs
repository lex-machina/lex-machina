//! Python interop module for lex-learning.
//!
//! This module provides the bridge between Rust and Python, enabling the
//! lex-learning crate to leverage Python's ML ecosystem (scikit-learn, XGBoost,
//! TensorFlow/Keras, SHAP) while maintaining a pure Rust API.
//!
//! # Architecture
//!
//! The Python interop layer consists of four submodules:
//!
//! - [`callback`]: Progress callback mechanism for training pipelines
//! - [`conversion`]: Type conversions between Rust types and Python objects
//! - [`embedded`]: Embedded Python source code for the `lex_learning` package
//! - [`runtime`]: Python runtime initialization and management
//!
//! # Usage
//!
//! The runtime must be initialized before any Python operations:
//!
//! ```rust,ignore
//! use lex_learning::python::runtime::PythonRuntime;
//!
//! let runtime = PythonRuntime::initialize()?;
//! // Python operations can now be performed
//! ```
//!
//! # Bundled Python Runtime
//!
//! This crate embeds a standalone Python 3.12 runtime (from python-build-standalone)
//! with all required ML dependencies pre-installed. The runtime is extracted to
//! a platform-specific directory on first initialization.

pub mod callback;
pub mod conversion;
pub mod embedded;
pub mod runtime;
