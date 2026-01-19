use std::env;
use std::path::PathBuf;

fn main() {
    configure_python_rpath();
    tauri_build::build();
}

fn configure_python_rpath() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let platform_dir = get_platform_dir();
    let python_runtime_dir = PathBuf::from(manifest_dir)
        .join("..")
        .join("crates")
        .join("lex-learning")
        .join("runtime")
        .join("python")
        .join(platform_dir);
    let python_lib_dir = python_runtime_dir.join("lib");

    println!(
        "cargo:rustc-link-search=native={}",
        python_lib_dir.display()
    );

    #[cfg(target_os = "linux")]
    {
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../crates/lex-learning/runtime/python/{}/lib",
            platform_dir
        );
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            python_lib_dir.display()
        );
    }

    #[cfg(target_os = "macos")]
    {
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../crates/lex-learning/runtime/python/{}/lib",
            platform_dir
        );
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            python_lib_dir.display()
        );
    }

    println!(
        "cargo:rerun-if-changed=../crates/lex-learning/runtime/python/{}",
        platform_dir
    );
    println!("cargo:rerun-if-changed=build.rs");
}

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
