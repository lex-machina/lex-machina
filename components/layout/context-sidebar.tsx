"use client";

import {
    useState,
    useCallback,
    useEffect,
    useRef,
    type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import ResizeHandle from "@/components/ui/resize-handle";
import type { UIState } from "@/types";

interface ContextSidebarProps {
    /** Content to render inside the sidebar */
    children?: ReactNode;
    /** Initial width in pixels (default: 280) */
    defaultWidth?: number;
    /** Minimum width in pixels (default: 200) */
    minWidth?: number;
    /** Maximum width in pixels (default: 500) */
    maxWidth?: number;
    /** Whether to show the sidebar (default: true) */
    visible?: boolean;
}

const DEFAULT_WIDTH = 280;
const MIN_WIDTH = 200;
const MAX_WIDTH = 500;

/**
 * Right sidebar wrapper component with resizable width.
 *
 * This component provides the layout structure for the right sidebar
 * and handles resize logic. Page-specific content is passed as children.
 *
 * Features:
 * - Resizable width with drag handle
 * - Width persisted to Rust backend
 * - Min/max width constraints
 * - Can be hidden via `visible` prop
 *
 * @example
 * ```tsx
 * // In a page component
 * <ContextSidebar>
 *   <FileInfoPanel />
 *   <ColumnListPanel />
 * </ContextSidebar>
 * ```
 */
const ContextSidebar = ({
    children,
    defaultWidth = DEFAULT_WIDTH,
    minWidth = MIN_WIDTH,
    maxWidth = MAX_WIDTH,
    visible = true,
}: ContextSidebarProps) => {
    // Start with null to avoid flash of default width
    const [width, setWidth] = useState<number | null>(null);
    const [isInitialized, setIsInitialized] = useState(false);

    // Ref to track latest width for use in callbacks (avoids stale closure)
    const widthRef = useRef(defaultWidth);
    useEffect(() => {
        if (width !== null) {
            widthRef.current = width;
        }
    }, [width]);

    // Restore sidebar width from Rust on mount
    useEffect(() => {
        const restoreWidth = async () => {
            try {
                const uiState = await invoke<UIState>("get_ui_state");
                if (uiState.sidebar_width > 0) {
                    // Clamp to min/max constraints
                    const restoredWidth = Math.max(
                        minWidth,
                        Math.min(maxWidth, uiState.sidebar_width),
                    );
                    setWidth(restoredWidth);
                    widthRef.current = restoredWidth;
                } else {
                    setWidth(defaultWidth);
                }
            } catch (err) {
                console.error("Failed to restore sidebar width:", err);
                setWidth(defaultWidth);
            } finally {
                setIsInitialized(true);
            }
        };
        restoreWidth();
    }, [minWidth, maxWidth, defaultWidth]);

    const handleResize = useCallback(
        (delta: number) => {
            setWidth((prev) => {
                const currentWidth = prev ?? defaultWidth;
                // Delta is positive when moving right, negative when moving left
                // For right sidebar, moving left (negative delta) should increase width
                const newWidth = Math.max(
                    minWidth,
                    Math.min(maxWidth, currentWidth - delta),
                );
                return newWidth;
            });
        },
        [minWidth, maxWidth, defaultWidth],
    );

    const handleResizeEnd = useCallback(async () => {
        const currentWidth = widthRef.current;
        try {
            await invoke("set_sidebar_width", { width: currentWidth });
        } catch (err) {
            console.error("Failed to persist sidebar width:", err);
        }
    }, []);

    // Don't render until we've restored width from Rust (avoids flash of default width)
    if (!visible || !isInitialized) {
        return null;
    }

    // At this point width is guaranteed to be a number
    const currentWidth = width ?? defaultWidth;

    return (
        <>
            <ResizeHandle
                direction="horizontal"
                onResize={handleResize}
                onResizeEnd={handleResizeEnd}
            />
            <aside
                style={{ width: currentWidth }}
                className="bg-background shrink-0 overflow-y-auto border-l"
            >
                {children ? (
                    children
                ) : (
                    <div className="p-5">
                        <p className="text-muted-foreground text-sm">
                            No content to display
                        </p>
                    </div>
                )}
            </aside>
        </>
    );
};

export default ContextSidebar;
