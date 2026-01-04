"use client";

import { useEffect } from "react";
import { Loader2 } from "lucide-react";

import { useSettings } from "@/lib/hooks/use-settings";
import { useTheme } from "@/lib/hooks/use-theme";
import { useSidebar } from "@/lib/contexts/sidebar-context";

import AppShell from "@/components/layout/app-shell";
import {
    ThemeSelector,
    AIProviderConfig,
    NavPositionSelector,
} from "@/components/settings";

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
 * - Navigation position selection (Merged, Left, Right)
 * - AI provider configuration (OpenRouter, Gemini)
 * - API key management with validation
 *
 * Layout:
 * - Two-column desktop layout (Appearance | AI Provider)
 * - Minimal version text at bottom
 *
 * This page opts out of sidebar content - shows vertical nav only.
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

    const { navBarPosition, requestSetNavBarPosition } = useSidebar();

    // Use the theme hook to ensure theme is applied to DOM
    useTheme();

    // Refresh settings when page is visited
    useEffect(() => {
        refresh();
    }, [refresh]);

    return (
        <AppShell sidebar={false}>
            <div className="flex min-h-0 flex-1 flex-col p-4">
                {/* Loading state */}
                {isLoading ? (
                    <div className="flex flex-1 items-center justify-center">
                        <div className="text-muted-foreground flex items-center gap-3">
                            <Loader2 className="h-4 w-4 animate-spin" />
                            <span className="text-sm">Loading settings...</span>
                        </div>
                    </div>
                ) : (
                    <>
                        {/* Two Column Grid */}
                        <div className="grid min-h-0 flex-1 grid-cols-2 gap-6">
                            {/* Left Column - Appearance */}
                            <div className="flex min-h-0 flex-col">
                                <div className="flex h-full flex-col rounded-lg border">
                                    <div className="bg-muted/30 border-b px-4 py-3">
                                        <h2 className="text-muted-foreground text-xs font-semibold tracking-wider uppercase">
                                            Appearance
                                        </h2>
                                    </div>
                                    <div className="flex-1 overflow-y-auto p-4">
                                        {/* Theme Section */}
                                        <div className="mb-6">
                                            <div className="mb-4 flex flex-col gap-2">
                                                <h3 className="text-sm font-medium">
                                                    Theme
                                                </h3>
                                                <p className="text-muted-foreground text-xs">
                                                    Choose how Lex Machina looks
                                                    on your device
                                                </p>
                                            </div>
                                            <ThemeSelector
                                                value={theme}
                                                onChange={setTheme}
                                            />
                                        </div>

                                        {/* Layout Section */}
                                        <div>
                                            <div className="mb-4 flex flex-col gap-2">
                                                <h3 className="text-sm font-medium">
                                                    Navigation Position
                                                </h3>
                                                <p className="text-muted-foreground text-xs">
                                                    Choose where the navigation
                                                    bar appears
                                                </p>
                                            </div>
                                            <NavPositionSelector
                                                value={navBarPosition}
                                                onChange={
                                                    requestSetNavBarPosition
                                                }
                                            />
                                        </div>
                                    </div>
                                </div>
                            </div>

                            {/* Right Column - AI Provider */}
                            <div className="flex min-h-0 flex-col">
                                <div className="flex h-full flex-col rounded-lg border">
                                    <div className="bg-muted/30 border-b px-4 py-3">
                                        <h2 className="text-muted-foreground text-xs font-semibold tracking-wider uppercase">
                                            AI Provider
                                        </h2>
                                    </div>
                                    <div className="flex-1 overflow-y-auto p-4">
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
                        <div className="mt-4 border-t pt-3 text-center">
                            <p className="text-muted-foreground text-xs">
                                Lex Machina {APP_VERSION}
                            </p>
                        </div>
                    </>
                )}
            </div>
        </AppShell>
    );
}
