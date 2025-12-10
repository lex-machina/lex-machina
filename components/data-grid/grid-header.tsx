"use client";

import { useState, useRef, useCallback } from "react";
import { cn } from "@/lib/utils";

import type { ColumnInfo } from "@/types";

interface GridHeaderProps {
  columns: ColumnInfo[];
  columnWidths: number[];
  onColumnResize: (colIndex: number, width: number) => void;
  onColumnResizeEnd: () => void;
}

const ROW_HEIGHT = 32;
const MIN_COLUMN_WIDTH = 60;

const GridHeader = ({
  columns,
  columnWidths,
  onColumnResize,
  onColumnResizeEnd,
}: GridHeaderProps) => {
  const [resizingCol, setResizingCol] = useState<number | null>(null);
  const startX = useRef(0);
  const startWidth = useRef(0);

  const handleResizeStart = useCallback(
    (e: React.MouseEvent, colIndex: number) => {
      e.preventDefault();
      setResizingCol(colIndex);
      startX.current = e.clientX;
      startWidth.current = columnWidths[colIndex] || 150;

      const handleMouseMove = (moveEvent: MouseEvent) => {
        const delta = moveEvent.clientX - startX.current;
        const newWidth = Math.max(MIN_COLUMN_WIDTH, startWidth.current + delta);
        onColumnResize(colIndex, newWidth);
      };

      const handleMouseUp = () => {
        setResizingCol(null);
        document.removeEventListener("mousemove", handleMouseMove);
        document.removeEventListener("mouseup", handleMouseUp);
        onColumnResizeEnd();
      };
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
    },
    [columnWidths, onColumnResize, onColumnResizeEnd],
  );

  return (
    <div
      className="shrink-0 border-b bg-muted sticky top-0 z-10"
      style={{ height: ROW_HEIGHT }}
    >
      <div className="flex h-full">
        {columns.map((col, index) => {
          const width = columnWidths[index] || 150;

          return (
            <div
              key={col.name}
              className="relative flex shrink-0 items-center  px-2 border-r text-sm font-semibold text-muted-foreground overflow-hidden"
              style={{ width, minWidth: width, maxWidth: width }}
            >
              <span className="truncate" title={`${col.name} (${col.dtype})`}>
                {col.name}
              </span>
              <div
                className={cn(
                  "absolute right-0 top-0 bottom-0 w-1 cursor-col-resize transition-colors",
                  "hover:bg-primary/50",
                  resizingCol === index && "bg-primary",
                )}
                onMouseDown={(e) => handleResizeStart(e, index)}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default GridHeader;
