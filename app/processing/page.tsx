"use client";

import { useState, useCallback } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import {
  Cog,
  Sparkles,
  Eraser,
  GitBranch,
  Table2,
  Play,
  FileText,
} from "lucide-react";

import type { ColumnInfo } from "@/types";
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
// HELPERS
// ============================================================================

/**
 * Format bytes to human readable string.
 */
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

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
  fileInfo: ReturnType<typeof useFileState>["fileInfo"];
  isProcessing: boolean;
  canStart: boolean;
  onStart: () => void;
}

function ProcessingToolbar({
  fileInfo,
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
      {fileInfo && (
        <div className="flex items-center gap-2 px-2.5 py-1 rounded bg-muted text-xs text-muted-foreground">
          <FileText className="w-3.5 h-3.5" />
          <span className="font-medium">{fileInfo.name}</span>
          <span>•</span>
          <span>{fileInfo.row_count.toLocaleString()} × {fileInfo.column_count}</span>
          <span>•</span>
          <span>{formatBytes(fileInfo.size_bytes)}</span>
        </div>
      )}
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

  return (
    <div className="flex flex-col h-full min-h-0">
      {/* Column Selector - takes remaining space with internal scroll */}
      <div className="flex-1 min-h-0 border rounded-lg overflow-hidden flex flex-col">
        <div className="flex items-center justify-between px-3 py-2 border-b bg-muted/30">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Columns
          </h3>
          <ColumnSelectorHeader
            totalCount={columns.length}
            selectedCount={selectedColumns.length}
            onSelectAll={handleSelectAll}
            onDeselectAll={handleDeselectAll}
            disabled={isProcessing}
          />
        </div>
        <div className="flex-1 min-h-0 overflow-hidden">
          <ColumnSelector
            columns={columns}
            selectedColumns={selectedColumns}
            onSelectionChange={onSelectionChange}
            disabled={isProcessing}
            hideHeader={true}
            className="h-full"
          />
        </div>
      </div>

      {/* Row Range Selector - fixed at bottom */}
      <div className="mt-3 border rounded-lg p-3 shrink-0">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-3">
          Row Range
        </h3>
        <RowRangeSelector
          totalRows={totalRows}
          rowRange={rowRange}
          onRangeChange={onRowRangeChange}
          disabled={isProcessing}
        />
      </div>
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
  hasAIProvider: boolean;
  isProcessing: boolean;
}

function CenterPanel({
  config,
  onConfigChange,
  columns,
  hasAIProvider,
  isProcessing,
}: CenterPanelProps) {
  return (
    <div className="h-full min-h-0 border rounded-lg overflow-hidden flex flex-col">
      <div className="px-3 py-2 border-b bg-muted/30">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Configuration
        </h3>
      </div>
      <div className="flex-1 min-h-0 overflow-y-auto">
        <ConfigPanel
          config={config}
          onConfigChange={onConfigChange}
          columns={columns}
          hasAIProvider={hasAIProvider}
          disabled={isProcessing}
        />
      </div>
    </div>
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
  } = usePreprocessing();

  // Local state for configuration
  const [selectedColumns, setSelectedColumns] = useState<string[]>([]);
  const [rowRange, setRowRange] = useState<RowRange | null>(null);
  const [config, setConfig] = useState<PipelineConfigRequest>(DEFAULT_PIPELINE_CONFIG);

  // Can start preprocessing?
  const canStart = isFileLoaded && !isProcessing;

  // Handle start preprocessing
  const handleStart = useCallback(async () => {
    if (!canStart) return;

    try {
      // Convert RowRange to tuple format expected by Rust
      const rowRangeTuple: [number, number] | null = rowRange
        ? [rowRange.start, rowRange.end]
        : null;

      await startPreprocessing(selectedColumns, rowRangeTuple, config);
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

      toast.info("Configuration loaded from history");
    },
    []
  );

  return (
    <AppShell
      toolbar={
        <ProcessingToolbar
          fileInfo={fileInfo}
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
        clearHistory={clearHistory}
        onSelectHistoryEntry={handleSelectHistoryEntry}
      />
    </AppShell>
  );
}
