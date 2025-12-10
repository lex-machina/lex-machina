"use client";

import { useCallback } from "react";
import { useRouter } from "next/navigation";
import { invoke } from "@tauri-apps/api/core";

import type { FileInfo } from "@/types";
import { useFileState } from "@/lib/hooks/use-file-state";
import AppShell from "@/components/layout/app-shell";
import { toast } from "@/components/ui/toast";

/**
 * Home Toolbar - Quick actions for the home page.
 */
const HomeToolbar = () => {
  return null; // Home page has no toolbar actions
};

/**
 * Quick action card component.
 */
interface ActionCardProps {
  title: string;
  description: string;
  icon: React.ReactNode;
  onClick: () => void;
  disabled?: boolean;
}

const ActionCard = ({
  title,
  description,
  icon,
  onClick,
  disabled,
}: ActionCardProps) => (
  <button
    onClick={onClick}
    disabled={disabled}
    className="flex flex-col items-center p-6 rounded-lg border bg-card text-card-foreground hover:bg-muted/50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed w-48"
  >
    <div className="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center mb-4 text-primary">
      {icon}
    </div>
    <h3 className="font-semibold mb-1">{title}</h3>
    <p className="text-xs text-muted-foreground text-center">{description}</p>
  </button>
);

/**
 * Icon components for action cards.
 */
const ImportIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
    <polyline points="17 8 12 3 7 8" />
    <line x1="12" y1="3" x2="12" y2="15" />
  </svg>
);

const ViewDataIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
    <line x1="3" y1="9" x2="21" y2="9" />
    <line x1="3" y1="15" x2="21" y2="15" />
    <line x1="9" y1="3" x2="9" y2="21" />
    <line x1="15" y1="3" x2="15" y2="21" />
  </svg>
);

const AnalyzeIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <line x1="18" y1="20" x2="18" y2="10" />
    <line x1="12" y1="20" x2="12" y2="4" />
    <line x1="6" y1="20" x2="6" y2="14" />
  </svg>
);

/**
 * Home page content - Welcome screen with quick actions.
 */
const HomeContent = () => {
  const router = useRouter();
  const { fileInfo, isFileLoaded } = useFileState();

  const handleImport = useCallback(async () => {
    try {
      const filePath = await invoke<string | null>("open_file_dialog");
      if (!filePath) return;

      if (fileInfo) {
        await invoke("close_file");
      }

      await invoke<FileInfo>("load_file", { path: filePath });
      toast.success("File loaded successfully");
      // Navigate to data page after successful import
      router.push("/data");
    } catch (err) {
      toast.error(`Failed to import file: ${err}`);
    }
  }, [fileInfo, router]);

  const handleViewData = useCallback(() => {
    router.push("/data");
  }, [router]);

  const handleAnalyze = useCallback(() => {
    router.push("/analysis");
  }, [router]);

  return (
    <div className="flex-1 flex flex-col items-center justify-center p-8">
      {/* Welcome Header */}
      <div className="text-center mb-12">
        <h1 className="text-4xl font-bold mb-2">Lex Machina</h1>
        <p className="text-lg text-muted-foreground">
          No-Code Data Analytics & AutoML
        </p>
      </div>

      {/* Current File Status */}
      {isFileLoaded && fileInfo && (
        <div className="mb-8 p-4 rounded-lg bg-muted/50 border text-center">
          <p className="text-sm text-muted-foreground mb-1">Current file</p>
          <p className="font-medium">{fileInfo.name}</p>
          <p className="text-xs text-muted-foreground">
            {fileInfo.row_count.toLocaleString()} rows, {fileInfo.column_count}{" "}
            columns
          </p>
        </div>
      )}

      {/* Quick Actions */}
      <div className="flex gap-4">
        <ActionCard
          title="Import Data"
          description="Load a CSV file to get started"
          icon={<ImportIcon />}
          onClick={handleImport}
        />
        <ActionCard
          title="View Data"
          description="Browse and explore your dataset"
          icon={<ViewDataIcon />}
          onClick={handleViewData}
        />
        <ActionCard
          title="Analyze"
          description="Run statistical analysis"
          icon={<AnalyzeIcon />}
          onClick={handleAnalyze}
          disabled={!isFileLoaded}
        />
      </div>

      {/* Getting Started Hint */}
      {!isFileLoaded && (
        <p className="mt-12 text-sm text-muted-foreground">
          Import a CSV file to begin your data analysis journey
        </p>
      )}
    </div>
  );
};

/**
 * Home page - Welcome screen with quick actions.
 *
 * Features:
 * - Quick import action
 * - Navigation to data/analysis pages
 * - Current file status display
 */
const HomePage = () => {
  return (
    <AppShell toolbar={<HomeToolbar />}>
      <HomeContent />
    </AppShell>
  );
};

export default HomePage;
