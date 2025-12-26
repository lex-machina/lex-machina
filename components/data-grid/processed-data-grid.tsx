"use client";

import { useProcessedData } from "@/lib/hooks/use-processed-data";
import { useGridScroll, SCROLLBAR_SIZE } from "@/components/data-grid/use-grid-scroll";
import GridHeader from "@/components/data-grid/grid-header";
import GridBody from "@/components/data-grid/grid-body";
import Scrollbar from "@/components/ui/scrollbar";

/**
 * ProcessedDataGrid - A virtualized data grid for processed DataFrame data.
 *
 * This component is similar to DataGrid but uses the processed data hook
 * instead of the original file data hook.
 *
 * Following "Rust Supremacy":
 * - All data comes from Rust via useProcessedData hook
 * - Column widths are managed locally (processed data is temporary)
 * - Component is purely reactive to Rust state
 *
 * @example
 * ```tsx
 * // Usage in data page processed tab
 * <ProcessedDataGrid />
 * ```
 */
export function ProcessedDataGrid() {
  // Data state from Rust
  const {
    columns,
    columnWidths,
    totalRows,
    rows,
    visibleStart,
    hasProcessedData,
    fetchRows,
    setColumnWidth,
  } = useProcessedData();

  // Scroll state management
  const {
    currentRowIndex,
    scrollLeft,
    viewportWidth,
    visibleRowCount,
    totalWidth,
    showHorizontalScrollbar,
    verticalScrollbarHeight,
    handleVerticalSeek,
    handleHorizontalSeek,
    handleWheel,
    containerRef,
    setViewportHeight,
  } = useGridScroll({
    totalRows,
    columnWidths,
    onVisibleRangeChange: fetchRows,
  });

  // Empty state
  if (!hasProcessedData) {
    return (
      <div className="flex-1 justify-center items-center flex flex-col gap-3">
        <NoDataIcon />
        <div className="text-center">
          <p className="text-muted-foreground font-medium">No processed data available</p>
          <p className="text-sm text-muted-foreground/70 mt-1">
            Go to the Processing page to preprocess your data
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-1 flex-col overflow-hidden relative">
      {/* Main content area with vertical scrollbar space reserved */}
      <div
        ref={containerRef}
        className="flex-1 flex flex-col overflow-hidden"
        style={{ marginRight: SCROLLBAR_SIZE }}
        onWheel={handleWheel}
      >
        {/* Clip container for horizontal scrolling */}
        <div
          className="flex-1 flex flex-col overflow-hidden"
          style={{
            marginBottom: showHorizontalScrollbar ? SCROLLBAR_SIZE : 0,
          }}
        >
          {/* Header - transforms with horizontal scroll */}
          <div className="shrink-0 overflow-hidden">
            <div
              style={{
                transform: `translateX(${-scrollLeft}px)`,
                width: totalWidth,
              }}
            >
              <GridHeader
                columns={columns}
                columnWidths={columnWidths}
                onColumnResize={setColumnWidth}
                onColumnResizeEnd={() => {
                  // No persistence for processed data - it's temporary
                }}
              />
            </div>
          </div>

          {/* Body - transforms with horizontal scroll */}
          <div className="flex-1 overflow-hidden min-h-0">
            <div
              className="h-full"
              style={{
                transform: `translateX(${-scrollLeft}px)`,
                width: totalWidth,
              }}
            >
              <GridBody
                rows={rows}
                totalRows={totalRows}
                columnWidths={columnWidths}
                onFetchRows={fetchRows}
                visibleStart={visibleStart}
                currentRowIndex={currentRowIndex}
                onRowIndexChange={handleVerticalSeek}
                onViewportChange={setViewportHeight}
              />
            </div>
          </div>
        </div>

        {/* Horizontal scrollbar - at bottom, hides when content fits */}
        {showHorizontalScrollbar && (
          <Scrollbar
            direction="horizontal"
            totalSize={totalWidth}
            currentPosition={scrollLeft}
            visibleSize={viewportWidth}
            onSeek={handleHorizontalSeek}
            containerSize={viewportWidth}
            hideWhenFits={true}
          />
        )}
      </div>

      {/* Vertical scrollbar - fixed to right edge, always visible */}
      <Scrollbar
        direction="vertical"
        totalSize={totalRows}
        currentPosition={currentRowIndex}
        visibleSize={visibleRowCount}
        onSeek={handleVerticalSeek}
        containerSize={verticalScrollbarHeight}
        hideWhenFits={false}
      />
    </div>
  );
}

/**
 * Icon shown when no processed data is available.
 */
function NoDataIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="48"
      height="48"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="text-muted-foreground/50"
    >
      <path d="M12 3v3" />
      <path d="M18.5 5.5l-2.1 2.1" />
      <path d="M21 12h-3" />
      <path d="M18.5 18.5l-2.1-2.1" />
      <path d="M12 21v-3" />
      <path d="M5.5 18.5l2.1-2.1" />
      <path d="M3 12h3" />
      <path d="M5.5 5.5l2.1 2.1" />
      <circle cx="12" cy="12" r="4" />
      <path d="m2 2 20 20" className="text-muted-foreground/30" />
    </svg>
  );
}

export default ProcessedDataGrid;
