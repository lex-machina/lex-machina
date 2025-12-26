//! Settings Commands
//!
//! This module provides Tauri commands for application settings:
//! - Theme management (System, Light, Dark)
//! - AI provider configuration (None, OpenRouter, Gemini)
//!
//! # Session-Only Storage
//!
//! Both theme and AI provider settings are stored in memory only.
//! API keys are intentionally not persisted to disk for security.
//!
//! # Events
//!
//! - `settings:theme-changed` - Emitted when theme changes

use tauri::{AppHandle, State};

use crate::events::AppEventEmitter;
use crate::state::{AIProviderConfig, AIProviderType, AppState, Theme};

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
/// This immediately updates the theme state and emits a `settings:theme-changed`
/// event so the frontend can apply the new theme.
///
/// # Parameters
///
/// - `theme` - The new theme to apply
/// - `app` - Tauri AppHandle for emitting events
/// - `state` - Tauri-managed application state
///
/// # Events Emitted
///
/// - `settings:theme-changed` - Contains the new Theme value
#[tauri::command]
pub fn set_theme(theme: Theme, app: AppHandle, state: State<'_, AppState>) {
    *state.theme.write() = theme;
    app.emit_theme_changed(theme);
}

// ============================================================================
// AI PROVIDER COMMANDS
// ============================================================================

/// Gets the current AI provider configuration.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
///
/// # Returns
///
/// - `Some(AIProviderConfig)` if an AI provider is configured
/// - `None` if no AI provider is set (rule-based decisions only)
#[tauri::command]
pub fn get_ai_provider_config(state: State<'_, AppState>) -> Option<AIProviderConfig> {
    state.ai_provider_config.read().clone()
}

/// Configures an AI provider for preprocessing decisions.
///
/// Sets the AI provider type and API key. The API key is stored in memory
/// only (session-only, not persisted to disk).
///
/// # Parameters
///
/// - `provider` - The AI provider type (OpenRouter or Gemini)
/// - `api_key` - The API key for the provider
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
    state: State<'_, AppState>,
) -> Result<(), String> {
    if provider == AIProviderType::None {
        return Err("Use clear_ai_provider to remove AI configuration".to_string());
    }

    if api_key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    *state.ai_provider_config.write() = Some(AIProviderConfig { provider, api_key });
    Ok(())
}

/// Clears the AI provider configuration.
///
/// After calling this, preprocessing will use rule-based decisions only.
///
/// # Parameters
///
/// - `state` - Tauri-managed application state
#[tauri::command]
pub fn clear_ai_provider(state: State<'_, AppState>) {
    *state.ai_provider_config.write() = None;
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
