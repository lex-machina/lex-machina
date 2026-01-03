"use client";

import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { FileInfo } from "@/types";
import { useFileState } from "@/lib/hooks/use-file-state";
import { useSettings } from "@/lib/hooks/use-settings";
import AppShell from "@/components/layout/app-shell";
import ContextSidebar from "@/components/layout/context-sidebar";
import { toast } from "@/components/ui/toast";
import { Logo, HomeSidebarContent, HomeMainContent } from "@/components/home";

// ============================================================================
// HOME PAGE
// ============================================================================

/**
 * Home page with VS Code-style layout.
 *
 * Layout:
 * - Main area: Logo centered with workflow/links below
 * - Right sidebar: Start actions, current file info, status (using ContextSidebar)
 *
 * The page adapts based on whether a file is loaded:
 * - No file: Shows logo + links, sidebar shows "Open Dataset"
 * - File loaded: Same layout, sidebar shows current file info
 */
export default function HomePage() {
  const { fileInfo, isFileLoaded } = useFileState();
  const { aiProviderType } = useSettings();

  // ==========================================================================
  // HANDLERS
  // ==========================================================================

  /**
   * Opens the native file dialog and loads the selected file.
   */
  const handleOpenDataset = useCallback(async () => {
    try {
      const filePath = await invoke<string | null>("open_file_dialog");
      if (!filePath) return;

      // Close existing file if one is loaded
      if (fileInfo) {
        await invoke("close_file");
      }

      await invoke<FileInfo>("load_file", { path: filePath });
      toast.success("Dataset loaded successfully");
    } catch (err) {
      toast.error(`Failed to load dataset: ${err}`);
    }
  }, [fileInfo]);

  /**
   * Closes the currently loaded file.
   */
  const handleCloseFile = useCallback(async () => {
    try {
      await invoke("close_file");
      toast.success("Dataset closed");
    } catch (err) {
      toast.error(`Failed to close dataset: ${err}`);
    }
  }, []);

  // ==========================================================================
  // RENDER
  // ==========================================================================

  return (
    <AppShell
      toolbar={null}
      sidebar={
        <ContextSidebar>
          <HomeSidebarContent
            fileInfo={isFileLoaded ? fileInfo : null}
            aiProvider={aiProviderType}
            onOpenDataset={handleOpenDataset}
            onCloseFile={handleCloseFile}
          />
        </ContextSidebar>
      }
    >
      {/* Main content area - centered logo with links below */}
      <div className="flex-1 flex flex-col items-center justify-center p-8">
        {/* Logo */}
        <Logo className="mb-12" />

        {/* Workflow and Links */}
        <HomeMainContent />
      </div>
    </AppShell>
  );
}
