"use client";

import { useState, useCallback, useEffect, useRef } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { invoke } from "@tauri-apps/api/core";
import {
  Cog,
  Sparkles,
  Eraser,
  GitBranch,
  Table2,
  Play,
  AlertTriangle,
} from "lucide-react";

import type { ColumnInfo, PreprocessingUIState } from "@/types";
import { useFileState } from "@/lib/hooks/use-file-state";
import { usePreprocessing } from "@/lib/hooks/use-preprocessing";
import { useSettings } from "@/lib/hooks/use-settings";
import {
  DEFAULT_PIPELINE_CONFIG,
  type PipelineConfigRequest,
  type RowRange,
  type PreprocessingHistoryEntry,
} from "@/types";

import AppShell from "@/components/layout/app-shell";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardContent, CardFooter } from "@/components/ui/card";
import { toast } from "@/components/ui/toast";

import {
  ColumnSelector,
  ColumnSelectorHeader,
  RowRangeSelector,
  ConfigPanel,
  ProgressPanel,
  ResultsPanel,
} from "@/components/preprocessing";

// ============================================================================
// EMPTY STATE
// ============================================================================

/**
 * Empty state component shown when no file is loaded.
 */
function NoFileLoadedState() {
  return (
    <div className="flex-1 flex items-center justify-center p-8">
      <div className="text-center max-w-md">
        {/* Icon */}
        <div className="mx-auto w-16 h-16 rounded-full bg-muted flex items-center justify-center mb-6">
          <Cog className="w-8 h-8 text-muted-foreground" />
        </div>

        {/* Title and description */}
        <h2 className="text-xl font-semibold mb-2">Data Preprocessing</h2>
        <p className="text-muted-foreground mb-6">
          Import a dataset to clean, transform, and prepare your data for analysis and ML.
        </p>

        {/* Features */}
        <ul className="text-sm text-muted-foreground space-y-2 mb-8 text-left">
          <li className="flex items-center gap-3">
            <Eraser className="w-4 h-4 shrink-0" />
            <span>Missing value imputation (KNN, statistical)</span>
          </li>
          <li className="flex items-center gap-3">
            <GitBranch className="w-4 h-4 shrink-0" />
            <span>Outlier detection and handling</span>
          </li>
          <li className="flex items-center gap-3">
            <Sparkles className="w-4 h-4 shrink-0" />
            <span>AI-guided preprocessing decisions</span>
          </li>
          <li className="flex items-center gap-3">
            <Cog className="w-4 h-4 shrink-0" />
            <span>Type correction and data cleaning</span>
          </li>
        </ul>

        {/* Action button */}
        <Button asChild size="lg">
          <Link href="/data">
            <Table2 className="w-4 h-4 mr-2" />
            Go to Data
          </Link>
        </Button>
      </div>
    </div>
  );
}

// ============================================================================
// TOOLBAR
// ============================================================================

interface ProcessingToolbarProps {
  isProcessing: boolean;
  canStart: boolean;
  onStart: () => void;
}

function ProcessingToolbar({
  isProcessing,
  canStart,
  onStart,
}: ProcessingToolbarProps) {
  return (
    <div className="flex items-center gap-4">
      <Button
        variant="default"
        size="sm"
        onClick={onStart}
        disabled={!canStart || isProcessing}
      >
        <Play className="w-3.5 h-3.5 mr-1.5" />
        {isProcessing ? "Processing..." : "Start Processing"}
      </Button>
    </div>
  );
}

// ============================================================================
// LEFT PANEL - COLUMNS & ROW RANGE
// ============================================================================

interface LeftPanelProps {
  columns: ColumnInfo[];
  selectedColumns: string[];
  onSelectionChange: (columns: string[]) => void;
  rowRange: RowRange | null;
  onRowRangeChange: (range: RowRange | null) => void;
  totalRows: number;
  isProcessing: boolean;
}

function LeftPanel({
  columns,
  selectedColumns,
  onSelectionChange,
  rowRange,
  onRowRangeChange,
  totalRows,
  isProcessing,
}: LeftPanelProps) {
  // Selection handlers for the header
  const handleSelectAll = useCallback(() => {
    onSelectionChange(columns.map((col) => col.name));
  }, [columns, onSelectionChange]);

  const handleDeselectAll = useCallback(() => {
    onSelectionChange([]);
  }, [onSelectionChange]);

  const noColumnsSelected = selectedColumns.length === 0;

  return (
    <div className="flex flex-col h-full min-h-0">
      {/* Column Selector - takes remaining space with internal scroll */}
      <Card className="flex-1 min-h-0">
        <CardHeader
          title="Columns"
          actions={
            <ColumnSelectorHeader
              totalCount={columns.length}
              selectedCount={selectedColumns.length}
              onSelectAll={handleSelectAll}
              onDeselectAll={handleDeselectAll}
              disabled={isProcessing}
            />
          }
        />
        <CardContent className="overflow-hidden">
          <ColumnSelector
            columns={columns}
            selectedColumns={selectedColumns}
            onSelectionChange={onSelectionChange}
            disabled={isProcessing}
            hideHeader={true}
            className="h-full"
          />
        </CardContent>
        {/* Warning when no columns selected */}
        {noColumnsSelected && columns.length > 0 && (
          <CardFooter className="flex items-center gap-2 text-xs text-muted-foreground">
            <AlertTriangle className="size-3.5 shrink-0" />
            <span>Select at least one column to process</span>
          </CardFooter>
        )}
      </Card>

      {/* Row Range Selector - fixed at bottom */}
      <Card className="mt-3 shrink-0">
        <CardHeader title="Row Range" />
        <CardContent padded>
          <RowRangeSelector
            totalRows={totalRows}
            rowRange={rowRange}
            onRangeChange={onRowRangeChange}
            disabled={isProcessing}
          />
        </CardContent>
      </Card>
    </div>
  );
}

// ============================================================================
// CENTER PANEL - CONFIGURATION
// ============================================================================

interface CenterPanelProps {
  config: PipelineConfigRequest;
  onConfigChange: (config: PipelineConfigRequest) => void;
  columns: ColumnInfo[];
  selectedColumns: string[];
  hasAIProvider: boolean;
  isProcessing: boolean;
}

function CenterPanel({
  config,
  onConfigChange,
  columns,
  selectedColumns,
  hasAIProvider,
  isProcessing,
}: CenterPanelProps) {
  return (
    <Card className="h-full min-h-0">
      <CardHeader title="Configuration" />
      <CardContent scrollable>
        <ConfigPanel
          config={config}
          onConfigChange={onConfigChange}
          columns={columns}
          selectedColumns={selectedColumns}
          hasAIProvider={hasAIProvider}
          disabled={isProcessing}
        />
      </CardContent>
    </Card>
  );
}

// ============================================================================
// RIGHT PANEL - PROGRESS / RESULTS+HISTORY
// ============================================================================

interface RightPanelProps {
  status: "idle" | "running" | "completed" | "cancelled" | "error";
  progress: ReturnType<typeof usePreprocessing>["progress"];
  summary: ReturnType<typeof usePreprocessing>["summary"];
  error: string | null;
  onCancel: () => void;
  onReset: () => void;
  onViewData: () => void;
  getHistory: () => Promise<PreprocessingHistoryEntry[]>;
  clearHistory: () => Promise<void>;
  onSelectHistoryEntry: (entry: PreprocessingHistoryEntry) => void;
  activeResultsTab: "results" | "history";
  onActiveResultsTabChange: (tab: "results" | "history") => void;
}

function RightPanel({
  status,
  progress,
  summary,
  error,
  onCancel,
  onReset,
  onViewData,
  getHistory,
  clearHistory,
  onSelectHistoryEntry,
  activeResultsTab,
  onActiveResultsTabChange,
}: RightPanelProps) {
  const isRunning = status === "running";
  const showProgress = isRunning || status === "error" || status === "cancelled";

  return (
    <div className="h-full min-h-0 flex flex-col">
      {/* Progress Panel - Show when running or error/cancelled */}
      {showProgress ? (
        <ProgressPanel
          status={status}
          progress={progress}
          onCancel={onCancel}
          onReset={onReset}
          error={error}
        />
      ) : (
        /* Results Panel with tabs (Results | History) - Show when idle or completed */
        <ResultsPanel
          summary={summary}
          onViewData={onViewData}
          getHistory={getHistory}
          onSelectHistoryEntry={onSelectHistoryEntry}
          onClearHistory={clearHistory}
          disabled={isRunning}
          className="flex-1"
          activeTab={activeResultsTab}
          onActiveTabChange={onActiveResultsTabChange}
        />
      )}
    </div>
  );
}

// ============================================================================
// MAIN CONTENT - THREE COLUMN LAYOUT
// ============================================================================

interface ProcessingContentProps {
  fileInfo: ReturnType<typeof useFileState>["fileInfo"];
  columns: ColumnInfo[];
  selectedColumns: string[];
  onSelectionChange: (columns: string[]) => void;
  rowRange: RowRange | null;
  onRowRangeChange: (range: RowRange | null) => void;
  config: PipelineConfigRequest;
  onConfigChange: (config: PipelineConfigRequest) => void;
  hasAIProvider: boolean;
  status: "idle" | "running" | "completed" | "cancelled" | "error";
  progress: ReturnType<typeof usePreprocessing>["progress"];
  summary: ReturnType<typeof usePreprocessing>["summary"];
  error: string | null;
  isProcessing: boolean;
  onCancel: () => void;
  onReset: () => void;
  onViewData: () => void;
  getHistory: () => Promise<PreprocessingHistoryEntry[]>;
  clearHistory: () => Promise<void>;
  onSelectHistoryEntry: (entry: PreprocessingHistoryEntry) => void;
  activeResultsTab: "results" | "history";
  onActiveResultsTabChange: (tab: "results" | "history") => void;
}

function ProcessingContent({
  fileInfo,
  columns,
  selectedColumns,
  onSelectionChange,
  rowRange,
  onRowRangeChange,
  config,
  onConfigChange,
  hasAIProvider,
  status,
  progress,
  summary,
  error,
  isProcessing,
  onCancel,
  onReset,
  onViewData,
  getHistory,
  clearHistory,
  onSelectHistoryEntry,
  activeResultsTab,
  onActiveResultsTabChange,
}: ProcessingContentProps) {
  // Show empty state when no file is loaded
  if (!fileInfo) {
    return <NoFileLoadedState />;
  }

  return (
    <div className="flex-1 grid grid-cols-3 gap-4 p-4 min-h-0">
      {/* Left Panel - Columns & Row Range */}
      <div className="min-h-0">
        <LeftPanel
          columns={columns}
          selectedColumns={selectedColumns}
          onSelectionChange={onSelectionChange}
          rowRange={rowRange}
          onRowRangeChange={onRowRangeChange}
          totalRows={fileInfo.row_count}
          isProcessing={isProcessing}
        />
      </div>

      {/* Center Panel - Configuration */}
      <div className="min-h-0">
        <CenterPanel
          config={config}
          onConfigChange={onConfigChange}
          columns={columns}
          selectedColumns={selectedColumns}
          hasAIProvider={hasAIProvider}
          isProcessing={isProcessing}
        />
      </div>

      {/* Right Panel - Progress/Results/History */}
      <div className="min-h-0">
        <RightPanel
          status={status}
          progress={progress}
          summary={summary}
          error={error}
          onCancel={onCancel}
          onReset={onReset}
          onViewData={onViewData}
          getHistory={getHistory}
          clearHistory={clearHistory}
          onSelectHistoryEntry={onSelectHistoryEntry}
          activeResultsTab={activeResultsTab}
          onActiveResultsTabChange={onActiveResultsTabChange}
        />
      </div>
    </div>
  );
}

// ============================================================================
// PROCESSING PAGE
// ============================================================================

/**
 * Processing page - Configure and run data preprocessing.
 *
 * Features:
 * - Three-column desktop layout (Columns | Configuration | Progress/Results)
 * - Column selection with data type badges
 * - Row range selection
 * - Preprocessing configuration
 * - Real-time progress tracking
 * - Results summary
 * - Processing history
 *
 * Layout is designed for desktop use with dense information display
 * and no scrolling on the main page (panels scroll internally).
 */
export default function ProcessingPage() {
  const router = useRouter();

  // Hooks
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

  // Track if we've loaded the persisted state (use state instead of ref to trigger re-render)
  const [hasLoadedPersistedState, setHasLoadedPersistedState] = useState(false);
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Local state for configuration
  const [rowRange, setRowRange] = useState<RowRange | null>(null);
  const [config, setConfig] = useState<PipelineConfigRequest>(DEFAULT_PIPELINE_CONFIG);
  const [activeResultsTab, setActiveResultsTab] = useState<"results" | "history">("history");

  // Get columns from fileInfo
  const columns = fileInfo?.columns ?? [];

  // Selected columns state - initialized to all columns when file changes
  // We use a compound state to track both selection and the file it was initialized for
  const [selectionState, setSelectionState] = useState<{
    filePath: string | null;
    selected: string[];
  }>({ filePath: null, selected: [] });

  // Derive selectedColumns - if file changed, return all columns (and update state)
  const currentFilePath = fileInfo?.path ?? null;
  let selectedColumns = selectionState.selected;

  if (currentFilePath !== selectionState.filePath && !hasLoadedPersistedState) {
    // File changed and we haven't loaded persisted state yet - select all columns
    selectedColumns = columns.map((col) => col.name);
  } else if (currentFilePath !== selectionState.filePath && hasLoadedPersistedState) {
    // Different file loaded after we restored state - reset to all columns
    selectedColumns = columns.map((col) => col.name);
  }

  // Wrapper for setSelectedColumns that also updates the file path
  const setSelectedColumns = useCallback(
    (newSelection: string[]) => {
      setSelectionState({
        filePath: currentFilePath,
        selected: newSelection,
      });
    },
    [currentFilePath]
  );

  // Load persisted UI state when file info becomes available
  useEffect(() => {
    // Only attempt to load once, and only when we have file info
    if (hasLoadedPersistedState || !fileInfo) return;

    const currentFileInfo = fileInfo; // Capture for async closure

    async function loadPersistedState() {
      try {
        const savedState = await invoke<PreprocessingUIState>("get_preprocessing_ui_state");
        
        // Only restore if we have saved columns (indicates previous session had data)
        if (savedState.selected_columns.length > 0) {
          setHasLoadedPersistedState(true);
          
          // Restore selected columns - filter to only include columns that exist in current file
          const validColumns = savedState.selected_columns.filter((col) =>
            currentFileInfo.columns.some((c) => c.name === col)
          );
          
          // Only restore if we have valid columns, otherwise keep all columns selected
          if (validColumns.length > 0) {
            setSelectionState({
              filePath: currentFileInfo.path,
              selected: validColumns,
            });
          }

          // Restore row range (clamped to current file's row count)
          if (savedState.row_range) {
            const maxRow = currentFileInfo.row_count;
            setRowRange({
              start: Math.min(savedState.row_range[0], maxRow - 1),
              end: Math.min(savedState.row_range[1], maxRow),
            });
          }

          // Restore config
          setConfig(savedState.config);
          
          // Restore active results tab
          if (savedState.active_results_tab === "results" || savedState.active_results_tab === "history") {
            setActiveResultsTab(savedState.active_results_tab);
          }
        } else {
          // No saved columns - mark as loaded so we don't keep trying
          setHasLoadedPersistedState(true);
        }
      } catch (err) {
        // Silently ignore - use defaults if loading fails
        console.warn("Failed to load persisted preprocessing state:", err);
        setHasLoadedPersistedState(true);
      }
    }

    loadPersistedState();
  }, [fileInfo, hasLoadedPersistedState]);

  // Save UI state to Rust when it changes (debounced)
  useEffect(() => {
    // Skip if we haven't finished initial load or no file is loaded
    if (!isFileLoaded) return;

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
  }, [selectedColumns, rowRange, config, activeResultsTab, isFileLoaded]);

  // Can start preprocessing?
  const hasColumnsSelected = selectedColumns.length > 0;
  const canStart = isFileLoaded && !isProcessing && hasColumnsSelected;

  // Handle start preprocessing
  const handleStart = useCallback(async () => {
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
  }, [canStart, selectedColumns, rowRange, config, startPreprocessing]);

  // Handle view processed data
  const handleViewData = useCallback(() => {
    router.push("/data?tab=processed");
  }, [router]);

  // Handle clearing history - also clears the displayed summary
  const handleClearHistory = useCallback(async () => {
    await clearHistory();
    reset(); // Clears summary and resets status to idle
  }, [clearHistory, reset]);

  // Handle history entry selection
  const handleSelectHistoryEntry = useCallback(
    (entry: PreprocessingHistoryEntry) => {
      // Load the config from the history entry
      const historyConfig: PipelineConfigRequest = {
        missing_column_threshold: entry.config.missing_column_threshold,
        missing_row_threshold: entry.config.missing_row_threshold,
        outlier_strategy: entry.config.outlier_strategy as PipelineConfigRequest["outlier_strategy"],
        numeric_imputation: entry.config.numeric_imputation as PipelineConfigRequest["numeric_imputation"],
        categorical_imputation: entry.config.categorical_imputation as PipelineConfigRequest["categorical_imputation"],
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
    [setSelectedColumns, setSummary]
  );

  return (
    <AppShell
      toolbar={
        <ProcessingToolbar
          isProcessing={isProcessing}
          canStart={canStart}
          onStart={handleStart}
        />
      }
    >
      <ProcessingContent
        fileInfo={fileInfo}
        columns={fileInfo?.columns ?? []}
        selectedColumns={selectedColumns}
        onSelectionChange={setSelectedColumns}
        rowRange={rowRange}
        onRowRangeChange={setRowRange}
        config={config}
        onConfigChange={setConfig}
        hasAIProvider={hasAIProvider}
        status={status}
        progress={progress}
        summary={summary}
        error={error}
        isProcessing={isProcessing}
        onCancel={cancelPreprocessing}
        onReset={reset}
        onViewData={handleViewData}
        getHistory={getHistory}
        clearHistory={handleClearHistory}
        onSelectHistoryEntry={handleSelectHistoryEntry}
        activeResultsTab={activeResultsTab}
        onActiveResultsTabChange={setActiveResultsTab}
      />
    </AppShell>
  );
}
