"use client";

import { Loader2, CheckCircle2, XCircle, StopCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { ProgressBar, StageProgress } from "@/components/ui/progress-bar";
import type { ProgressUpdate, PreprocessingStage } from "@/types";
import type { PreprocessingStatus } from "@/lib/hooks/use-preprocessing";

// ============================================================================
// TYPES
// ============================================================================

export interface ProgressPanelProps {
    /** Current preprocessing status */
    status: PreprocessingStatus;
    /** Current progress update (null when idle) */
    progress: ProgressUpdate | null;
    /** Callback to cancel preprocessing */
    onCancel: () => void;
    /** Callback to reset/dismiss the panel */
    onReset?: () => void;
    /** Error message if status is "error" */
    error?: string | null;
    /** Elapsed time in milliseconds (optional, managed by parent) */
    elapsedMs?: number;
    /** Additional class names */
    className?: string;
}

// ============================================================================
// HELPERS
// ============================================================================

/**
 * Human-readable labels for preprocessing stages.
 */
const STAGE_LABELS: Record<PreprocessingStage, string> = {
    initializing: "Initializing",
    profiling: "Profiling Dataset",
    quality_analysis: "Analyzing Quality",
    type_correction: "Correcting Types",
    decision_making: "Making Decisions",
    cleaning: "Cleaning Data",
    imputation: "Imputing Values",
    outlier_handling: "Handling Outliers",
    report_generation: "Generating Report",
    complete: "Complete",
    cancelled: "Cancelled",
    failed: "Failed",
};

/**
 * Get a user-friendly label for a stage.
 */
function getStageLabel(stage: PreprocessingStage): string {
    return STAGE_LABELS[stage] ?? stage;
}

/**
 * Format elapsed time in human-readable format.
 */
function formatElapsedTime(elapsedMs: number): string {
    const seconds = Math.floor(elapsedMs / 1000);
    const minutes = Math.floor(seconds / 60);

    if (minutes > 0) {
        const remainingSeconds = seconds % 60;
        return `${minutes}m ${remainingSeconds}s`;
    }
    return `${seconds}s`;
}

/**
 * Get status color class.
 */
function getStatusColorClass(status: PreprocessingStatus): string {
    switch (status) {
        case "completed":
            return "text-foreground";
        case "error":
            return "text-muted-foreground";
        case "cancelled":
            return "text-muted-foreground";
        default:
            return "text-muted-foreground";
    }
}

/**
 * Get status icon.
 */
function StatusIcon({ status }: { status: PreprocessingStatus }) {
    switch (status) {
        case "running":
            return <Loader2 className="h-4 w-4 animate-spin" />;
        case "completed":
            return <CheckCircle2 className="h-4 w-4" />;
        case "error":
            return <XCircle className="h-4 w-4" />;
        case "cancelled":
            return <StopCircle className="h-4 w-4" />;
        default:
            return null;
    }
}

// ============================================================================
// PROGRESS PANEL COMPONENT
// ============================================================================

/**
 * A panel showing preprocessing progress with cancel button.
 *
 * Displays real-time progress updates including stage, sub-stage,
 * and detailed messages. Shows appropriate status after completion.
 *
 * @example
 * ```tsx
 * const { status, progress, cancelPreprocessing, reset, error } = usePreprocessing();
 *
 * <ProgressPanel
 *   status={status}
 *   progress={progress}
 *   onCancel={cancelPreprocessing}
 *   onReset={reset}
 *   error={error}
 * />
 * ```
 */
export function ProgressPanel({
    status,
    progress,
    onCancel,
    onReset,
    error,
    elapsedMs,
    className,
}: ProgressPanelProps) {
    // Don't render if idle
    if (status === "idle") {
        return null;
    }

    const isRunning = status === "running";
    const isComplete = status === "completed";
    const isError = status === "error";
    const isCancelled = status === "cancelled";
    const isDone = isComplete || isError || isCancelled;

    // Get current stage info
    const currentStage = progress?.stage ?? "initializing";
    const stageLabel = getStageLabel(currentStage);
    const subStage = progress?.sub_stage;
    const message = progress?.message ?? "";
    const overallProgress = progress?.progress ?? 0;
    const stageProgress = progress?.stage_progress ?? 0;
    const itemsProcessed = progress?.items_processed;
    const itemsTotal = progress?.items_total;

    return (
        <div
            className={cn(
                "border-border bg-card flex flex-col gap-4 rounded-lg border p-4",
                className,
            )}
            data-slot="progress-panel"
        >
            {/* Header */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <StatusIcon status={status} />
                    <span
                        className={cn(
                            "text-sm font-medium",
                            getStatusColorClass(status),
                        )}
                    >
                        {isRunning
                            ? "Processing..."
                            : isComplete
                              ? "Complete"
                              : isError
                                ? "Error"
                                : isCancelled
                                  ? "Cancelled"
                                  : "Idle"}
                    </span>
                </div>
                {elapsedMs !== undefined && isRunning && (
                    <span className="text-muted-foreground text-xs tabular-nums">
                        Elapsed: {formatElapsedTime(elapsedMs)}
                    </span>
                )}
            </div>

            {/* Progress display */}
            {isRunning && progress && (
                <StageProgress
                    overallProgress={overallProgress}
                    stageProgress={stageProgress}
                    stageName={stageLabel}
                    subStage={subStage}
                    message={message}
                />
            )}

            {/* Items counter */}
            {isRunning &&
                itemsProcessed !== undefined &&
                itemsTotal !== undefined && (
                    <div className="text-muted-foreground flex items-center justify-between text-xs">
                        <span>Items processed</span>
                        <span className="tabular-nums">
                            {itemsProcessed.toLocaleString()} /{" "}
                            {itemsTotal.toLocaleString()}
                        </span>
                    </div>
                )}

            {/* Completion message */}
            {isComplete && (
                <div className="flex flex-col gap-2">
                    <ProgressBar value={100} variant="default" size="default" />
                    <p className="text-muted-foreground text-sm">
                        Preprocessing completed successfully.
                    </p>
                </div>
            )}

            {/* Error message */}
            {isError && error && (
                <div className="flex flex-col gap-2">
                    <ProgressBar
                        value={overallProgress * 100}
                        variant="default"
                        size="default"
                    />
                    <p className="text-muted-foreground text-sm break-words">
                        {error}
                    </p>
                </div>
            )}

            {/* Cancelled message */}
            {isCancelled && (
                <div className="flex flex-col gap-2">
                    <ProgressBar
                        value={overallProgress * 100}
                        variant="default"
                        size="default"
                    />
                    <p className="text-muted-foreground text-sm">
                        Preprocessing was cancelled at{" "}
                        {Math.round(overallProgress * 100)}%
                    </p>
                </div>
            )}

            {/* Actions */}
            <div className="border-border flex items-center gap-2 border-t pt-2">
                {isRunning && (
                    <Button variant="destructive" size="sm" onClick={onCancel}>
                        Cancel
                    </Button>
                )}
                {isDone && onReset && (
                    <Button variant="outline" size="sm" onClick={onReset}>
                        Dismiss
                    </Button>
                )}
            </div>
        </div>
    );
}

export default ProgressPanel;
