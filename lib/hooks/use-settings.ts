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
   * Clears the AI provider configuration.
   * After calling this, preprocessing will use rule-based decisions only.
   */
  clearAIProvider: () => Promise<void>;

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
 * API keys are stored in session memory only (not persisted to disk)
 * for security reasons.
 */
export function useSettings(): UseSettingsReturn {
  // State
  const [theme, setThemeState] = useState<Theme>("system");
  const [aiConfig, setAIConfig] = useState<AIProviderConfig | null>(null);
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
      const [fetchedTheme, fetchedAIConfig] = await Promise.all([
        invoke<Theme>("get_theme"),
        invoke<AIProviderConfig | null>("get_ai_provider_config"),
      ]);

      setThemeState(fetchedTheme);
      setAIConfig(fetchedAIConfig);
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
   * Clears the AI provider configuration.
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
    isLoading,
    validationStatus,
    validationError,

    // Actions
    setTheme,
    configureAIProvider,
    clearAIProvider,
    validateAPIKey,
    refresh,
  };
}
