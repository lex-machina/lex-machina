//! Python runtime initialization and management.
//!
//! This module handles setting up the bundled Python 3.12 runtime for ML operations.
//! The runtime is extracted from `python-build-standalone` archives and configured
//! to work in an embedded context with PyO3.
//!
//! # Overview
//!
//! The initialization process involves:
//!
//! 1. **Runtime Discovery** - Locating the platform-specific Python runtime
//! 2. **Source Extraction** - Writing embedded Python source to the app data directory
//! 3. **Environment Setup** - Configuring `PYTHONHOME`, `PYTHONPATH`, etc.
//! 4. **Interpreter Init** - Starting the Python interpreter via PyO3
//! 5. **Executable Fix** - Patching `sys.executable` for joblib/multiprocessing
//! 6. **Verification** - Confirming `lex_learning` module imports successfully
//!
//! # Usage
//!
//! Call [`initialize()`] once at application startup before any ML operations:
//!
//! ```rust,ignore
//! use lex_learning::initialize;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize Python runtime (required before any ML operations)
//!     initialize()?;
//!
//!     // Now safe to use Pipeline, TrainedModel, etc.
//!     let config = PipelineConfig::builder()
//!         .problem_type(ProblemType::Classification)
//!         .build()?;
//!     // ...
//!
//!     Ok(())
//! }
//! ```
//!
//! # Thread Safety
//!
//! - [`initialize()`] is safe to call from multiple threads; only the first call
//!   performs initialization, subsequent calls return the cached result.
//! - Environment variables are set during initialization before any threads are
//!   spawned, which is safe per Rust's env var safety guidelines.
//!
//! # Runtime Location
//!
//! The Python runtime is searched for in this order:
//!
//! 1. `LEX_PYTHON_RUNTIME_DIR` environment variable (for testing/override)
//! 2. `{exe_dir}/../runtime/python/{platform}/` (relative to executable)
//! 3. `{exe_dir}/runtime/python/{platform}/` (for installed apps)
//! 4. Compile-time path from `build.rs` (for development)
//!
//! # Platform Support
//!
//! | Platform | Directory Name |
//! |----------|---------------|
//! | Linux x86_64 | `linux-x86_64` |
//! | Linux ARM64 | `linux-aarch64` |
//! | macOS Intel | `darwin-x86_64` |
//! | macOS Apple Silicon | `darwin-aarch64` |
//! | Windows x86_64 | `windows-x86_64` |
//!
//! # The sys.executable Fix
//!
//! When PyO3 embeds Python, `sys.executable` points to the Rust binary rather than
//! the Python interpreter. This breaks libraries like `joblib` and `loky` that spawn
//! worker processes using `sys.executable`. We fix this by patching `sys.executable`
//! to point to the bundled Python interpreter after initialization but before
//! importing any ML libraries.

use crate::error::LexLearningError;
use crate::python::embedded::EMBEDDED_FILES;
use pyo3::types::PyAnyMethods;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Stores the result of initialization to ensure it only happens once.
///
/// Uses [`OnceLock`] for thread-safe, lazy, one-time initialization.
/// The stored `Result<(), String>` captures any initialization error message.
static INIT_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

/// The application name used for the app data directory.
///
/// This is used to create platform-specific directories for storing
/// extracted Python source files:
/// - Linux: `~/.local/share/lex-learning/`
/// - macOS: `~/Library/Application Support/lex-learning/`
/// - Windows: `%APPDATA%/lex-learning/`
const APP_NAME: &str = "lex-learning";

/// The Python version string for path construction.
///
/// Used to locate the correct `lib/python3.12/` directory within the runtime.
/// Must match the version of the bundled `python-build-standalone` runtime.
const PYTHON_VERSION: &str = "python3.12";

/// Initializes the Python runtime for ML operations.
///
/// This function **must** be called once before using any Python-dependent
/// functionality such as [`Pipeline`](crate::Pipeline) or [`TrainedModel`](crate::TrainedModel).
///
/// # What It Does
///
/// 1. Discovers the Python runtime directory (see module docs for search order)
/// 2. Extracts embedded Python source to the app data directory
/// 3. Sets `PYTHONHOME` and `PYTHONPATH` environment variables
/// 4. Initializes the Python interpreter via PyO3
/// 5. Patches `sys.executable` to enable joblib/multiprocessing
/// 6. Verifies the `lex_learning` module can be imported
///
/// # Thread Safety
///
/// This function is safe to call from multiple threads. Only the first call
/// performs initialization; subsequent calls return the cached result immediately.
///
/// # Errors
///
/// Returns [`LexLearningError::RuntimeInit`] if:
/// - The Python runtime directory cannot be found
/// - Python source extraction fails (disk full, permissions)
/// - Environment variable setup fails
/// - Python interpreter initialization fails
/// - The `lex_learning` module cannot be imported
///
/// # Example
///
/// ```rust,ignore
/// use lex_learning::{initialize, Pipeline, PipelineConfig};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Must initialize before any ML operations
///     initialize()?;
///
///     // Now we can use the library
///     let config = PipelineConfig::builder()
///         .problem_type(ProblemType::Classification)
///         .target_column("target")
///         .build()?;
///
///     let pipeline = Pipeline::builder().config(config).build()?;
///     // ...
///
///     Ok(())
/// }
/// ```
///
/// # Panics
///
/// This function does not panic. All errors are returned as `Result`.
#[must_use = "initialization may fail; check the Result"]
pub fn initialize() -> Result<(), LexLearningError> {
    let result = INIT_RESULT.get_or_init(do_initialize);

    match result {
        Ok(()) => Ok(()),
        Err(msg) => Err(LexLearningError::RuntimeInit(msg.clone())),
    }
}

/// Performs the actual initialization sequence.
///
/// This is the internal implementation called by [`initialize()`].
/// It is separated to allow the `OnceLock` pattern to work correctly.
///
/// # Steps
///
/// 1. Find Python runtime directory
/// 2. Extract embedded Python source
/// 3. Set up environment variables (PYTHONHOME, PYTHONPATH)
/// 4. Initialize Python interpreter
/// 5. Fix sys.executable for multiprocessing
/// 6. Verify lex_learning import works
fn do_initialize() -> Result<(), String> {
    // 1. Find the Python runtime directory
    let runtime_dir = find_runtime_dir()?;

    // 2. Extract embedded Python source to app data directory
    let python_src_dir = extract_python_source()?;

    // 3. Set up environment variables
    setup_python_environment(&runtime_dir, &python_src_dir)?;

    // 4. Initialize Python interpreter
    // Note: We call this explicitly because we're using a custom Python runtime,
    // not the system Python. Environment variables must be set before this call.
    pyo3::Python::initialize();

    // 5. Fix sys.executable to point to the bundled Python interpreter
    // This MUST happen before any imports that use joblib/multiprocessing
    fix_sys_executable(&runtime_dir)?;

    // 6. Verify we can import lex_learning
    verify_python_setup()?;

    Ok(())
}

/// Finds the Python runtime directory.
///
/// Searches for the platform-specific runtime directory in multiple locations
/// to support different deployment scenarios.
///
/// # Search Order
///
/// 1. `LEX_PYTHON_RUNTIME_DIR` environment variable (testing/override)
/// 2. `{exe_dir}/../runtime/python/{platform}/` (development layout)
/// 3. `{exe_dir}/runtime/python/{platform}/` (installed app layout)
/// 4. Compile-time path from `build.rs` (development fallback)
///
/// # Returns
///
/// The canonicalized path to the runtime directory.
///
/// # Errors
///
/// Returns an error if no valid runtime directory is found, with a message
/// listing all locations that were searched.
fn find_runtime_dir() -> Result<PathBuf, String> {
    let platform = get_platform_dir();

    // 1. Check environment variable override
    if let Ok(dir) = env::var("LEX_PYTHON_RUNTIME_DIR") {
        let path = PathBuf::from(dir);
        if path.exists() {
            return Ok(path);
        }
    }

    // 2. Check relative to executable
    if let Ok(exe_path) = env::current_exe() {
        // Try {exe_dir}/../runtime/python/{platform}/
        if let Some(exe_dir) = exe_path.parent() {
            let runtime_path = exe_dir
                .join("..")
                .join("runtime")
                .join("python")
                .join(platform);
            if runtime_path.exists() {
                return runtime_path.canonicalize().map_err(|e| e.to_string());
            }

            // Also try {exe_dir}/runtime/python/{platform}/ (for installed apps)
            let runtime_path = exe_dir.join("runtime").join("python").join(platform);
            if runtime_path.exists() {
                return runtime_path.canonicalize().map_err(|e| e.to_string());
            }
        }
    }

    // 3. Fall back to compile-time path (for development)
    let compile_time_dir = PathBuf::from(env!("LEX_PYTHON_RUNTIME_DIR"));
    if compile_time_dir.exists() {
        return Ok(compile_time_dir);
    }

    Err(format!(
        "Python runtime not found. Searched:\n\
         - LEX_PYTHON_RUNTIME_DIR env var\n\
         - Relative to executable\n\
         - Compile-time path: {}",
        compile_time_dir.display()
    ))
}

/// Returns the platform-specific app data directory for storing extracted files.
///
/// This follows platform conventions for user-specific application data:
///
/// | Platform | Directory |
/// |----------|-----------|
/// | Linux | `$XDG_DATA_HOME/lex-learning/` or `~/.local/share/lex-learning/` |
/// | macOS | `~/Library/Application Support/lex-learning/` |
/// | Windows | `%APPDATA%/lex-learning/` |
///
/// # Fallback
///
/// If no platform-specific directory can be determined, falls back to
/// `{temp_dir}/lex-learning/`.
///
/// # Errors
///
/// This function does not fail; it always returns a valid path (possibly the
/// temp directory fallback).
fn get_app_data_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg_data).join(APP_NAME));
        }
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home).join(".local").join("share").join(APP_NAME));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join(APP_NAME));
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            return Ok(PathBuf::from(appdata).join(APP_NAME));
        }
    }

    // Fallback to temp directory
    Ok(env::temp_dir().join(APP_NAME))
}

/// Extracts embedded Python source files to the app data directory.
///
/// The Python source files for `lex_learning` are embedded in the Rust binary
/// at compile time (see [`EMBEDDED_FILES`]). This function writes them to disk
/// so Python can import them.
///
/// # Returns
///
/// The path to the directory containing the `lex_learning` package
/// (the parent of `lex_learning/`).
///
/// # Errors
///
/// Returns an error if:
/// - The app data directory cannot be created
/// - Any embedded file cannot be written (permissions, disk full)
///
/// # Note
///
/// Currently, files are re-extracted on every startup. A future optimization
/// could check file hashes to skip extraction when files haven't changed.
fn extract_python_source() -> Result<PathBuf, String> {
    let app_data_dir = get_app_data_dir()?;
    let python_dir = app_data_dir.join("python");
    let lex_learning_dir = python_dir.join("lex_learning");

    // Check if we need to extract (simple version check using __init__.py hash)
    // For now, we always extract on startup to ensure consistency
    // TODO: Add version checking to avoid re-extraction on every startup

    // Create the lex_learning package directory
    fs::create_dir_all(&lex_learning_dir).map_err(|e| {
        format!(
            "Failed to create directory {}: {}",
            lex_learning_dir.display(),
            e
        )
    })?;

    // Extract all embedded files
    for file in EMBEDDED_FILES {
        let file_path = lex_learning_dir.join(file.path);

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!("Failed to create directory {}: {}", parent.display(), e)
            })?;
        }

        // Write the file
        fs::write(&file_path, file.content).map_err(|e| {
            format!("Failed to write {}: {}", file_path.display(), e)
        })?;
    }

    Ok(python_dir)
}

/// Sets up Python environment variables for the bundled runtime.
///
/// Configures the following environment variables:
///
/// | Variable | Purpose |
/// |----------|---------|
/// | `PYTHONHOME` | Tells Python where to find its standard library |
/// | `PYTHONPATH` | Module search path: our source, stdlib, site-packages |
/// | `PYTHONNOUSERSITE` | Disables user site-packages to avoid conflicts |
/// | `PYTHONDONTWRITEBYTECODE` | Disables `__pycache__` to avoid permission issues |
///
/// # Arguments
///
/// * `runtime_dir` - Path to the Python runtime directory
/// * `python_src_dir` - Path to the extracted `lex_learning` source
///
/// # Safety
///
/// This function modifies environment variables using `env::set_var()`, which is
/// marked as `unsafe` in Rust 2024 edition due to potential data races in
/// multi-threaded programs.
///
/// **This is safe here because:**
/// - It is called during single-threaded initialization
/// - No other threads exist yet
/// - Python has not been initialized yet
///
/// # Errors
///
/// This function currently cannot fail, but returns `Result` for API consistency
/// and future error handling.
fn setup_python_environment(runtime_dir: &Path, python_src_dir: &Path) -> Result<(), String> {
    // Set PYTHONHOME to the runtime directory
    // This tells Python where to find its standard library
    //
    // SAFETY: We're setting environment variables during single-threaded initialization,
    // before spawning any threads or initializing Python. This is the intended use case.
    unsafe {
        env::set_var("PYTHONHOME", runtime_dir);
    }

    // Build PYTHONPATH with:
    // 1. The directory containing our lex_learning package
    // 2. The runtime's lib/python3.12 (standard library)
    // 3. The runtime's site-packages (third-party packages)
    let python_lib = runtime_dir.join("lib").join(PYTHON_VERSION);
    let site_packages = python_lib.join("site-packages");

    // Use platform-specific path separator
    #[cfg(windows)]
    let separator = ";";
    #[cfg(not(windows))]
    let separator = ":";

    let pythonpath = format!(
        "{}{}{}{}{}",
        python_src_dir.display(),
        separator,
        python_lib.display(),
        separator,
        site_packages.display()
    );

    // SAFETY: Same as above - single-threaded initialization.
    unsafe {
        env::set_var("PYTHONPATH", &pythonpath);

        // Disable user site-packages to avoid conflicts with system Python
        env::set_var("PYTHONNOUSERSITE", "1");

        // Disable __pycache__ to avoid permission issues
        env::set_var("PYTHONDONTWRITEBYTECODE", "1");
    }

    Ok(())
}

/// Patches `sys.executable` to point to the bundled Python interpreter.
///
/// # The Problem
///
/// When PyO3 embeds Python, `sys.executable` points to the Rust binary (the host
/// executable), not the Python interpreter. This breaks libraries that spawn
/// worker processes using `sys.executable`:
///
/// - `joblib` (used by scikit-learn for parallel processing)
/// - `loky` (joblib's process executor)
/// - `multiprocessing` (Python's built-in parallel processing)
///
/// # The Solution
///
/// We patch both `sys.executable` and `sys._base_executable` to point to the
/// actual Python interpreter in the bundled runtime. This must happen:
///
/// 1. **After** `Python::initialize()` (so `sys` module exists)
/// 2. **Before** importing any ML libraries (before they cache `sys.executable`)
///
/// # Arguments
///
/// * `runtime_dir` - Path to the Python runtime directory
///
/// # Platform-Specific Paths
///
/// - Unix: `{runtime_dir}/bin/python3`
/// - Windows: `{runtime_dir}/python.exe`
///
/// # Errors
///
/// Returns an error if:
/// - The `sys` module cannot be imported
/// - The executable path is not valid UTF-8
/// - Setting the attribute fails
fn fix_sys_executable(runtime_dir: &Path) -> Result<(), String> {
    #[cfg(windows)]
    let python_exe = runtime_dir.join("python.exe");
    #[cfg(not(windows))]
    let python_exe = runtime_dir.join("bin").join("python3");

    pyo3::Python::attach(|py| {
        let sys = py
            .import("sys")
            .map_err(|e| format!("Failed to import sys: {}", e))?;

        let exe_path = python_exe
            .to_str()
            .ok_or_else(|| "Python executable path is not valid UTF-8".to_string())?;

        sys.setattr("executable", exe_path)
            .map_err(|e| format!("Failed to set sys.executable: {}", e))?;

        // Also set _base_executable for completeness (used by venv and some tools)
        sys.setattr("_base_executable", exe_path)
            .map_err(|e| format!("Failed to set sys._base_executable: {}", e))?;

        Ok(())
    })
}

/// Verifies that Python is initialized correctly and `lex_learning` works.
///
/// This is the final step of initialization. It confirms that:
///
/// 1. The `lex_learning` module can be imported
/// 2. Key classes (`Pipeline`, `PipelineConfig`, `TrainedModel`) are accessible
///
/// # Errors
///
/// Returns an error with a descriptive message if:
/// - The `lex_learning` module cannot be imported (path issues, syntax errors)
/// - Any of the key classes are missing (incomplete installation)
fn verify_python_setup() -> Result<(), String> {
    pyo3::Python::attach(|py| {
        // Try importing the lex_learning module
        py.import("lex_learning")
            .map_err(|e| format!("Failed to import lex_learning: {}", e))?;

        // Verify we can access key classes
        let lex_learning = py.import("lex_learning").unwrap();
        lex_learning
            .getattr("Pipeline")
            .map_err(|e| format!("Failed to access Pipeline class: {}", e))?;
        lex_learning
            .getattr("PipelineConfig")
            .map_err(|e| format!("Failed to access PipelineConfig class: {}", e))?;
        lex_learning
            .getattr("TrainedModel")
            .map_err(|e| format!("Failed to access TrainedModel class: {}", e))?;

        Ok(())
    })
}

/// Returns the platform-specific runtime directory name.
///
/// This maps the target OS and architecture to the directory names used
/// by `python-build-standalone` releases.
///
/// # Returns
///
/// A static string like `"linux-x86_64"`, `"darwin-aarch64"`, etc.
///
/// # Compile-Time Error
///
/// If compiled for an unsupported platform, this function produces a
/// compile-time error.
///
/// # Supported Platforms
///
/// | OS | Architecture | Directory |
/// |----|-------------|-----------|
/// | Linux | x86_64 | `linux-x86_64` |
/// | Linux | aarch64 | `linux-aarch64` |
/// | macOS | x86_64 | `darwin-x86_64` |
/// | macOS | aarch64 | `darwin-aarch64` |
/// | Windows | x86_64 | `windows-x86_64` |
fn get_platform_dir() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "linux-x86_64"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "linux-aarch64"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "darwin-x86_64"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "darwin-aarch64"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "windows-x86_64"
    }
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
    )))]
    {
        compile_error!("Unsupported platform")
    }
}

/// Checks whether the Python runtime has been successfully initialized.
///
/// This can be used to verify initialization status without triggering
/// initialization (unlike [`initialize()`] which performs initialization
/// if not already done).
///
/// # Returns
///
/// - `true` if [`initialize()`] has been called and succeeded
/// - `false` if [`initialize()`] has not been called, or if it failed
///
/// # Example
///
/// ```rust,ignore
/// use lex_learning::{initialize, is_initialized};
///
/// assert!(!is_initialized());
///
/// initialize()?;
///
/// assert!(is_initialized());
/// ```
#[must_use]
pub fn is_initialized() -> bool {
    INIT_RESULT.get().is_some_and(|r| r.is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::PyAnyMethods;

    #[test]
    fn test_get_platform_dir() {
        let dir = get_platform_dir();
        assert!(!dir.is_empty());
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        assert_eq!(dir, "linux-x86_64");
    }

    #[test]
    fn test_get_app_data_dir() {
        let dir = get_app_data_dir();
        assert!(dir.is_ok());
        let dir = dir.unwrap();
        assert!(dir.to_string_lossy().contains("lex-learning"));
    }

    #[test]
    fn test_find_runtime_dir() {
        // This should find the runtime via the compile-time path
        let dir = find_runtime_dir();
        assert!(dir.is_ok(), "Failed to find runtime: {:?}", dir);
        let dir = dir.unwrap();
        assert!(dir.exists(), "Runtime dir does not exist: {}", dir.display());
        assert!(
            dir.join("bin").join("python3").exists() || dir.join("python.exe").exists(),
            "Python executable not found in runtime"
        );
    }

    // Integration test - tests actual Python initialization
    // Run with: cargo test -- --ignored test_initialize
    #[test]
    #[ignore = "Modifies global state, run separately"]
    fn test_initialize() {
        let result = initialize();
        assert!(result.is_ok(), "Initialization failed: {:?}", result);
        assert!(is_initialized());

        // Verify we can use Python
        pyo3::Python::attach(|py| {
            let lex_learning = py.import("lex_learning").unwrap();
            let version = lex_learning.getattr("__version__").unwrap();
            assert!(!version.to_string().is_empty());
        });
    }

    #[test]
    #[ignore = "Modifies global state, run separately"]
    fn test_initialize_is_idempotent() {
        // First call
        assert!(initialize().is_ok());
        // Second call should also succeed (no-op)
        assert!(initialize().is_ok());
    }
}
