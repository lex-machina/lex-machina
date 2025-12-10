"use client";

import { useFileState } from "@/lib/hooks/use-file-state";
import { useAppStatus } from "@/lib/hooks/use-app-status";
import { formatBytes, formatNumber } from "@/lib/utils";

/**
 * Status bar component at the bottom of the application.
 *
 * This component subscribes directly to Rust events and displays:
 * - Loading status with message (when loading)
 * - File statistics when a file is loaded (rows, columns, size)
 * - "No file loaded" message when empty
 *
 * Following "Rust Supremacy", this component is purely reactive -
 * it receives all state from Rust events, not from props.
 */
const StatusBar = () => {
  const { fileInfo } = useFileState();
  const { isLoading, loadingMessage } = useAppStatus();

  return (
    <footer className="flex items-center justify-between h-6 px-5 text-xs border-t bg-background text-muted-foreground">
      {/* Left side: File info or status */}
      <div className="flex items-center gap-5">
        {isLoading && loadingMessage ? (
          <span className="text-primary animate-pulse">{loadingMessage}</span>
        ) : fileInfo ? (
          <>
            <span>Rows: {formatNumber(fileInfo.row_count)}</span>
            <span>Columns: {fileInfo.column_count}</span>
            <span>Size: {formatBytes(fileInfo.size_bytes)}</span>
          </>
        ) : (
          <span>No file loaded</span>
        )}
      </div>

      {/* Right side: Loading indicator (when loading but no message) */}
      {isLoading && !loadingMessage && (
        <span className="text-primary animate-pulse">Loading...</span>
      )}
    </footer>
  );
};

export default StatusBar;
