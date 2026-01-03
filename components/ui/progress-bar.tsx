"use client";

import { cn } from "@/lib/utils";

// ============================================================================
// TYPES
// ============================================================================

export interface ProgressBarProps {
    /** Current progress value (0-100 or 0-1 depending on max) */
    value: number;
    /** Maximum value (default: 100) */
    max?: number;
    /** Whether to show the percentage label */
    showLabel?: boolean;
    /** Custom label formatter */
    formatLabel?: (value: number, max: number) => string;
    /** Size variant */
    size?: "sm" | "default" | "lg";
    /** Color variant */
    variant?: "default" | "success" | "warning" | "error";
    /** Whether the progress is indeterminate (animated) */
    indeterminate?: boolean;
    /** Label text above the progress bar */
    label?: string;
    /** Additional class names */
    className?: string;
}

// ============================================================================
// PROGRESS BAR COMPONENT
// ============================================================================

/**
 * A progress bar for showing completion status.
 *
 * @example
 * ```tsx
 * // Simple progress
 * <ProgressBar value={50} />
 *
 * // With label
 * <ProgressBar
 *   value={75}
 *   label="Processing..."
 *   showLabel
 * />
 *
 * // Indeterminate (loading)
 * <ProgressBar indeterminate label="Loading..." />
 *
 * // Custom formatting
 * <ProgressBar
 *   value={3}
 *   max={10}
 *   showLabel
 *   formatLabel={(v, m) => `${v} of ${m} steps`}
 * />
 * ```
 */
export function ProgressBar({
    value,
    max = 100,
    showLabel = false,
    formatLabel,
    size = "default",
    variant = "default",
    indeterminate = false,
    label,
    className,
}: ProgressBarProps) {
    // Clamp value between 0 and max
    const clampedValue = Math.max(0, Math.min(value, max));
    const percentage = (clampedValue / max) * 100;

    // Default label formatter
    const defaultFormatLabel = (v: number, m: number) => {
        if (m === 100) {
            return `${Math.round(v)}%`;
        }
        return `${v}/${m}`;
    };

    const displayLabel = formatLabel
        ? formatLabel(clampedValue, max)
        : defaultFormatLabel(clampedValue, max);

    const sizeClasses = {
        sm: "h-1.5",
        default: "h-2",
        lg: "h-3",
    };

    const variantClasses = {
        default: "bg-primary",
        success: "bg-green-500",
        warning: "bg-yellow-500",
        error: "bg-destructive",
    };

    return (
        <div
            className={cn("flex flex-col gap-1.5", className)}
            data-slot="progress"
        >
            {(label || showLabel) && (
                <div className="flex items-center justify-between">
                    {label && (
                        <span className="text-sm leading-none font-medium">
                            {label}
                        </span>
                    )}
                    {showLabel && !indeterminate && (
                        <span className="text-muted-foreground text-sm tabular-nums">
                            {displayLabel}
                        </span>
                    )}
                </div>
            )}
            <div
                role="progressbar"
                aria-valuenow={indeterminate ? undefined : clampedValue}
                aria-valuemin={0}
                aria-valuemax={max}
                aria-label={label}
                className={cn(
                    // Base track styles
                    "bg-secondary relative w-full overflow-hidden rounded-full",
                    sizeClasses[size],
                )}
            >
                <div
                    className={cn(
                        // Base fill styles
                        "h-full rounded-full transition-all duration-300 ease-in-out",
                        variantClasses[variant],
                        // Indeterminate animation
                        indeterminate && "animate-progress-indeterminate w-1/3",
                    )}
                    style={
                        indeterminate
                            ? undefined
                            : {
                                  width: `${percentage}%`,
                              }
                    }
                />
            </div>
        </div>
    );
}

// ============================================================================
// STAGE PROGRESS (for multi-stage operations)
// ============================================================================

export interface StageProgressProps {
    /** Overall progress (0-1) */
    overallProgress: number;
    /** Current stage progress (0-1) */
    stageProgress: number;
    /** Current stage name */
    stageName: string;
    /** Optional sub-stage or current item */
    subStage?: string;
    /** Optional message */
    message?: string;
    /** Additional class names */
    className?: string;
}

/**
 * A compound progress indicator for multi-stage operations.
 *
 * Shows both overall progress and current stage progress.
 *
 * @example
 * ```tsx
 * <StageProgress
 *   overallProgress={0.45}
 *   stageProgress={0.8}
 *   stageName="Imputation"
 *   subStage="Column: Age"
 *   message="Imputing missing values using KNN..."
 * />
 * ```
 */
export function StageProgress({
    overallProgress,
    stageProgress,
    stageName,
    subStage,
    message,
    className,
}: StageProgressProps) {
    return (
        <div
            className={cn("flex flex-col gap-3", className)}
            data-slot="stage-progress"
        >
            {/* Overall progress */}
            <ProgressBar
                value={overallProgress * 100}
                label="Overall Progress"
                showLabel
                size="default"
            />

            {/* Stage progress */}
            <div className="flex flex-col gap-1.5">
                <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">{stageName}</span>
                    {subStage && (
                        <span className="text-muted-foreground text-xs">
                            {subStage}
                        </span>
                    )}
                </div>
                <ProgressBar
                    value={stageProgress * 100}
                    size="sm"
                    variant="default"
                />
            </div>

            {/* Message */}
            {message && (
                <p className="text-muted-foreground truncate text-xs">
                    {message}
                </p>
            )}
        </div>
    );
}

export default ProgressBar;
