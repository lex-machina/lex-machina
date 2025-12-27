"use client";

import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRustEvent } from "./use-rust-event";
import {
  RUST_EVENTS,
  type Theme,
  type AIProviderType,
  type AIProviderConfig,
  type ThemeChangedPayload,
} from "@/types";

// ============================================================================
// TYPES
// ============================================================================

/**
 * Validation status for an API key.
 */
export type ValidationStatus =
  | "idle"
  | "validating"
  | "valid"
  | "invalid"
  | "error";

/**
 * State returned by the useSettings hook.
 */
export interface SettingsState {
  /** Current application theme */
  theme: Theme;
  /** Current AI provider configuration (null if not configured) */
  aiConfig: AIProviderConfig | null;
  /** Whether an AI provider is configured */
  hasAIProvider: boolean;
  /** Current AI provider type (for display) */
  aiProviderType: AIProviderType;
  /** List of providers that have saved API keys in the keychain */
  savedProviders: AIProviderType[];
  /** Whether settings are being loaded */
  isLoading: boolean;
  /** Validation status for the current API key */
  validationStatus: ValidationStatus;
  /** Validation error message */
  validationError: string | null;
}

/**
 * Actions returned by the useSettings hook.
 */
export interface SettingsActions {
  /**
   * Sets the application theme.
   *
   * @param theme - The theme to apply ("system", "light", or "dark")
   *
   * @example
   * ```tsx
   * setTheme("dark");
   * ```
   */
  setTheme: (theme: Theme) => Promise<void>;

  /**
   * Configures an AI provider.
   *
   * @param provider - The provider type ("openrouter" or "gemini")
   * @param apiKey - The API key for the provider
   * @returns Promise that resolves on success or rejects with error message
   *
   * @example
   * ```tsx
   * try {
   *   await configureAIProvider("openrouter", "sk-or-xxx");
   *   console.log("AI provider configured!");
   * } catch (err) {
   *   console.error("Failed to configure:", err);
   * }
   * ```
   */
  configureAIProvider: (
    provider: AIProviderType,
    apiKey: string
  ) => Promise<void>;

  /**
   * Clears the active AI provider configuration.
   * The API key remains saved in the keychain for later reactivation.
   * Use `deleteSavedProvider` to permanently remove a saved key.
   */
  clearAIProvider: () => Promise<void>;

  /**
   * Switches to a provider that has a saved API key.
   *
   * @param provider - The provider to switch to (must have a saved key)
   * @returns Promise that resolves on success or rejects if no key is saved
   *
   * @example
   * ```tsx
   * // Switch to a previously saved provider
   * if (savedProviders.includes("gemini")) {
   *   await switchProvider("gemini");
   * }
   * ```
   */
  switchProvider: (provider: AIProviderType) => Promise<void>;

  /**
   * Permanently deletes a saved API key from the keychain.
   *
   * @param provider - The provider whose key should be deleted
   *
   * @example
   * ```tsx
   * // Remove the saved OpenRouter key
   * await deleteSavedProvider("openrouter");
   * ```
   */
  deleteSavedProvider: (provider: AIProviderType) => Promise<void>;

  /**
   * Validates an API key without saving it.
   *
   * @param provider - The provider type to validate against
   * @param apiKey - The API key to validate
   * @returns Promise that resolves to true if valid, false if invalid
   *
   * @example
   * ```tsx
   * const isValid = await validateAPIKey("gemini", "my-api-key");
   * if (isValid) {
   *   await configureAIProvider("gemini", "my-api-key");
   * }
   * ```
   */
  validateAPIKey: (provider: AIProviderType, apiKey: string) => Promise<boolean>;

  /**
   * Refreshes settings from Rust.
   * Call this when navigating to the settings page.
   */
  refresh: () => Promise<void>;
}

/**
 * Return type of the useSettings hook.
 */
export type UseSettingsReturn = SettingsState & SettingsActions;

// ============================================================================
// HOOK IMPLEMENTATION
// ============================================================================

/**
 * Hook for managing application settings.
 *
 * This hook provides access to:
 * - Theme settings (System, Light, Dark)
 * - AI provider configuration (None, OpenRouter, Gemini)
 * - API key validation
 *
 * @returns State and actions for settings management
 *
 * @example
 * ```tsx
 * function SettingsPage() {
 *   const {
 *     theme,
 *     setTheme,
 *     aiConfig,
 *     hasAIProvider,
 *     configureAIProvider,
 *     clearAIProvider,
 *     validateAPIKey,
 *     validationStatus,
 *   } = useSettings();
 *
 *   const handleThemeChange = (newTheme: Theme) => {
 *     setTheme(newTheme);
 *   };
 *
 *   const handleSaveAPIKey = async (apiKey: string) => {
 *     const isValid = await validateAPIKey("openrouter", apiKey);
 *     if (isValid) {
 *       await configureAIProvider("openrouter", apiKey);
 *     }
 *   };
 *
 *   return (
 *     <div>
 *       <ThemeSelector value={theme} onChange={handleThemeChange} />
 *       <AIProviderConfig
 *         config={aiConfig}
 *         onSave={handleSaveAPIKey}
 *         onClear={clearAIProvider}
 *         validationStatus={validationStatus}
 *       />
 *     </div>
 *   );
 * }
 * ```
 *
 * @remarks
 * Following "Rust Supremacy", all settings are stored and managed in Rust.
 * This hook handles IPC communication and local UI state.
 *
 * **Persistence:**
 * - Theme and sidebar width are stored in a JSON settings file
 * - API keys are stored securely in the OS keychain (Keychain on macOS,
 *   Credential Manager on Windows, Secret Service on Linux)
 * - Settings persist across app restarts
 */
export function useSettings(): UseSettingsReturn {
  // State
  const [theme, setThemeState] = useState<Theme>("system");
  const [aiConfig, setAIConfig] = useState<AIProviderConfig | null>(null);
  const [savedProviders, setSavedProviders] = useState<AIProviderType[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [validationStatus, setValidationStatus] =
    useState<ValidationStatus>("idle");
  const [validationError, setValidationError] = useState<string | null>(null);

  // Derived state
  const hasAIProvider = aiConfig !== null && aiConfig.provider !== "none";
  const aiProviderType: AIProviderType = aiConfig?.provider ?? "none";

  // ============================================================================
  // DATA FETCHING
  // ============================================================================

  /**
   * Fetch all settings from Rust.
   */
  const fetchSettings = useCallback(async () => {
    setIsLoading(true);
    try {
      const [fetchedTheme, fetchedAIConfig, fetchedSavedProviders] = await Promise.all([
        invoke<Theme>("get_theme"),
        invoke<AIProviderConfig | null>("get_ai_provider_config"),
        invoke<AIProviderType[]>("get_saved_providers"),
      ]);

      setThemeState(fetchedTheme);
      setAIConfig(fetchedAIConfig);
      setSavedProviders(fetchedSavedProviders);
    } catch (err) {
      console.error("Failed to fetch settings:", err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // ============================================================================
  // ACTIONS
  // ============================================================================

  /**
   * Sets the application theme.
   */
  const setTheme = useCallback(async (newTheme: Theme) => {
    try {
      await invoke("set_theme", { theme: newTheme });
      // State will be updated via event
    } catch (err) {
      console.error("Failed to set theme:", err);
      throw err;
    }
  }, []);

  /**
   * Configures an AI provider.
   */
  const configureAIProvider = useCallback(
    async (provider: AIProviderType, apiKey: string) => {
      if (provider === "none") {
        throw new Error("Use clearAIProvider to remove AI configuration");
      }

      try {
        await invoke("configure_ai_provider", { provider, apiKey });
        // Update local state
        setAIConfig({ provider, api_key: apiKey });
        // Add to saved providers if not already there
        setSavedProviders((prev) =>
          prev.includes(provider) ? prev : [...prev, provider]
        );
        setValidationStatus("valid");
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setValidationError(message);
        throw new Error(message);
      }
    },
    []
  );

  /**
   * Clears the active AI provider configuration.
   * Keys remain saved in the keychain.
   */
  const clearAIProvider = useCallback(async () => {
    try {
      await invoke("clear_ai_provider");
      setAIConfig(null);
      setValidationStatus("idle");
      setValidationError(null);
    } catch (err) {
      console.error("Failed to clear AI provider:", err);
      throw err;
    }
  }, []);

  /**
   * Switches to a provider that has a saved API key.
   */
  const switchProvider = useCallback(async (provider: AIProviderType) => {
    try {
      await invoke("switch_ai_provider", { provider });
      // Refresh to get the new config with masked key
      const newConfig = await invoke<AIProviderConfig | null>("get_ai_provider_config");
      setAIConfig(newConfig);
      setValidationStatus("valid");
      setValidationError(null);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error("Failed to switch provider:", err);
      throw new Error(message);
    }
  }, []);

  /**
   * Permanently deletes a saved API key.
   */
  const deleteSavedProvider = useCallback(async (provider: AIProviderType) => {
    try {
      await invoke("delete_saved_provider", { provider });
      // Remove from saved providers list
      setSavedProviders((prev) => prev.filter((p) => p !== provider));
      // If this was the active provider, clear the config
      setAIConfig((prev) => (prev?.provider === provider ? null : prev));
      setValidationStatus("idle");
      setValidationError(null);
    } catch (err) {
      console.error("Failed to delete saved provider:", err);
      throw err;
    }
  }, []);

  /**
   * Validates an API key without saving it.
   */
  const validateAPIKey = useCallback(
    async (provider: AIProviderType, apiKey: string): Promise<boolean> => {
      setValidationStatus("validating");
      setValidationError(null);

      try {
        const isValid = await invoke<boolean>("validate_ai_api_key", {
          provider,
          apiKey,
        });

        setValidationStatus(isValid ? "valid" : "invalid");
        if (!isValid) {
          setValidationError("API key validation failed");
        }

        return isValid;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setValidationStatus("error");
        setValidationError(message);
        return false;
      }
    },
    []
  );

  /**
   * Refresh settings from Rust.
   */
  const refresh = useCallback(async () => {
    await fetchSettings();
  }, [fetchSettings]);

  // ============================================================================
  // EVENT SUBSCRIPTIONS
  // ============================================================================

  /**
   * Handle theme changed event from Rust.
   */
  const handleThemeChanged = useCallback((newTheme: ThemeChangedPayload) => {
    setThemeState(newTheme);
  }, []);

  useRustEvent<ThemeChangedPayload>(
    RUST_EVENTS.THEME_CHANGED,
    handleThemeChanged
  );

  // ============================================================================
  // INITIALIZATION
  // ============================================================================

  /**
   * Fetch initial settings on mount.
   */
  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  // ============================================================================
  // RETURN
  // ============================================================================

  return {
    // State
    theme,
    aiConfig,
    hasAIProvider,
    aiProviderType,
    savedProviders,
    isLoading,
    validationStatus,
    validationError,

    // Actions
    setTheme,
    configureAIProvider,
    clearAIProvider,
    switchProvider,
    deleteSavedProvider,
    validateAPIKey,
    refresh,
  };
}
