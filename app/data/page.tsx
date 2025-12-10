"use client";

import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { FileInfo } from "@/types";
import { useFileState } from "@/lib/hooks/use-file-state";
import { useAppStatus } from "@/lib/hooks/use-app-status";
import AppShell from "@/components/layout/app-shell";
import DataGrid from "@/components/data-grid";
import ContextSidebar from "@/components/layout/context-sidebar";
import { Button } from "@/components/ui/button";
import { toast } from "@/components/ui/toast";
import { formatBytes, formatNumber } from "@/lib/utils";

/**
 * Data Toolbar - Import/Clear buttons for the data page.
 */
const DataToolbar = () => {
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
    <>
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
    </>
  );
};

/**
 * Data Sidebar - File info and column list.
 */
const DataSidebar = () => {
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
};

/**
 * Data page - CSV data viewer with grid display.
 *
 * Features:
 * - Import CSV files
 * - Virtual scrolling data grid
 * - File info sidebar
 * - Column resizing
 */
const DataPage = () => {
  return (
    <AppShell
      toolbar={<DataToolbar />}
      sidebar={
        <ContextSidebar visible={true}>
          <DataSidebar />
        </ContextSidebar>
      }
    >
      <DataGrid />
    </AppShell>
  );
};

export default DataPage;
