"use client";

import { useState, useMemo, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { toast } from "@/components/ui/toast";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { HistoryList } from "@/components/preprocessing/history-list";
import type {
  PreprocessingSummary,
  PreprocessingAction,
  ColumnSummary,
  ActionType,
  ExportResult,
  PreprocessingHistoryEntry,
} from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export type ResultsTabValue = "results" | "history";

export interface ResultsPanelProps {
  /** The preprocessing summary to display (null if no results yet) */
  summary: PreprocessingSummary | null;
  /** Callback when user wants to view the processed data */
  onViewData?: () => void;
  /** Callback when user wants to export the processed data */
  onExport?: () => void;
  /** Function to fetch history entries */
  getHistory: () => Promise<PreprocessingHistoryEntry[]>;
  /** Callback when a history entry is selected */
  onSelectHistoryEntry?: (entry: PreprocessingHistoryEntry) => void;
  /** Callback to clear all history */
  onClearHistory?: () => Promise<void>;
  /** Whether panel is disabled (e.g., during processing) */
  disabled?: boolean;
  /** Additional class names */
  className?: string;
  /** Controlled active tab value */
  activeTab?: ResultsTabValue;
  /** Callback when active tab changes */
  onActiveTabChange?: (tab: ResultsTabValue) => void;
}

// ============================================================================
// HELPERS
// ============================================================================

/**
 * Format a number with locale-aware formatting.
 */
function formatNumber(num: number): string {
  return num.toLocaleString();
}

/**
 * Format a percentage (0-1 scale).
 */
function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`;
}

/**
 * Get the quality score color.
 */
function getQualityColor(score: number): string {
  return score >= 0.5 ? "text-foreground" : "text-muted-foreground";
}

/**
 * Get quality badge variant.
 */
function getQualityBadgeClass(score: number): string {
  return score >= 0.7 ? "bg-muted text-foreground" : "bg-muted text-muted-foreground";
}

/**
 * Get action type label.
 */
function getActionTypeLabel(type: ActionType): string {
  const labels: Record<ActionType, string> = {
    column_removed: "Column Removed",
    rows_removed: "Rows Removed",
    type_corrected: "Type Corrected",
    value_imputed: "Value Imputed",
    outlier_handled: "Outlier Handled",
    duplicates_removed: "Duplicates Removed",
    target_identified: "Target Identified",
    problem_type_detected: "Problem Detected",
    column_renamed: "Column Renamed",
    value_cleaned: "Value Cleaned",
    data_normalized: "Data Normalized",
    categories_encoded: "Categories Encoded",
  };
  return labels[type] ?? type;
}



// ============================================================================
// SUB-COMPONENTS
// ============================================================================

interface StatCardProps {
  label: string;
  before: number | string;
  after: number | string;
  change?: number;
  formatFn?: (value: number | string) => string;
}

function StatCard({ label, before, after, change, formatFn }: StatCardProps) {
  const format = formatFn ?? ((v) => String(v));
  const hasChange = change !== undefined && change !== 0;
  const showBefore = before !== 0;

  return (
    <div className="flex flex-col gap-1 p-3 rounded-md bg-muted/50">
      <span className="text-xs text-muted-foreground">{label}</span>
      <div className="flex items-baseline gap-2">
        <span className="text-lg font-semibold tabular-nums">{format(after)}</span>
        {hasChange && (
          <span className="text-xs tabular-nums text-muted-foreground">
            ({typeof change === "number" && change > 0 ? "+" : ""}
            {typeof change === "number" ? formatNumber(change) : change})
          </span>
        )}
      </div>
      {showBefore && (
        <span className="text-xs text-muted-foreground">
          was {format(before)}
        </span>
      )}
    </div>
  );
}

interface ActionItemProps {
  action: PreprocessingAction;
}

function ActionItem({ action }: ActionItemProps) {
  return (
    <div className="flex items-start gap-3 py-2 border-b border-border last:border-0">
      <span
        className="mt-0.5 text-xs font-medium px-1.5 py-0.5 rounded bg-muted text-muted-foreground"
      >
        {getActionTypeLabel(action.action_type)}
      </span>
      <div className="flex-1 min-w-0">
        <p className="text-sm">{action.description}</p>
        {action.details && (
          <p className="text-xs text-muted-foreground mt-0.5">{action.details}</p>
        )}
        <p className="text-xs text-muted-foreground mt-0.5">
          Target: <span className="font-mono">{action.target}</span>
        </p>
      </div>
    </div>
  );
}

interface ColumnSummaryItemProps {
  column: ColumnSummary;
}

function ColumnSummaryItem({ column }: ColumnSummaryItemProps) {
  const hasChanges =
    column.missing_before !== column.missing_after ||
    column.type_corrections > 0 ||
    column.outliers_handled > 0 ||
    column.values_cleaned > 0 ||
    column.was_removed;

  return (
    <div
      className={cn(
        "flex flex-col gap-2 p-3 rounded-md border",
        column.was_removed
          ? "border-border bg-muted/30"
          : hasChanges
            ? "border-border bg-muted/30"
            : "border-border/50 bg-transparent"
      )}
    >
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium truncate" title={column.name}>
          {column.name}
        </span>
        {column.was_removed && (
          <span className="text-xs text-muted-foreground px-1.5 py-0.5 rounded bg-muted">
            Removed
          </span>
        )}
      </div>

      {!column.was_removed && (
        <>
          {/* Type info */}
          <div className="flex items-center gap-2 text-xs">
            <span className="text-muted-foreground">Type:</span>
            <span className="font-mono">{column.original_type}</span>
            {column.original_type !== column.final_type && (
              <>
                <span className="text-muted-foreground">→</span>
                <span className="font-mono">{column.final_type}</span>
              </>
            )}
          </div>

          {/* Stats */}
          <div className="grid grid-cols-2 gap-2 text-xs">
            {column.missing_before > 0 && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Missing:</span>
                <span className="tabular-nums">
                  {column.missing_before} → {column.missing_after}
                </span>
              </div>
            )}
            {column.outliers_handled > 0 && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Outliers:</span>
                <span className="tabular-nums">{column.outliers_handled}</span>
              </div>
            )}
            {column.type_corrections > 0 && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Type fixes:</span>
                <span className="tabular-nums">{column.type_corrections}</span>
              </div>
            )}
            {column.values_cleaned > 0 && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Cleaned:</span>
                <span className="tabular-nums">{column.values_cleaned}</span>
              </div>
            )}
          </div>

          {/* Imputation method */}
          {column.imputation_method && (
            <div className="text-xs text-muted-foreground">
              Imputation: <span>{column.imputation_method}</span>
            </div>
          )}
        </>
      )}

      {column.was_removed && column.removal_reason && (
        <p className="text-xs text-muted-foreground">{column.removal_reason}</p>
      )}
    </div>
  );
}

// ============================================================================
// RESULTS CONTENT (when summary exists)
// ============================================================================

interface ResultsContentProps {
  summary: PreprocessingSummary;
  onViewData?: () => void;
  onExport?: () => void;
}

function ResultsContent({ summary, onViewData, onExport }: ResultsContentProps) {
  const [activeTab, setActiveTab] = useState("overview");

  // Handle export button click
  const handleExport = useCallback(async () => {
    if (onExport) {
      onExport();
      return;
    }
    // Default export behavior
    try {
      const result = await invoke<ExportResult>("export_processed_data");
      toast.success(`Exported to ${result.csv_path}`);
    } catch (err) {
      // Silently handle user cancellation
      if (err !== "Export cancelled by user") {
        toast.error(`Export failed: ${err}`);
      }
    }
  }, [onExport]);

  // Calculate derived stats
  const qualityImprovement = summary.data_quality_score_after - summary.data_quality_score_before;

  // Group actions by type for summary
  const actionCounts = useMemo(() => {
    const counts: Partial<Record<ActionType, number>> = {};
    for (const action of summary.actions) {
      counts[action.action_type] = (counts[action.action_type] ?? 0) + 1;
    }
    return counts;
  }, [summary.actions]);

  // Separate removed and modified columns
  const { removedColumns, modifiedColumns } = useMemo(() => {
    const removed: ColumnSummary[] = [];
    const modified: ColumnSummary[] = [];

    for (const col of summary.column_summaries) {
      if (col.was_removed) {
        removed.push(col);
      } else if (
        col.missing_before !== col.missing_after ||
        col.type_corrections > 0 ||
        col.outliers_handled > 0 ||
        col.values_cleaned > 0 ||
        col.original_type !== col.final_type
      ) {
        modified.push(col);
      }
    }

    return { removedColumns: removed, modifiedColumns: modified };
  }, [summary.column_summaries]);

  return (
    <>
      {/* Scrollable content */}
      <div className="flex-1 min-h-0 overflow-y-auto p-3">
        <div className="flex flex-col gap-4">
          {/* Quality Score Highlight */}
          <div className="flex items-center justify-between p-3 rounded-md bg-muted/50">
            <div className="flex flex-col gap-1">
              <span className="text-xs text-muted-foreground">Data Quality Score</span>
              <div className="flex items-baseline gap-3">
                <span className={cn("text-2xl font-bold", getQualityColor(summary.data_quality_score_after))}>
                  {formatPercent(summary.data_quality_score_after)}
                </span>
                {qualityImprovement !== 0 && (
                  <span className="text-sm text-muted-foreground">
                    ({qualityImprovement > 0 ? "+" : ""}
                    {formatPercent(qualityImprovement)})
                  </span>
                )}
              </div>
            </div>
            <div
              className={cn(
                "px-2 py-1 rounded text-xs font-medium",
                getQualityBadgeClass(summary.data_quality_score_after)
              )}
            >
              {summary.data_quality_score_after >= 0.9
                ? "Excellent"
                : summary.data_quality_score_after >= 0.7
                  ? "Good"
                  : summary.data_quality_score_after >= 0.5
                    ? "Fair"
                    : "Poor"}
            </div>
          </div>

          {/* Inner Tabs for Overview/Actions/Columns */}
          <Tabs value={activeTab} onValueChange={setActiveTab}>
            <TabsList className="w-full">
              <TabsTrigger value="overview" className="flex-1 min-w-0 px-2">Overview</TabsTrigger>
              <TabsTrigger value="actions" className="flex-1 min-w-0 px-2">
                Actions ({summary.actions.length})
              </TabsTrigger>
              <TabsTrigger value="columns" className="flex-1 min-w-0 px-2">
                Columns ({summary.column_summaries.length})
              </TabsTrigger>
            </TabsList>

            {/* Overview Tab */}
            <TabsContent value="overview">
              <div className="flex flex-col gap-4">
                {/* Stats Grid */}
                <div className="grid grid-cols-2 gap-3">
                  <StatCard
                    label="Rows"
                    before={summary.rows_before}
                    after={summary.rows_after}
                    change={-summary.rows_removed}
                    formatFn={(v) => formatNumber(Number(v))}
                  />
                  <StatCard
                    label="Columns"
                    before={summary.columns_before}
                    after={summary.columns_after}
                    change={-summary.columns_removed}
                    formatFn={(v) => formatNumber(Number(v))}
                  />
                  <StatCard
                    label="Issues Found"
                    before={0}
                    after={summary.issues_found}
                    formatFn={(v) => formatNumber(Number(v))}
                  />
                  <StatCard
                    label="Issues Resolved"
                    before={0}
                    after={summary.issues_resolved}
                    formatFn={(v) => formatNumber(Number(v))}
                  />
                </div>

                {/* Action Summary */}
                {Object.keys(actionCounts).length > 0 && (
                  <div className="flex flex-col gap-1">
                    {Object.entries(actionCounts).map(([type, count]) => (
                      <span
                        key={type}
                        className="text-xs text-muted-foreground"
                      >
                        • {count} {getActionTypeLabel(type as ActionType).toLowerCase()}
                      </span>
                    ))}
                  </div>
                )}

                {/* Warnings */}
                {summary.warnings.length > 0 && (
                  <div className="flex flex-col gap-2 p-3 rounded-md border border-border bg-muted/30">
                    <span className="text-xs font-medium text-muted-foreground">
                      Warnings ({summary.warnings.length})
                    </span>
                    <ul className="text-xs text-muted-foreground space-y-1">
                      {summary.warnings.map((warning, idx) => (
                        <li key={idx} className="flex items-start gap-2">
                          <span>•</span>
                          {warning}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            </TabsContent>

            {/* Actions Tab */}
            <TabsContent value="actions">
              <div className="flex flex-col">
                {summary.actions.length === 0 ? (
                  <p className="text-sm text-muted-foreground text-center py-4">
                    No actions were taken
                  </p>
                ) : (
                  summary.actions.map((action, idx) => (
                    <ActionItem key={idx} action={action} />
                  ))
                )}
              </div>
            </TabsContent>

            {/* Columns Tab */}
            <TabsContent value="columns">
              <div className="flex flex-col gap-4">
                {/* Removed columns */}
                {removedColumns.length > 0 && (
                  <div className="flex flex-col gap-2">
                    <span className="text-xs font-medium text-muted-foreground">
                      Removed ({removedColumns.length})
                    </span>
                    <div className="grid grid-cols-1 gap-2">
                      {removedColumns.map((col) => (
                        <ColumnSummaryItem key={col.name} column={col} />
                      ))}
                    </div>
                  </div>
                )}

                {/* Modified columns */}
                {modifiedColumns.length > 0 && (
                  <div className="flex flex-col gap-2">
                    <span className="text-xs font-medium text-muted-foreground">
                      Modified ({modifiedColumns.length})
                    </span>
                    <div className="grid grid-cols-1 gap-2">
                      {modifiedColumns.map((col) => (
                        <ColumnSummaryItem key={col.name} column={col} />
                      ))}
                    </div>
                  </div>
                )}

                {removedColumns.length === 0 && modifiedColumns.length === 0 && (
                  <p className="text-sm text-muted-foreground text-center py-4">
                    No columns were modified
                  </p>
                )}
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </div>

      {/* Footer Actions - fixed at bottom */}
      <div className="flex items-center gap-2 p-3 border-t border-border shrink-0">
        {onViewData && (
          <Button variant="default" size="sm" onClick={onViewData}>
            View Data
          </Button>
        )}
        <Button variant="outline" size="sm" onClick={handleExport}>
          Export
        </Button>
      </div>
    </>
  );
}

// ============================================================================
// EMPTY RESULTS STATE
// ============================================================================

function EmptyResultsState() {
  return (
    <div className="flex-1 flex flex-col items-center justify-center p-6 text-center">
      <p className="text-sm font-medium text-muted-foreground">No results yet</p>
      <p className="text-xs text-muted-foreground mt-1">
        Run preprocessing to see results
      </p>
    </div>
  );
}

// ============================================================================
// RESULTS PANEL COMPONENT
// ============================================================================

/**
 * A panel with two tabs: Results and History.
 *
 * - Results tab: Shows the last preprocessing result summary, or empty state
 * - History tab: Shows preprocessing history with selectable entries
 *
 * Auto-switches to Results tab when a new result comes in.
 *
 * @example
 * ```tsx
 * <ResultsPanel
 *   summary={summary}
 *   onViewData={() => router.push("/data")}
 *   getHistory={getHistory}
 *   onSelectHistoryEntry={handleSelectEntry}
 *   onClearHistory={clearHistory}
 * />
 * ```
 */
export function ResultsPanel({
  summary,
  onViewData,
  onExport,
  getHistory,
  onSelectHistoryEntry,
  onClearHistory,
  disabled = false,
  className,
  activeTab: controlledActiveTab,
  onActiveTabChange,
}: ResultsPanelProps) {
  // Internal state for uncontrolled mode
  const [internalActiveTab, setInternalActiveTab] = useState<ResultsTabValue>("history");
  
  // Use controlled value if provided, otherwise use internal state
  const activeTab = controlledActiveTab ?? internalActiveTab;
  
  // Handler that notifies parent and updates internal state
  const handleTabChange = useCallback((tab: ResultsTabValue) => {
    setInternalActiveTab(tab);
    onActiveTabChange?.(tab);
  }, [onActiveTabChange]);

  return (
    <div
      className={cn(
        "h-full min-h-0 flex flex-col border rounded-lg overflow-hidden",
        className
      )}
      data-slot="results-panel"
    >
      {/* Header with tabs */}
      <div className="px-3 py-2 border-b bg-muted/30 shrink-0">
        <div className="grid grid-cols-2">
          <button
            type="button"
            onClick={() => handleTabChange("results")}
            className={cn(
              "text-xs font-semibold uppercase tracking-wider transition-colors text-center",
              activeTab === "results"
                ? "text-foreground"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            Results
          </button>
          <button
            type="button"
            onClick={() => handleTabChange("history")}
            className={cn(
              "text-xs font-semibold uppercase tracking-wider transition-colors text-center",
              activeTab === "history"
                ? "text-foreground"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            History
          </button>
        </div>
      </div>

      {/* Tab Content */}
      {activeTab === "results" ? (
        summary ? (
          <ResultsContent
            summary={summary}
            onViewData={onViewData}
            onExport={onExport}
          />
        ) : (
          <EmptyResultsState />
        )
      ) : (
        <div className="flex-1 min-h-0 overflow-y-auto p-3">
          <HistoryList
            getHistory={getHistory}
            onSelectEntry={onSelectHistoryEntry}
            onClearHistory={onClearHistory}
            disabled={disabled}
            className="h-full"
          />
        </div>
      )}
    </div>
  );
}

export default ResultsPanel;
