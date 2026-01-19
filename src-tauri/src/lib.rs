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
//! |                                    |  - Preprocessing cmds   |  |
//! |                                    |  - Settings commands    |  |
//! |                                    ---------------------------  |
//! |                                                                 |
//! |  -----------------------------------------------------------    |
//! |  |                    Events (Rust → Frontend)              |    |
//! |  |  file:loaded, file:closed, app:loading, app:error        |    |
//! |  |  preprocessing:progress, preprocessing:complete          |    |
//! |  |  preprocessing:error, settings:theme-changed             |    |
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
//! - **Preprocessing**: Data cleaning pipeline operations
//! - **Settings**: Theme and AI provider configuration
//!
//! # Event System
//!
//! Events allow Rust to push state changes to the frontend:
//! - `file:loaded` - File successfully loaded (contains FileInfo)
//! - `file:closed` - File closed
//! - `app:loading` - Loading state changed
//! - `app:error` - Error occurred
//! - `preprocessing:progress` - Pipeline progress update
//! - `preprocessing:complete` - Pipeline finished successfully
//! - `preprocessing:error` - Pipeline failed
//! - `preprocessing:cancelled` - Pipeline cancelled by user
//! - `settings:theme-changed` - Theme setting changed

mod commands;
pub mod events;
mod state;

use state::AppState;
use tauri::Manager;

/// Tauri mobile entry point attribute.
/// This macro generates the appropriate entry point for mobile platforms.
/// On desktop, it has no effect.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        // ====================================================================
        // PLUGINS
        // ====================================================================
        // Dialog plugin: Provides native file open/save dialogs.
        // Used by `open_file_dialog` command to show OS-native file picker.
        // Required permission in capabilities/default.json
        .plugin(tauri_plugin_dialog::init())
        // Opener plugin: Opens URLs and files in the system's default application.
        // Used for external links (e.g., GitHub) to open in the system browser.
        // Required permission in capabilities/default.json
        .plugin(tauri_plugin_opener::init())
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
            // Sidebar collapsed state
            // Toggles sidebar collapsed state and returns new state
            commands::toggle_sidebar,
            // Explicitly sets sidebar collapsed state
            commands::set_sidebar_collapsed,
            // Preprocessing commands
            // Starts the preprocessing pipeline on the loaded DataFrame
            commands::start_preprocessing,
            // Cancels the currently running preprocessing pipeline
            commands::cancel_preprocessing,
            // Gets the preprocessing history
            commands::get_preprocessing_history,
            // Loads a history entry (currently returns error - not fully implemented)
            commands::load_history_entry,
            // Clears all preprocessing history
            commands::clear_preprocessing_history,
            // Gets file info for the processed DataFrame
            commands::get_processed_file_info,
            // Fetches rows from the processed DataFrame for virtual scrolling
            commands::get_processed_rows,
            // Clears the processed DataFrame from memory
            commands::clear_processed_data,
            // Gets the last preprocessing result summary (persists across navigation)
            commands::get_last_preprocessing_result,
            // Clears the last preprocessing result (when user dismisses)
            commands::clear_last_preprocessing_result,
            // Exports processed data to CSV with JSON report
            commands::export_processed_data,
            // Settings commands
            // Gets the current theme setting
            commands::get_theme,
            // Sets the application theme
            commands::set_theme,
            // Gets the current AI provider configuration
            commands::get_ai_provider_config,
            // Configures an AI provider for preprocessing decisions
            commands::configure_ai_provider,
            // Clears the active AI provider (keeps saved keys)
            commands::clear_ai_provider,
            // Validates an AI provider API key
            commands::validate_ai_api_key,
            // Gets list of providers with saved API keys
            commands::get_saved_providers,
            // Switches to a provider with a saved key
            commands::switch_ai_provider,
            // Deletes a saved provider's API key
            commands::delete_saved_provider,
            // Navigation bar position commands
            // Gets the current navigation bar position
            commands::get_nav_bar_position,
            // Sets the navigation bar position
            commands::set_nav_bar_position,
            // Keyring commands (secure credential storage)
            // Stores an API key in the OS keychain
            commands::set_api_key,
            // Retrieves an API key from the OS keychain
            commands::get_api_key,
            // Deletes an API key from the OS keychain
            commands::delete_api_key,
            // Checks if an API key exists in the OS keychain
            commands::has_api_key,
            // Preprocessing UI state persistence
            // Gets saved preprocessing page state (columns, row range, config)
            commands::get_preprocessing_ui_state,
            // Saves preprocessing page state for navigation persistence
            commands::set_preprocessing_ui_state,
            // ML commands
            commands::is_ml_initialized,
            commands::initialize_ml,
            commands::start_training,
            commands::cancel_training,
            commands::get_training_result,
            commands::get_shap_plot,
            commands::get_model_info,
            commands::save_model,
            commands::load_model,
            commands::predict_single,
            commands::predict_batch,
            commands::get_training_history,
            commands::clear_training_history,
            commands::get_ml_ui_state,
            commands::set_ml_ui_state,
            commands::get_auto_start_ml_kernel,
            commands::set_auto_start_ml_kernel,
        ])
        // ====================================================================
        // SETUP HOOK
        // ====================================================================
        // Setup runs once after the app is initialized but before the window opens.
        // Used here to:
        // 1. Initialize logging (debug builds only)
        // 2. Restore persisted settings from store and keychain
        .setup(|app| {
            // Only enable logging plugin in debug builds
            // This prevents log spam in production releases
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize settings from persisted store
            // This restores theme, sidebar width, and AI provider config
            let state = app.state::<AppState>();
            if let Err(e) = commands::settings::init_settings_from_store(app.handle(), &state) {
                log::warn!("Failed to restore settings: {}", e);
                // Non-fatal - app continues with default settings
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
