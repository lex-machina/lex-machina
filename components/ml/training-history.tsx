"use client";

import { useEffect, useState } from "react";
import { Clock, Trash2 } from "lucide-react";

import type { TrainingHistoryEntry } from "@/types";
import { Button } from "@/components/ui/button";
import { cn, formatNumber } from "@/lib/utils";

interface TrainingHistoryProps {
    entries: TrainingHistoryEntry[];
    onRefresh: () => Promise<TrainingHistoryEntry[]>;
    onClear: () => Promise<void>;
    onSelect: (entry: TrainingHistoryEntry) => void;
    disabled?: boolean;
}

function formatTimestamp(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString(undefined, {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
    });
}

export function TrainingHistory({
    entries,
    onRefresh,
    onClear,
    onSelect,
    disabled = false,
}: TrainingHistoryProps) {
    const [history, setHistory] = useState(entries);
    const [isLoading, setIsLoading] = useState(false);

    useEffect(() => {
        setHistory(entries);
    }, [entries]);

    useEffect(() => {
        setIsLoading(true);
        onRefresh()
            .then((updated) => {
                setHistory(updated);
            })
            .finally(() => setIsLoading(false));
    }, [onRefresh]);

    return (
        <div className="flex h-full flex-col">
            <div className="flex items-center justify-between border-b px-3 py-2">
                <div className="text-muted-foreground flex items-center gap-2 text-xs">
                    <Clock className="h-3.5 w-3.5" />
                    Training history
                </div>
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onClear}
                    disabled={disabled || history.length === 0}
                    className="h-6 px-2 text-xs"
                >
                    <Trash2 className="h-3.5 w-3.5" />
                    Clear
                </Button>
            </div>
            <div className="flex-1 overflow-y-auto p-3">
                {isLoading && (
                    <p className="text-muted-foreground text-xs">Loading...</p>
                )}
                {history.length === 0 && !isLoading && (
                    <p className="text-muted-foreground text-xs">
                        No training history yet.
                    </p>
                )}
                <div className="flex flex-col gap-2">
                    {history.map((entry) => (
                        <button
                            key={entry.id}
                            type="button"
                            onClick={() => onSelect(entry)}
                            disabled={disabled}
                            className={cn(
                                "border-border flex flex-col gap-2 rounded-md border px-3 py-2 text-left",
                                "hover:bg-muted/40 transition-colors",
                                disabled && "opacity-60",
                            )}
                        >
                            <div className="text-muted-foreground flex items-center justify-between text-xs">
                                <span>{formatTimestamp(entry.timestamp)}</span>
                                <span>
                                    {entry.result_summary.best_model_name}
                                </span>
                            </div>
                            <div className="flex items-center justify-between text-sm">
                                <span className="font-medium">
                                    {formatNumber(
                                        entry.result_summary.test_score,
                                    )}
                                </span>
                                <span className="text-muted-foreground text-xs">
                                    {entry.config.problem_type}
                                </span>
                            </div>
                        </button>
                    ))}
                </div>
            </div>
        </div>
    );
}
