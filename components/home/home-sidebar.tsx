"use client";

import { FolderOpen, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardContent } from "@/components/ui/card";
import { cn, formatBytes } from "@/lib/utils";
import type { FileInfo, AIProviderType } from "@/types";

// ============================================================================
// START SECTION
// ============================================================================

interface StartSectionProps {
  onOpenDataset: () => void;
}

/**
 * Start section with the primary "Open Dataset" action.
 */
const StartSection = ({ onOpenDataset }: StartSectionProps) => {
  return (
    <Card>
      <CardHeader title="Start" />
      <CardContent padded>
        <Button
          variant="ghost"
          className="w-full justify-start gap-2 h-8 px-2 text-sm -mx-2"
          onClick={onOpenDataset}
        >
          <FolderOpen className="w-4 h-4" />
          Open Dataset
        </Button>
      </CardContent>
    </Card>
  );
};

// ============================================================================
// CURRENT FILE SECTION
// ============================================================================

interface CurrentFileSectionProps {
  fileInfo: FileInfo | null;
  onClose: () => void;
}

/**
 * Current file section showing the loaded dataset info or empty state.
 */
const CurrentFileSection = ({ fileInfo, onClose }: CurrentFileSectionProps) => {
  if (!fileInfo) {
    return (
      <Card>
        <CardHeader title="Current" />
        <CardContent padded>
          <p className="text-sm text-muted-foreground">No file loaded</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader
        title="Current"
        actions={
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
            title="Close file"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        }
      />
      <CardContent padded>
        <div className="min-w-0">
          <p
            className="text-sm font-medium truncate"
            title={fileInfo.name}
          >
            {fileInfo.name}
          </p>
          <p className="text-xs text-muted-foreground mt-1">
            {fileInfo.row_count.toLocaleString()} rows · {fileInfo.column_count} cols
          </p>
          <p className="text-xs text-muted-foreground">
            {formatBytes(fileInfo.size_bytes)}
          </p>
        </div>
      </CardContent>
    </Card>
  );
};

// ============================================================================
// STATUS SECTION
// ============================================================================

interface StatusSectionProps {
  aiProvider: AIProviderType;
}

/**
 * Status section showing AI provider status.
 */
const StatusSection = ({ aiProvider }: StatusSectionProps) => {
  const getProviderLabel = (provider: AIProviderType): string => {
    switch (provider) {
      case "openrouter":
        return "OpenRouter";
      case "gemini":
        return "Gemini";
      case "none":
      default:
        return "Off";
    }
  };

  return (
    <Card>
      <CardHeader title="Status" />
      <CardContent padded>
        <div className="text-xs text-muted-foreground space-y-1">
          <div className="flex items-center justify-between">
            <span>AI</span>
            <span
              className={cn(
                aiProvider !== "none" ? "text-foreground" : "text-muted-foreground"
              )}
            >
              {getProviderLabel(aiProvider)}
            </span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

// ============================================================================
// HOME SIDEBAR CONTENT
// ============================================================================

interface HomeSidebarContentProps {
  fileInfo: FileInfo | null;
  aiProvider: AIProviderType;
  onOpenDataset: () => void;
  onCloseFile: () => void;
}

/**
 * Content for the home page sidebar.
 *
 * This is meant to be used as children of ContextSidebar.
 * It provides the inner content without the sidebar wrapper.
 *
 * Shows:
 * - Current file info (if loaded)
 * - Start section with Open Dataset action
 * - Status section with AI provider info
 */
const HomeSidebarContent = ({
  fileInfo,
  aiProvider,
  onOpenDataset,
  onCloseFile,
}: HomeSidebarContentProps) => {
  return (
    <div className="flex flex-col p-3 gap-3">
      {/* Start section */}
      <StartSection onOpenDataset={onOpenDataset} />

      {/* Current file section - always shown */}
      <CurrentFileSection fileInfo={fileInfo} onClose={onCloseFile} />

      {/* Status section */}
      <StatusSection aiProvider={aiProvider} />
    </div>
  );
};

export default HomeSidebarContent;
