"use client";

import {
    createContext,
    useContext,
    useMemo,
    useCallback,
    type ReactNode,
} from "react";
import { useRouter } from "next/navigation";

import type {
    ColumnInfo,
    FileInfo,
    PipelineConfigRequest,
    RowRange,
    ProgressUpdate,
    PreprocessingSummary,
    PreprocessingHistoryEntry,
} from "@/types";
import type { PreprocessingStatus } from "@/lib/hooks/use-preprocessing";
import { useFileState } from "@/lib/hooks/use-file-state";
import { usePreprocessing } from "@/lib/hooks/use-preprocessing";
import { useSettings } from "@/lib/hooks/use-settings";
import { toast } from "@/components/ui/toast";

// ============================================================================
// TYPES
// ============================================================================

export type ResultsTabValue = "results" | "history";

export interface ProcessingContextValue {
    // File state
    fileInfo: FileInfo | null;
    isFileLoaded: boolean;
    columns: ColumnInfo[];

    // Selection state
    selectedColumns: string[];
    setSelectedColumns: (columns: string[]) => void;
    rowRange: RowRange | null;
    setRowRange: (range: RowRange | null) => void;

    // Config state
    config: PipelineConfigRequest;
    setConfig: (config: PipelineConfigRequest) => void;

    // Processing state
    status: PreprocessingStatus;
    isProcessing: boolean;
    progress: ProgressUpdate | null;
    summary: PreprocessingSummary | null;
    error: string | null;

    // Settings
    hasAIProvider: boolean;

    // UI state
    activeResultsTab: ResultsTabValue;
    setActiveResultsTab: (tab: ResultsTabValue) => void;

    // Actions
    startProcessing: () => Promise<void>;
    cancelProcessing: () => Promise<void>;
    reset: () => void;
    getHistory: () => Promise<PreprocessingHistoryEntry[]>;
    clearHistory: () => Promise<void>;
    loadHistoryEntry: (entry: PreprocessingHistoryEntry) => void;
    viewProcessedData: () => void;

    // Derived
    canStart: boolean;
    hasColumnsSelected: boolean;
}

// ============================================================================
// CONTEXT
// ============================================================================

const ProcessingContext = createContext<ProcessingContextValue | null>(null);

// ============================================================================
// PROVIDER PROPS
// ============================================================================

export interface ProcessingProviderProps {
    children: ReactNode;
    // State managed by parent (from UI persistence hook)
    selectedColumns: string[];
    setSelectedColumns: (columns: string[]) => void;
    rowRange: RowRange | null;
    setRowRange: (range: RowRange | null) => void;
    config: PipelineConfigRequest;
    setConfig: (config: PipelineConfigRequest) => void;
    activeResultsTab: ResultsTabValue;
    setActiveResultsTab: (tab: ResultsTabValue) => void;
}

// ============================================================================
// PROVIDER
// ============================================================================

/**
 * Provider for the processing page context.
 *
 * This wraps the processing page and combines:
 * - File state from useFileState
 * - Processing operations from usePreprocessing
 * - Settings from useSettings
 * - UI state (columns, config, etc.) from props (managed by page)
 *
 * The UI state is passed in as props so it can be managed by the
 * useProcessingUIState hook for persistence.
 */
export function ProcessingProvider({
    children,
    selectedColumns,
    setSelectedColumns,
    rowRange,
    setRowRange,
    config,
    setConfig,
    activeResultsTab,
    setActiveResultsTab,
}: ProcessingProviderProps) {
    const router = useRouter();

    // External hooks
    const { fileInfo, isFileLoaded } = useFileState();
    const { hasAIProvider } = useSettings();
    const {
        status,
        isProcessing,
        progress,
        summary,
        error,
        startPreprocessing,
        cancelPreprocessing,
        reset,
        getHistory,
        clearHistory,
        setSummary,
    } = usePreprocessing();

    // Derived state - memoize columns to avoid recreating array on every render
    const columns = useMemo(() => fileInfo?.columns ?? [], [fileInfo?.columns]);
    const hasColumnsSelected = selectedColumns.length > 0;
    const canStart = isFileLoaded && !isProcessing && hasColumnsSelected;

    // ========================================================================
    // ACTIONS
    // ========================================================================

    /**
     * Start the preprocessing pipeline with current configuration.
     */
    const startProcessing = useCallback(async () => {
        if (!canStart) return;

        try {
            // Convert RowRange to tuple format expected by Rust
            const rowRangeTuple: [number, number] | null = rowRange
                ? [rowRange.start, rowRange.end]
                : null;

            await startPreprocessing(selectedColumns, rowRangeTuple, config);
            // Switch to results tab on successful completion
            setActiveResultsTab("results");
            toast.success("Preprocessing completed successfully");
        } catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            toast.error(`Preprocessing failed: ${message}`);
        }
    }, [
        canStart,
        selectedColumns,
        rowRange,
        config,
        startPreprocessing,
        setActiveResultsTab,
    ]);

    /**
     * Cancel the running preprocessing operation.
     */
    const cancelProcessing = useCallback(async () => {
        await cancelPreprocessing();
    }, [cancelPreprocessing]);

    /**
     * Clear history and reset display state.
     */
    const handleClearHistory = useCallback(async () => {
        await clearHistory();
        reset(); // Clears summary and resets status to idle
    }, [clearHistory, reset]);

    /**
     * Load configuration from a history entry.
     */
    const loadHistoryEntry = useCallback(
        (entry: PreprocessingHistoryEntry) => {
            // Load the config from the history entry
            const historyConfig: PipelineConfigRequest = {
                missing_column_threshold: entry.config.missing_column_threshold,
                missing_row_threshold: entry.config.missing_row_threshold,
                outlier_strategy: entry.config
                    .outlier_strategy as PipelineConfigRequest["outlier_strategy"],
                numeric_imputation: entry.config
                    .numeric_imputation as PipelineConfigRequest["numeric_imputation"],
                categorical_imputation: entry.config
                    .categorical_imputation as PipelineConfigRequest["categorical_imputation"],
                enable_type_correction: entry.config.enable_type_correction,
                remove_duplicates: entry.config.remove_duplicates,
                knn_neighbors: entry.config.knn_neighbors,
                use_ai_decisions: entry.config.use_ai_decisions,
                target_column: entry.config.target_column,
            };

            setConfig(historyConfig);
            setSelectedColumns(entry.config.selected_columns);

            if (entry.config.row_range) {
                setRowRange({
                    start: entry.config.row_range[0],
                    end: entry.config.row_range[1],
                });
            } else {
                setRowRange(null);
            }

            // Show the results from this history entry
            setSummary(entry.summary);
            setActiveResultsTab("results");

            toast.info("Configuration loaded from history");
        },
        [
            setConfig,
            setSelectedColumns,
            setRowRange,
            setSummary,
            setActiveResultsTab,
        ],
    );

    /**
     * Navigate to the data page to view processed data.
     */
    const viewProcessedData = useCallback(() => {
        router.push("/data?tab=processed");
    }, [router]);

    // ========================================================================
    // CONTEXT VALUE
    // ========================================================================

    const value = useMemo<ProcessingContextValue>(
        () => ({
            // File state
            fileInfo,
            isFileLoaded,
            columns,

            // Selection state
            selectedColumns,
            setSelectedColumns,
            rowRange,
            setRowRange,

            // Config state
            config,
            setConfig,

            // Processing state
            status,
            isProcessing,
            progress,
            summary,
            error,

            // Settings
            hasAIProvider,

            // UI state
            activeResultsTab,
            setActiveResultsTab,

            // Actions
            startProcessing,
            cancelProcessing,
            reset,
            getHistory,
            clearHistory: handleClearHistory,
            loadHistoryEntry,
            viewProcessedData,

            // Derived
            canStart,
            hasColumnsSelected,
        }),
        [
            fileInfo,
            isFileLoaded,
            columns,
            selectedColumns,
            setSelectedColumns,
            rowRange,
            setRowRange,
            config,
            setConfig,
            status,
            isProcessing,
            progress,
            summary,
            error,
            hasAIProvider,
            activeResultsTab,
            setActiveResultsTab,
            startProcessing,
            cancelProcessing,
            reset,
            getHistory,
            handleClearHistory,
            loadHistoryEntry,
            viewProcessedData,
            canStart,
            hasColumnsSelected,
        ],
    );

    return (
        <ProcessingContext.Provider value={value}>
            {children}
        </ProcessingContext.Provider>
    );
}

// ============================================================================
// HOOK
// ============================================================================

/**
 * Hook to access the processing context.
 *
 * Must be used within a ProcessingProvider.
 *
 * @throws {Error} If used outside of ProcessingProvider
 *
 * @example
 * ```tsx
 * function ProcessingToolbar() {
 *     const { canStart, isProcessing, startProcessing } = useProcessingContext();
 *
 *     return (
 *         <Button onClick={startProcessing} disabled={!canStart || isProcessing}>
 *             {isProcessing ? "Processing..." : "Start Processing"}
 *         </Button>
 *     );
 * }
 * ```
 */
export function useProcessingContext(): ProcessingContextValue {
    const context = useContext(ProcessingContext);

    if (!context) {
        throw new Error(
            "useProcessingContext must be used within a ProcessingProvider",
        );
    }

    return context;
}
