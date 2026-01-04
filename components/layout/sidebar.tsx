"use client";

/**
 * Sidebar Component
 *
 * Context sidebar for page-specific content.
 *
 * ## Modes based on NavBarPosition
 *
 * 1. **Merged mode** (default): Navigation integrated at top of sidebar
 *    - Expanded: Horizontal nav row + content + resize handle
 *    - Collapsed: Vertical nav strip only
 *
 * 2. **Left/Right mode**: Navigation rendered separately by AppShell
 *    - Sidebar shows content only (no nav)
 *    - Has resize handle
 *    - When collapsed or no content, sidebar is hidden entirely
 *
 * ## State Management (Rust Supremacy)
 *
 * All state is managed in Rust. This component:
 * - Reads cached state from SidebarContext
 * - Invokes Rust commands for state changes
 * - Does NO logic - only rendering
 *
 * ## Layouts
 *
 * ### Merged - Expanded (collapsed=false, has content)
 * ```
 * ┌─────────────────────┐
 * │ 🏠 📊 ⚙️ 📈 🧠 ⚙️    │ ← Horizontal nav
 * ├─────────────────────┤
 * │                     │
 * │   Page sidebar      │
 * │   content           │
 * │                     │
 * └─────────────────────┘
 * ```
 *
 * ### Merged - Collapsed (collapsed=true OR no content)
 * ```
 * ┌────┐
 * │ 🏠 │
 * │ 📊 │
 * │ ⚙️ │ ← Vertical nav
 * │ 📈 │
 * │ 🧠 │
 * │ ⚙️ │
 * └────┘
 * ```
 *
 * ### Left/Right - Expanded (has content, not collapsed)
 * ```
 * ┌─────────────────────┐
 * │                     │
 * │   Page sidebar      │ ← Content only (no nav)
 * │   content           │
 * │                     │
 * └─────────────────────┘
 * ```
 *
 * ### Left/Right - Collapsed or no content
 * ```
 * (nothing rendered - nav is separate)
 * ```
 */

import { useCallback, useEffect, useRef, type ReactNode } from "react";
import {
    useSidebar,
    DEFAULT_SIDEBAR_WIDTH,
} from "@/lib/contexts/sidebar-context";
import {
    SidebarNav,
    NAV_STRIP_WIDTH,
    MIN_HORIZONTAL_NAV_WIDTH,
} from "./sidebar-nav";
import ResizeHandle from "@/components/ui/resize-handle";

// ============================================================================
// TYPES
// ============================================================================

interface SidebarProps {
    /**
     * Page-specific sidebar content.
     * If undefined or false, sidebar shows vertical nav only (opt-out mode).
     */
    children?: ReactNode | false;

    /**
     * Whether navigation is rendered separately (left/right mode).
     * When true, sidebar shows content only - no nav icons.
     * @default false
     */
    navSeparate?: boolean;
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

/**
 * Sidebar Component
 *
 * Unified context sidebar that handles content display.
 * Navigation rendering depends on navSeparate prop.
 *
 * @example
 * ```tsx
 * // Merged mode (default) - nav integrated
 * <Sidebar>
 *   <FileInfoPanel />
 * </Sidebar>
 *
 * // Left/Right mode - nav rendered separately
 * <Sidebar navSeparate>
 *   <FileInfoPanel />
 * </Sidebar>
 *
 * // Opt-out (Settings page)
 * <Sidebar>{false}</Sidebar>
 * ```
 */
export function Sidebar({ children, navSeparate = false }: SidebarProps) {
    const { width, collapsed, isInitialized, requestSetWidth } = useSidebar();

    // Track width during drag for smooth resize
    const widthRef = useRef(width);
    useEffect(() => {
        widthRef.current = width;
    }, [width]);

    // ========================================================================
    // Computed State
    // ========================================================================

    /**
     * Whether this page has sidebar content.
     * If children is false or undefined, page opts out.
     */
    const hasContent = children !== false && children !== undefined;

    /**
     * Whether to show expanded mode.
     * Expanded = not collapsed AND has content.
     */
    const isExpanded = !collapsed && hasContent;

    // ========================================================================
    // Resize Handlers
    // ========================================================================

    /**
     * Handle resize drag.
     * Delta is positive when moving right, negative when moving left.
     * For right sidebar, moving left (negative delta) should increase width.
     */
    const handleResize = useCallback(
        (delta: number) => {
            const newWidth = widthRef.current - delta;
            requestSetWidth(newWidth);
        },
        [requestSetWidth],
    );

    /**
     * Handle double-click on resize handle.
     * Resets width to default (280px).
     */
    const handleDoubleClick = useCallback(() => {
        requestSetWidth(DEFAULT_SIDEBAR_WIDTH);
    }, [requestSetWidth]);

    // ========================================================================
    // Render
    // ========================================================================

    // Don't render until state is initialized from Rust
    if (!isInitialized) {
        return null;
    }

    // ========================================================================
    // LEFT/RIGHT MODE (nav rendered separately by AppShell)
    // ========================================================================

    if (navSeparate) {
        // In left/right mode, when collapsed or no content, render nothing
        // (the nav bar is rendered separately by AppShell)
        if (!isExpanded) {
            return null;
        }

        // Expanded: show content only (no nav)
        return (
            <>
                {/* Resize handle on the left edge */}
                <div onDoubleClick={handleDoubleClick}>
                    <ResizeHandle
                        direction="horizontal"
                        onResize={handleResize}
                    />
                </div>

                {/* Sidebar container - content only */}
                <aside
                    className="bg-background flex h-full shrink-0 flex-col overflow-hidden border-l"
                    style={{ width }}
                >
                    {/* Page-specific content */}
                    <div className="flex-1 overflow-y-auto">{children}</div>
                </aside>
            </>
        );
    }

    // ========================================================================
    // MERGED MODE (default) - nav integrated into sidebar
    // ========================================================================

    // ========================================================================
    // COLLAPSED / OPT-OUT MODE: Vertical nav strip only
    // ========================================================================

    if (!isExpanded) {
        return <SidebarNav orientation="vertical" side="right" />;
    }

    // ========================================================================
    // EXPANDED MODE: Resize handle + horizontal nav + content
    // ========================================================================

    return (
        <>
            {/* Resize handle on the left edge */}
            <div onDoubleClick={handleDoubleClick}>
                <ResizeHandle direction="horizontal" onResize={handleResize} />
            </div>

            {/* Sidebar container */}
            <aside
                className="bg-background flex h-full shrink-0 flex-col overflow-hidden border-l"
                style={{ width, minWidth: MIN_HORIZONTAL_NAV_WIDTH }}
            >
                {/* Horizontal nav at top */}
                <SidebarNav orientation="horizontal" />

                {/* Page-specific content */}
                <div className="flex-1 overflow-y-auto">{children}</div>
            </aside>
        </>
    );
}

export default Sidebar;
export { NAV_STRIP_WIDTH };
