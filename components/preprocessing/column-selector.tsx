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
  /** Hide the internal header (use when embedding in a panel with its own header) */
  hideHeader?: boolean;
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
 * Uses muted, subtle colors to fit with the rest of the app.
 */
function getDtypeBadgeClass(): string {
  // Use consistent muted styling for all types
  return "bg-muted text-muted-foreground";
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
  const dtypeBadgeClass = getDtypeBadgeClass();
  const hasNulls = column.null_count > 0;

  return (
    <div
      className={cn(
        "flex items-center gap-2 px-2 py-1.5 rounded",
        "hover:bg-muted/50 transition-colors",
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
      <span className="flex-1 text-sm truncate" title={column.name}>
        {column.name}
      </span>

      {/* Null count indicator */}
      {hasNulls && (
        <span
          className="text-xs text-muted-foreground tabular-nums shrink-0"
          title={`${column.null_count} null values`}
        >
          {column.null_count}
        </span>
      )}

      {/* Data type badge */}
      <span
        className={cn(
          "px-1.5 py-0.5 text-xs rounded shrink-0",
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
// COLUMN SELECTOR HEADER COMPONENT
// ============================================================================

export interface ColumnSelectorHeaderProps {
  /** Total number of columns */
  totalCount: number;
  /** Number of selected columns */
  selectedCount: number;
  /** Callback to select all columns */
  onSelectAll: () => void;
  /** Callback to deselect all columns */
  onDeselectAll: () => void;
  /** Whether the controls are disabled */
  disabled?: boolean;
}

/**
 * A standalone header component with selection count and All/None buttons.
 * Use this when you want to place the controls in a custom header.
 */
export function ColumnSelectorHeader({
  totalCount,
  selectedCount,
  onSelectAll,
  onDeselectAll,
  disabled = false,
}: ColumnSelectorHeaderProps) {
  const allSelected = totalCount > 0 && selectedCount === totalCount;
  const noneSelected = selectedCount === 0;

  return (
    <div className="flex items-center gap-2">
      <span className="text-xs text-muted-foreground tabular-nums">
        {selectedCount}/{totalCount}
      </span>
      <Button
        variant="ghost"
        size="sm"
        onClick={onSelectAll}
        disabled={disabled || allSelected}
        className="h-5 px-1.5 text-xs"
      >
        All
      </Button>
      <Button
        variant="ghost"
        size="sm"
        onClick={onDeselectAll}
        disabled={disabled || noneSelected}
        className="h-5 px-1.5 text-xs"
      >
        None
      </Button>
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
  hideHeader = false,
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
      {/* Header with actions - only shown if not hidden */}
      {!hideHeader && (
        <div className="flex items-center justify-between px-3 py-2 border-b border-border">
          <span className="text-xs text-muted-foreground">
            {selectedCount}/{totalCount}
          </span>
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleSelectAll}
              disabled={disabled || allSelected}
              className="h-6 px-2 text-xs"
            >
              All
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleDeselectAll}
              disabled={disabled || noneSelected}
              className="h-6 px-2 text-xs"
            >
              None
            </Button>
          </div>
        </div>
      )}

      {/* Column list - fills available space with internal scroll */}
      <div className="flex-1 min-h-0 flex flex-col overflow-y-auto px-1">
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
    </div>
  );
}

export default ColumnSelector;
