//! Tauri Command Modules
//!
//! This module serves as the central hub for all Tauri IPC commands.
//! Commands are organized into logical groups:
//!
//! # Module Organizatioin
//!
//! - **dialog**: Native OS file dialogs (open file picker)
//! - **file_io**: File loading and metadata extraction (CSV parsing with Polars)
//! - **dataframe**: Data Operations (row fetching for virtual scroll, closing files)
//! - **ui**: UI state management (sidebar width, column widths)
//!
//! # How Commands Work
//!
//! Each command is a function decorated with `#[tauri::command]`.
//! The frontend calls these via `invoke("command_name", { args })`.
//! Return values are automatically serialized to JSON.
//!
//! # Re-exports
//!
//! All commands are re-exported at the module level for convenience.
//! This allows `lib.rs` to import all commands with `use commands::*;`

pub mod dataframe;
pub mod dialog;
pub mod file_io;
pub mod ui;

// Re-export all commands for easy access in lib.rs
pub use dataframe::*;
pub use dialog::*;
pub use file_io::*;
pub use ui::*;
