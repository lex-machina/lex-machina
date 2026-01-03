"use client";

import { useState, useCallback, useEffect, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

import type {
    ColumnInfo,
    FileInfo,
    PipelineConfigRequest,
    RowRange,
    PreprocessingUIState,
} from "@/types";
import { DEFAULT_PIPELINE_CONFIG } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export type ResultsTabValue = "results" | "history";

export interface UseProcessingUIStateReturn {
    /** Whether persisted state has been loaded from Rust */
    isLoaded: boolean;

    /** Selected columns for preprocessing */
    selectedColumns: string[];
    /** Update selected columns */
    setSelectedColumns: (columns: string[]) => void;

    /** Row range to process, or null for all rows */
    rowRange: RowRange | null;
    /** Update row range */
    setRowRange: (range: RowRange | null) => void;

    /** Pipeline configuration */
    config: PipelineConfigRequest;
    /** Update pipeline configuration */
    setConfig: (config: PipelineConfigRequest) => void;

    /** Active tab in results panel */
    activeResultsTab: ResultsTabValue;
    /** Update active results tab */
    setActiveResultsTab: (tab: ResultsTabValue) => void;

    /** Reinitialize columns when file changes (selects all columns) */
    initializeColumnsForFile: (columns: ColumnInfo[]) => void;
}

// ============================================================================
// HOOK
// ============================================================================

/**
 * Hook for managing preprocessing UI state with persistence to Rust.
 *
 * This hook handles:
 * - Loading persisted state from Rust on mount
 * - Debounced saving to Rust when state changes
 * - Reinitializing columns when a new file is loaded
 *
 * @param fileInfo - Current file info, used to detect file changes
 * @returns UI state and setters
 *
 * @example
 * ```tsx
 * function ProcessingPage() {
 *     const { fileInfo } = useFileState();
 *     const {
 *         isLoaded,
 *         selectedColumns,
 *         setSelectedColumns,
 *         // ...
 *     } = useProcessingUIState(fileInfo);
 *
 *     // Don't render until state is loaded
 *     if (!isLoaded) return <Loading />;
 *
 *     // ...
 * }
 * ```
 */
export function useProcessingUIState(
    fileInfo: FileInfo | null,
): UseProcessingUIStateReturn {
    // Track if we've loaded the persisted state
    const [isLoaded, setIsLoaded] = useState(false);
    const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    // Local state for configuration
    const [config, setConfig] = useState<PipelineConfigRequest>(
        DEFAULT_PIPELINE_CONFIG,
    );
    const [activeResultsTab, setActiveResultsTab] =
        useState<ResultsTabValue>("history");

    // Selected columns state with file tracking
    const [selectionState, setSelectionState] = useState<{
        filePath: string | null;
        selected: string[];
    }>({ filePath: null, selected: [] });

    // Row range state with file tracking (resets when file changes)
    const [rowRangeState, setRowRangeStateInternal] = useState<{
        filePath: string | null;
        range: RowRange | null;
    }>({ filePath: null, range: null });

    // ========================================================================
    // COLUMN SELECTION
    // ========================================================================

    const currentFilePath = fileInfo?.path ?? null;

    // Memoize column names to avoid recreating on every render
    // Use fileInfo?.columns directly in the dependency, not a derived variable
    const allColumnNames = useMemo(
        () => (fileInfo?.columns ?? []).map((col) => col.name),
        [fileInfo?.columns],
    );

    // Determine selected columns based on file state
    // If the file changed, return all columns for the new file
    const selectedColumns = useMemo(() => {
        if (currentFilePath !== selectionState.filePath) {
            return allColumnNames;
        }
        return selectionState.selected;
    }, [currentFilePath, selectionState, allColumnNames]);

    // Compute rowRange - reset to null when file changes
    const rowRange = useMemo(() => {
        if (currentFilePath !== rowRangeState.filePath) {
            // File changed, reset row range
            return null;
        }
        return rowRangeState.range;
    }, [currentFilePath, rowRangeState]);

    // Update selection and track file path
    const setSelectedColumns = useCallback(
        (newSelection: string[]) => {
            setSelectionState({
                filePath: currentFilePath,
                selected: newSelection,
            });
        },
        [currentFilePath],
    );

    // Wrapper for setRowRange that tracks file path
    const setRowRange = useCallback(
        (range: RowRange | null) => {
            setRowRangeStateInternal({
                filePath: currentFilePath,
                range,
            });
        },
        [currentFilePath],
    );

    // Initialize columns when file changes
    const initializeColumnsForFile = useCallback((newColumns: ColumnInfo[]) => {
        setSelectionState((prev) => ({
            filePath: prev.filePath,
            selected: newColumns.map((col) => col.name),
        }));
    }, []);

    // ========================================================================
    // LOAD PERSISTED STATE
    // ========================================================================

    useEffect(() => {
        // Only attempt to load once, and only when we have file info
        if (isLoaded || !fileInfo) return;

        const currentFileInfo = fileInfo; // Capture for async closure

        async function loadPersistedState() {
            try {
                const savedState = await invoke<PreprocessingUIState>(
                    "get_preprocessing_ui_state",
                );

                // Only restore if we have saved columns (indicates previous session)
                if (savedState.selected_columns.length > 0) {
                    // Restore selected columns - filter to only include columns that exist
                    const validColumns = savedState.selected_columns.filter(
                        (col) =>
                            currentFileInfo.columns.some((c) => c.name === col),
                    );

                    // Only restore if we have valid columns
                    if (validColumns.length > 0) {
                        setSelectionState({
                            filePath: currentFileInfo.path,
                            selected: validColumns,
                        });
                    }

                    // Restore row range (clamped to current file's row count)
                    if (savedState.row_range) {
                        const maxRow = currentFileInfo.row_count;
                        setRowRangeStateInternal({
                            filePath: currentFileInfo.path,
                            range: {
                                start: Math.min(
                                    savedState.row_range[0],
                                    maxRow - 1,
                                ),
                                end: Math.min(savedState.row_range[1], maxRow),
                            },
                        });
                    }

                    // Restore config
                    setConfig(savedState.config);

                    // Restore active results tab
                    if (
                        savedState.active_results_tab === "results" ||
                        savedState.active_results_tab === "history"
                    ) {
                        setActiveResultsTab(savedState.active_results_tab);
                    }
                }

                setIsLoaded(true);
            } catch (err) {
                // Silently ignore - use defaults if loading fails
                console.warn(
                    "Failed to load persisted preprocessing state:",
                    err,
                );
                setIsLoaded(true);
            }
        }

        loadPersistedState();
    }, [fileInfo, isLoaded]);

    // ========================================================================
    // SAVE STATE TO RUST (DEBOUNCED)
    // ========================================================================

    useEffect(() => {
        // Skip if not loaded yet or no file
        if (!isLoaded || !fileInfo) return;

        // Clear any pending save
        if (saveTimeoutRef.current) {
            clearTimeout(saveTimeoutRef.current);
        }

        // Debounce the save to avoid too many IPC calls
        saveTimeoutRef.current = setTimeout(() => {
            const uiState: PreprocessingUIState = {
                selected_columns: selectedColumns,
                row_range: rowRange ? [rowRange.start, rowRange.end] : null,
                config,
                active_results_tab: activeResultsTab,
            };

            invoke("set_preprocessing_ui_state", { uiState }).catch((err) => {
                console.warn("Failed to save preprocessing UI state:", err);
            });
        }, 300);

        return () => {
            if (saveTimeoutRef.current) {
                clearTimeout(saveTimeoutRef.current);
            }
        };
    }, [
        selectedColumns,
        rowRange,
        config,
        activeResultsTab,
        isLoaded,
        fileInfo,
    ]);

    // ========================================================================
    // RETURN
    // ========================================================================

    return {
        isLoaded,
        selectedColumns,
        setSelectedColumns,
        rowRange,
        setRowRange,
        config,
        setConfig,
        activeResultsTab,
        setActiveResultsTab,
        initializeColumnsForFile,
    };
}
