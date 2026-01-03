"use client";

import { useState, useEffect, useCallback } from "react";
import { Loader2 } from "lucide-react";
import { cn, formatNumber, formatPercent, formatDuration } from "@/lib/utils";
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
 * Get quality score color class - uses muted styling for consistency.
 */
function getQualityColorClass(score: number): string {
    return score >= 0.5 ? "text-foreground" : "text-muted-foreground";
}

// ============================================================================
// HISTORY ENTRY ITEM
// ============================================================================

interface HistoryEntryItemProps {
    entry: PreprocessingHistoryEntry;
    onSelect?: (entry: PreprocessingHistoryEntry) => void;
    disabled?: boolean;
}

function HistoryEntryItem({
    entry,
    onSelect,
    disabled,
}: HistoryEntryItemProps) {
    const { summary, config, timestamp } = entry;
    const qualityImprovement =
        summary.data_quality_score_after - summary.data_quality_score_before;

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
                "border-border flex w-full flex-col gap-2 rounded-md border p-3",
                "text-left transition-colors",
                "hover:bg-accent/50 hover:border-accent",
                "focus:ring-ring focus:ring-2 focus:ring-offset-2 focus:outline-none",
                "disabled:pointer-events-none disabled:opacity-50",
            )}
        >
            {/* Header row */}
            <div className="flex items-center justify-between">
                <span className="text-muted-foreground text-xs">
                    {formatTimestamp(timestamp)}
                </span>
                <span className="text-muted-foreground text-xs">
                    {formatDuration(summary.duration_ms)}
                </span>
            </div>

            {/* Quality score */}
            <div className="flex items-center gap-2">
                <span className="text-sm font-medium">Quality:</span>
                <span
                    className={cn(
                        "text-sm font-semibold",
                        getQualityColorClass(summary.data_quality_score_after),
                    )}
                >
                    {formatPercent(summary.data_quality_score_after)}
                </span>
                {qualityImprovement !== 0 && (
                    <span className="text-muted-foreground text-xs">
                        ({qualityImprovement > 0 ? "+" : ""}
                        {formatPercent(qualityImprovement)})
                    </span>
                )}
            </div>

            {/* Stats row */}
            <div className="text-muted-foreground flex items-center gap-4 text-xs">
                <span>
                    {formatNumber(summary.rows_after)} rows
                    {summary.rows_removed > 0 && (
                        <span> (-{formatNumber(summary.rows_removed)})</span>
                    )}
                </span>
                <span>
                    {formatNumber(summary.columns_after)} cols
                    {summary.columns_removed > 0 && (
                        <span> (-{formatNumber(summary.columns_removed)})</span>
                    )}
                </span>
                <span>
                    {formatNumber(summary.issues_resolved)}/
                    {formatNumber(summary.issues_found)} issues
                </span>
            </div>

            {/* Config summary */}
            <div className="flex flex-wrap gap-1.5">
                {config.use_ai_decisions && (
                    <span className="bg-muted text-muted-foreground rounded px-1.5 py-0.5 text-xs">
                        AI
                    </span>
                )}
                <span className="bg-muted text-muted-foreground rounded px-1.5 py-0.5 text-xs">
                    {config.outlier_strategy}
                </span>
                <span className="bg-muted text-muted-foreground rounded px-1.5 py-0.5 text-xs">
                    {config.numeric_imputation}
                </span>
                {config.selected_columns.length > 0 && (
                    <span className="bg-muted text-muted-foreground rounded px-1.5 py-0.5 text-xs">
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
            setError(
                err instanceof Error ? err.message : "Failed to load history",
            );
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
            setError(
                err instanceof Error ? err.message : "Failed to clear history",
            );
        }
    };

    // Loading state
    if (isLoading) {
        return (
            <div
                className={cn(
                    "flex items-center justify-center p-6",
                    "text-muted-foreground text-sm",
                    className,
                )}
            >
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
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
                    className,
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
                    "flex flex-col items-center justify-center p-6 text-center",
                    className,
                )}
            >
                <p className="text-muted-foreground text-sm font-medium">
                    No preprocessing history
                </p>
                <p className="text-muted-foreground mt-1 text-xs">
                    Run preprocessing to see results here
                </p>
            </div>
        );
    }

    return (
        <div
            className={cn("flex flex-col gap-3", className)}
            data-slot="history-list"
        >
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
                        className="h-7 text-xs"
                    >
                        Clear All
                    </Button>
                )}
            </div>

            {/* Entries list */}
            <div className="flex max-h-[400px] flex-col gap-2 overflow-y-auto">
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
            <p className="text-muted-foreground text-center text-xs">
                Click an entry to view details
            </p>
        </div>
    );
}

export default HistoryList;
