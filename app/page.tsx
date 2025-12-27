"use client";

import { useCallback } from "react";
import { useRouter } from "next/navigation";
import { invoke } from "@tauri-apps/api/core";
import {
  FileUp,
  Table2,
  Cog,
  Settings,
  X,
  CheckCircle2,
  Circle,
  ChevronRight,
} from "lucide-react";

import type { FileInfo } from "@/types";
import { useFileState } from "@/lib/hooks/use-file-state";
import { usePreprocessing } from "@/lib/hooks/use-preprocessing";
import AppShell from "@/components/layout/app-shell";
import { Button } from "@/components/ui/button";
import { toast } from "@/components/ui/toast";
import { cn } from "@/lib/utils";

// ============================================================================
// HELPERS
// ============================================================================

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

// ============================================================================
// WELCOME STATE (No file loaded)
// ============================================================================

interface WelcomeStateProps {
  onImport: () => void;
}

const WelcomeState = ({ onImport }: WelcomeStateProps) => {
  return (
    <div className="flex-1 flex flex-col items-center justify-center p-8">
      {/* Logo/Title */}
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2">Lex Machina</h1>
        <p className="text-muted-foreground">No-Code AutoML for Everyone</p>
      </div>

      {/* Import Card - Settings style */}
      <div className="border rounded-lg max-w-md w-full">
        <div className="px-4 py-3 border-b bg-muted/30">
          <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Get Started
          </h2>
        </div>
        <div className="p-6 text-center">
          <div className="w-14 h-14 rounded-full bg-muted flex items-center justify-center mx-auto mb-4">
            <FileUp className="w-7 h-7 text-muted-foreground" />
          </div>
          <p className="text-sm text-muted-foreground mb-5">
            Import a CSV file to begin analyzing your data
          </p>
          <Button onClick={onImport} className="gap-2">
            <FileUp className="w-4 h-4" />
            Import Data
          </Button>
        </div>
      </div>

      {/* Features Row - text only, no icons */}
      <div className="flex items-center gap-6 mt-8 text-xs text-muted-foreground">
        <span>Local-first</span>
        <span className="text-muted-foreground/30">·</span>
        <span>AutoML</span>
        <span className="text-muted-foreground/30">·</span>
        <span>Explainable</span>
        <span className="text-muted-foreground/30">·</span>
        <span>No-Code</span>
      </div>
    </div>
  );
};

// ============================================================================
// WORKING STATE (File loaded)
// ============================================================================

interface WorkflowStep {
  id: string;
  label: string;
  completed: boolean;
  active: boolean;
}

interface WorkingStateProps {
  fileInfo: FileInfo;
  hasProcessed: boolean;
  onClear: () => void;
  onNavigate: (path: string) => void;
}

const WorkingState = ({ fileInfo, hasProcessed, onClear, onNavigate }: WorkingStateProps) => {
  const steps: WorkflowStep[] = [
    { id: "import", label: "Import", completed: true, active: false },
    { id: "process", label: "Process", completed: hasProcessed, active: !hasProcessed },
    { id: "analyze", label: "Analyze", completed: false, active: false },
    { id: "train", label: "Train", completed: false, active: false },
  ];

  const quickActions = [
    { id: "data", label: "View Data", icon: Table2, path: "/data" },
    { id: "process", label: "Process", icon: Cog, path: "/processing" },
    { id: "settings", label: "Settings", icon: Settings, path: "/settings" },
  ];

  return (
    <div className="flex-1 flex flex-col items-center justify-center p-8">
      {/* Logo/Title */}
      <div className="text-center mb-6">
        <h1 className="text-3xl font-bold mb-2">Lex Machina</h1>
        <p className="text-muted-foreground">No-Code AutoML for Everyone</p>
      </div>

      {/* File Info Card */}
      <div className="border rounded-lg w-full max-w-lg mb-4">
        <div className="px-4 py-2 border-b bg-muted/30 flex items-center justify-between">
          <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Current File
          </h2>
          <button
            onClick={onClear}
            className="p-1 rounded hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
            title="Close file"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
        <div className="px-4 py-3 flex items-center justify-between">
          <div>
            <p className="text-sm font-medium truncate" title={fileInfo.name}>
              {fileInfo.name}
            </p>
            <p className="text-xs text-muted-foreground">
              {fileInfo.row_count.toLocaleString()} rows · {fileInfo.column_count} cols · {formatBytes(fileInfo.size_bytes)}
            </p>
          </div>
        </div>
      </div>

      {/* Workflow Stepper - inline */}
      <div className="flex items-center gap-1 mb-6">
        {steps.map((step, index) => (
          <div key={step.id} className="flex items-center">
            <div className="flex items-center gap-1.5">
              {step.completed ? (
                <CheckCircle2 className="w-4 h-4 text-green-500" />
              ) : step.active ? (
                <Circle className="w-4 h-4 text-primary fill-primary/20" />
              ) : (
                <Circle className="w-4 h-4 text-muted-foreground/30" />
              )}
              <span
                className={cn(
                  "text-sm",
                  step.completed && "text-foreground",
                  step.active && "text-foreground font-medium",
                  !step.completed && !step.active && "text-muted-foreground/50"
                )}
              >
                {step.label}
              </span>
            </div>
            {index < steps.length - 1 && (
              <ChevronRight className="w-4 h-4 mx-2 text-muted-foreground/30" />
            )}
          </div>
        ))}
      </div>

      {/* Quick Actions - inline buttons */}
      <div className="flex items-center gap-2 mb-8">
        {quickActions.map((action) => (
          <Button
            key={action.id}
            variant="outline"
            size="sm"
            onClick={() => onNavigate(action.path)}
            className="gap-1.5"
          >
            <action.icon className="w-4 h-4" />
            {action.label}
          </Button>
        ))}
      </div>

      {/* Features Row - text only */}
      <div className="flex items-center gap-6 text-xs text-muted-foreground">
        <span>Local-first</span>
        <span className="text-muted-foreground/30">·</span>
        <span>AutoML</span>
        <span className="text-muted-foreground/30">·</span>
        <span>Explainable</span>
        <span className="text-muted-foreground/30">·</span>
        <span>No-Code</span>
      </div>
    </div>
  );
};

// ============================================================================
// HOME PAGE
// ============================================================================

export default function HomePage() {
  const router = useRouter();
  const { fileInfo, isFileLoaded } = useFileState();
  const { status } = usePreprocessing();

  const hasProcessed = status === "completed";

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

  const handleClearFile = useCallback(async () => {
    try {
      await invoke("close_file");
      toast.success("File closed");
    } catch (err) {
      toast.error(`Failed to close file: ${err}`);
    }
  }, []);

  const handleNavigate = useCallback(
    (path: string) => {
      router.push(path);
    },
    [router]
  );

  return (
    <AppShell toolbar={null}>
      {isFileLoaded && fileInfo ? (
        <WorkingState
          fileInfo={fileInfo}
          hasProcessed={hasProcessed}
          onClear={handleClearFile}
          onNavigate={handleNavigate}
        />
      ) : (
        <WelcomeState onImport={handleImport} />
      )}
    </AppShell>
  );
}
