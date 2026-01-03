"use client";

import type { ReactNode } from "react";

import NavSidebar from "@/components/layout/nav-sidebar";
import Toolbar from "@/components/layout/toolbar";
import StatusBar from "@/components/layout/status-bar";
import ToastContainer from "@/components/ui/toast";

interface AppShellProps {
    /** Content for the toolbar (left side buttons) */
    toolbar?: ReactNode;
    /** Content for the right context sidebar */
    sidebar?: ReactNode;
    /** Main content area */
    children: ReactNode;
}

/**
 * Application shell providing the main layout structure.
 *
 * This is a pure layout component with no business logic or state.
 * It provides slots for:
 * - Toolbar: Page-specific action buttons
 * - Sidebar: Page-specific context panel (right side)
 * - Children: Main content area
 *
 * Fixed elements (always present):
 * - NavSidebar: Left navigation (subscribes to events internally)
 * - StatusBar: Bottom status (subscribes to events internally)
 * - ToastContainer: Toast notifications
 *
 * Layout structure:
 * ```
 * ┌──────────────────────────────────────────────────────┐
 * │                      Toolbar                         │
 * ├────┬─────────────────────────────────────────┬───────┤
 * │    │                                         │       │
 * │ N  │              Main Content               │ Side  │
 * │ a  │               (children)                │ bar   │
 * │ v  │                                         │       │
 * │    │                                         │       │
 * ├────┴─────────────────────────────────────────┴───────┤
 * │                     StatusBar                        │
 * └──────────────────────────────────────────────────────┘
 * ```
 *
 * @example
 * ```tsx
 * // In app/page.tsx or app/data/page.tsx
 * export default function DataPage() {
 *   return (
 *     <AppShell
 *       toolbar={<DataToolbar />}
 *       sidebar={<DataSidebar />}
 *     >
 *       <DataGrid />
 *     </AppShell>
 *   );
 * }
 * ```
 */
const AppShell = ({ toolbar, sidebar, children }: AppShellProps) => {
    return (
        <div className="bg-background flex h-screen w-screen flex-col overflow-hidden">
            {/* Top toolbar with page-specific content */}
            <Toolbar>{toolbar}</Toolbar>

            {/* Main area: NavSidebar + Content + ContextSidebar */}
            <div className="flex flex-1 overflow-hidden">
                {/* Left navigation - always visible */}
                <NavSidebar />

                {/* Main content area */}
                <main className="flex flex-1 overflow-hidden">{children}</main>

                {/* Right context sidebar - page-specific */}
                {sidebar}
            </div>

            {/* Bottom status bar - subscribes to events internally */}
            <StatusBar />

            {/* Toast notifications */}
            <ToastContainer />
        </div>
    );
};

export default AppShell;
