"use client";

import { useRef, useState, useEffect, useCallback } from "react";

import type { Row } from "@/types";
import GridCell from "@/components/data-grid/grid-cell";

interface GridBodyProps {
  rows: Row[];
  totalRows: number;
  columnWidths: number[];
  onFetchRows: (start: number, count: number) => void;
  visibleStart: number;
  currentRowIndex: number;
  onRowIndexChange: (rowIndex: number) => void;
  onViewportChange: (height: number) => void;
}

const ROW_HEIGHT = 32;
const BUFFER_ROWS = 10;
const PIXELS_PER_ROW = 40; // Accumulated delta threshold for scrolling one row

const GridBody = ({
  rows,
  totalRows,
  columnWidths,
  onFetchRows,
  visibleStart,
  currentRowIndex,
  onRowIndexChange,
  onViewportChange,
}: GridBodyProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [viewportHeight, setViewportHeight] = useState(0);
  const lastFetchRef = useRef({ start: 0, count: 0 });
  const accumulatedDeltaRef = useRef(0);

  const visibleRowCount = Math.ceil(viewportHeight / ROW_HEIGHT);
  const maxRowIndex = Math.max(0, totalRows - visibleRowCount);

  // Clamp helper
  const clamp = (value: number, min: number, max: number) =>
    Math.max(min, Math.min(max, value));

  // Calculate fetch range based on current row index
  const fetchStart = Math.max(0, currentRowIndex - BUFFER_ROWS);
  const fetchCount = visibleRowCount + 2 * BUFFER_ROWS;

  // Fetch rows when currentRowIndex changes
  useEffect(() => {
    const lastFetch = lastFetchRef.current;
    if (fetchStart !== lastFetch.start || fetchCount !== lastFetch.count) {
      lastFetchRef.current = { start: fetchStart, count: fetchCount };
      onFetchRows(fetchStart, fetchCount);
    }
  }, [fetchStart, fetchCount, onFetchRows]);

  // Observe container resize
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const height = entry.contentRect.height;
        setViewportHeight(height);
        onViewportChange(height);
      }
    });

    observer.observe(container);
    const initialHeight = container.clientHeight;
    setViewportHeight(initialHeight);
    onViewportChange(initialHeight);

    return () => observer.disconnect();
  }, [onViewportChange]);

  // Handle mouse wheel - accumulate deltas for smooth touchpad support
  const handleWheel = useCallback(
    (e: React.WheelEvent) => {
      e.preventDefault();
      
      // Accumulate delta
      accumulatedDeltaRef.current += e.deltaY;
      
      // Calculate how many rows to move
      const rowDelta = Math.trunc(accumulatedDeltaRef.current / PIXELS_PER_ROW);
      
      if (rowDelta !== 0) {
        // Subtract consumed delta
        accumulatedDeltaRef.current -= rowDelta * PIXELS_PER_ROW;
        onRowIndexChange(clamp(currentRowIndex + rowDelta, 0, maxRowIndex));
      }
    },
    [currentRowIndex, maxRowIndex, onRowIndexChange],
  );

  // Handle keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      let rowDelta = 0;

      switch (e.key) {
        case "ArrowUp":
          rowDelta = -1;
          break;
        case "ArrowDown":
          rowDelta = 1;
          break;
        case "PageUp":
          rowDelta = -visibleRowCount;
          break;
        case "PageDown":
          rowDelta = visibleRowCount;
          break;
        case "Home":
          onRowIndexChange(0);
          return;
        case "End":
          onRowIndexChange(maxRowIndex);
          return;
        default:
          return;
      }

      e.preventDefault();
      onRowIndexChange(clamp(currentRowIndex + rowDelta, 0, maxRowIndex));
    },
    [visibleRowCount, maxRowIndex, currentRowIndex, onRowIndexChange],
  );

  // Render rows relative to currentRowIndex
  const renderRows = () => {
    return rows.map((row, localIndex) => {
      const absoluteIndex = visibleStart + localIndex;

      // Only render rows in the visible window (with buffer)
      if (
        absoluteIndex < currentRowIndex - BUFFER_ROWS ||
        absoluteIndex >= currentRowIndex + visibleRowCount + BUFFER_ROWS
      ) {
        return null;
      }

      // Position relative to currentRowIndex
      const topPosition = (absoluteIndex - currentRowIndex) * ROW_HEIGHT;

      return (
        <div
          key={absoluteIndex}
          className="flex absolute left-0"
          style={{
            top: topPosition,
            height: ROW_HEIGHT,
          }}
        >
          {row.map((cellValue, colIndex) => (
            <GridCell
              key={colIndex}
              value={cellValue}
              width={columnWidths[colIndex] || 150}
            />
          ))}
        </div>
      );
    });
  };

  return (
    <div
      ref={containerRef}
      className="h-full w-full relative overflow-hidden outline-none"
      tabIndex={0}
      onWheel={handleWheel}
      onKeyDown={handleKeyDown}
    >
      {/* Row container - no native scrolling */}
      <div className="absolute inset-0 overflow-hidden">
        {renderRows()}
      </div>
    </div>
  );
};

export default GridBody;
