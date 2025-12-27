//! Secure Credential Storage Commands
//!
//! This module provides Tauri commands for secure API key storage using the
//! operating system's native keychain:
//! - **macOS**: Keychain
//! - **Windows**: Credential Manager
//! - **Linux**: Secret Service (GNOME Keyring, KWallet, etc.)
//!
//! # Security
//!
//! API keys are stored encrypted by the OS, not in plain text files.
//! Each provider's API key is stored as a separate credential entry.
//!
//! # Service Name
//!
//! All credentials use "lex-machina" as the service name.
//! The username is the provider name (e.g., "openrouter", "gemini").

use keyring::Entry;

use crate::state::AIProviderType;

/// The service name used for all keyring entries.
const KEYRING_SERVICE: &str = "lex-machina";

/// Converts an AI provider type to a keyring username.
fn provider_to_username(provider: AIProviderType) -> &'static str {
    match provider {
        AIProviderType::None => "none", // Should never be used
        AIProviderType::OpenRouter => "openrouter",
        AIProviderType::Gemini => "gemini",
    }
}

/// Stores an API key securely in the OS keychain.
///
/// # Parameters
///
/// - `provider` - The AI provider type (determines the credential name)
/// - `api_key` - The API key to store
///
/// # Returns
///
/// - `Ok(())` on success
/// - `Err(String)` if storage fails (e.g., keychain unavailable, permission denied)
///
/// # Platform Behavior
///
/// - **macOS**: Stores in Keychain, may prompt for permission on first access
/// - **Windows**: Stores in Credential Manager
/// - **Linux**: Stores via Secret Service (requires GNOME Keyring or similar)
#[tauri::command]
pub fn set_api_key(provider: AIProviderType, api_key: String) -> Result<(), String> {
    if provider == AIProviderType::None {
        return Err("Cannot store API key for 'None' provider".to_string());
    }

    if api_key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    let username = provider_to_username(provider);
    let entry = Entry::new(KEYRING_SERVICE, username).map_err(|e| {
        log::error!("Failed to create keyring entry: {}", e);
        format!("Failed to access secure storage: {}", e)
    })?;

    entry.set_password(&api_key).map_err(|e| {
        log::error!("Failed to store API key: {}", e);
        format!("Failed to store API key securely: {}", e)
    })?;

    log::info!("Stored API key for provider: {:?}", provider);
    Ok(())
}

/// Retrieves an API key from the OS keychain.
///
/// # Parameters
///
/// - `provider` - The AI provider type to retrieve the key for
///
/// # Returns
///
/// - `Ok(Some(String))` if the key exists
/// - `Ok(None)` if no key is stored for this provider
/// - `Err(String)` if retrieval fails (e.g., keychain unavailable)
#[tauri::command]
pub fn get_api_key(provider: AIProviderType) -> Result<Option<String>, String> {
    if provider == AIProviderType::None {
        return Ok(None);
    }

    let username = provider_to_username(provider);
    let entry = Entry::new(KEYRING_SERVICE, username).map_err(|e| {
        log::error!("Failed to create keyring entry: {}", e);
        format!("Failed to access secure storage: {}", e)
    })?;

    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => {
            log::error!("Failed to retrieve API key: {}", e);
            Err(format!("Failed to retrieve API key: {}", e))
        }
    }
}

/// Deletes an API key from the OS keychain.
///
/// # Parameters
///
/// - `provider` - The AI provider type to delete the key for
///
/// # Returns
///
/// - `Ok(())` on success (including if no key existed)
/// - `Err(String)` if deletion fails (e.g., keychain unavailable)
#[tauri::command]
pub fn delete_api_key(provider: AIProviderType) -> Result<(), String> {
    if provider == AIProviderType::None {
        return Ok(());
    }

    let username = provider_to_username(provider);
    let entry = Entry::new(KEYRING_SERVICE, username).map_err(|e| {
        log::error!("Failed to create keyring entry: {}", e);
        format!("Failed to access secure storage: {}", e)
    })?;

    match entry.delete_credential() {
        Ok(()) => {
            log::info!("Deleted API key for provider: {:?}", provider);
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            // Not an error - key didn't exist
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to delete API key: {}", e);
            Err(format!("Failed to delete API key: {}", e))
        }
    }
}

/// Checks if an API key exists for a provider without retrieving it.
///
/// # Parameters
///
/// - `provider` - The AI provider type to check
///
/// # Returns
///
/// - `Ok(true)` if a key exists
/// - `Ok(false)` if no key exists
/// - `Err(String)` if the check fails
#[tauri::command]
pub fn has_api_key(provider: AIProviderType) -> Result<bool, String> {
    if provider == AIProviderType::None {
        return Ok(false);
    }

    let username = provider_to_username(provider);
    let entry = Entry::new(KEYRING_SERVICE, username).map_err(|e| {
        log::error!("Failed to create keyring entry: {}", e);
        format!("Failed to access secure storage: {}", e)
    })?;

    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => {
            log::error!("Failed to check API key existence: {}", e);
            Err(format!("Failed to check secure storage: {}", e))
        }
    }
}
