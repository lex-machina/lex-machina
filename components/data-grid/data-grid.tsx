"use client";

import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Table2, FileUp, Columns, Search, Download } from "lucide-react";

import { useGridData } from "@/components/data-grid/use-grid-data";
import { useGridScroll, SCROLLBAR_SIZE } from "@/components/data-grid/use-grid-scroll";
import GridHeader from "@/components/data-grid/grid-header";
import GridBody from "@/components/data-grid/grid-body";
import Scrollbar from "@/components/ui/scrollbar";
import { Button } from "@/components/ui/button";
import { toast } from "@/components/ui/toast";
import type { FileInfo } from "@/types";

/**
 * Empty state component shown when no file is loaded.
 * Provides informative content about page features and an action to import data.
 */
function NoFileLoadedState() {
  const handleImport = useCallback(async () => {
    try {
      const filePath = await invoke<string | null>("open_file_dialog");
      if (!filePath) return;

      await invoke<FileInfo>("load_file", { path: filePath });
      toast.success("File loaded successfully");
    } catch (err) {
      toast.error(`Failed to import file: ${err}`);
    }
  }, []);

  return (
    <div className="flex-1 flex items-center justify-center p-8">
      <div className="text-center max-w-md">
        {/* Icon */}
        <div className="mx-auto w-16 h-16 rounded-full bg-muted flex items-center justify-center mb-6">
          <Table2 className="w-8 h-8 text-muted-foreground" />
        </div>

        {/* Title and description */}
        <h2 className="text-xl font-semibold mb-2">Data Viewer</h2>
        <p className="text-muted-foreground mb-6">
          Import a CSV file to explore and analyze your data with a high-performance grid.
        </p>

        {/* Features */}
        <ul className="text-sm text-muted-foreground space-y-2 mb-8 text-left">
          <li className="flex items-center gap-3">
            <FileUp className="w-4 h-4 shrink-0" />
            <span>Import CSV files of any size</span>
          </li>
          <li className="flex items-center gap-3">
            <Columns className="w-4 h-4 shrink-0" />
            <span>Resizable columns with type detection</span>
          </li>
          <li className="flex items-center gap-3">
            <Search className="w-4 h-4 shrink-0" />
            <span>Virtual scrolling for large datasets</span>
          </li>
          <li className="flex items-center gap-3">
            <Download className="w-4 h-4 shrink-0" />
            <span>Export processed data to CSV</span>
          </li>
        </ul>

        {/* Action button */}
        <Button onClick={handleImport} size="lg">
          <FileUp className="w-4 h-4 mr-2" />
          Import File
        </Button>
      </div>
    </div>
  );
}

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

  // Empty state - show enhanced empty state with features
  if (!hasData) {
    return <NoFileLoadedState />;
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
