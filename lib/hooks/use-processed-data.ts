"use client";

import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRustEvent } from "./use-rust-event";
import {
    RUST_EVENTS,
    type FileInfo,
    type Row,
    type ProcessedRowsResponse,
    type PreprocessingSummary,
} from "@/types";

// ============================================================================
// CONSTANTS
// ============================================================================

/**
 * Number of extra rows to fetch beyond the visible area.
 * This provides buffer for smooth scrolling.
 */
const BUFFER_ROWS = 10;

// ============================================================================
// TYPES
// ============================================================================

/**
 * State returned by the useProcessedData hook.
 */
export interface ProcessedDataState {
    /** File info for the processed DataFrame (null if no processed data) */
    fileInfo: FileInfo | null;
    /** Whether processed data is available */
    hasProcessedData: boolean;
    /** Total number of rows in the processed DataFrame */
    totalRows: number;
    /** Column information from the processed DataFrame */
    columns: FileInfo["columns"];
    /** Current column widths */
    columnWidths: number[];
    /** Currently loaded rows buffer */
    rows: Row[];
    /** Starting index of the rows buffer */
    visibleStart: number;
    /** Summary from the preprocessing run that created this data */
    summary: PreprocessingSummary | null;
}

/**
 * Actions returned by the useProcessedData hook.
 */
export interface ProcessedDataActions {
    /**
     * Fetch rows for virtual scrolling.
     *
     * @param currentRowIndex - Current top visible row index
     * @param visibleRowCount - Number of rows visible in viewport
     */
    fetchRows: (currentRowIndex: number, visibleRowCount: number) => void;

    /**
     * Update a column width (local state only).
     *
     * @param colIndex - Column index to update
     * @param width - New width in pixels
     */
    setColumnWidth: (colIndex: number, width: number) => void;

    /**
     * Refresh the processed data state from Rust.
     * Call this after preprocessing completes or when navigating to the data view.
     */
    refresh: () => Promise<void>;

    /**
     * Clear the processed data from memory.
     */
    clearProcessedData: () => Promise<void>;
}

/**
 * Return type of the useProcessedData hook.
 */
export type UseProcessedDataReturn = ProcessedDataState & ProcessedDataActions;

// ============================================================================
// HOOK IMPLEMENTATION
// ============================================================================

/**
 * Hook for accessing and displaying processed DataFrame data.
 *
 * This hook provides:
 * - File info for the processed DataFrame
 * - Virtual scrolling with row fetching
 * - Column width management
 * - Automatic refresh on preprocessing completion
 *
 * @returns State and actions for processed data access
 *
 * @example
 * ```tsx
 * function ProcessedDataGrid() {
 *   const {
 *     hasProcessedData,
 *     columns,
 *     columnWidths,
 *     totalRows,
 *     rows,
 *     visibleStart,
 *     fetchRows,
 *     setColumnWidth,
 *   } = useProcessedData();
 *
 *   if (!hasProcessedData) {
 *     return <div>No processed data available. Run preprocessing first.</div>;
 *   }
 *
 *   return (
 *     <DataGrid
 *       columns={columns}
 *       columnWidths={columnWidths}
 *       rows={rows}
 *       visibleStart={visibleStart}
 *       totalRows={totalRows}
 *       onScroll={(rowIndex, visibleCount) => fetchRows(rowIndex, visibleCount)}
 *       onColumnResize={(idx, width) => setColumnWidth(idx, width)}
 *     />
 *   );
 * }
 * ```
 *
 * @remarks
 * Following "Rust Supremacy", all data is fetched from Rust.
 * This hook manages the IPC communication and local UI state for display.
 */
export function useProcessedData(): UseProcessedDataReturn {
    // State
    const [fileInfo, setFileInfo] = useState<FileInfo | null>(null);
    const [rows, setRows] = useState<Row[]>([]);
    const [visibleStart, setVisibleStart] = useState(0);
    const [columnWidths, setColumnWidths] = useState<number[]>([]);
    const [summary, setSummary] = useState<PreprocessingSummary | null>(null);

    // Track last fetch to avoid duplicate requests
    const lastFetchRef = useRef({ start: -1, count: -1 });

    // Derived state
    const hasProcessedData = fileInfo !== null;
    const totalRows = fileInfo?.row_count ?? 0;
    const columns = fileInfo?.columns ?? [];

    // ============================================================================
    // DATA FETCHING
    // ============================================================================

    /**
     * Fetch processed file info from Rust.
     */
    const fetchFileInfo = useCallback(async () => {
        try {
            const info = await invoke<FileInfo | null>(
                "get_processed_file_info",
            );
            setFileInfo(info);

            if (info) {
                // Initialize column widths from file info
                setColumnWidths(info.columns.map((col) => col.width));
            } else {
                // Clear state if no processed data
                setColumnWidths([]);
                setRows([]);
                setVisibleStart(0);
                lastFetchRef.current = { start: -1, count: -1 };
            }
        } catch (err) {
            console.error("Failed to get processed file info:", err);
        }
    }, []);

    /**
     * Fetch rows for virtual scrolling.
     */
    const fetchRows = useCallback(
        async (currentRowIndex: number, visibleRowCount: number) => {
            if (!fileInfo) return;

            const fetchStart = Math.max(0, currentRowIndex - BUFFER_ROWS);
            const fetchCount = visibleRowCount + 2 * BUFFER_ROWS;

            // Avoid duplicate fetches
            if (
                lastFetchRef.current.start === fetchStart &&
                lastFetchRef.current.count === fetchCount
            ) {
                return;
            }

            lastFetchRef.current = { start: fetchStart, count: fetchCount };

            try {
                const response = await invoke<ProcessedRowsResponse>(
                    "get_processed_rows",
                    {
                        start: fetchStart,
                        count: fetchCount,
                    },
                );

                if (response) {
                    setRows(response.rows);
                    setVisibleStart(response.start);
                }
            } catch (err) {
                console.error("Failed to fetch processed rows:", err);
            }
        },
        [fileInfo],
    );

    // ============================================================================
    // COLUMN WIDTH MANAGEMENT
    // ============================================================================

    /**
     * Update a single column width (local state only).
     */
    const setColumnWidth = useCallback((colIndex: number, width: number) => {
        setColumnWidths((prev) => {
            const next = [...prev];
            next[colIndex] = width;
            return next;
        });
    }, []);

    // ============================================================================
    // ACTIONS
    // ============================================================================

    /**
     * Refresh processed data state from Rust.
     */
    const refresh = useCallback(async () => {
        await fetchFileInfo();
        // Reset fetch tracking to allow fresh data
        lastFetchRef.current = { start: -1, count: -1 };
    }, [fetchFileInfo]);

    /**
     * Clear processed data from memory.
     */
    const clearProcessedData = useCallback(async () => {
        try {
            await invoke("clear_processed_data");
            setFileInfo(null);
            setRows([]);
            setVisibleStart(0);
            setColumnWidths([]);
            setSummary(null);
            lastFetchRef.current = { start: -1, count: -1 };
        } catch (err) {
            console.error("Failed to clear processed data:", err);
        }
    }, []);

    // ============================================================================
    // EVENT SUBSCRIPTIONS
    // ============================================================================

    /**
     * Handle preprocessing complete event.
     * Automatically refresh data when preprocessing finishes.
     */
    const handlePreprocessingComplete = useCallback(
        (completeSummary: PreprocessingSummary) => {
            setSummary(completeSummary);
            // Refresh file info to get the new processed data
            fetchFileInfo();
        },
        [fetchFileInfo],
    );

    useRustEvent<PreprocessingSummary>(
        RUST_EVENTS.PREPROCESSING_COMPLETE,
        handlePreprocessingComplete,
    );

    // ============================================================================
    // INITIALIZATION
    // ============================================================================

    /**
     * Fetch initial state on mount.
     * This handles the case where processed data already exists
     * (e.g., navigating back to the data view after preprocessing).
     */
    useEffect(() => {
        // eslint-disable-next-line react-hooks/set-state-in-effect -- Initial data fetch on mount is standard pattern
        fetchFileInfo();
    }, [fetchFileInfo]);

    // ============================================================================
    // RETURN
    // ============================================================================

    return {
        // State
        fileInfo,
        hasProcessedData,
        totalRows,
        columns,
        columnWidths,
        rows,
        visibleStart,
        summary,

        // Actions
        fetchRows,
        setColumnWidth,
        refresh,
        clearProcessedData,
    };
}
