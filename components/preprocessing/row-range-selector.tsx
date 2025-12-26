"use client";

import { useCallback, useMemo } from "react";
import { cn } from "@/lib/utils";
import { Input } from "@/components/ui/input";
import { Toggle } from "@/components/ui/toggle";
import type { RowRange } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface RowRangeSelectorProps {
  /** Total number of rows in the dataset */
  totalRows: number;
  /** Current row range selection (null = all rows) */
  rowRange: RowRange | null;
  /** Callback when row range changes */
  onRangeChange: (range: RowRange | null) => void;
  /** Whether the selector is disabled */
  disabled?: boolean;
  /** Additional class names */
  className?: string;
}

// ============================================================================
// HELPERS
// ============================================================================

/**
 * Format a number with thousand separators.
 */
function formatNumber(num: number): string {
  return num.toLocaleString();
}

/**
 * Clamp a value between min and max.
 */
function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

/**
 * Parse input value to number, returning null if invalid.
 */
function parseRowInput(value: string): number | null {
  const trimmed = value.trim();
  if (trimmed === "") return null;

  const parsed = parseInt(trimmed, 10);
  if (isNaN(parsed) || parsed < 0) return null;

  return parsed;
}

// ============================================================================
// ROW RANGE SELECTOR COMPONENT
// ============================================================================

/**
 * A row range selector for preprocessing configuration.
 *
 * Allows selecting a subset of rows to preprocess. When disabled,
 * all rows are included in preprocessing.
 *
 * @example
 * ```tsx
 * const [rowRange, setRowRange] = useState<RowRange | null>(null);
 * const { fileInfo } = useFileState();
 *
 * <RowRangeSelector
 *   totalRows={fileInfo?.row_count ?? 0}
 *   rowRange={rowRange}
 *   onRangeChange={setRowRange}
 * />
 * ```
 */
export function RowRangeSelector({
  totalRows,
  rowRange,
  onRangeChange,
  disabled = false,
  className,
}: RowRangeSelectorProps) {
  // Whether row range filtering is enabled
  const isEnabled = rowRange !== null;

  // Current values (with defaults when not enabled)
  const startValue = rowRange?.start ?? 0;
  const endValue = rowRange?.end ?? totalRows;

  // Calculate row count in selection
  const selectedRowCount = isEnabled ? Math.max(0, endValue - startValue) : totalRows;

  // Toggle row range filtering
  const handleToggle = useCallback(
    (pressed: boolean) => {
      if (pressed) {
        // Enable with default range (all rows)
        onRangeChange({ start: 0, end: totalRows });
      } else {
        // Disable (use all rows)
        onRangeChange(null);
      }
    },
    [totalRows, onRangeChange]
  );

  // Handle start index change
  const handleStartChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const parsed = parseRowInput(e.target.value);
      if (parsed === null) return;

      const newStart = clamp(parsed, 0, endValue);
      onRangeChange({ start: newStart, end: endValue });
    },
    [endValue, onRangeChange]
  );

  // Handle end index change
  const handleEndChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const parsed = parseRowInput(e.target.value);
      if (parsed === null) return;

      const newEnd = clamp(parsed, startValue, totalRows);
      onRangeChange({ start: startValue, end: newEnd });
    },
    [startValue, totalRows, onRangeChange]
  );

  // Handle blur to ensure valid values
  const handleStartBlur = useCallback(() => {
    if (!isEnabled) return;

    // Ensure start is valid
    const validStart = clamp(startValue, 0, endValue);
    if (validStart !== startValue) {
      onRangeChange({ start: validStart, end: endValue });
    }
  }, [isEnabled, startValue, endValue, onRangeChange]);

  const handleEndBlur = useCallback(() => {
    if (!isEnabled) return;

    // Ensure end is valid
    const validEnd = clamp(endValue, startValue, totalRows);
    if (validEnd !== endValue) {
      onRangeChange({ start: startValue, end: validEnd });
    }
  }, [isEnabled, startValue, endValue, totalRows, onRangeChange]);

  // Validation errors
  const errors = useMemo(() => {
    if (!isEnabled) return { start: undefined, end: undefined };

    const startErr =
      startValue < 0
        ? "Must be >= 0"
        : startValue > endValue
          ? "Must be <= end"
          : undefined;

    const endErr =
      endValue > totalRows
        ? `Must be <= ${formatNumber(totalRows)}`
        : endValue < startValue
          ? "Must be >= start"
          : undefined;

    return { start: startErr, end: endErr };
  }, [isEnabled, startValue, endValue, totalRows]);

  // Empty state
  if (totalRows === 0) {
    return (
      <div
        className={cn(
          "flex flex-col items-center justify-center p-4",
          "text-muted-foreground text-sm",
          className
        )}
      >
        <p>No rows available</p>
        <p className="text-xs mt-1">Load a file to select rows</p>
      </div>
    );
  }

  return (
    <div className={cn("flex flex-col gap-4", className)} data-slot="row-range-selector">
      {/* Enable toggle */}
      <div className="flex items-center justify-between">
        <Toggle
          pressed={isEnabled}
          onPressedChange={handleToggle}
          disabled={disabled}
          label="Limit row range"
          description="Process only a subset of rows"
        />
      </div>

      {/* Row range inputs */}
      <div
        className={cn(
          "grid grid-cols-2 gap-4 transition-opacity",
          !isEnabled && "opacity-50 pointer-events-none"
        )}
      >
        <Input
          label="Start row"
          type="number"
          min={0}
          max={endValue}
          value={startValue}
          onChange={handleStartChange}
          onBlur={handleStartBlur}
          error={errors.start}
          disabled={disabled || !isEnabled}
          helperText={`Min: 0`}
        />
        <Input
          label="End row"
          type="number"
          min={startValue}
          max={totalRows}
          value={endValue}
          onChange={handleEndChange}
          onBlur={handleEndBlur}
          error={errors.end}
          disabled={disabled || !isEnabled}
          helperText={`Max: ${formatNumber(totalRows)}`}
        />
      </div>

      {/* Summary */}
      <div className="flex items-center justify-between px-1 text-xs text-muted-foreground">
        <span>
          {isEnabled ? (
            <>
              Rows {formatNumber(startValue)} to {formatNumber(endValue)}
            </>
          ) : (
            "All rows selected"
          )}
        </span>
        <span className="tabular-nums">
          {formatNumber(selectedRowCount)} of {formatNumber(totalRows)} rows
        </span>
      </div>

      {/* Visual indicator bar */}
      <div className="h-2 w-full rounded-full bg-secondary overflow-hidden">
        {isEnabled ? (
          <div
            className="h-full bg-primary transition-all"
            style={{
              marginLeft: `${(startValue / totalRows) * 100}%`,
              width: `${((endValue - startValue) / totalRows) * 100}%`,
            }}
          />
        ) : (
          <div className="h-full w-full bg-primary" />
        )}
      </div>
    </div>
  );
}

export default RowRangeSelector;
