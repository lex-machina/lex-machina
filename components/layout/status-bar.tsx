"use client";

import { useEffect, useMemo, useState } from "react";
import { usePathname } from "next/navigation";
import { invoke } from "@tauri-apps/api/core";
import { useFileState } from "@/lib/hooks/use-file-state";
import { useAppStatus } from "@/lib/hooks/use-app-status";
import { usePreprocessing } from "@/lib/hooks/use-preprocessing";
import type { AnalysisResult, AnalysisUIState } from "@/types";
import { formatBytes, formatDuration, formatNumber } from "@/lib/utils";

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
    const [analysisResult, setAnalysisResult] = useState<AnalysisResult | null>(
        null,
    );
    const [analysisUiState, setAnalysisUiState] =
        useState<AnalysisUIState | null>(null);

    const isAnalysisPage = pathname === "/analysis";
    const isAnalysisLoading =
        isAnalysisPage &&
        isLoading &&
        Boolean(loadingMessage?.toLowerCase().includes("analysis"));

    const analysisTabLabel = useMemo(() => {
        if (!analysisUiState) {
            return "Overview";
        }
        const labels: Record<string, string> = {
            overview: "Overview",
            columns: "Columns",
            missingness: "Missingness",
            correlations: "Correlations",
            associations: "Associations",
            quality: "Quality",
        };
        return labels[analysisUiState.active_tab] ?? "Overview";
    }, [analysisUiState]);

    useEffect(() => {
        if (!isAnalysisPage) {
            setAnalysisResult(null);
            setAnalysisUiState(null);
            return;
        }

        let isActive = true;

        const loadUiState = async () => {
            try {
                const state = await invoke<AnalysisUIState>(
                    "get_analysis_ui_state",
                );
                if (isActive) {
                    setAnalysisUiState(state);
                }
            } catch (err) {
                if (isActive) {
                    setAnalysisUiState(null);
                }
            }
        };

        loadUiState();
        const intervalId = setInterval(loadUiState, 1200);

        return () => {
            isActive = false;
            clearInterval(intervalId);
        };
    }, [isAnalysisPage]);

    useEffect(() => {
        if (!isAnalysisPage || !analysisUiState) {
            return;
        }

        if (isAnalysisLoading) {
            return;
        }

        let isActive = true;

        invoke<AnalysisResult | null>("get_analysis_result", {
            useProcessedData: analysisUiState.use_processed_data,
        })
            .then((result) => {
                if (isActive) {
                    setAnalysisResult(result);
                }
            })
            .catch(() => {
                if (isActive) {
                    setAnalysisResult(null);
                }
            });

        return () => {
            isActive = false;
        };
    }, [isAnalysisPage, analysisUiState, isAnalysisLoading]);

    /**
     * Renders the left content based on current page.
     */
    const renderLeftContent = () => {
        // Loading state takes precedence on all pages
        if (isLoading && loadingMessage) {
            return (
                <span className="text-primary animate-pulse">
                    {loadingMessage}
                </span>
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
                            {formatNumber(fileInfo.row_count)} x{" "}
                            {fileInfo.column_count}
                        </span>
                    </>
                ) : (
                    <span>No file loaded</span>
                );

            case "/analysis":
                if (!analysisUiState) {
                    return <span>Analysis</span>;
                }

                return (
                    <>
                        <span className="text-foreground font-medium">
                            {analysisUiState.use_processed_data
                                ? "Processed"
                                : "Original"}{" "}
                            dataset
                        </span>
                        <span className="text-muted-foreground/60">|</span>
                        <span>Tab: {analysisTabLabel}</span>
                        {analysisResult?.generated_at && (
                            <>
                                <span className="text-muted-foreground/60">
                                    |
                                </span>
                                <span>
                                    Last run:{" "}
                                    {new Date(
                                        analysisResult.generated_at,
                                    ).toLocaleString()}
                                </span>
                            </>
                        )}
                        {analysisResult?.duration_ms !== undefined && (
                            <>
                                <span className="text-muted-foreground/60">
                                    |
                                </span>
                                <span>
                                    Runtime:{" "}
                                    {formatDuration(analysisResult.duration_ms)}
                                </span>
                            </>
                        )}
                        <span className="text-muted-foreground/60">|</span>
                        <span className="tracking-wide uppercase">
                            {isAnalysisLoading && "Running"}
                            {!isAnalysisLoading && analysisResult && "Complete"}
                            {!isAnalysisLoading && !analysisResult && "Idle"}
                        </span>
                    </>
                );

            case "/ml":
                // ML: File info or no file
                return fileInfo ? (
                    <>
                        <span>{fileInfo.name}</span>
                        <span className="text-muted-foreground/60">|</span>
                        <span>{formatNumber(fileInfo.row_count)} rows</span>
                    </>
                ) : (
                    <span>No file loaded</span>
                );

            case "/visualizations":
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
                    <span className="text-primary animate-pulse">
                        Loading...
                    </span>
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
                return <span>{APP_VERSION}</span>;

            case "/visualizations":
                return (
                    <>
                        <span>Visualizations</span>
                        <span className="text-muted-foreground/60">|</span>
                        <span>{APP_VERSION}</span>
                    </>
                );

            case "/ml":
                return (
                    <>
                        <span>ML</span>
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
                return <span>Processing...</span>;
            case "completed":
                return <span>Complete</span>;
            case "cancelled":
                return <span>Cancelled</span>;
            case "error":
                return <span>Error</span>;
            default:
                return <span>Idle</span>;
        }
    };

    return (
        <footer className="bg-background text-muted-foreground flex h-6 items-center justify-between border-t px-5 text-xs">
            {/* Left side: Page-specific content */}
            <div className="flex items-center gap-2">{renderLeftContent()}</div>

            {/* Right side: Page-specific content */}
            <div className="flex items-center gap-2">
                {renderRightContent()}
            </div>
        </footer>
    );
};

export default StatusBar;
