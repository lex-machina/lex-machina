"use client";

import { Suspense, useCallback, useEffect } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { invoke } from "@tauri-apps/api/core";

import type { FileInfo, ExportResult } from "@/types";
import { useFileState } from "@/lib/hooks/use-file-state";
import { useProcessedData } from "@/lib/hooks/use-processed-data";
import { useAppStatus } from "@/lib/hooks/use-app-status";
import AppShell from "@/components/layout/app-shell";
import { DataGrid, ProcessedDataGrid } from "@/components/data-grid";
import ContextSidebar from "@/components/layout/context-sidebar";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { toast } from "@/components/ui/toast";
import { formatBytes, formatNumber } from "@/lib/utils";

// ============================================================================
// TYPES
// ============================================================================

type DataTab = "original" | "processed";

// ============================================================================
// DATA TOOLBAR
// ============================================================================

interface DataToolbarProps {
  activeTab: DataTab;
  onTabChange: (tab: DataTab) => void;
  hasProcessedData: boolean;
}

/**
 * Data Toolbar - Import/Clear buttons and tab navigation.
 */
function DataToolbar({ activeTab, onTabChange, hasProcessedData }: DataToolbarProps) {
  const { fileInfo } = useFileState();
  const { isLoading, loadingMessage } = useAppStatus();

  const handleImport = useCallback(async () => {
    try {
      const filePath = await invoke<string | null>("open_file_dialog");
      if (!filePath) return;

      if (fileInfo) {
        await invoke("close_file");
      }

      await invoke<FileInfo>("load_file", { path: filePath });
      toast.success("File loaded successfully");
    } catch (err) {
      toast.error(`Failed to import file: ${err}`);
    }
  }, [fileInfo]);

  const handleClear = useCallback(async () => {
    try {
      await invoke("close_file");
      toast.info("File closed");
    } catch (err) {
      toast.error(`Failed to clear file: ${err}`);
    }
  }, []);

  return (
    <div className="flex items-center gap-4 w-full">
      {/* Left side: Import/Clear buttons */}
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={handleImport}
          disabled={isLoading}
        >
          {isLoading && loadingMessage ? loadingMessage : "Import File"}
        </Button>
        {fileInfo && (
          <Button
            variant="ghost"
            size="sm"
            onClick={handleClear}
            disabled={isLoading}
          >
            Clear File
          </Button>
        )}
      </div>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Right side: Tabs */}
      <Tabs value={activeTab} onValueChange={(v) => onTabChange(v as DataTab)}>
        <TabsList>
          <TabsTrigger value="original">Original</TabsTrigger>
          <TabsTrigger value="processed" disabled={!hasProcessedData}>
            Processed
          </TabsTrigger>
        </TabsList>
      </Tabs>
    </div>
  );
}

// ============================================================================
// SIDEBAR COMPONENTS
// ============================================================================

/**
 * Original Data Sidebar - File info and column list for original data.
 */
function OriginalDataSidebar() {
  const { fileInfo } = useFileState();

  if (!fileInfo) {
    return (
      <div className="p-5">
        <p className="text-sm text-muted-foreground">
          Import a file to view details
        </p>
      </div>
    );
  }

  return (
    <div className="p-5 space-y-5">
      <section>
        <h2 className="text-xs font-semibold uppercase text-muted-foreground mb-3">
          File Info
        </h2>
        <dl className="space-y-2 text-sm">
          <div>
            <dt className="text-muted-foreground">Name</dt>
            <dd className="font-medium truncate" title={fileInfo.name}>
              {fileInfo.name}
            </dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Path</dt>
            <dd className="font-mono text-xs truncate" title={fileInfo.path}>
              {fileInfo.path}
            </dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Size</dt>
            <dd className="font-medium">
              {formatBytes(fileInfo.size_bytes)}
            </dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Rows</dt>
            <dd className="font-medium">
              {formatNumber(fileInfo.row_count)}
            </dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Columns</dt>
            <dd className="font-medium">
              {formatNumber(fileInfo.column_count)}
            </dd>
          </div>
        </dl>
      </section>

      <section>
        <h2 className="text-xs font-semibold uppercase text-muted-foreground mb-3">
          Columns
        </h2>
        <ul className="space-y-1">
          {fileInfo.columns.map((col) => (
            <li
              key={col.name}
              className="flex items-center justify-between text-sm py-1 px-2 rounded hover:bg-muted/50"
            >
              <span className="truncate font-medium" title={col.name}>
                {col.name}
              </span>
              <span className="text-xs text-muted-foreground ml-2 shrink-0">
                {col.dtype}
              </span>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}

/**
 * Processed Data Sidebar - Info about processed data.
 */
function ProcessedDataSidebar() {
  const { fileInfo, summary, clearProcessedData } = useProcessedData();

  const handleExport = useCallback(async () => {
    try {
      const result = await invoke<ExportResult>("export_processed_data");
      toast.success(`Exported to ${result.csv_path}`);
    } catch (err) {
      // Silently handle user cancellation
      if (err !== "Export cancelled by user") {
        toast.error(`Export failed: ${err}`);
      }
    }
  }, []);

  const handleClear = useCallback(async () => {
    try {
      await clearProcessedData();
      toast.info("Processed data cleared");
    } catch (err) {
      toast.error(`Failed to clear processed data: ${err}`);
    }
  }, [clearProcessedData]);

  if (!fileInfo) {
    return (
      <div className="p-5">
        <p className="text-sm text-muted-foreground">
          No processed data available
        </p>
      </div>
    );
  }

  return (
    <div className="p-5 space-y-5">
      {/* Preprocessing Summary */}
      {summary && (
        <section>
          <h2 className="text-xs font-semibold uppercase text-muted-foreground mb-3">
            Preprocessing Summary
          </h2>
          <dl className="space-y-2 text-sm">
            <div className="flex items-center justify-between">
              <dt className="text-muted-foreground">Quality Score</dt>
              <dd className="font-medium">
                <span className="text-green-500">
                  {(summary.data_quality_score_after * 100).toFixed(0)}%
                </span>
                <span className="text-muted-foreground mx-1">from</span>
                <span className="text-amber-500">
                  {(summary.data_quality_score_before * 100).toFixed(0)}%
                </span>
              </dd>
            </div>
            <div className="flex items-center justify-between">
              <dt className="text-muted-foreground">Issues Resolved</dt>
              <dd className="font-medium">
                {summary.issues_resolved} / {summary.issues_found}
              </dd>
            </div>
            <div className="flex items-center justify-between">
              <dt className="text-muted-foreground">Rows Removed</dt>
              <dd className="font-medium">{summary.rows_removed}</dd>
            </div>
            <div className="flex items-center justify-between">
              <dt className="text-muted-foreground">Columns Removed</dt>
              <dd className="font-medium">{summary.columns_removed}</dd>
            </div>
          </dl>
        </section>
      )}

      {/* Processed Data Info */}
      <section>
        <h2 className="text-xs font-semibold uppercase text-muted-foreground mb-3">
          Processed Data
        </h2>
        <dl className="space-y-2 text-sm">
          <div>
            <dt className="text-muted-foreground">Rows</dt>
            <dd className="font-medium">
              {formatNumber(fileInfo.row_count)}
            </dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Columns</dt>
            <dd className="font-medium">
              {formatNumber(fileInfo.column_count)}
            </dd>
          </div>
        </dl>
      </section>

      {/* Columns */}
      <section>
        <h2 className="text-xs font-semibold uppercase text-muted-foreground mb-3">
          Columns
        </h2>
        <ul className="space-y-1">
          {fileInfo.columns.map((col) => (
            <li
              key={col.name}
              className="flex items-center justify-between text-sm py-1 px-2 rounded hover:bg-muted/50"
            >
              <span className="truncate font-medium" title={col.name}>
                {col.name}
              </span>
              <span className="text-xs text-muted-foreground ml-2 shrink-0">
                {col.dtype}
              </span>
            </li>
          ))}
        </ul>
      </section>

      {/* Action Buttons */}
      <section className="space-y-2">
        <Button
          variant="outline"
          size="sm"
          onClick={handleExport}
          className="w-full"
        >
          Export CSV
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleClear}
          className="w-full text-destructive hover:text-destructive hover:bg-destructive/10"
        >
          Clear Processed Data
        </Button>
      </section>
    </div>
  );
}

// ============================================================================
// DATA PAGE
// ============================================================================

/**
 * Data page content - uses useSearchParams which requires Suspense boundary.
 */
function DataPageContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  
  const { hasProcessedData, refresh: refreshProcessedData } = useProcessedData();
  
  // Determine active tab from URL params and available data
  // Use URL param if it's "processed" and we have processed data, otherwise "original"
  const tabParam = searchParams.get("tab");
  const activeTab: DataTab = tabParam === "processed" && hasProcessedData 
    ? "processed" 
    : "original";

  // Refresh processed data on mount to check if it exists
  useEffect(() => {
    refreshProcessedData();
  }, [refreshProcessedData]);

  // Handle tab change - update URL
  const handleTabChange = useCallback(
    (tab: DataTab) => {
      // Update URL without full navigation
      const url = new URL(window.location.href);
      if (tab === "processed") {
        url.searchParams.set("tab", "processed");
      } else {
        url.searchParams.delete("tab");
      }
      router.replace(url.pathname + url.search, { scroll: false });
    },
    [router]
  );

  return (
    <AppShell
      toolbar={
        <DataToolbar
          activeTab={activeTab}
          onTabChange={handleTabChange}
          hasProcessedData={hasProcessedData}
        />
      }
      sidebar={
        <ContextSidebar visible={true}>
          {activeTab === "original" ? (
            <OriginalDataSidebar />
          ) : (
            <ProcessedDataSidebar />
          )}
        </ContextSidebar>
      }
    >
      {/* Tab Content */}
      {activeTab === "original" ? (
        <DataGrid />
      ) : (
        <ProcessedDataGrid />
      )}
    </AppShell>
  );
}

/**
 * Data page - CSV data viewer with grid display.
 *
 * Features:
 * - Import CSV files
 * - Virtual scrolling data grid
 * - File info sidebar
 * - Column resizing
 * - Tabs for Original and Processed data
 *
 * URL Parameters:
 * - ?tab=processed - Switch to processed data tab
 */
export default function DataPage() {
  return (
    <Suspense fallback={<DataPageFallback />}>
      <DataPageContent />
    </Suspense>
  );
}

/**
 * Fallback UI while Suspense boundary is loading.
 */
function DataPageFallback() {
  return (
    <AppShell
      toolbar={<div className="h-8" />}
      sidebar={<ContextSidebar visible={true}><div className="p-5" /></ContextSidebar>}
    >
      <div className="flex items-center justify-center h-full text-muted-foreground">
        Loading...
      </div>
    </AppShell>
  );
}
