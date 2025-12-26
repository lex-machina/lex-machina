"use client";

import { useState, useCallback } from "react";
import { useRouter } from "next/navigation";

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
import ContextSidebar from "@/components/layout/context-sidebar";
import { Button } from "@/components/ui/button";
import { toast } from "@/components/ui/toast";

import {
  DatasetPreview,
  ColumnSelector,
  RowRangeSelector,
  ConfigPanel,
  ProgressPanel,
  ResultsPanel,
  HistoryList,
} from "@/components/preprocessing";

// ============================================================================
// TOOLBAR
// ============================================================================

interface ProcessingToolbarProps {
  isFileLoaded: boolean;
  isProcessing: boolean;
  canStart: boolean;
  onStart: () => void;
}

function ProcessingToolbar({
  isFileLoaded,
  isProcessing,
  canStart,
  onStart,
}: ProcessingToolbarProps) {
  return (
    <>
      <Button
        variant="default"
        size="sm"
        onClick={onStart}
        disabled={!canStart || isProcessing}
      >
        {isProcessing ? "Processing..." : "Start Preprocessing"}
      </Button>
      {!isFileLoaded && (
        <span className="text-xs text-muted-foreground">
          Load a file first to enable preprocessing
        </span>
      )}
    </>
  );
}

// ============================================================================
// SIDEBAR
// ============================================================================

interface ProcessingSidebarProps {
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

function ProcessingSidebar({
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
}: ProcessingSidebarProps) {
  const isIdle = status === "idle";
  const isRunning = status === "running";
  const isComplete = status === "completed";

  return (
    <div className="flex flex-col gap-4 p-4 h-full overflow-y-auto">
      {/* Progress Panel - Show when running or just finished */}
      {(isRunning || status === "error" || status === "cancelled") && (
        <ProgressPanel
          status={status}
          progress={progress}
          onCancel={onCancel}
          onReset={onReset}
          error={error}
        />
      )}

      {/* Results Panel - Show when completed */}
      {isComplete && summary && (
        <ResultsPanel
          summary={summary}
          onViewData={onViewData}
          onDismiss={onReset}
        />
      )}

      {/* History List - Show when idle */}
      {isIdle && (
        <HistoryList
          getHistory={getHistory}
          onSelectEntry={onSelectHistoryEntry}
          onClearHistory={clearHistory}
          disabled={isRunning}
        />
      )}
    </div>
  );
}

// ============================================================================
// MAIN CONTENT
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
  isProcessing: boolean;
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
  isProcessing,
}: ProcessingContentProps) {
  return (
    <div className="flex flex-col gap-6 p-6 h-full overflow-y-auto">
      {/* Dataset Preview */}
      <section>
        <h2 className="text-sm font-semibold mb-3">Dataset Overview</h2>
        <DatasetPreview fileInfo={fileInfo} />
      </section>

      {/* Only show config options if file is loaded */}
      {fileInfo && (
        <>
          {/* Column Selection */}
          <section>
            <h2 className="text-sm font-semibold mb-3">Select Columns</h2>
            <p className="text-xs text-muted-foreground mb-3">
              Choose which columns to include in preprocessing. Leave empty to process all columns.
            </p>
            <div className="border border-border rounded-lg overflow-hidden">
              <ColumnSelector
                columns={columns ?? []}
                selectedColumns={selectedColumns}
                onSelectionChange={onSelectionChange}
                disabled={isProcessing}
              />
            </div>
          </section>

          {/* Row Range Selection */}
          <section>
            <h2 className="text-sm font-semibold mb-3">Row Range</h2>
            <div className="border border-border rounded-lg p-4">
              <RowRangeSelector
                totalRows={fileInfo.row_count}
                rowRange={rowRange}
                onRangeChange={onRowRangeChange}
                disabled={isProcessing}
              />
            </div>
          </section>

          {/* Configuration */}
          <section>
            <h2 className="text-sm font-semibold mb-3">Configuration</h2>
            <div className="border border-border rounded-lg">
              <ConfigPanel
                config={config}
                onConfigChange={onConfigChange}
                columns={columns ?? []}
                hasAIProvider={hasAIProvider}
                disabled={isProcessing}
              />
            </div>
          </section>
        </>
      )}
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
 * - Dataset preview with statistics
 * - Column selection
 * - Row range selection
 * - Preprocessing configuration
 * - Real-time progress tracking
 * - Results summary
 * - Processing history
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
          isFileLoaded={isFileLoaded}
          isProcessing={isProcessing}
          canStart={canStart}
          onStart={handleStart}
        />
      }
      sidebar={
        <ContextSidebar visible={true}>
          <ProcessingSidebar
            status={status}
            progress={progress}
            summary={summary}
            error={error}
            onCancel={cancelPreprocessing}
            onReset={reset}
            onViewData={handleViewData}
            getHistory={getHistory}
            clearHistory={clearHistory}
            onSelectHistoryEntry={handleSelectHistoryEntry}
          />
        </ContextSidebar>
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
        isProcessing={isProcessing}
      />
    </AppShell>
  );
}
