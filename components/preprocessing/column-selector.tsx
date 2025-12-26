"use client";

import { useMemo, useCallback } from "react";
import { cn } from "@/lib/utils";
import { Checkbox } from "@/components/ui/checkbox";
import { Button } from "@/components/ui/button";
import type { ColumnInfo } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface ColumnSelectorProps {
  /** All available columns from the loaded file */
  columns: ColumnInfo[];
  /** Currently selected column names */
  selectedColumns: string[];
  /** Callback when selection changes */
  onSelectionChange: (selectedColumns: string[]) => void;
  /** Whether the selector is disabled (e.g., during processing) */
  disabled?: boolean;
  /** Additional class names */
  className?: string;
}

// ============================================================================
// HELPERS
// ============================================================================

/**
 * Maps Polars dtype to a short display label.
 */
function getDtypeLabel(dtype: string): string {
  const dtypeLower = dtype.toLowerCase();

  if (dtypeLower.includes("int")) return "int";
  if (dtypeLower.includes("float") || dtypeLower.includes("f64") || dtypeLower.includes("f32")) return "float";
  if (dtypeLower.includes("bool")) return "bool";
  if (dtypeLower.includes("str") || dtypeLower.includes("utf8") || dtypeLower.includes("string")) return "str";
  if (dtypeLower.includes("date")) return "date";
  if (dtypeLower.includes("time")) return "time";
  if (dtypeLower.includes("datetime")) return "datetime";
  if (dtypeLower.includes("duration")) return "duration";
  if (dtypeLower.includes("categorical") || dtypeLower.includes("cat")) return "cat";
  if (dtypeLower.includes("null")) return "null";
  if (dtypeLower.includes("object")) return "obj";

  // Return first 6 chars if unknown
  return dtype.slice(0, 6).toLowerCase();
}

/**
 * Get badge color based on dtype category.
 */
function getDtypeBadgeClass(dtype: string): string {
  const dtypeLower = dtype.toLowerCase();

  // Numeric types - blue
  if (dtypeLower.includes("int") || dtypeLower.includes("float") || dtypeLower.includes("f64") || dtypeLower.includes("f32")) {
    return "bg-blue-500/20 text-blue-400 border-blue-500/30";
  }

  // String types - green
  if (dtypeLower.includes("str") || dtypeLower.includes("utf8") || dtypeLower.includes("string")) {
    return "bg-green-500/20 text-green-400 border-green-500/30";
  }

  // Boolean - purple
  if (dtypeLower.includes("bool")) {
    return "bg-purple-500/20 text-purple-400 border-purple-500/30";
  }

  // Date/Time types - orange
  if (dtypeLower.includes("date") || dtypeLower.includes("time") || dtypeLower.includes("duration")) {
    return "bg-orange-500/20 text-orange-400 border-orange-500/30";
  }

  // Categorical - yellow
  if (dtypeLower.includes("categorical") || dtypeLower.includes("cat")) {
    return "bg-yellow-500/20 text-yellow-400 border-yellow-500/30";
  }

  // Null/Unknown - gray
  return "bg-muted text-muted-foreground border-border";
}

// ============================================================================
// COLUMN ITEM COMPONENT
// ============================================================================

interface ColumnItemProps {
  column: ColumnInfo;
  isSelected: boolean;
  onToggle: (columnName: string, checked: boolean) => void;
  disabled?: boolean;
}

function ColumnItem({ column, isSelected, onToggle, disabled }: ColumnItemProps) {
  const handleChange = useCallback(
    (checked: boolean) => {
      onToggle(column.name, checked);
    },
    [column.name, onToggle]
  );

  const dtypeLabel = getDtypeLabel(column.dtype);
  const dtypeBadgeClass = getDtypeBadgeClass(column.dtype);
  const hasNulls = column.null_count > 0;

  return (
    <div
      className={cn(
        "flex items-center gap-3 px-3 py-2 rounded-md",
        "hover:bg-accent/50 transition-colors",
        disabled && "opacity-50 pointer-events-none"
      )}
    >
      <Checkbox
        checked={isSelected}
        onCheckedChange={handleChange}
        disabled={disabled}
        aria-label={`Select column ${column.name}`}
      />

      {/* Column name */}
      <span className="flex-1 text-sm font-medium truncate" title={column.name}>
        {column.name}
      </span>

      {/* Null count indicator */}
      {hasNulls && (
        <span
          className="text-xs text-muted-foreground tabular-nums"
          title={`${column.null_count} null values`}
        >
          {column.null_count} nulls
        </span>
      )}

      {/* Data type badge */}
      <span
        className={cn(
          "px-1.5 py-0.5 text-xs font-mono rounded border",
          dtypeBadgeClass
        )}
        title={`Data type: ${column.dtype}`}
      >
        {dtypeLabel}
      </span>
    </div>
  );
}

// ============================================================================
// COLUMN SELECTOR COMPONENT
// ============================================================================

/**
 * A visual column selector for preprocessing configuration.
 *
 * Displays all columns from the loaded file with their data types and null counts.
 * Allows multi-select with Select All / Deselect All functionality.
 *
 * @example
 * ```tsx
 * const [selectedColumns, setSelectedColumns] = useState<string[]>([]);
 * const { fileInfo } = useFileState();
 *
 * <ColumnSelector
 *   columns={fileInfo?.columns ?? []}
 *   selectedColumns={selectedColumns}
 *   onSelectionChange={setSelectedColumns}
 * />
 * ```
 */
export function ColumnSelector({
  columns,
  selectedColumns,
  onSelectionChange,
  disabled = false,
  className,
}: ColumnSelectorProps) {
  // Create a Set for O(1) lookup
  const selectedSet = useMemo(() => new Set(selectedColumns), [selectedColumns]);

  // Selection counts
  const totalCount = columns.length;
  const selectedCount = selectedColumns.length;
  const allSelected = totalCount > 0 && selectedCount === totalCount;
  const noneSelected = selectedCount === 0;

  // Handle individual column toggle
  const handleColumnToggle = useCallback(
    (columnName: string, checked: boolean) => {
      if (checked) {
        onSelectionChange([...selectedColumns, columnName]);
      } else {
        onSelectionChange(selectedColumns.filter((name) => name !== columnName));
      }
    },
    [selectedColumns, onSelectionChange]
  );

  // Select all columns
  const handleSelectAll = useCallback(() => {
    onSelectionChange(columns.map((col) => col.name));
  }, [columns, onSelectionChange]);

  // Deselect all columns
  const handleDeselectAll = useCallback(() => {
    onSelectionChange([]);
  }, [onSelectionChange]);

  // Empty state
  if (columns.length === 0) {
    return (
      <div
        className={cn(
          "flex flex-col items-center justify-center p-6",
          "text-muted-foreground text-sm",
          className
        )}
      >
        <p>No columns available</p>
        <p className="text-xs mt-1">Load a file to see columns</p>
      </div>
    );
  }

  return (
    <div className={cn("flex flex-col", className)} data-slot="column-selector">
      {/* Header with actions */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-border">
        <span className="text-sm text-muted-foreground">
          {selectedCount} of {totalCount} selected
        </span>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleSelectAll}
            disabled={disabled || allSelected}
          >
            Select All
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleDeselectAll}
            disabled={disabled || noneSelected}
          >
            Deselect All
          </Button>
        </div>
      </div>

      {/* Column list */}
      <div className="flex flex-col overflow-y-auto max-h-[400px]">
        {columns.map((column) => (
          <ColumnItem
            key={column.name}
            column={column}
            isSelected={selectedSet.has(column.name)}
            onToggle={handleColumnToggle}
            disabled={disabled}
          />
        ))}
      </div>

      {/* Footer with summary */}
      <div className="flex items-center gap-4 px-3 py-2 border-t border-border text-xs text-muted-foreground">
        <span className="flex items-center gap-1.5">
          <span className="w-2 h-2 rounded-full bg-blue-500/50" />
          Numeric
        </span>
        <span className="flex items-center gap-1.5">
          <span className="w-2 h-2 rounded-full bg-green-500/50" />
          String
        </span>
        <span className="flex items-center gap-1.5">
          <span className="w-2 h-2 rounded-full bg-orange-500/50" />
          Date/Time
        </span>
        <span className="flex items-center gap-1.5">
          <span className="w-2 h-2 rounded-full bg-purple-500/50" />
          Boolean
        </span>
      </div>
    </div>
  );
}

export default ColumnSelector;
