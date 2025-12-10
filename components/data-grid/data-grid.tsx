"use client";

import { useGridData } from "@/components/data-grid/use-grid-data";
import { useGridScroll, SCROLLBAR_SIZE } from "@/components/data-grid/use-grid-scroll";
import GridHeader from "@/components/data-grid/grid-header";
import GridBody from "@/components/data-grid/grid-body";
import Scrollbar from "@/components/ui/scrollbar";

/**
 * DataGrid - A self-contained virtualized data grid component.
 *
 * This component:
 * - Subscribes to file state events from Rust (via useGridData)
 * - Manages its own scroll state (via useGridScroll)
 * - Fetches row data from Rust as needed
 * - Handles column resizing with persistence to Rust
 *
 * Following "Rust Supremacy":
 * - All data comes from Rust events/commands
 * - Column widths persist to Rust backend
 * - Component is purely reactive to Rust state
 *
 * @example
 * ```tsx
 * // Minimal usage - component manages everything internally
 * <DataGrid />
 * ```
 */
const DataGrid = () => {
  // Data state from Rust events
  const {
    columns,
    columnWidths,
    totalRows,
    rows,
    visibleStart,
    hasData,
    fetchRows,
    setColumnWidth,
    persistColumnWidths,
  } = useGridData();

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
  if (!hasData) {
    return (
      <div className="flex-1 justify-center items-center flex">
        <p className="text-muted-foreground">Upload a file to get started</p>
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
                onColumnResizeEnd={persistColumnWidths}
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
};

export default DataGrid;
