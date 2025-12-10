"use client";

import type { CellValue } from "@/types";
import { cn } from "@/lib/utils";

interface GridCellProps {
  value: CellValue;
  width: number;
  isHeader?: boolean;
}

const formatCellValue = (value: CellValue): string => {
  if (value === null) {
    return "";
  }
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  if (typeof value === "number") {
    if (Number.isInteger(value)) {
      return value.toLocaleString();
    }
    return value.toLocaleString(undefined, { maximumFractionDigits: 4 });
  }
  return String(value);
};

const GridCell = ({ value, width, isHeader = false }: GridCellProps) => {
  const displayValue = formatCellValue(value);
  const isNull = value === null;

  return (
    <div
      className={cn(
        "shrink-0 px-2 py-1 border-r border-b overflow-hidden text-ellipsis whitespace-nowrap",
        isHeader
          ? "font-semibold bg-muted text-muted-foreground"
          : "bg-background",
        isNull && "text-muted-foreground/50 italic",
      )}
      style={{ width, minWidth: width, maxWidth: width }}
      title={displayValue}
    >
      {isNull ? "null" : displayValue}
    </div>
  );
};

export default GridCell;
