"use client";

/**
 * Toolbar Component
 *
 * Top toolbar with app branding and sidebar toggle button.
 *
 * ## Layout
 *
 * ```
 * ┌────────────────────────────────────────────────────────┐
 * │ [Lex Machina]                              [≡ toggle]  │
 * └────────────────────────────────────────────────────────┘
 * ```
 *
 * - **Left:** "Lex Machina" branding
 * - **Right:** Sidebar toggle button (hidden on opt-out pages)
 *
 * ## Toggle Button Behavior
 *
 * - Visible when page has sidebar content
 * - Hidden on opt-out pages (e.g., Settings)
 * - Calls Rust `toggle_sidebar` command via context
 * - Sized and positioned to align with nav bar icons
 */

import { PanelRight } from "lucide-react";
import { useSidebar } from "@/lib/contexts/sidebar-context";
import { cn } from "@/lib/utils";
import { NAV_STRIP_WIDTH } from "@/components/layout/sidebar-nav";

/** Toggle button size matches nav icons (h-10 w-10 = 40px) */
const TOGGLE_BUTTON_SIZE = 40;

/** Padding to center button within nav strip width */
const TOGGLE_BUTTON_PADDING = (NAV_STRIP_WIDTH - TOGGLE_BUTTON_SIZE) / 2;

interface ToolbarProps {
    /**
     * Whether to show the sidebar toggle button.
     * Set to false for pages that opt-out of sidebar content.
     * @default true
     */
    showToggle?: boolean;
}

/**
 * Top toolbar component with app title and sidebar toggle.
 *
 * @example
 * ```tsx
 * // Default - shows toggle button
 * <Toolbar />
 *
 * // Opt-out page - hide toggle button
 * <Toolbar showToggle={false} />
 * ```
 */
const Toolbar = ({ showToggle = true }: ToolbarProps) => {
    const { collapsed, requestToggle, isInitialized } = useSidebar();

    return (
        <header className="bg-background flex h-12 items-center justify-between border-b pl-5">
            {/* Left: App branding */}
            <h1 className="text-muted-foreground text-lg font-bold">
                Lex Machina
            </h1>

            {/* Right: Sidebar toggle button - aligned with nav bar */}
            {showToggle && isInitialized && (
                <button
                    onClick={requestToggle}
                    title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
                    aria-label={
                        collapsed ? "Expand sidebar" : "Collapse sidebar"
                    }
                    className={cn(
                        "flex h-10 w-10 items-center justify-center rounded-lg",
                        "transition-colors duration-150",
                        "text-muted-foreground hover:bg-muted hover:text-foreground",
                    )}
                    style={{ marginRight: TOGGLE_BUTTON_PADDING }}
                >
                    <PanelRight size={20} />
                </button>
            )}
        </header>
    );
};

export default Toolbar;
