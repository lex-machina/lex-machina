"use client";

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
      return "text-green-500";
    case "error":
      return "text-destructive";
    case "cancelled":
      return "text-yellow-500";
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
      return (
        <svg
          className="h-4 w-4 animate-spin"
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
      );
    case "completed":
      return (
        <svg
          className="h-4 w-4 text-green-500"
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
      );
    case "error":
      return (
        <svg
          className="h-4 w-4 text-destructive"
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
        >
          <path
            fillRule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clipRule="evenodd"
          />
        </svg>
      );
    case "cancelled":
      return (
        <svg
          className="h-4 w-4 text-yellow-500"
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
        >
          <path
            fillRule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8 7a1 1 0 00-1 1v4a1 1 0 001 1h4a1 1 0 001-1V8a1 1 0 00-1-1H8z"
            clipRule="evenodd"
          />
        </svg>
      );
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
        "flex flex-col gap-4 p-4 rounded-lg border border-border bg-card",
        className
      )}
      data-slot="progress-panel"
    >
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <StatusIcon status={status} />
          <span className={cn("text-sm font-medium", getStatusColorClass(status))}>
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
          <span className="text-xs text-muted-foreground tabular-nums">
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
      {isRunning && itemsProcessed !== undefined && itemsTotal !== undefined && (
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span>Items processed</span>
          <span className="tabular-nums">
            {itemsProcessed.toLocaleString()} / {itemsTotal.toLocaleString()}
          </span>
        </div>
      )}

      {/* Completion message */}
      {isComplete && (
        <div className="flex flex-col gap-2">
          <ProgressBar value={100} variant="success" size="default" />
          <p className="text-sm text-green-500">
            Preprocessing completed successfully!
          </p>
        </div>
      )}

      {/* Error message */}
      {isError && error && (
        <div className="flex flex-col gap-2">
          <ProgressBar
            value={overallProgress * 100}
            variant="error"
            size="default"
          />
          <p className="text-sm text-destructive break-words">{error}</p>
        </div>
      )}

      {/* Cancelled message */}
      {isCancelled && (
        <div className="flex flex-col gap-2">
          <ProgressBar
            value={overallProgress * 100}
            variant="warning"
            size="default"
          />
          <p className="text-sm text-yellow-500">
            Preprocessing was cancelled at {Math.round(overallProgress * 100)}%
          </p>
        </div>
      )}

      {/* Actions */}
      <div className="flex items-center gap-2 pt-2 border-t border-border">
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
