"use client";

/**
 * AppShell Component
 *
 * Application shell providing the main layout structure.
 * This is a pure layout component with no business logic or state.
 *
 * ## Layout Modes (based on navBarPosition)
 *
 * ### 1. Merged Mode (default)
 * Navigation integrated into right sidebar:
 * ```
 * ┌──────────────────────────────────────────────────────────────────┐
 * │ [Lex Machina]                                      [≡ toggle]    │ ← Toolbar
 * ├─────────────────────────────────────────────┬────────────────────┤
 * │                                             │ 🏠 📊 ⚙️ 📈 🧠 ⚙️    │ ← Horizontal nav (expanded)
 * │                                             ├────────────────────┤
 * │              Main Content                   │   Sidebar content  │
 * │              (children)                     │                    │
 * ├─────────────────────────────────────────────┴────────────────────┤
 * │                           StatusBar                              │
 * └──────────────────────────────────────────────────────────────────┘
 * ```
 *
 * ### 2. Left Mode
 * Separate navigation bar on left side:
 * ```
 * ┌──────────────────────────────────────────────────────────────────┐
 * │ [Lex Machina]                                      [≡ toggle]    │ ← Toolbar
 * ├────┬────────────────────────────────────────┬────────────────────┤
 * │ 🏠 │                                        │                    │
 * │ 📊 │                                        │   Sidebar content  │
 * │ ⚙️ │          Main Content                  │                    │
 * │ 📈 │          (children)                    │                    │
 * │ 🧠 │                                        │                    │
 * │ ⚙️ │                                        │                    │
 * ├────┴────────────────────────────────────────┴────────────────────┤
 * │                           StatusBar                              │
 * └──────────────────────────────────────────────────────────────────┘
 * ```
 *
 * ### 3. Right Mode
 * Separate navigation bar on right side (before sidebar):
 * ```
 * ┌──────────────────────────────────────────────────────────────────┐
 * │ [Lex Machina]                                      [≡ toggle]    │ ← Toolbar
 * ├─────────────────────────────────────────────┬────┬───────────────┤
 * │                                             │ 🏠 │               │
 * │                                             │ 📊 │ Sidebar       │
 * │              Main Content                   │ ⚙️ │ content       │
 * │              (children)                     │ 📈 │               │
 * │                                             │ 🧠 │               │
 * │                                             │ ⚙️ │               │
 * ├─────────────────────────────────────────────┴────┴───────────────┤
 * │                           StatusBar                              │
 * └──────────────────────────────────────────────────────────────────┘
 * ```
 */

import type { ReactNode } from "react";

import Toolbar from "@/components/layout/toolbar";
import StatusBar from "@/components/layout/status-bar";
import { Sidebar } from "@/components/layout/sidebar";
import { SidebarNav } from "@/components/layout/sidebar-nav";
import ToastContainer from "@/components/ui/toast";
import { useSidebar } from "@/lib/contexts/sidebar-context";

interface AppShellProps {
    /**
     * Content for the right sidebar.
     * - `ReactNode`: Sidebar content (expandable sidebar with toggle)
     * - `false`: Opt-out of sidebar (vertical nav only, no toggle button)
     * - `undefined`: Same as false
     */
    sidebar?: ReactNode | false;
    /** Main content area */
    children: ReactNode;
}

/**
 * Application shell providing the main layout structure.
 *
 * @example
 * ```tsx
 * // Page WITH sidebar content
 * export default function DataPage() {
 *   return (
 *     <AppShell sidebar={<DataSidebar />}>
 *       <DataGrid />
 *     </AppShell>
 *   );
 * }
 *
 * // Page WITHOUT sidebar (opt-out)
 * export default function SettingsPage() {
 *   return (
 *     <AppShell sidebar={false}>
 *       <SettingsGrid />
 *     </AppShell>
 *   );
 * }
 * ```
 */
const AppShell = ({ sidebar, children }: AppShellProps) => {
    const { navBarPosition, isInitialized } = useSidebar();

    /**
     * Whether this page has sidebar content.
     * Used to show/hide toggle button in toolbar.
     */
    const hasSidebarContent = sidebar !== false && sidebar !== undefined;

    // Don't render layout until sidebar state is initialized
    if (!isInitialized) {
        return (
            <div className="bg-background flex h-screen w-screen items-center justify-center">
                <div className="text-muted-foreground text-sm">Loading...</div>
            </div>
        );
    }

    // ========================================================================
    // MERGED MODE (default) - Navigation integrated into sidebar
    // ========================================================================

    if (navBarPosition === "merged") {
        return (
            <div className="bg-background flex h-screen w-screen flex-col overflow-hidden">
                {/* Top toolbar with toggle button (hidden on opt-out pages) */}
                <Toolbar showToggle={hasSidebarContent} />

                {/* Main area: Content + Sidebar */}
                <div className="flex flex-1 overflow-hidden">
                    {/* Main content area */}
                    <main className="flex flex-1 overflow-hidden">
                        {children}
                    </main>

                    {/* Right sidebar (unified nav + content) */}
                    <Sidebar>{sidebar}</Sidebar>
                </div>

                {/* Bottom status bar */}
                <StatusBar />

                {/* Toast notifications */}
                <ToastContainer />
            </div>
        );
    }

    // ========================================================================
    // LEFT MODE - Separate nav bar on left side
    // ========================================================================

    if (navBarPosition === "left") {
        return (
            <div className="bg-background flex h-screen w-screen flex-col overflow-hidden">
                {/* Top toolbar with toggle button (hidden on opt-out pages) */}
                <Toolbar showToggle={hasSidebarContent} />

                {/* Main area: NavBar (left) + Content + Sidebar (right) */}
                <div className="flex flex-1 overflow-hidden">
                    {/* Left navigation bar */}
                    <SidebarNav orientation="vertical" side="left" />

                    {/* Main content area */}
                    <main className="flex flex-1 overflow-hidden">
                        {children}
                    </main>

                    {/* Right sidebar (content only, no nav) */}
                    <Sidebar navSeparate>{sidebar}</Sidebar>
                </div>

                {/* Bottom status bar */}
                <StatusBar />

                {/* Toast notifications */}
                <ToastContainer />
            </div>
        );
    }

    // ========================================================================
    // RIGHT MODE - Separate nav bar on right edge, sidebar to its left
    // ========================================================================

    return (
        <div className="bg-background flex h-screen w-screen flex-col overflow-hidden">
            {/* Top toolbar with toggle button (hidden on opt-out pages) */}
            <Toolbar showToggle={hasSidebarContent} />

            {/* Main area: Content + Sidebar + NavBar (right edge) */}
            <div className="flex flex-1 overflow-hidden">
                {/* Main content area */}
                <main className="flex flex-1 overflow-hidden">{children}</main>

                {/* Right sidebar (content only, no nav) */}
                <Sidebar navSeparate>{sidebar}</Sidebar>

                {/* Right navigation bar (far right edge) */}
                <SidebarNav orientation="vertical" side="right" />
            </div>

            {/* Bottom status bar */}
            <StatusBar />

            {/* Toast notifications */}
            <ToastContainer />
        </div>
    );
};

export default AppShell;
