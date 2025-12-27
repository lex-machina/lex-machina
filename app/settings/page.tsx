"use client";

import { useEffect } from "react";
import { Settings as SettingsIcon, Loader2 } from "lucide-react";

import { useSettings } from "@/lib/hooks/use-settings";
import { useTheme } from "@/lib/hooks/use-theme";

import AppShell from "@/components/layout/app-shell";
import { ThemeSelector, AIProviderConfig } from "@/components/settings";

// ============================================================================
// CONSTANTS
// ============================================================================

const APP_VERSION = "v0.1.0";

// ============================================================================
// SETTINGS PAGE
// ============================================================================

/**
 * Settings page - Configure application preferences.
 *
 * Features:
 * - Theme selection (System, Light, Dark)
 * - AI provider configuration (OpenRouter, Gemini)
 * - API key management with validation
 *
 * Layout:
 * - Two-column desktop layout (Appearance | AI Provider)
 * - Minimal version text at bottom
 *
 * Settings are stored in memory (session-only).
 * Theme changes are applied immediately to the DOM.
 */
export default function SettingsPage() {
  // Hooks
  const {
    theme,
    setTheme,
    aiConfig,
    savedProviders,
    configureAIProvider,
    clearAIProvider,
    switchProvider,
    deleteSavedProvider,
    validateAPIKey,
    validationStatus,
    validationError,
    isLoading,
    refresh,
  } = useSettings();

  // Use the theme hook to ensure theme is applied to DOM
  useTheme();

  // Refresh settings when page is visited
  useEffect(() => {
    refresh();
  }, [refresh]);

  return (
    <AppShell
      toolbar={
        <div className="flex items-center gap-2">
          <SettingsIcon className="w-4 h-4" />
          <h1 className="text-sm font-medium">Settings</h1>
        </div>
      }
    >
      <div className="flex-1 flex flex-col p-4 min-h-0">
        {/* Loading state */}
        {isLoading ? (
          <div className="flex-1 flex items-center justify-center">
            <div className="flex items-center gap-3 text-muted-foreground">
              <Loader2 className="w-4 h-4 animate-spin" />
              <span className="text-sm">Loading settings...</span>
            </div>
          </div>
        ) : (
          <>
            {/* Two Column Grid */}
            <div className="flex-1 grid grid-cols-2 gap-6 min-h-0">
              {/* Left Column - Appearance */}
              <div className="flex flex-col min-h-0">
                <div className="border rounded-lg flex flex-col h-full">
                  <div className="px-4 py-3 border-b bg-muted/30">
                    <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      Appearance
                    </h2>
                  </div>
                  <div className="flex-1 p-4 overflow-y-auto">
                    <div className="flex flex-col gap-2 mb-4">
                      <h3 className="text-sm font-medium">Theme</h3>
                      <p className="text-xs text-muted-foreground">
                        Choose how Lex Machina looks on your device
                      </p>
                    </div>
                    <ThemeSelector
                      value={theme}
                      onChange={setTheme}
                    />
                  </div>
                </div>
              </div>

              {/* Right Column - AI Provider */}
              <div className="flex flex-col min-h-0">
                <div className="border rounded-lg flex flex-col h-full">
                  <div className="px-4 py-3 border-b bg-muted/30">
                    <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                      AI Provider
                    </h2>
                  </div>
                  <div className="flex-1 p-4 overflow-y-auto">
                    <AIProviderConfig
                      config={aiConfig}
                      savedProviders={savedProviders}
                      onConfigure={configureAIProvider}
                      onClear={clearAIProvider}
                      onSwitch={switchProvider}
                      onDelete={deleteSavedProvider}
                      onValidate={validateAPIKey}
                      validationStatus={validationStatus}
                      validationError={validationError}
                    />
                  </div>
                </div>
              </div>
            </div>

            {/* Version Footer */}
            <div className="mt-4 pt-3 border-t text-center">
              <p className="text-xs text-muted-foreground">
                Lex Machina {APP_VERSION}
              </p>
            </div>
          </>
        )}
      </div>
    </AppShell>
  );
}
