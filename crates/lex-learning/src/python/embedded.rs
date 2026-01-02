//! Embedded Python source files for the `lex_learning` package.
//!
//! This module contains all Python source files for the `lex_learning` package,
//! embedded at compile time using [`include_str!`]. These files are extracted
//! to the application data directory at runtime by the [`runtime`] module.
//!
//! # Embedding Strategy
//!
//! The Python source files are embedded directly into the Rust binary during
//! compilation. This approach provides several benefits:
//!
//! 1. **Self-contained binary**: No need to distribute Python files separately
//! 2. **Version consistency**: Python code is always in sync with Rust code
//! 3. **Tamper resistance**: Source files cannot be modified after compilation
//! 4. **Simplified deployment**: Single binary contains everything needed
//!
//! # Package Structure
//!
//! The embedded files mirror the Python package structure:
//!
//! ```text
//! lex_learning/
//! ├── __init__.py          # Package root, public API exports
//! ├── config.py            # PipelineConfig, ProblemType
//! ├── errors.py            # Exception hierarchy
//! ├── core/                # Core types and protocols
//! │   ├── __init__.py
//! │   ├── metrics.py       # Metrics dataclasses
//! │   ├── protocols.py     # Type protocols
//! │   └── types.py         # Result types
//! ├── explainability/      # SHAP explanations
//! │   ├── __init__.py
//! │   ├── explainer.py     # Main Explainer class
//! │   ├── plots.py         # Plot generation
//! │   └── shap_strategies.py
//! ├── inference/           # Model loading and prediction
//! │   ├── __init__.py
//! │   ├── artifact.py      # Model artifacts
//! │   └── model.py         # TrainedModel class
//! ├── models/              # ML model definitions
//! │   ├── __init__.py
//! │   ├── boosting.py      # XGBoost, LightGBM
//! │   ├── neural.py        # Keras neural networks
//! │   └── sklearn_models.py
//! ├── pipeline/            # Training orchestration
//! │   ├── __init__.py
//! │   ├── orchestrator.py  # Pipeline class
//! │   └── stages.py        # Training stages
//! ├── preprocessing/       # Data preprocessing
//! │   ├── __init__.py
//! │   └── preprocessor.py
//! ├── progress/            # Progress reporting
//! │   ├── __init__.py
//! │   └── reporter.py
//! └── training/            # Model training
//!     ├── __init__.py
//!     ├── optimizer.py     # Optuna optimization
//!     ├── selector.py      # Algorithm selection
//!     └── trainer.py       # Training loop
//! ```
//!
//! # Runtime Extraction
//!
//! At runtime, these files are extracted to:
//!
//! ```text
//! {app_data_dir}/python/lex_learning/
//! ```
//!
//! Where `{app_data_dir}` is platform-specific:
//! - **Linux**: `~/.local/share/lex-learning/`
//! - **macOS**: `~/Library/Application Support/lex-learning/`
//! - **Windows**: `%APPDATA%\lex-learning\`
//!
//! # Usage
//!
//! This module is primarily used by [`runtime::extract_python_source`]:
//!
//! ```rust,ignore
//! use crate::python::embedded::EMBEDDED_FILES;
//!
//! for file in EMBEDDED_FILES {
//!     let target_path = base_dir.join(file.path);
//!     std::fs::write(&target_path, file.content)?;
//! }
//! ```
//!
//! [`runtime`]: crate::python::runtime
//! [`runtime::extract_python_source`]: crate::python::runtime

/// A single embedded Python source file.
///
/// This struct represents a Python file that has been embedded into the Rust
/// binary at compile time. It contains the relative path (within the package)
/// and the file contents as a static string.
///
/// # Fields
///
/// * `path` - Relative path within the `lex_learning` package (e.g., `"__init__.py"`,
///   `"core/types.py"`). Uses forward slashes as path separators on all platforms.
/// * `content` - The complete file contents as a UTF-8 string.
///
/// # Example
///
/// ```rust
/// use lex_learning::python::embedded::EmbeddedFile;
///
/// let file = EmbeddedFile {
///     path: "example.py",
///     content: "print('Hello, world!')",
/// };
///
/// assert_eq!(file.path, "example.py");
/// assert!(file.content.contains("print"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddedFile {
    /// Relative path within the `lex_learning` package.
    ///
    /// Examples: `"__init__.py"`, `"core/types.py"`, `"training/trainer.py"`
    pub path: &'static str,

    /// The complete file contents as a UTF-8 string.
    pub content: &'static str,
}

/// All embedded Python source files for the `lex_learning` package.
///
/// This constant contains all 29 Python source files that make up the
/// `lex_learning` package. The files are organized by subpackage and
/// embedded at compile time using [`include_str!`].
///
/// # File Organization
///
/// Files are listed in a logical order:
/// 1. Root package files (`__init__.py`, `config.py`, `errors.py`)
/// 2. Subpackages in alphabetical order, each with its own `__init__.py` first
///
/// # Extraction
///
/// These files are extracted to the filesystem at runtime. The extraction
/// preserves the directory structure, creating subdirectories as needed.
///
/// # Updating Files
///
/// When Python source files are modified:
/// 1. The Rust binary must be recompiled to include the changes
/// 2. The extracted files are overwritten on next initialization
///
/// # File Count
///
/// The current count is 29 files across 8 subpackages:
/// - Root: 3 files
/// - core: 4 files
/// - explainability: 4 files
/// - inference: 3 files
/// - models: 4 files
/// - pipeline: 3 files
/// - preprocessing: 2 files
/// - progress: 2 files
/// - training: 4 files
pub const EMBEDDED_FILES: &[EmbeddedFile] = &[
    // Root package files
    EmbeddedFile {
        path: "__init__.py",
        content: include_str!("../../python/lex_learning/src/__init__.py"),
    },
    EmbeddedFile {
        path: "config.py",
        content: include_str!("../../python/lex_learning/src/config.py"),
    },
    EmbeddedFile {
        path: "errors.py",
        content: include_str!("../../python/lex_learning/src/errors.py"),
    },
    // core/ subpackage
    EmbeddedFile {
        path: "core/__init__.py",
        content: include_str!("../../python/lex_learning/src/core/__init__.py"),
    },
    EmbeddedFile {
        path: "core/metrics.py",
        content: include_str!("../../python/lex_learning/src/core/metrics.py"),
    },
    EmbeddedFile {
        path: "core/protocols.py",
        content: include_str!("../../python/lex_learning/src/core/protocols.py"),
    },
    EmbeddedFile {
        path: "core/types.py",
        content: include_str!("../../python/lex_learning/src/core/types.py"),
    },
    // explainability/ subpackage
    EmbeddedFile {
        path: "explainability/__init__.py",
        content: include_str!("../../python/lex_learning/src/explainability/__init__.py"),
    },
    EmbeddedFile {
        path: "explainability/explainer.py",
        content: include_str!("../../python/lex_learning/src/explainability/explainer.py"),
    },
    EmbeddedFile {
        path: "explainability/plots.py",
        content: include_str!("../../python/lex_learning/src/explainability/plots.py"),
    },
    EmbeddedFile {
        path: "explainability/shap_strategies.py",
        content: include_str!("../../python/lex_learning/src/explainability/shap_strategies.py"),
    },
    // inference/ subpackage
    EmbeddedFile {
        path: "inference/__init__.py",
        content: include_str!("../../python/lex_learning/src/inference/__init__.py"),
    },
    EmbeddedFile {
        path: "inference/artifact.py",
        content: include_str!("../../python/lex_learning/src/inference/artifact.py"),
    },
    EmbeddedFile {
        path: "inference/model.py",
        content: include_str!("../../python/lex_learning/src/inference/model.py"),
    },
    // models/ subpackage
    EmbeddedFile {
        path: "models/__init__.py",
        content: include_str!("../../python/lex_learning/src/models/__init__.py"),
    },
    EmbeddedFile {
        path: "models/boosting.py",
        content: include_str!("../../python/lex_learning/src/models/boosting.py"),
    },
    EmbeddedFile {
        path: "models/neural.py",
        content: include_str!("../../python/lex_learning/src/models/neural.py"),
    },
    EmbeddedFile {
        path: "models/sklearn_models.py",
        content: include_str!("../../python/lex_learning/src/models/sklearn_models.py"),
    },
    // pipeline/ subpackage
    EmbeddedFile {
        path: "pipeline/__init__.py",
        content: include_str!("../../python/lex_learning/src/pipeline/__init__.py"),
    },
    EmbeddedFile {
        path: "pipeline/orchestrator.py",
        content: include_str!("../../python/lex_learning/src/pipeline/orchestrator.py"),
    },
    EmbeddedFile {
        path: "pipeline/stages.py",
        content: include_str!("../../python/lex_learning/src/pipeline/stages.py"),
    },
    // preprocessing/ subpackage
    EmbeddedFile {
        path: "preprocessing/__init__.py",
        content: include_str!("../../python/lex_learning/src/preprocessing/__init__.py"),
    },
    EmbeddedFile {
        path: "preprocessing/preprocessor.py",
        content: include_str!("../../python/lex_learning/src/preprocessing/preprocessor.py"),
    },
    // progress/ subpackage
    EmbeddedFile {
        path: "progress/__init__.py",
        content: include_str!("../../python/lex_learning/src/progress/__init__.py"),
    },
    EmbeddedFile {
        path: "progress/reporter.py",
        content: include_str!("../../python/lex_learning/src/progress/reporter.py"),
    },
    // training/ subpackage
    EmbeddedFile {
        path: "training/__init__.py",
        content: include_str!("../../python/lex_learning/src/training/__init__.py"),
    },
    EmbeddedFile {
        path: "training/optimizer.py",
        content: include_str!("../../python/lex_learning/src/training/optimizer.py"),
    },
    EmbeddedFile {
        path: "training/selector.py",
        content: include_str!("../../python/lex_learning/src/training/selector.py"),
    },
    EmbeddedFile {
        path: "training/trainer.py",
        content: include_str!("../../python/lex_learning/src/training/trainer.py"),
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_files_not_empty() {
        assert!(!EMBEDDED_FILES.is_empty());
        // Should have 29 Python files
        assert_eq!(EMBEDDED_FILES.len(), 29);
    }

    #[test]
    fn test_all_files_have_content() {
        for file in EMBEDDED_FILES {
            assert!(!file.path.is_empty(), "File path should not be empty");
            assert!(!file.content.is_empty(), "File {} should have content", file.path);
        }
    }

    #[test]
    fn test_init_file_exists() {
        let init_file = EMBEDDED_FILES.iter().find(|f| f.path == "__init__.py");
        assert!(init_file.is_some(), "__init__.py should be embedded");
        assert!(
            init_file.unwrap().content.contains("lex-learning"),
            "__init__.py should contain package docstring"
        );
    }
}
