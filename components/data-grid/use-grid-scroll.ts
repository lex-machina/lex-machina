"use client";

import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { GridScrollPosition } from "@/types";

/**
 * Constants for scroll behavior.
 */
const ROW_HEIGHT = 32;
const SCROLLBAR_SIZE = 10;
const PIXELS_PER_HORIZONTAL_SCROLL = 20;
const SCROLL_PERSIST_DEBOUNCE_MS = 200;

/**
 * Configuration for the scroll hook.
 */
export interface GridScrollConfig {
    /** Total number of rows in the dataset */
    totalRows: number;
    /** Array of column widths */
    columnWidths: number[];
    /** Callback when visible rows change (for data fetching) */
    onVisibleRangeChange?: (
        currentRowIndex: number,
        visibleRowCount: number,
    ) => void;
}

/**
 * State returned by the useGridScroll hook.
 */
export interface GridScrollState {
    /** Current top row index (vertical scroll position) */
    currentRowIndex: number;
    /** Current horizontal scroll offset in pixels */
    scrollLeft: number;
    /** Height of the visible viewport in pixels */
    viewportHeight: number;
    /** Width of the visible viewport in pixels */
    viewportWidth: number;
    /** Full container height (including scrollbars) */
    containerHeight: number;
    /** Number of rows visible in the viewport */
    visibleRowCount: number;
    /** Maximum row index for scrolling */
    maxRowIndex: number;
    /** Maximum horizontal scroll position */
    maxScrollLeft: number;
    /** Total width of all columns */
    totalWidth: number;
    /** Whether horizontal scrollbar should be shown */
    showHorizontalScrollbar: boolean;
    /** Height available for vertical scrollbar */
    verticalScrollbarHeight: number;
    /** Handler for vertical scrollbar seek */
    handleVerticalSeek: (rowIndex: number) => void;
    /** Handler for horizontal scrollbar seek */
    handleHorizontalSeek: (position: number) => void;
    /** Handler for wheel events */
    handleWheel: (e: React.WheelEvent) => void;
    /** Ref callback for the container element */
    containerRef: (node: HTMLDivElement | null) => void;
    /** Setter for viewport height (called by GridBody) */
    setViewportHeight: (height: number) => void;
}

/**
 * Hook for managing DataGrid scroll state.
 *
 * This hook:
 * - Manages vertical scroll position (row index)
 * - Manages horizontal scroll position (pixel offset)
 * - Observes container size for responsive behavior
 * - Handles wheel events for smooth scrolling
 * - Notifies parent when visible range changes (for data fetching)
 * - Persists scroll position to Rust (debounced)
 * - Restores scroll position from Rust on mount
 *
 * Following "Rust Supremacy":
 * - Scroll position is persisted to Rust for session restoration
 * - Local state provides instant feedback during scrolling
 * - Debounced sync to Rust avoids excessive IPC calls
 *
 * @param config - Scroll configuration
 * @returns Scroll state and handlers
 *
 * @example
 * ```tsx
 * const {
 *   currentRowIndex,
 *   scrollLeft,
 *   handleWheel,
 *   containerRef,
 *   ...
 * } = useGridScroll({
 *   totalRows: 1000,
 *   columnWidths: [100, 150, 200],
 *   onVisibleRangeChange: fetchRows,
 * });
 * ```
 */
export function useGridScroll(config: GridScrollConfig): GridScrollState {
    const { totalRows, columnWidths, onVisibleRangeChange } = config;

    // Scroll position state
    const [currentRowIndex, setCurrentRowIndex] = useState(0);
    const [scrollLeft, setScrollLeft] = useState(0);

    // Viewport dimensions
    const [viewportHeight, setViewportHeight] = useState(0);
    const [viewportWidth, setViewportWidth] = useState(0);
    const [containerHeight, setContainerHeight] = useState(0);

    // Refs
    const containerNodeRef = useRef<HTMLDivElement | null>(null);
    const accumulatedHorizontalDelta = useRef(0);
    const resizeObserverRef = useRef<ResizeObserver | null>(null);
    const persistTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(
        null,
    );
    const initializedRef = useRef(false);

    // Derived values
    const totalWidth = columnWidths.reduce((sum, w) => sum + w, 0);
    const visibleRowCount = Math.ceil(viewportHeight / ROW_HEIGHT);
    const maxRowIndex = Math.max(0, totalRows - visibleRowCount);
    const maxScrollLeft = Math.max(
        0,
        totalWidth - viewportWidth + SCROLLBAR_SIZE,
    );
    const showHorizontalScrollbar = totalWidth > viewportWidth;
    const verticalScrollbarHeight = showHorizontalScrollbar
        ? containerHeight - SCROLLBAR_SIZE
        : containerHeight;

    // Clamp helper
    const clamp = useCallback(
        (value: number, min: number, max: number) =>
            Math.max(min, Math.min(max, value)),
        [],
    );

    // Persist scroll position to Rust (debounced)
    const persistScrollPosition = useCallback(
        (rowIndex: number, scrollLeftPos: number) => {
            // Clear any pending persist
            if (persistTimeoutRef.current) {
                clearTimeout(persistTimeoutRef.current);
            }

            // Debounce the persist call
            persistTimeoutRef.current = setTimeout(async () => {
                try {
                    await invoke("set_grid_scroll", {
                        rowIndex,
                        scrollLeft: scrollLeftPos,
                    });
                } catch (err) {
                    console.error("Failed to persist scroll position:", err);
                }
            }, SCROLL_PERSIST_DEBOUNCE_MS);
        },
        [],
    );

    // Restore scroll position from Rust on mount
    useEffect(() => {
        if (initializedRef.current || totalRows === 0) return;

        const restoreScrollPosition = async () => {
            try {
                const pos = await invoke<GridScrollPosition>("get_grid_scroll");
                // Only restore if we have data and position is valid
                if (pos.row_index > 0 || pos.scroll_left > 0) {
                    setCurrentRowIndex(Math.min(pos.row_index, maxRowIndex));
                    setScrollLeft(pos.scroll_left);
                }
                initializedRef.current = true;
            } catch (err) {
                console.error("Failed to restore scroll position:", err);
                initializedRef.current = true;
            }
        };

        restoreScrollPosition();
    }, [totalRows, maxRowIndex]);

    // Reset scroll position when file changes (totalRows goes to 0 then back up)
    // Using a ref to track the previous state to avoid the lint warning
    // about setState in effects. This is necessary because we need to reset
    // scroll state when the file is closed or when currentRowIndex exceeds max.
    const prevTotalRowsRef = useRef(totalRows);
    useEffect(() => {
        const wasTotalRows = prevTotalRowsRef.current;
        prevTotalRowsRef.current = totalRows;

        // Reset if totalRows went to 0
        if (wasTotalRows > 0 && totalRows === 0) {
            // eslint-disable-next-line react-hooks/set-state-in-effect -- Necessary to reset scroll when file closes
            setCurrentRowIndex(0);
            setScrollLeft(0);
            initializedRef.current = false; // Allow restore on next file
        } else if (totalRows > 0 && currentRowIndex > maxRowIndex) {
            // Clamp currentRowIndex if it exceeds the new max
            setCurrentRowIndex(maxRowIndex);
        }
    }, [totalRows, maxRowIndex, currentRowIndex]);

    // Notify parent when visible range changes
    useEffect(() => {
        if (onVisibleRangeChange && visibleRowCount > 0) {
            onVisibleRangeChange(currentRowIndex, visibleRowCount);
        }
    }, [currentRowIndex, visibleRowCount, onVisibleRangeChange]);

    // Persist scroll position when it changes
    useEffect(() => {
        if (initializedRef.current && totalRows > 0) {
            persistScrollPosition(currentRowIndex, scrollLeft);
        }
    }, [currentRowIndex, scrollLeft, totalRows, persistScrollPosition]);

    // Cleanup timeout on unmount
    useEffect(() => {
        return () => {
            if (persistTimeoutRef.current) {
                clearTimeout(persistTimeoutRef.current);
            }
        };
    }, []);

    // Handle vertical scrollbar seek
    const handleVerticalSeek = useCallback(
        (rowIndex: number) => {
            setCurrentRowIndex(clamp(rowIndex, 0, maxRowIndex));
        },
        [clamp, maxRowIndex],
    );

    // Handle horizontal scrollbar seek
    const handleHorizontalSeek = useCallback(
        (position: number) => {
            setScrollLeft(clamp(position, 0, maxScrollLeft));
        },
        [clamp, maxScrollLeft],
    );

    // Handle wheel events (horizontal scrolling)
    const handleWheel = useCallback(
        (e: React.WheelEvent) => {
            // Handle horizontal scrolling (shift+wheel or trackpad horizontal)
            if (e.deltaX !== 0) {
                e.preventDefault();
                accumulatedHorizontalDelta.current += e.deltaX;

                const scrollDelta =
                    Math.trunc(
                        accumulatedHorizontalDelta.current /
                            PIXELS_PER_HORIZONTAL_SCROLL,
                    ) * PIXELS_PER_HORIZONTAL_SCROLL;

                if (scrollDelta !== 0) {
                    accumulatedHorizontalDelta.current -= scrollDelta;
                    setScrollLeft((prev) =>
                        clamp(prev + scrollDelta, 0, maxScrollLeft),
                    );
                }
            }
        },
        [clamp, maxScrollLeft],
    );

    // Container ref callback with ResizeObserver
    const containerRef = useCallback((node: HTMLDivElement | null) => {
        // Cleanup previous observer
        if (resizeObserverRef.current) {
            resizeObserverRef.current.disconnect();
            resizeObserverRef.current = null;
        }

        containerNodeRef.current = node;

        if (node) {
            const observer = new ResizeObserver((entries) => {
                for (const entry of entries) {
                    setViewportWidth(entry.contentRect.width);
                    setContainerHeight(entry.contentRect.height);
                }
            });
            observer.observe(node);
            resizeObserverRef.current = observer;

            // Set initial values
            setViewportWidth(node.clientWidth);
            setContainerHeight(node.clientHeight);
        }
    }, []);

    // Handler for viewport height change from GridBody
    const handleViewportHeightChange = useCallback((height: number) => {
        setViewportHeight(height);
    }, []);

    return {
        currentRowIndex,
        scrollLeft,
        viewportHeight,
        viewportWidth,
        containerHeight,
        visibleRowCount,
        maxRowIndex,
        maxScrollLeft,
        totalWidth,
        showHorizontalScrollbar,
        verticalScrollbarHeight,
        handleVerticalSeek,
        handleHorizontalSeek,
        handleWheel,
        containerRef,
        setViewportHeight: handleViewportHeightChange,
    };
}

/**
 * Exported constants for use by other grid components.
 */
export { ROW_HEIGHT, SCROLLBAR_SIZE };
