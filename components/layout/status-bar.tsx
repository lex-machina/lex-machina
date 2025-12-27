"use client";

import { usePathname } from "next/navigation";
import { useFileState } from "@/lib/hooks/use-file-state";
import { useAppStatus } from "@/lib/hooks/use-app-status";
import { usePreprocessing } from "@/lib/hooks/use-preprocessing";
import { formatBytes, formatNumber } from "@/lib/utils";

/** Application version - displayed in status bar */
const APP_VERSION = "v0.1.0";

/**
 * Status bar component at the bottom of the application.
 *
 * This component displays page-aware contextual information:
 * - Home: "Ready" or "File: {name}" | version
 * - Data: File info (rows, cols, size) | active tab indicator
 * - Processing: File info compact | processing status
 * - Analysis/ML: File info | "Coming Soon"
 * - Settings: "Settings" | version
 *
 * Following "Rust Supremacy", this component is purely reactive -
 * it receives all state from Rust events, not from props.
 */
const StatusBar = () => {
  const pathname = usePathname();
  const { fileInfo } = useFileState();
  const { isLoading, loadingMessage } = useAppStatus();
  const { status: preprocessingStatus } = usePreprocessing();

  /**
   * Renders the left content based on current page.
   */
  const renderLeftContent = () => {
    // Loading state takes precedence on all pages
    if (isLoading && loadingMessage) {
      return (
        <span className="text-primary animate-pulse">{loadingMessage}</span>
      );
    }

    switch (pathname) {
      case "/":
        // Home: "Ready" or "File: {name}"
        return fileInfo ? (
          <span>File: {fileInfo.name}</span>
        ) : (
          <span>Ready</span>
        );

      case "/data":
        // Data: Full file info
        return fileInfo ? (
          <>
            <span>{fileInfo.name}</span>
            <span className="text-muted-foreground/60">|</span>
            <span>{formatNumber(fileInfo.row_count)} rows</span>
            <span className="text-muted-foreground/60">|</span>
            <span>{fileInfo.column_count} cols</span>
            <span className="text-muted-foreground/60">|</span>
            <span>{formatBytes(fileInfo.size_bytes)}</span>
          </>
        ) : (
          <span>No file loaded</span>
        );

      case "/processing":
        // Processing: Compact file info
        return fileInfo ? (
          <>
            <span>{fileInfo.name}</span>
            <span className="text-muted-foreground/60">|</span>
            <span>
              {formatNumber(fileInfo.row_count)} x {fileInfo.column_count}
            </span>
          </>
        ) : (
          <span>No file loaded</span>
        );

      case "/analysis":
      case "/ml":
        // Analysis/ML: File info or no file
        return fileInfo ? (
          <>
            <span>{fileInfo.name}</span>
            <span className="text-muted-foreground/60">|</span>
            <span>{formatNumber(fileInfo.row_count)} rows</span>
          </>
        ) : (
          <span>No file loaded</span>
        );

      case "/settings":
        // Settings: Just "Settings" label
        return <span>Settings</span>;

      default:
        // Fallback: Same as home
        return fileInfo ? (
          <span>File: {fileInfo.name}</span>
        ) : (
          <span>Ready</span>
        );
    }
  };

  /**
   * Renders the right content based on current page.
   * Always includes version, with page-specific info prepended.
   */
  const renderRightContent = () => {
    // Loading indicator when loading but no specific message
    if (isLoading && !loadingMessage) {
      return (
        <>
          <span className="text-primary animate-pulse">Loading...</span>
          <span className="text-muted-foreground/60">|</span>
          <span>{APP_VERSION}</span>
        </>
      );
    }

    switch (pathname) {
      case "/":
        // Home: Just version
        return <span>{APP_VERSION}</span>;

      case "/data":
        // Data: Active tab + version
        return (
          <>
            <span>Original</span>
            <span className="text-muted-foreground/60">|</span>
            <span>{APP_VERSION}</span>
          </>
        );

      case "/processing":
        // Processing: Status + version
        return (
          <>
            {renderPreprocessingStatus()}
            <span className="text-muted-foreground/60">|</span>
            <span>{APP_VERSION}</span>
          </>
        );

      case "/analysis":
      case "/ml":
        // Analysis/ML: Coming soon + version
        return (
          <>
            <span className="text-muted-foreground/60">Coming Soon</span>
            <span className="text-muted-foreground/60">|</span>
            <span>{APP_VERSION}</span>
          </>
        );

      case "/settings":
        // Settings: Just version
        return <span>{APP_VERSION}</span>;

      default:
        return <span>{APP_VERSION}</span>;
    }
  };

  /**
   * Renders the preprocessing status indicator.
   */
  const renderPreprocessingStatus = () => {
    switch (preprocessingStatus) {
      case "idle":
        return <span>Idle</span>;
      case "running":
        return <span className="text-primary animate-pulse">Processing...</span>;
      case "completed":
        return <span className="text-green-500">Complete</span>;
      case "cancelled":
        return <span className="text-yellow-500">Cancelled</span>;
      case "error":
        return <span className="text-red-500">Error</span>;
      default:
        return <span>Idle</span>;
    }
  };

  return (
    <footer className="flex items-center justify-between h-6 px-5 text-xs border-t bg-background text-muted-foreground">
      {/* Left side: Page-specific content */}
      <div className="flex items-center gap-2">{renderLeftContent()}</div>

      {/* Right side: Page-specific content */}
      <div className="flex items-center gap-2">{renderRightContent()}</div>
    </footer>
  );
};

export default StatusBar;
