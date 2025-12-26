/**
 * DataGrid - Public API
 *
 * The DataGrid component is fully self-contained and manages its own state:
 * - Subscribes to file events from Rust (file loaded/closed)
 * - Fetches row data from Rust as needed for virtual scrolling
 * - Manages column widths with persistence to Rust
 * - Handles all scroll and resize interactions internally
 *
 * @example
 * ```tsx
 * import DataGrid from '@/components/data-grid';
 *
 * // Simple usage - no props needed
 * <DataGrid />
 * ```
 *
 * Internal Architecture:
 * - useGridData: Manages data state (columns, rows, widths) via Rust events
 * - useGridScroll: Manages scroll state (position, viewport dimensions)
 * - GridHeader: Column headers with resize handles
 * - GridBody: Virtualized row rendering with keyboard navigation
 * - Scrollbar: Custom scrollbars for both axes
 */
export { default } from "./data-grid";
export { default as DataGrid } from "./data-grid";

// ProcessedDataGrid - for displaying processed DataFrame data
export { ProcessedDataGrid } from "./processed-data-grid";

// Export hooks for advanced use cases (e.g., building custom grid variants)
export { useGridData } from "./use-grid-data";
export { useGridScroll, ROW_HEIGHT, SCROLLBAR_SIZE } from "./use-grid-scroll";
export type { GridDataState } from "./use-grid-data";
export type { GridScrollState, GridScrollConfig } from "./use-grid-scroll";
