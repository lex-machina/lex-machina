//! Build script for lex-learning.
//!
//! This script configures the build environment for the bundled Python runtime,
//! setting up linker paths and runtime library discovery.
//!
//! # What This Script Does
//!
//! 1. **Locates Python Runtime**: Determines the platform-specific directory
//!    containing the bundled Python 3.12 runtime.
//!
//! 2. **Configures Linker Search Path**: Tells the Rust linker where to find
//!    `libpython3.12.so` (or equivalent) at build time.
//!
//! 3. **Sets RPATH**: Embeds runtime library paths so the compiled binary can
//!    find `libpython` at runtime without `LD_LIBRARY_PATH`.
//!
//! 4. **Exports Environment Variable**: Sets `LEX_PYTHON_RUNTIME_DIR` for use
//!    by the crate at runtime.
//!
//! # Platform Support
//!
//! | Platform | Directory |
//! |----------|-----------|
//! | Linux x86_64 | `runtime/python/linux-x86_64/` |
//! | Linux aarch64 | `runtime/python/linux-aarch64/` |
//! | macOS x86_64 | `runtime/python/darwin-x86_64/` |
//! | macOS aarch64 | `runtime/python/darwin-aarch64/` |
//! | Windows x86_64 | `runtime/python/windows-x86_64/` |
//!
//! Unsupported platforms will fail with a compile error.
//!
//! # RPATH Strategy
//!
//! The script sets two rpath entries:
//!
//! 1. **Relative path** (`$ORIGIN/../runtime/python/.../lib` on Linux,
//!    `@executable_path/../runtime/python/.../lib` on macOS): For deployed
//!    binaries where the runtime is installed relative to the executable.
//!
//! 2. **Absolute path** (the actual `lib/` directory): For development,
//!    allowing `cargo run` from the `target/debug` directory.
//!
//! # Cargo Directives
//!
//! - `cargo:rustc-link-search`: Build-time library search path
//! - `cargo:rustc-link-arg`: Linker arguments for RPATH
//! - `cargo:rustc-env`: Environment variable for runtime use
//! - `cargo:rerun-if-changed`: Rebuild triggers

use std::env;
use std::path::PathBuf;

/// Build script entry point.
///
/// Sets up the Python runtime paths for linking and runtime discovery.
/// This function is called automatically by Cargo during the build process.
fn main() {
    // Get the directory where the crate is located
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Determine the platform-specific runtime directory
    let platform_dir = get_platform_dir();
    let python_runtime_dir = PathBuf::from(&manifest_dir)
        .join("runtime")
        .join("python")
        .join(platform_dir);

    let python_lib_dir = python_runtime_dir.join("lib");

    // Tell the linker where to find libpython at build time
    println!(
        "cargo:rustc-link-search=native={}",
        python_lib_dir.display()
    );

    // Embed rpath so the binary can find libpython at runtime
    // $ORIGIN means "relative to the executable"
    #[cfg(target_os = "linux")]
    {
        // Relative path for deployed binaries
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../runtime/python/{}/lib",
            platform_dir
        );
        // Absolute path for development (running from target/debug)
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            python_lib_dir.display()
        );
    }

    #[cfg(target_os = "macos")]
    {
        // @executable_path is macOS equivalent of $ORIGIN
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../runtime/python/{}/lib",
            platform_dir
        );
        // Absolute path for development
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            python_lib_dir.display()
        );
    }

    // Export the runtime directory for use in the crate
    println!(
        "cargo:rustc-env=LEX_PYTHON_RUNTIME_DIR={}",
        python_runtime_dir.display()
    );

    // Re-run if runtime directory changes
    println!("cargo:rerun-if-changed=runtime/python/{}", platform_dir);
    println!("cargo:rerun-if-changed=build.rs");
}

/// Get the platform-specific runtime directory name.
///
/// Returns a static string identifying the current build target's
/// runtime directory within `runtime/python/`.
///
/// # Supported Platforms
///
/// - `linux-x86_64`: Linux on x86_64 (Intel/AMD 64-bit)
/// - `linux-aarch64`: Linux on ARM64 (e.g., Raspberry Pi 4, AWS Graviton)
/// - `darwin-x86_64`: macOS on Intel
/// - `darwin-aarch64`: macOS on Apple Silicon (M1/M2/M3)
/// - `windows-x86_64`: Windows on x86_64
///
/// # Compile-Time Error
///
/// This function generates a compile error on unsupported platforms,
/// ensuring build failures are clear and early.
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
