"use client";

/**
 * Sidebar Context - Rendering Cache for Sidebar State
 *
 * This context provides a caching layer for sidebar state from Rust.
 * Following "Rust Supremacy", all state and logic lives in Rust - this
 * context only caches values for rendering and invokes Rust commands.
 *
 * ## Key Principles
 *
 * 1. **Rust is the source of truth** - State is fetched from Rust on mount
 * 2. **No TypeScript logic** - All toggle/width logic happens in Rust
 * 3. **Cache for rendering** - Prevents duplicate fetches across pages
 * 4. **Invoke for mutations** - All state changes go through Rust commands
 *
 * ## State Flow
 *
 * ```
 * User clicks toggle button
 *        │
 *        ▼
 * invoke("toggle_sidebar")  ← TypeScript calls Rust
 *        │
 *        ▼
 * Rust toggles AppState.ui_state.sidebar_collapsed
 *        │
 *        ▼
 * Rust persists via tauri-plugin-store
 *        │
 *        ▼
 * Rust returns new collapsed state (bool)
 *        │
 *        ▼
 * TypeScript updates cached state for re-render
 * ```
 */

import {
    createContext,
    useContext,
    useState,
    useCallback,
    useEffect,
    useMemo,
    type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import type { UIState, NavBarPosition } from "@/types";
import { MIN_HORIZONTAL_NAV_WIDTH } from "@/components/layout/sidebar-nav";

// ============================================================================
// CONSTANTS
// ============================================================================

/** Default sidebar width in pixels */
const DEFAULT_SIDEBAR_WIDTH = 280;

/**
 * Minimum sidebar width in pixels.
 * Uses calculated width from horizontal nav to ensure icons always fit.
 */
const MIN_SIDEBAR_WIDTH = MIN_HORIZONTAL_NAV_WIDTH;

/**
 * Calculate maximum sidebar width as 1/3 of window width.
 * Falls back to 500px if window is not available.
 */
const getMaxSidebarWidth = (): number => {
    if (typeof window === "undefined") return 500;
    return Math.floor(window.innerWidth / 3);
};

// ============================================================================
// TYPES
// ============================================================================

/**
 * Sidebar context value exposed to consumers.
 *
 * Contains cached state from Rust and methods to request state changes.
 * All methods invoke Rust commands - no TypeScript logic.
 */
interface SidebarContextValue {
    // ========================================================================
    // Cached State (read-only for rendering)
    // ========================================================================

    /** Current sidebar width in pixels (from Rust) */
    width: number;

    /** Whether the sidebar is collapsed (from Rust) */
    collapsed: boolean;

    /** Current navigation bar position (from Rust) */
    navBarPosition: NavBarPosition;

    /** Whether the current page opts out of sidebar content */
    isOptOut: boolean;

    /** Whether initial state has been fetched from Rust */
    isInitialized: boolean;

    // ========================================================================
    // Actions (invoke Rust commands - no TS logic)
    // ========================================================================

    /**
     * Toggle the sidebar collapsed state.
     *
     * Calls `invoke("toggle_sidebar")` and updates cached state
     * with the returned value.
     */
    requestToggle: () => Promise<void>;

    /**
     * Set the sidebar width.
     *
     * Calls `invoke("set_sidebar_width")` to persist the width.
     * Updates cached state immediately for smooth UX.
     *
     * @param width - New width in pixels (clamped by Rust if needed)
     */
    requestSetWidth: (width: number) => Promise<void>;

    /**
     * Set the navigation bar position.
     *
     * Calls `invoke("set_nav_bar_position")` to persist the position.
     * Updates cached state on success.
     *
     * @param position - New nav bar position ("left" | "right" | "merged")
     */
    requestSetNavBarPosition: (position: NavBarPosition) => Promise<void>;

    /**
     * Set the opt-out state for the current page.
     *
     * This is a local-only operation (not persisted to Rust).
     * Pages call this on mount to indicate they don't have sidebar content.
     *
     * @param optOut - Whether the current page opts out of sidebar content
     */
    setOptOut: (optOut: boolean) => void;
}

/**
 * Props for the SidebarProvider component.
 */
interface SidebarProviderProps {
    children: ReactNode;
}

// ============================================================================
// CONTEXT
// ============================================================================

/**
 * React Context for sidebar state.
 *
 * Undefined when accessed outside of SidebarProvider.
 */
const SidebarContext = createContext<SidebarContextValue | undefined>(
    undefined,
);

// ============================================================================
// PROVIDER
// ============================================================================

/**
 * Sidebar Provider Component
 *
 * Wraps the application to provide sidebar state to all pages.
 * Fetches initial state from Rust on mount and caches it.
 *
 * @example
 * ```tsx
 * // In app/layout.tsx
 * <SidebarProvider>
 *   <ThemeProvider>{children}</ThemeProvider>
 * </SidebarProvider>
 * ```
 */
export function SidebarProvider({ children }: SidebarProviderProps) {
    // ========================================================================
    // State (cached from Rust)
    // ========================================================================

    const [width, setWidth] = useState(DEFAULT_SIDEBAR_WIDTH);
    const [collapsed, setCollapsed] = useState(false);
    const [navBarPosition, setNavBarPosition] =
        useState<NavBarPosition>("merged");
    const [isOptOut, setIsOptOut] = useState(false);
    const [isInitialized, setIsInitialized] = useState(false);

    // ========================================================================
    // Initial Fetch
    // ========================================================================

    /**
     * Fetch initial state from Rust on mount.
     */
    useEffect(() => {
        const fetchInitialState = async () => {
            try {
                // Fetch UI state (sidebar width and collapsed state)
                const uiState = await invoke<UIState>("get_ui_state");
                setWidth(uiState.sidebar_width);
                setCollapsed(uiState.sidebar_collapsed);

                // Fetch nav bar position separately
                const position = await invoke<NavBarPosition>(
                    "get_nav_bar_position",
                );
                setNavBarPosition(position);

                setIsInitialized(true);
            } catch (err) {
                console.error("Failed to fetch sidebar state from Rust:", err);
                // Use defaults on error, but still mark as initialized
                setIsInitialized(true);
            }
        };

        fetchInitialState();
    }, []);

    // ========================================================================
    // Actions
    // ========================================================================

    /**
     * Toggle sidebar collapsed state via Rust.
     */
    const requestToggle = useCallback(async () => {
        try {
            // Rust toggles and returns the new state
            const newCollapsed = await invoke<boolean>("toggle_sidebar");
            setCollapsed(newCollapsed);
        } catch (err) {
            console.error("Failed to toggle sidebar:", err);
        }
    }, []);

    /**
     * Set sidebar width via Rust.
     */
    const requestSetWidth = useCallback(async (newWidth: number) => {
        // Clamp width on the frontend for immediate feedback
        // Max width is 1/3 of window width
        const maxWidth = getMaxSidebarWidth();
        const clampedWidth = Math.max(
            MIN_SIDEBAR_WIDTH,
            Math.min(maxWidth, newWidth),
        );

        // Update local state immediately for smooth UX during drag
        setWidth(clampedWidth);

        try {
            // Persist to Rust (async, best effort)
            await invoke("set_sidebar_width", { width: clampedWidth });
        } catch (err) {
            console.error("Failed to set sidebar width:", err);
        }
    }, []);

    /**
     * Set nav bar position via Rust.
     */
    const requestSetNavBarPosition = useCallback(
        async (position: NavBarPosition) => {
            // Update local state immediately for responsive UI
            setNavBarPosition(position);

            try {
                // Persist to Rust
                await invoke("set_nav_bar_position", { position });
            } catch (err) {
                console.error("Failed to set nav bar position:", err);
            }
        },
        [],
    );

    /**
     * Set opt-out state (local only, not persisted).
     */
    const setOptOut = useCallback((optOut: boolean) => {
        setIsOptOut(optOut);
    }, []);

    // ========================================================================
    // Context Value
    // ========================================================================

    const contextValue = useMemo<SidebarContextValue>(
        () => ({
            // State
            width,
            collapsed,
            navBarPosition,
            isOptOut,
            isInitialized,

            // Actions
            requestToggle,
            requestSetWidth,
            requestSetNavBarPosition,
            setOptOut,
        }),
        [
            width,
            collapsed,
            navBarPosition,
            isOptOut,
            isInitialized,
            requestToggle,
            requestSetWidth,
            requestSetNavBarPosition,
            setOptOut,
        ],
    );

    return (
        <SidebarContext.Provider value={contextValue}>
            {children}
        </SidebarContext.Provider>
    );
}

// ============================================================================
// HOOK
// ============================================================================

/**
 * Hook to access sidebar context.
 *
 * Must be used within a SidebarProvider.
 *
 * @returns Sidebar context value with cached state and actions
 *
 * @throws Error if used outside of SidebarProvider
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const {
 *     collapsed,
 *     requestToggle,
 *     width,
 *     requestSetWidth,
 *     navBarPosition,
 *     requestSetNavBarPosition,
 *   } = useSidebar();
 *
 *   return (
 *     <div>
 *       <button onClick={requestToggle}>
 *         {collapsed ? "Expand" : "Collapse"}
 *       </button>
 *       <p>Width: {width}px</p>
 *       <p>Nav Position: {navBarPosition}</p>
 *     </div>
 *   );
 * }
 * ```
 */
export function useSidebar(): SidebarContextValue {
    const context = useContext(SidebarContext);

    if (context === undefined) {
        throw new Error("useSidebar must be used within a SidebarProvider");
    }

    return context;
}

// ============================================================================
// EXPORTS
// ============================================================================

export { MIN_SIDEBAR_WIDTH, getMaxSidebarWidth, DEFAULT_SIDEBAR_WIDTH };
