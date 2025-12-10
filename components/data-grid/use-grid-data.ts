"use client";

import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

import { useFileState } from "@/lib/hooks/use-file-state";
import type { Row, RowsResponse, ColumnInfo, UIState } from "@/types";

/**
 * Constants for grid data management.
 */
const BUFFER_ROWS = 10;

/**
 * State returned by the useGridData hook.
 */
export interface GridDataState {
  /** Column information from the loaded file */
  columns: ColumnInfo[];
  /** Current column widths (can be modified by user) */
  columnWidths: number[];
  /** Total number of rows in the dataset */
  totalRows: number;
  /** Currently loaded rows buffer */
  rows: Row[];
  /** Starting index of the rows buffer */
  visibleStart: number;
  /** Whether any data is available */
  hasData: boolean;
  /** Fetch rows for a given scroll position */
  fetchRows: (currentRowIndex: number, visibleRowCount: number) => void;
  /** Update a column width (local state) */
  setColumnWidth: (colIndex: number, width: number) => void;
  /** Persist column widths to Rust */
  persistColumnWidths: () => Promise<void>;
}

/**
 * Hook for managing DataGrid data state.
 *
 * This hook:
 * - Subscribes to file state events (file loaded/closed)
 * - Manages rows buffer for virtual scrolling
 * - Handles column width state
 * - Communicates with Rust backend for data fetching
 *
 * Following "Rust Supremacy", this hook:
 * - Receives file state from Rust events
 * - Fetches row data from Rust commands
 * - Persists column widths to Rust
 *
 * @returns Grid data state and methods
 *
 * @example
 * ```tsx
 * const {
 *   columns,
 *   columnWidths,
 *   totalRows,
 *   rows,
 *   visibleStart,
 *   hasData,
 *   fetchRows,
 *   setColumnWidth,
 *   persistColumnWidths,
 * } = useGridData();
 * ```
 */
export function useGridData(): GridDataState {
  // Subscribe to file state from Rust events
  const { fileInfo } = useFileState();

  // Local state for rows buffer
  const [rows, setRows] = useState<Row[]>([]);
  const [visibleStart, setVisibleStart] = useState(0);

  // Local state for column widths (can be modified during resize)
  const [columnWidths, setColumnWidths] = useState<number[]>([]);

  // Ref to track latest column widths for use in callbacks (avoids stale closure)
  const columnWidthsRef = useRef(columnWidths);
  useEffect(() => {
    columnWidthsRef.current = columnWidths;
  }, [columnWidths]);

  // Track last fetch to avoid duplicate requests
  const lastFetchRef = useRef({ start: -1, count: -1 });

  // Track if we've attempted to restore column widths for the current file
  const hasRestoredWidthsRef = useRef(false);

  // Restore column widths from UIState when file loads
  // Priority: 1) User-customized widths from UIState, 2) Default widths from fileInfo
  useEffect(() => {
    if (!fileInfo) {
      hasRestoredWidthsRef.current = false;
      return;
    }

    // Only restore once per file load
    if (hasRestoredWidthsRef.current) return;
    hasRestoredWidthsRef.current = true;

    const restoreColumnWidths = async () => {
      try {
        const uiState = await invoke<UIState>("get_ui_state");
        // Use persisted widths if they match current column count (same file)
        if (
          uiState.column_widths &&
          uiState.column_widths.length === fileInfo.columns.length
        ) {
          setColumnWidths(uiState.column_widths);
        } else {
          // Fall back to default widths from file info
          setColumnWidths(fileInfo.columns.map((col) => col.width));
        }
      } catch (err) {
        console.error("Failed to restore column widths:", err);
        // Fall back to default widths
        setColumnWidths(fileInfo.columns.map((col) => col.width));
      }
    };

    restoreColumnWidths();
  }, [fileInfo]);

  // Reset state when file changes/closes
  useEffect(() => {
    if (!fileInfo) {
      setRows([]);
      setVisibleStart(0);
      setColumnWidths([]);
      lastFetchRef.current = { start: -1, count: -1 };
    }
  }, [fileInfo]);

  // Fetch rows from Rust backend
  const fetchRows = useCallback(
    async (currentRowIndex: number, visibleRowCount: number) => {
      if (!fileInfo) return;

      const fetchStart = Math.max(0, currentRowIndex - BUFFER_ROWS);
      const fetchCount = visibleRowCount + 2 * BUFFER_ROWS;

      // Avoid duplicate fetches
      if (
        lastFetchRef.current.start === fetchStart &&
        lastFetchRef.current.count === fetchCount
      ) {
        return;
      }

      lastFetchRef.current = { start: fetchStart, count: fetchCount };

      try {
        const response = await invoke<RowsResponse>("get_rows", {
          start: fetchStart,
          count: fetchCount,
        });
        setRows(response.rows);
        setVisibleStart(response.start);
      } catch (err) {
        console.error("Failed to fetch rows:", err);
      }
    },
    [fileInfo]
  );

  // Update a single column width (local state only)
  const setColumnWidth = useCallback((colIndex: number, width: number) => {
    setColumnWidths((prev) => {
      const next = [...prev];
      next[colIndex] = width;
      return next;
    });
  }, []);

  // Persist column widths to Rust backend
  const persistColumnWidths = useCallback(async () => {
    const currentWidths = columnWidthsRef.current;
    try {
      await invoke("set_column_widths", { widths: currentWidths });
    } catch (err) {
      console.error("Failed to persist column widths:", err);
    }
  }, []);

  return {
    columns: fileInfo?.columns ?? [],
    columnWidths,
    totalRows: fileInfo?.row_count ?? 0,
    rows,
    visibleStart,
    hasData: fileInfo !== null,
    fetchRows,
    setColumnWidth,
    persistColumnWidths,
  };
}
