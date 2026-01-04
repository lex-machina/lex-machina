//! UI State Commands
//!
//! This module provides commands for managing UI layout state:
//! - Sidebar width
//! - Sidebar collapsed state
//! - Column widths
//! - Grid scroll position
//!
//! # Why Store UI State in Rust?
//!
//! Following the "Rust Supremacy" principle, all state lives in Rust.
//! Benefits:
//! - Single source of truth (no sync issues between frontend/backend)
//! - State survives webview reloads
//! - Could be persisted to disk in the future
//! - Consistent with the rest of the architecture
//!
//! # State Flow
//!
//! ```text
//! => User drags column resize handle
//! => Frontend updates local state (instant visual feedback)
//! => On drag end: invoke("set_column_widths", { widths })
//! => Rust stores widths in `AppState.ui_state`
//! ```
//!
//! The frontend maintains its own state for instant feedback during drag,
//! then persists to Rust when the drag ends. This gives smooth UX while
//! keeping Rust as the source of truth.

use tauri::{AppHandle, State};

use crate::state::{AppState, GridScrollPosition, UIState};

// ============================================================================
// TAURI COMMANDS
// ============================================================================

/// Return the current UI state.
///
/// Allows the frontend to query the current UI layout settings.
/// Useful for restoring state after page reload.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// A clone of the current `UIState` containing:
/// - `sidebar_width` - Current sidebar width in pixels
/// - `column_widths` - Array of column widths in pixels
///
/// # Frontend usage
///
/// ```typescript
/// const uiState = await invoke<UIState>("get_ui_state");
/// setSidebarWidth(uiState.sidebar_width);
/// setColumnWidths(uiState.column_widths);
/// ```
#[tauri::command]
pub fn get_ui_state(state: State<'_, AppState>) -> UIState {
    // Aquire read lock and clone the state
    // Clone is necessary because we can't return a reference
    state.ui_state.read().clone()
}

/// Update the sidebar width.
///
/// Called when the user finishes dragging the sidebar resize handle.
/// The width is persisted to the settings store for restoration on next launch.
///
/// # Parameters
///
/// - `width` - New sidebar width in pixels
/// - `app` - Tauri AppHandle for accessing the settings store
/// - `state` - Tauri-managed application state
///
/// # Frontend Usage
///
/// ```typescript
/// // Called in onResizeEnd callback
/// await invoke("set_sidebar_width", { width: 300 });
/// ```
///
/// # Note
///
/// This command persists the width to disk. The frontend has already updated
/// its local state for instant visual feedback; this call persists to Rust
/// and the settings store.
#[tauri::command]
pub fn set_sidebar_width(width: f32, app: AppHandle, state: State<'_, AppState>) {
    // Update in-memory state
    let mut guard = state.ui_state.write();
    guard.sidebar_width = width;
    drop(guard); // Release lock before IO

    // Persist to settings store (best effort - don't fail the command)
    if let Err(e) = super::settings::persist_sidebar_width(&app, width) {
        log::warn!("Failed to persist sidebar width: {}", e);
    }
}

/// Updates a single column's width.
///
/// Called when the user finishes resizing a specific column.
/// Less efficient that `set_column_widths` if updating multiple columns.
///
/// # Parameters
///
/// - `col` - Column index (0-indexed)
/// - `width` - New width in pixels
/// - `state` - Tauri-managed application state
///
/// # Bounds Handling
///
/// If `col` is beyond the current vector length, the vector is
/// automatically resized with a default width of 150px for new entries.
/// This handles edge cases where columns might be added dynamically.
///
/// # Frontend Usage
///
/// ```typescript
/// // Called in onColumnResizeEnd callback
/// await invoke("set_column_width", { col: 2, width: 200 });
/// ```
#[tauri::command]
pub fn set_column_width(col: usize, width: f32, state: State<'_, AppState>) {
    let mut guard = state.ui_state.write();

    // Ensure vector is large enough to hold this column index
    // If not, extend with default width (150px)
    if col >= guard.column_widths.len() {
        guard.column_widths.resize(col + 1, 150.0);
    }

    guard.column_widths[col] = width;
}

/// Updates all column widths at once.
///
/// More efficient than calling `set_column_width` multiple times.
/// Replaces the entire column widths array.
///
/// # Parameters
///
/// - `widths` - Array of column widths in pixels (one per column)
/// - `state` - Tauri-managed application state
///
/// # Frontend Usage
///
/// ```typescript
/// // Called after any column resize ends
/// await invoke("set_column_widths", { widths: [100, 150, 200, 250] });
/// ```
///
/// # Note
///
/// This replaces the entire array, so make sure to pass widths
/// for ALL columns, not just the one that changed.
#[tauri::command]
pub fn set_column_widths(widths: Vec<f32>, state: State<'_, AppState>) {
    let mut guard = state.ui_state.write();
    guard.column_widths = widths;
}

// ============================================================================
// SCROLL POSITION COMMANDS
// ============================================================================

/// Returns the current grid scroll position.
///
/// Allows the frontend to restore scroll position when navigating
/// back to the data page or after page reload.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// The current `GridScrollPosition` containing:
/// - `row_index` - Current top row index (vertical scroll)
/// - `scroll_left` - Horizontal scroll offset in pixels
///
/// # Frontend Usage
///
/// ```typescript
/// const pos = await invoke<GridScrollPosition>("get_grid_scroll");
/// setCurrentRowIndex(pos.row_index);
/// setScrollLeft(pos.scroll_left);
/// ```
#[tauri::command]
pub fn get_grid_scroll(state: State<'_, AppState>) -> GridScrollPosition {
    state.ui_state.read().grid_scroll.clone()
}

/// Updates the grid scroll position.
///
/// Called when the user scrolls the data grid. This allows the scroll
/// position to be restored when navigating between pages.
///
/// # Parameters
///
/// - `row_index` - Current top row index (vertical scroll position)
/// - `scroll_left` - Horizontal scroll offset in pixels
/// - `state` - Tauri-managed application state
///
/// # Frontend Usage
///
/// ```typescript
/// // Called when scroll position changes (debounced)
/// await invoke("set_grid_scroll", { rowIndex: 100, scrollLeft: 50 });
/// ```
///
/// # Performance Note
///
/// This should be called with debouncing to avoid excessive IPC calls
/// during rapid scrolling. Typical debounce: 100-200ms after scroll stops.
#[tauri::command]
pub fn set_grid_scroll(row_index: usize, scroll_left: f32, state: State<'_, AppState>) {
    let mut guard = state.ui_state.write();
    guard.grid_scroll.row_index = row_index;
    guard.grid_scroll.scroll_left = scroll_left;
}

// ============================================================================
// SIDEBAR COLLAPSED COMMANDS
// ============================================================================

/// Toggles the sidebar collapsed state.
///
/// This toggles between expanded and collapsed states, persists the change,
/// and returns the new collapsed state.
///
/// # Parameters
///
/// - `app` - Tauri AppHandle for accessing the settings store
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// The new collapsed state (true = collapsed, false = expanded).
///
/// # Frontend Usage
///
/// ```typescript
/// const newState = await invoke<boolean>("toggle_sidebar");
/// // newState is true if now collapsed, false if now expanded
/// ```
#[tauri::command]
pub fn toggle_sidebar(app: AppHandle, state: State<'_, AppState>) -> bool {
    let mut guard = state.ui_state.write();
    let new_collapsed = !guard.sidebar_collapsed;
    guard.sidebar_collapsed = new_collapsed;
    drop(guard); // Release lock before IO

    // Persist to settings store (best effort - don't fail the command)
    if let Err(e) = super::settings::persist_sidebar_collapsed(&app, new_collapsed) {
        log::warn!("Failed to persist sidebar collapsed state: {}", e);
    }

    new_collapsed
}

/// Sets the sidebar collapsed state explicitly.
///
/// Unlike `toggle_sidebar`, this sets the state to a specific value.
/// Useful for programmatic control or restoring state.
///
/// # Parameters
///
/// - `collapsed` - The new collapsed state (true = collapsed, false = expanded)
/// - `app` - Tauri AppHandle for accessing the settings store
/// - `state` - Tauri-managed application state
///
/// # Frontend Usage
///
/// ```typescript
/// // Collapse the sidebar
/// await invoke("set_sidebar_collapsed", { collapsed: true });
///
/// // Expand the sidebar
/// await invoke("set_sidebar_collapsed", { collapsed: false });
/// ```
#[tauri::command]
pub fn set_sidebar_collapsed(collapsed: bool, app: AppHandle, state: State<'_, AppState>) {
    let mut guard = state.ui_state.write();
    guard.sidebar_collapsed = collapsed;
    drop(guard); // Release lock before IO

    // Persist to settings store (best effort - don't fail the command)
    if let Err(e) = super::settings::persist_sidebar_collapsed(&app, collapsed) {
        log::warn!("Failed to persist sidebar collapsed state: {}", e);
    }
}
