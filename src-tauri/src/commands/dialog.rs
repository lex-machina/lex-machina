//! Native File Dialog Commands
//!
//! This module provides commands for opening native OS file dialogs.
//! Uses the `tauri-plugin-dialog` for cross-platform file picking.
//!
//! # Why Native Dialogs?
//!
//! Using native OS dialogs (instead of web-style `<input type="file">`) give us:
//! - Familiar look and feel for desktop users
//! - Better integration with the OS (recent files, favorites, etc.)
//! - Proper file system access permissions on all platforms
//! - Professional desktop application appearance
//!
//! # permissions
//!
//! The dialog plugin requires permissions in `capabilities/default.json`:
//! ```json
//!"permissions": ["dialog:default", "dialog:allow-open"]
//! ```

use tauri_plugin_dialog::DialogExt;

/// Opens a native file dialog filtered for CSV files.
///
/// This command displays the OS-native file picker dialog with a filter
/// that only shows CSV files. The user can navigate their file system
/// and select a file to open
///
/// # Returns
///
/// - `Some(String)` - The full path to the selected file
/// - `None` - If the user cancelled the dialog
///
/// # Frontend Usage
///
/// ```typescript
/// const filePath = await invoke<string | null>("open_file_dialog");
/// if (filePath) {
///     // User selected a file, now load it
///     const info = await invoke("load_file", { path: filePath });
/// }
/// ```
///
/// # Platform Behavior
///
/// - **Windows**: Opens standart Windows file picker
/// - **macOS**: Opens Filder-style file picker
/// - **Linux**: Opens GTK/Qt file picker (depends on desktop environment)
///
/// # Notes
///
/// - Uses `blocking_pick_file` which blocks the thread until dialog closes
/// - This is fine because Tauri commands run in a thread pool, not the main thread
/// - The `async` keyword is present for Tauri command compatability
#[tauri::command]
pub async fn open_file_dialog(app: tauri::AppHandle) -> Option<String> {
    // Get the dialog extension from the app handle
    let file_path = app
        .dialog()
        // Create a file dialog (as opposed to folder dialog)
        .file()
        // Add filter: only show files with .csv extension
        // First parameter is the display name, second is the extensions array
        .add_filter("CSV Files", &["csv"])
        // Open the dialog and block until user makes a selection or cancels
        // Returns Option<FilePath> where FilePath is a path wrapper type
        .blocking_pick_file();

    // Convert FilePath to String (is user selected a file)
    // FilePath implements ToString, giving us the full path
    file_path.map(|p| p.to_string())
}
