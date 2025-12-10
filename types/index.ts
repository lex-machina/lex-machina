export interface ColumnInfo {
  name: string;
  dtype: string;
  null_count: number;
  width: number;
}

export interface FileInfo {
  path: string;
  name: string;
  size_bytes: number;
  row_count: number;
  column_count: number;
  columns: ColumnInfo[];
}

export type CellValue = string | number | boolean | null;

export type Row = CellValue[];

export interface RowsResponse {
  rows: CellValue[][];
  start: number;
  total_rows: number;
}

// ============================================================================
// EVENT TYPES (Mirrors Rust events.rs payloads)
// ============================================================================

/**
 * Event names emitted by Rust backend.
 * These must match the constants in src-tauri/src/events.rs
 */
export const RUST_EVENTS = {
  FILE_LOADED: "file:loaded",
  FILE_CLOSED: "file:closed",
  LOADING: "app:loading",
  ERROR: "app:error",
} as const;

export type RustEventName = (typeof RUST_EVENTS)[keyof typeof RUST_EVENTS];

/**
 * Payload for the `file:loaded` event.
 * Mirrors: events.rs::FileLoadedPayload
 */
export interface FileLoadedPayload {
  file_info: FileInfo;
}

/**
 * Payload for the `app:loading` event.
 * Mirrors: events.rs::LoadingPayload
 */
export interface LoadingPayload {
  is_loading: boolean;
  message: string | null;
}

/**
 * Payload for the `app:error` event.
 * Mirrors: events.rs::ErrorPayload
 */
export interface ErrorPayload {
  code: string;
  message: string;
}

/**
 * Error codes from Rust backend.
 * Mirrors: events.rs::error_codes
 */
export const ERROR_CODES = {
  FILE_NOT_FOUND: "FILE_NOT_FOUND",
  FILE_READ_ERROR: "FILE_READ_ERROR",
  FILE_PARSE_ERROR: "FILE_PARSE_ERROR",
  FILE_METADATA_ERROR: "FILE_METADATA_ERROR",
  UNKNOWN_ERROR: "UNKNOWN_ERROR",
} as const;

export type ErrorCode = (typeof ERROR_CODES)[keyof typeof ERROR_CODES];

// ============================================================================
// UI STATE TYPES (Mirrors Rust state.rs)
// ============================================================================

/**
 * Grid scroll position.
 * Mirrors: state.rs::GridScrollPosition
 */
export interface GridScrollPosition {
  row_index: number;
  scroll_left: number;
}

/**
 * UI state persisted in Rust.
 * Mirrors: state.rs::UIState
 */
export interface UIState {
  sidebar_width: number;
  column_widths: number[];
  grid_scroll: GridScrollPosition;
}

