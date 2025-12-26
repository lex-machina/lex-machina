"use client";

import { useState, useEffect, useCallback } from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import type { PreprocessingHistoryEntry } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface HistoryListProps {
  /** Function to fetch history entries */
  getHistory: () => Promise<PreprocessingHistoryEntry[]>;
  /** Callback when a history entry is selected */
  onSelectEntry?: (entry: PreprocessingHistoryEntry) => void;
  /** Callback to clear all history */
  onClearHistory?: () => Promise<void>;
  /** Whether the list is disabled */
  disabled?: boolean;
  /** Additional class names */
  className?: string;
}

// ============================================================================
// HELPERS
// ============================================================================

/**
 * Format a Unix timestamp to a human-readable date/time.
 */
function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  // Relative time for recent entries
  if (diffMins < 1) return "Just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;

  // Absolute date for older entries
  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/**
 * Format duration in human-readable format.
 */
function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const seconds = ms / 1000;
  if (seconds < 60) return `${seconds.toFixed(1)}s`;
  const minutes = Math.floor(seconds / 60);
  return `${minutes}m ${Math.round(seconds % 60)}s`;
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
 * Get quality score color class.
 */
function getQualityColorClass(score: number): string {
  if (score >= 0.9) return "text-green-500";
  if (score >= 0.7) return "text-yellow-500";
  if (score >= 0.5) return "text-orange-500";
  return "text-red-500";
}

// ============================================================================
// HISTORY ENTRY ITEM
// ============================================================================

interface HistoryEntryItemProps {
  entry: PreprocessingHistoryEntry;
  onSelect?: (entry: PreprocessingHistoryEntry) => void;
  disabled?: boolean;
}

function HistoryEntryItem({ entry, onSelect, disabled }: HistoryEntryItemProps) {
  const { summary, config, timestamp } = entry;
  const qualityImprovement = summary.data_quality_score_after - summary.data_quality_score_before;

  const handleClick = () => {
    if (!disabled && onSelect) {
      onSelect(entry);
    }
  };

  return (
    <button
      type="button"
      onClick={handleClick}
      disabled={disabled || !onSelect}
      className={cn(
        "w-full flex flex-col gap-2 p-3 rounded-md border border-border",
        "text-left transition-colors",
        "hover:bg-accent/50 hover:border-accent",
        "focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
        "disabled:opacity-50 disabled:pointer-events-none"
      )}
    >
      {/* Header row */}
      <div className="flex items-center justify-between">
        <span className="text-xs text-muted-foreground">
          {formatTimestamp(timestamp)}
        </span>
        <span className="text-xs text-muted-foreground">
          {formatDuration(summary.duration_ms)}
        </span>
      </div>

      {/* Quality score */}
      <div className="flex items-center gap-2">
        <span className="text-sm font-medium">Quality:</span>
        <span className={cn("text-sm font-semibold", getQualityColorClass(summary.data_quality_score_after))}>
          {formatPercent(summary.data_quality_score_after)}
        </span>
        {qualityImprovement !== 0 && (
          <span
            className={cn(
              "text-xs",
              qualityImprovement > 0 ? "text-green-500" : "text-red-500"
            )}
          >
            ({qualityImprovement > 0 ? "+" : ""}{formatPercent(qualityImprovement)})
          </span>
        )}
      </div>

      {/* Stats row */}
      <div className="flex items-center gap-4 text-xs text-muted-foreground">
        <span>
          {formatNumber(summary.rows_after)} rows
          {summary.rows_removed > 0 && (
            <span className="text-red-400"> (-{formatNumber(summary.rows_removed)})</span>
          )}
        </span>
        <span>
          {formatNumber(summary.columns_after)} cols
          {summary.columns_removed > 0 && (
            <span className="text-red-400"> (-{formatNumber(summary.columns_removed)})</span>
          )}
        </span>
        <span>
          {formatNumber(summary.issues_resolved)}/{formatNumber(summary.issues_found)} issues
        </span>
      </div>

      {/* Config summary */}
      <div className="flex flex-wrap gap-1.5">
        {config.use_ai_decisions && (
          <span className="text-xs px-1.5 py-0.5 rounded bg-purple-500/20 text-purple-400">
            AI
          </span>
        )}
        <span className="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
          {config.outlier_strategy}
        </span>
        <span className="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
          {config.numeric_imputation}
        </span>
        {config.selected_columns.length > 0 && (
          <span className="text-xs px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-400">
            {config.selected_columns.length} cols selected
          </span>
        )}
      </div>
    </button>
  );
}

// ============================================================================
// HISTORY LIST COMPONENT
// ============================================================================

/**
 * A list component showing preprocessing history entries.
 *
 * Displays previous preprocessing runs with their configuration and results.
 * Allows selecting an entry to view details or load that result.
 *
 * @example
 * ```tsx
 * const { getHistory, clearHistory } = usePreprocessing();
 *
 * <HistoryList
 *   getHistory={getHistory}
 *   onSelectEntry={(entry) => console.log("Selected:", entry)}
 *   onClearHistory={clearHistory}
 * />
 * ```
 */
export function HistoryList({
  getHistory,
  onSelectEntry,
  onClearHistory,
  disabled = false,
  className,
}: HistoryListProps) {
  const [entries, setEntries] = useState<PreprocessingHistoryEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Fetch history on mount
  const fetchHistory = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const history = await getHistory();
      setEntries(history);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load history");
    } finally {
      setIsLoading(false);
    }
  }, [getHistory]);

  useEffect(() => {
    fetchHistory();
  }, [fetchHistory]);

  // Handle clear history
  const handleClearHistory = async () => {
    if (!onClearHistory) return;

    try {
      await onClearHistory();
      setEntries([]);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to clear history");
    }
  };

  // Loading state
  if (isLoading) {
    return (
      <div
        className={cn(
          "flex items-center justify-center p-6",
          "text-muted-foreground text-sm",
          className
        )}
      >
        <svg
          className="h-4 w-4 animate-spin mr-2"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
        >
          <circle
            className="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            strokeWidth="4"
          />
          <path
            className="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          />
        </svg>
        Loading history...
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div
        className={cn(
          "flex flex-col items-center justify-center gap-2 p-6",
          "text-sm",
          className
        )}
      >
        <p className="text-destructive">{error}</p>
        <Button variant="outline" size="sm" onClick={fetchHistory}>
          Retry
        </Button>
      </div>
    );
  }

  // Empty state
  if (entries.length === 0) {
    return (
      <div
        className={cn(
          "flex flex-col items-center justify-center p-6",
          "text-muted-foreground text-sm",
          className
        )}
      >
        <svg
          className="h-8 w-8 mb-2 opacity-50"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
        <p>No preprocessing history</p>
        <p className="text-xs mt-1">Run preprocessing to see results here</p>
      </div>
    );
  }

  return (
    <div className={cn("flex flex-col gap-3", className)} data-slot="history-list">
      {/* Header */}
      <div className="flex items-center justify-between px-1">
        <span className="text-sm font-medium">
          History ({entries.length})
        </span>
        {onClearHistory && (
          <Button
            variant="ghost"
            size="sm"
            onClick={handleClearHistory}
            disabled={disabled}
            className="text-xs h-7"
          >
            Clear All
          </Button>
        )}
      </div>

      {/* Entries list */}
      <div className="flex flex-col gap-2 max-h-[400px] overflow-y-auto">
        {entries.map((entry) => (
          <HistoryEntryItem
            key={entry.id}
            entry={entry}
            onSelect={onSelectEntry}
            disabled={disabled}
          />
        ))}
      </div>

      {/* Footer hint */}
      <p className="text-xs text-muted-foreground text-center">
        Click an entry to view details
      </p>
    </div>
  );
}

export default HistoryList;
