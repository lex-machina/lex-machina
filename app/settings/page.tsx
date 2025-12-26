"use client";

import { useEffect } from "react";

import { useSettings } from "@/lib/hooks/use-settings";
import { useTheme } from "@/lib/hooks/use-theme";

import AppShell from "@/components/layout/app-shell";
import { ThemeSelector, AIProviderConfig } from "@/components/settings";

// ============================================================================
// SECTION COMPONENT
// ============================================================================

interface SettingsSectionProps {
  title: string;
  description: string;
  children: React.ReactNode;
}

function SettingsSection({ title, description, children }: SettingsSectionProps) {
  return (
    <section className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <h2 className="text-base font-semibold">{title}</h2>
        <p className="text-sm text-muted-foreground">{description}</p>
      </div>
      <div className="border border-border rounded-lg p-4">
        {children}
      </div>
    </section>
  );
}

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
 * Settings are stored in memory (session-only).
 * Theme changes are applied immediately to the DOM.
 */
export default function SettingsPage() {
  // Hooks
  const {
    theme,
    setTheme,
    aiConfig,
    configureAIProvider,
    clearAIProvider,
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
          <h1 className="text-sm font-medium">Settings</h1>
        </div>
      }
    >
      <div className="flex flex-col gap-8 p-6 max-w-2xl">
        {/* Loading state */}
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <div className="flex items-center gap-3 text-muted-foreground">
              <LoadingSpinner />
              <span className="text-sm">Loading settings...</span>
            </div>
          </div>
        ) : (
          <>
            {/* Appearance Section */}
            <SettingsSection
              title="Appearance"
              description="Customize how Lex Machina looks on your device."
            >
              <ThemeSelector
                value={theme}
                onChange={setTheme}
              />
            </SettingsSection>

            {/* AI Provider Section */}
            <SettingsSection
              title="AI Provider"
              description="Configure an AI provider for intelligent preprocessing decisions. API keys are stored in memory only and cleared when the application closes."
            >
              <AIProviderConfig
                config={aiConfig}
                onConfigure={configureAIProvider}
                onClear={clearAIProvider}
                onValidate={validateAPIKey}
                validationStatus={validationStatus}
                validationError={validationError}
              />
            </SettingsSection>

            {/* About Section */}
            <SettingsSection
              title="About"
              description="Information about this application."
            >
              <div className="flex flex-col gap-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Application</span>
                  <span className="text-sm font-medium">Lex Machina</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Version</span>
                  <span className="text-sm font-mono">0.1.0</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Platform</span>
                  <span className="text-sm font-mono">Desktop (Tauri)</span>
                </div>
                <hr className="border-border my-2" />
                <p className="text-xs text-muted-foreground">
                  Lex Machina is a local-first AutoML application that democratizes 
                  data analytics for SMEs, non-profits, and non-technical individuals.
                  All data processing happens locally on your machine.
                </p>
              </div>
            </SettingsSection>
          </>
        )}
      </div>
    </AppShell>
  );
}

// ============================================================================
// LOADING SPINNER
// ============================================================================

function LoadingSpinner() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="animate-spin"
    >
      <path d="M21 12a9 9 0 1 1-6.219-8.56" />
    </svg>
  );
}
