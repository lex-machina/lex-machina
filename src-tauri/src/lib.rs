//! Lex Machina - Tauri Application Entry Point
//!
//! This module sets up and configures the Tauri application. It:
//! 1. Initializes plugins (dialog, logging)
//! 2. Creates and manages application state
//! 3. Registers all IPC command handlers
//! 4. Starts the Tauri runtime
//!
//! # Architecture Overview
//!
//! ```text
//! -------------------------------------------------------------------
//! |                      Tauri Application                          |
//! |                                                                 |
//! |  ---------------  ---------------  ---------------------------  |
//! |  |   Plugins   |  |    State    |  |    Command Handlers     |  |
//! |  |  - dialog   |  |  AppState   |  |  - open_file_dialog     |  |
//! |  |  - log      |  |  (managed)  |  |  - load_file            |  |
//! |  ---------------  ---------------  |  - get_rows             |  |
//! |                                    |  - close_file           |  |
//! |                                    |  - UI state commands    |  |
//! |                                    ---------------------------  |
//! |                                                                 |
//! |  -----------------------------------------------------------    |
//! |  |                    Events (Rust → Frontend)              |    |
//! |  |  file:loaded, file:closed, app:loading, app:error        |    |
//! |  -----------------------------------------------------------    |
//! |                                                                 |
//! |  -----------------------------------------------------------    |
//! |  |                    WebView (Next.js)                    |    |
//! |  |               Communicates via IPC (invoke)             |    |
//! |  -----------------------------------------------------------    |
//! -------------------------------------------------------------------
//! ```
//!
//! # Command Categories
//!
//! Commands are organized by function:
//! - **Dialog**: Native OS file dialogs
//! - **File I/O**: Loading/reading CSV files
//! - **DataFrame**: Row fetching for virtual scroll, closing files
//! - **UI State**: Persisting layout preferences
//!
//! # Event System
//!
//! Events allow Rust to push state changes to the frontend:
//! - `file:loaded` - File successfully loaded (contains FileInfo)
//! - `file:closed` - File closed
//! - `app:loading` - Loading state changed
//! - `app:error` - Error occurred

mod commands;
pub mod events;
mod state;

use state::AppState;

/// Tauri mobile entry point attribute.
/// This macro generates the appropriate entry point for mobile platforms.
/// On desktop, it has no effect.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // ====================================================================
        // PLUGINS
        // ====================================================================
        // Dialog plugin: Provides native file open/save dialogs.
        // Used by `open_file_dialog` command to show OS-native file picker.
        // Required permission in capabilities/default.json
        .plugin(tauri_plugin_dialog::init())
        // ====================================================================
        // STATE MANAGEMENT
        // ====================================================================
        // Register `AppState` as managed state.
        // This makes it available to all command handlers via `State<'_, AppState>`
        // Tauri ensures thread-safe access across multiple async invocations.
        .manage(AppState::new())
        // ====================================================================
        // COMMAND HANDLERS
        // ====================================================================
        // Register all IPC commands that the frontend can invoke.
        // Commands are called from TypeScript via: `invoke("command_name", { args })`
        .invoke_handler(tauri::generate_handler![
            // Dialog commands
            // Opens native file picker filtered to CSV files
            commands::open_file_dialog,
            // File I/O commands
            // Loads a CSV file into memory using Polars
            commands::load_file,
            // Returns cached file metadata (if already loaded)
            commands::get_file_info,
            // DataFrame Commands
            // Fetches a range of rows for virtual scrolling
            commands::get_rows,
            // Closes the current file and frees memory
            commands::close_file,
            // UI state commands
            // Gets current UI layout state
            commands::get_ui_state,
            // Updates sidebar width
            commands::set_sidebar_width,
            // Updates a single column width
            commands::set_column_width,
            // Updates all column widths at once
            commands::set_column_widths,
            // Grid scroll position
            // Gets current scroll position (for restoring after navigation)
            commands::get_grid_scroll,
            // Updates scroll position (debounced from frontend)
            commands::set_grid_scroll,
        ])
        // ====================================================================
        // SETUP HOOK
        // ====================================================================
        // Setup runs once after the app is initialized but before the window opens.
        // Used here to conditionally enable logging in debug builds
        .setup(|app| {
            // Only enable logging plugin in debug builds
            // Thiss prevents log spam in production releases
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        // ====================================================================
        // RUN
        // ====================================================================
        // Start the Tauri application.
        // generate_context!() reads tauri.conf.json at compile time.
        // This call blocks until the application exits.
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
