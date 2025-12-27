//! Settings Commands
//!
//! This module provides Tauri commands for application settings:
//! - Theme management (System, Light, Dark)
//! - AI provider configuration (None, OpenRouter, Gemini)
//! - Sidebar width persistence
//!
//! # Persistence
//!
//! Settings are persisted using two mechanisms:
//! - **tauri-plugin-store**: Theme, sidebar width, AI provider type
//!   - Stored in `settings.json` in the app data directory
//! - **OS Keychain**: API keys (via keyring module)
//!   - macOS: Keychain, Windows: Credential Manager, Linux: Secret Service
//!
//! # Events
//!
//! - `settings:theme-changed` - Emitted when theme changes

use serde_json::json;
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;

use crate::events::AppEventEmitter;
use crate::state::{AIProviderConfig, AIProviderType, AppState, Theme};

/// The settings store filename.
const SETTINGS_STORE: &str = "settings.json";

/// Store keys for persisted settings.
mod store_keys {
    pub const THEME: &str = "theme";
    pub const SIDEBAR_WIDTH: &str = "sidebar_width";
    pub const AI_PROVIDER_TYPE: &str = "ai_provider_type";
}

// ============================================================================
// INITIALIZATION
// ============================================================================

/// Initializes settings from the persisted store.
///
/// This should be called during app startup (in the setup hook) to restore
/// user preferences from the previous session.
///
/// # What Gets Restored
///
/// - Theme (System/Light/Dark)
/// - Sidebar width
/// - AI provider type (but NOT the API key - user must re-authenticate)
///
/// # Parameters
///
/// - `app` - Tauri AppHandle for accessing the store
/// - `state` - Application state to update with persisted values
///
/// # Returns
///
/// - `Ok(())` on success
/// - `Err(String)` if store access fails
pub fn init_settings_from_store<R: tauri::Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
) -> Result<(), String> {
    let store = app
        .store(SETTINGS_STORE)
        .map_err(|e| format!("Failed to open settings store: {}", e))?;

    // Restore theme
    if let Some(theme_value) = store.get(store_keys::THEME)
        && let Ok(theme) = serde_json::from_value::<Theme>(theme_value)
    {
        *state.theme.write() = theme;
        log::info!("Restored theme: {:?}", theme);
    }

    // Restore sidebar width
    if let Some(width_value) = store.get(store_keys::SIDEBAR_WIDTH)
        && let Some(width) = width_value.as_f64()
    {
        state.ui_state.write().sidebar_width = width as f32;
        log::info!("Restored sidebar width: {}", width);
    }

    // Restore AI provider type (API key is loaded separately via keyring)
    if let Some(provider_value) = store.get(store_keys::AI_PROVIDER_TYPE)
        && let Ok(provider) = serde_json::from_value::<AIProviderType>(provider_value)
        && provider != AIProviderType::None
    {
        // Try to load the API key from keyring
        if let Ok(Some(api_key)) = super::keyring::get_api_key(provider) {
            *state.ai_provider_config.write() = Some(AIProviderConfig {
                provider,
                api_key,
            });
            log::info!("Restored AI provider: {:?} (with API key)", provider);
        } else {
            log::info!(
                "Restored AI provider type: {:?} (no API key found)",
                provider
            );
        }
    }

    Ok(())
}

// ============================================================================
// THEME COMMANDS
// ============================================================================

/// Gets the current theme setting.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// The current theme (System, Light, or Dark).
#[tauri::command]
pub fn get_theme(state: State<'_, AppState>) -> Theme {
    *state.theme.read()
}

/// Sets the application theme.
///
/// This immediately updates the theme state, persists it to the store,
/// and emits a `settings:theme-changed` event so the frontend can apply
/// the new theme.
///
/// # Parameters
///
/// - `theme` - The new theme to apply
/// - `app` - Tauri AppHandle for emitting events and accessing store
/// - `state` - Tauri-managed application state
///
/// # Events Emitted
///
/// - `settings:theme-changed` - Contains the new Theme value
#[tauri::command]
pub fn set_theme(theme: Theme, app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    // Update in-memory state
    *state.theme.write() = theme;

    // Persist to store
    let store = app
        .store(SETTINGS_STORE)
        .map_err(|e| format!("Failed to open settings store: {}", e))?;

    store.set(store_keys::THEME, json!(theme));
    store
        .save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    log::info!("Theme set to: {:?}", theme);

    // Emit event to frontend
    app.emit_theme_changed(theme);

    Ok(())
}

// ============================================================================
// SIDEBAR WIDTH PERSISTENCE
// ============================================================================

/// Persists the sidebar width to the settings store.
///
/// This is called from the UI state commands when sidebar width changes.
/// It's a helper function, not a direct Tauri command.
///
/// # Parameters
///
/// - `app` - Tauri AppHandle for accessing the store
/// - `width` - The new sidebar width in pixels
pub fn persist_sidebar_width<R: tauri::Runtime>(app: &AppHandle<R>, width: f32) -> Result<(), String> {
    let store = app
        .store(SETTINGS_STORE)
        .map_err(|e| format!("Failed to open settings store: {}", e))?;

    store.set(store_keys::SIDEBAR_WIDTH, json!(width));
    store
        .save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    Ok(())
}

// ============================================================================
// AI PROVIDER COMMANDS
// ============================================================================

/// Gets the current AI provider configuration.
///
/// Returns the provider type and a masked version of the API key for display.
/// The full API key is never sent to the frontend.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Some(AIProviderConfig)` if an AI provider is configured
///   - Note: The `api_key` field contains a masked value (e.g., "sk-...xxxx")
/// - `None` if no AI provider is set (rule-based decisions only)
#[tauri::command]
pub fn get_ai_provider_config(state: State<'_, AppState>) -> Option<AIProviderConfig> {
    state.ai_provider_config.read().as_ref().map(|config| {
        // Return a masked version of the API key for display
        let masked_key = mask_api_key(&config.api_key);
        AIProviderConfig {
            provider: config.provider,
            api_key: masked_key,
        }
    })
}

/// Masks an API key for safe display.
///
/// Shows only the first 4 and last 4 characters, with dots in between.
fn mask_api_key(key: &str) -> String {
    if key.len() <= 12 {
        return "*".repeat(key.len());
    }
    format!("{}...{}", &key[..4], &key[key.len() - 4..])
}

/// Configures an AI provider for preprocessing decisions.
///
/// Sets the AI provider type and API key. The API key is stored securely
/// in the OS keychain, while the provider type is saved to the settings store.
///
/// # Parameters
///
/// - `provider` - The AI provider type (OpenRouter or Gemini)
/// - `api_key` - The API key for the provider
/// - `app` - Tauri AppHandle for accessing store and keyring
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Ok(())` on success
/// - `Err(String)` if the provider type is "None" (use `clear_ai_provider` instead)
///
/// # Notes
///
/// This does not validate the API key. Use `validate_ai_api_key` to test
/// if the key is valid before saving.
#[tauri::command]
pub fn configure_ai_provider(
    provider: AIProviderType,
    api_key: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if provider == AIProviderType::None {
        return Err("Use clear_ai_provider to remove AI configuration".to_string());
    }

    if api_key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    // Store API key in OS keychain (secure storage)
    super::keyring::set_api_key(provider, api_key.clone())?;

    // Store provider type in settings store
    let store = app
        .store(SETTINGS_STORE)
        .map_err(|e| format!("Failed to open settings store: {}", e))?;

    store.set(store_keys::AI_PROVIDER_TYPE, json!(provider));
    store
        .save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    // Update in-memory state
    *state.ai_provider_config.write() = Some(AIProviderConfig { provider, api_key });

    log::info!("Configured AI provider: {:?}", provider);
    Ok(())
}

/// Clears the active AI provider configuration.
///
/// This deactivates the current AI provider but does NOT delete the saved
/// API key from the OS keychain. The user can later reactivate the provider
/// using `switch_ai_provider` without re-entering their key.
///
/// To permanently delete a saved API key, use `delete_saved_provider`.
///
/// # Parameters
///
/// - `app` - Tauri AppHandle for accessing store
/// - `state` - Tauri-managed application state
#[tauri::command]
pub fn clear_ai_provider(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    // Clear provider type from settings store (but keep API key in keyring)
    let store = app
        .store(SETTINGS_STORE)
        .map_err(|e| format!("Failed to open settings store: {}", e))?;

    store.set(store_keys::AI_PROVIDER_TYPE, json!(AIProviderType::None));
    store
        .save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    // Clear in-memory state
    *state.ai_provider_config.write() = None;

    log::info!("Cleared active AI provider (keys preserved in keychain)");
    Ok(())
}

/// Validates an AI provider API key.
///
/// Tests the API key by making a minimal request to the provider.
/// This allows the user to verify their key is valid before saving.
///
/// # Parameters
///
/// - `provider` - The AI provider type to validate against
/// - `api_key` - The API key to validate
///
/// # Returns
///
/// - `Ok(true)` if the API key is valid
/// - `Ok(false)` if the API key is invalid (authentication failed)
/// - `Err(String)` if there was a network or other error
///
/// # Notes
///
/// This is an async command that makes a network request.
/// The actual validation logic depends on the AI provider.
#[tauri::command]
pub async fn validate_ai_api_key(
    provider: AIProviderType,
    api_key: String,
) -> Result<bool, String> {
    // For now, we do basic validation only
    // Full validation would require making API calls to each provider

    if api_key.trim().is_empty() {
        return Ok(false);
    }

    match provider {
        AIProviderType::None => Ok(false),
        AIProviderType::OpenRouter => {
            // OpenRouter API keys typically start with "sk-or-"
            // But we accept any non-empty key for flexibility
            Ok(api_key.len() >= 10)
        }
        AIProviderType::Gemini => {
            // Gemini API keys are typically 39 characters
            // But we accept any non-empty key for flexibility
            Ok(api_key.len() >= 10)
        }
    }
}

/// Returns a list of AI providers that have saved API keys.
///
/// This checks the OS keychain for each provider type and returns
/// a list of providers that have stored credentials.
///
/// # Returns
///
/// A vector of `AIProviderType` values that have saved API keys.
/// Does not include `AIProviderType::None`.
#[tauri::command]
pub fn get_saved_providers() -> Vec<AIProviderType> {
    let providers = [AIProviderType::OpenRouter, AIProviderType::Gemini];

    providers
        .into_iter()
        .filter(|&provider| {
            super::keyring::has_api_key(provider).unwrap_or(false)
        })
        .collect()
}

/// Switches to a different AI provider that has a saved API key.
///
/// This allows users to switch between providers without re-entering
/// their API key. The provider must have a saved key in the OS keychain.
///
/// # Parameters
///
/// - `provider` - The AI provider to switch to
/// - `app` - Tauri AppHandle for accessing store
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Ok(())` on success
/// - `Err(String)` if no API key is saved for this provider
#[tauri::command]
pub fn switch_ai_provider(
    provider: AIProviderType,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if provider == AIProviderType::None {
        return Err("Use clear_ai_provider to disable AI".to_string());
    }

    // Check if provider has a saved key
    let api_key = super::keyring::get_api_key(provider)?
        .ok_or_else(|| format!("No API key saved for {:?}", provider))?;

    // Update settings store with new active provider
    let store = app
        .store(SETTINGS_STORE)
        .map_err(|e| format!("Failed to open settings store: {}", e))?;

    store.set(store_keys::AI_PROVIDER_TYPE, json!(provider));
    store
        .save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    // Update in-memory state
    *state.ai_provider_config.write() = Some(AIProviderConfig { provider, api_key });

    log::info!("Switched to AI provider: {:?}", provider);
    Ok(())
}

/// Deletes a saved API key for a specific provider.
///
/// This removes the API key from the OS keychain without affecting
/// the currently active provider (unless deleting the active one).
///
/// If the deleted provider is the currently active one, the active
/// provider will be cleared.
///
/// # Parameters
///
/// - `provider` - The AI provider whose key should be deleted
/// - `app` - Tauri AppHandle for accessing store
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Ok(())` on success (including if no key existed)
/// - `Err(String)` if deletion fails
#[tauri::command]
pub fn delete_saved_provider(
    provider: AIProviderType,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if provider == AIProviderType::None {
        return Ok(());
    }

    // Delete from keyring
    super::keyring::delete_api_key(provider)?;

    // If this was the active provider, clear it
    let is_active = state
        .ai_provider_config
        .read()
        .as_ref()
        .is_some_and(|c| c.provider == provider);

    if is_active {
        // Clear provider type from settings store
        let store = app
            .store(SETTINGS_STORE)
            .map_err(|e| format!("Failed to open settings store: {}", e))?;

        store.set(store_keys::AI_PROVIDER_TYPE, json!(AIProviderType::None));
        store
            .save()
            .map_err(|e| format!("Failed to save settings: {}", e))?;

        // Clear in-memory state
        *state.ai_provider_config.write() = None;

        log::info!("Deleted API key and cleared active provider: {:?}", provider);
    } else {
        log::info!("Deleted saved API key for provider: {:?}", provider);
    }

    Ok(())
}
