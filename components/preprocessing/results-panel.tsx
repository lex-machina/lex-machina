"use client";

import { useState, useMemo, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { toast } from "@/components/ui/toast";
import type {
  PreprocessingSummary,
  PreprocessingAction,
  ColumnSummary,
  ActionType,
  ExportResult,
} from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface ResultsPanelProps {
  /** The preprocessing summary to display */
  summary: PreprocessingSummary;
  /** Callback when user wants to view the processed data */
  onViewData?: () => void;
  /** Callback when user wants to export the processed data */
  onExport?: () => void;
  /** Callback to dismiss/close the panel */
  onDismiss?: () => void;
  /** Additional class names */
  className?: string;
}

// ============================================================================
// HELPERS
// ============================================================================

/**
 * Format duration in human-readable format.
 */
function formatDuration(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`;
  }
  const seconds = ms / 1000;
  if (seconds < 60) {
    return `${seconds.toFixed(1)}s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.round(seconds % 60);
  return `${minutes}m ${remainingSeconds}s`;
}

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
  if (score >= 0.9) return "text-green-500";
  if (score >= 0.7) return "text-yellow-500";
  if (score >= 0.5) return "text-orange-500";
  return "text-red-500";
}

/**
 * Get quality badge variant.
 */
function getQualityBadgeClass(score: number): string {
  if (score >= 0.9) return "bg-green-500/20 text-green-400 border-green-500/30";
  if (score >= 0.7) return "bg-yellow-500/20 text-yellow-400 border-yellow-500/30";
  if (score >= 0.5) return "bg-orange-500/20 text-orange-400 border-orange-500/30";
  return "bg-red-500/20 text-red-400 border-red-500/30";
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

/**
 * Get action type icon color.
 */
function getActionTypeColor(type: ActionType): string {
  switch (type) {
    case "column_removed":
    case "rows_removed":
      return "text-red-400";
    case "type_corrected":
    case "value_cleaned":
      return "text-blue-400";
    case "value_imputed":
      return "text-purple-400";
    case "outlier_handled":
      return "text-orange-400";
    case "duplicates_removed":
      return "text-yellow-400";
    case "target_identified":
    case "problem_type_detected":
      return "text-green-400";
    default:
      return "text-muted-foreground";
  }
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
  const isPositive = change !== undefined && change > 0;
  const isNegative = change !== undefined && change < 0;

  return (
    <div className="flex flex-col gap-1 p-3 rounded-md bg-muted/50">
      <span className="text-xs text-muted-foreground">{label}</span>
      <div className="flex items-baseline gap-2">
        <span className="text-lg font-semibold tabular-nums">{format(after)}</span>
        {hasChange && (
          <span
            className={cn(
              "text-xs tabular-nums",
              isPositive && "text-green-500",
              isNegative && "text-red-500"
            )}
          >
            {isPositive ? "+" : ""}
            {typeof change === "number" ? formatNumber(change) : change}
          </span>
        )}
      </div>
      <span className="text-xs text-muted-foreground">
        was {format(before)}
      </span>
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
        className={cn(
          "mt-0.5 text-xs font-medium px-1.5 py-0.5 rounded",
          getActionTypeColor(action.action_type),
          "bg-current/10"
        )}
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
          ? "border-red-500/30 bg-red-500/5"
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
          <span className="text-xs text-red-400 px-1.5 py-0.5 rounded bg-red-500/10">
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
                <span className="font-mono text-blue-400">{column.final_type}</span>
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
              Imputation: <span className="text-purple-400">{column.imputation_method}</span>
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
// RESULTS PANEL COMPONENT
// ============================================================================

/**
 * A panel displaying the preprocessing results summary.
 *
 * Shows before/after statistics, quality scores, actions taken,
 * and per-column summaries.
 *
 * @example
 * ```tsx
 * const { summary } = usePreprocessing();
 *
 * {summary && (
 *   <ResultsPanel
 *     summary={summary}
 *     onViewData={() => router.push("/data")}
 *     onDismiss={reset}
 *   />
 * )}
 * ```
 */
export function ResultsPanel({
  summary,
  onViewData,
  onExport,
  onDismiss,
  className,
}: ResultsPanelProps) {
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
    <div
      className={cn(
        "flex flex-col gap-4 p-4 rounded-lg border border-border bg-card",
        className
      )}
      data-slot="results-panel"
    >
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <svg
            className="h-5 w-5 text-green-500"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
              clipRule="evenodd"
            />
          </svg>
          <span className="text-sm font-medium">Preprocessing Complete</span>
        </div>
        <span className="text-xs text-muted-foreground">
          {formatDuration(summary.duration_ms)}
        </span>
      </div>

      {/* Quality Score Highlight */}
      <div className="flex items-center justify-between p-3 rounded-md bg-muted/50">
        <div className="flex flex-col gap-1">
          <span className="text-xs text-muted-foreground">Data Quality Score</span>
          <div className="flex items-baseline gap-3">
            <span className={cn("text-2xl font-bold", getQualityColor(summary.data_quality_score_after))}>
              {formatPercent(summary.data_quality_score_after)}
            </span>
            {qualityImprovement !== 0 && (
              <span
                className={cn(
                  "text-sm",
                  qualityImprovement > 0 ? "text-green-500" : "text-red-500"
                )}
              >
                {qualityImprovement > 0 ? "+" : ""}
                {formatPercent(qualityImprovement)}
              </span>
            )}
          </div>
        </div>
        <div
          className={cn(
            "px-2 py-1 rounded border text-xs font-medium",
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

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="actions">
            Actions ({summary.actions.length})
          </TabsTrigger>
          <TabsTrigger value="columns">
            Columns ({summary.column_summaries.length})
          </TabsTrigger>
        </TabsList>

        {/* Overview Tab */}
        <TabsContent value="overview" className="mt-4">
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
              <div className="flex flex-wrap gap-2">
                {Object.entries(actionCounts).map(([type, count]) => (
                  <span
                    key={type}
                    className={cn(
                      "text-xs px-2 py-1 rounded",
                      getActionTypeColor(type as ActionType),
                      "bg-current/10"
                    )}
                  >
                    {count} {getActionTypeLabel(type as ActionType).toLowerCase()}
                  </span>
                ))}
              </div>
            )}

            {/* Warnings */}
            {summary.warnings.length > 0 && (
              <div className="flex flex-col gap-2 p-3 rounded-md border border-yellow-500/30 bg-yellow-500/5">
                <span className="text-xs font-medium text-yellow-500">
                  Warnings ({summary.warnings.length})
                </span>
                <ul className="text-xs text-muted-foreground space-y-1">
                  {summary.warnings.map((warning, idx) => (
                    <li key={idx} className="flex items-start gap-2">
                      <span className="text-yellow-500">•</span>
                      {warning}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </TabsContent>

        {/* Actions Tab */}
        <TabsContent value="actions" className="mt-4">
          <div className="flex flex-col max-h-[300px] overflow-y-auto">
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
        <TabsContent value="columns" className="mt-4">
          <div className="flex flex-col gap-4 max-h-[300px] overflow-y-auto">
            {/* Removed columns */}
            {removedColumns.length > 0 && (
              <div className="flex flex-col gap-2">
                <span className="text-xs font-medium text-red-400">
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

      {/* Actions */}
      <div className="flex items-center gap-2 pt-2 border-t border-border">
        {onViewData && (
          <Button variant="default" size="sm" onClick={onViewData}>
            View Processed Data
          </Button>
        )}
        <Button variant="outline" size="sm" onClick={handleExport}>
          Export CSV
        </Button>
        {onDismiss && (
          <Button variant="ghost" size="sm" onClick={onDismiss}>
            Dismiss
          </Button>
        )}
      </div>
    </div>
  );
}

export default ResultsPanel;
