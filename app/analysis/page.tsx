"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import { BarChart3 } from "lucide-react";

import AppShell from "@/components/layout/app-shell";
import { AnalysisSidebar, AnalysisWorkspace } from "@/components/analysis";
import { Button } from "@/components/ui/button";
import { useAnalysis } from "@/lib/hooks/use-analysis";
import { useAnalysisUIState } from "@/lib/hooks/use-analysis-ui-state";
import { useFileState } from "@/lib/hooks/use-file-state";
import { useProcessedData } from "@/lib/hooks/use-processed-data";
import { cn } from "@/lib/utils";

/**
 * Empty state component shown when no file is loaded.
 */
function NoFileLoadedState() {
    return (
        <div className="flex flex-1 items-center justify-center p-8">
            <div className="max-w-md text-center">
                {/* Icon */}
                <div className="bg-muted mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full">
                    <BarChart3 className="text-muted-foreground h-8 w-8" />
                </div>

                {/* Title and description */}
                <h2 className="mb-2 text-xl font-semibold">
                    Statistical Analysis
                </h2>
                <p className="text-muted-foreground mb-6">
                    Import a dataset to run full statistical profiling,
                    correlations, and quality diagnostics.
                </p>

                {/* Action button */}
                <Button asChild size="lg">
                    <Link href="/data">Go to Data</Link>
                </Button>
            </div>
        </div>
    );
}

const NoAnalysisState = ({
    onRun,
    isRunning,
    error,
}: {
    onRun: () => void;
    isRunning: boolean;
    error: string | null;
}) => {
    return (
        <div className="flex flex-1 items-center justify-center">
            <div className="max-w-md text-center">
                <h2 className="mb-2 text-xl font-semibold">Ready to analyze</h2>
                <p className="text-muted-foreground mb-4">
                    Run the analysis pipeline to generate full profiling,
                    diagnostics, and charts.
                </p>
                {error && (
                    <p className="text-destructive mb-4 text-sm">{error}</p>
                )}
                <Button onClick={onRun} disabled={isRunning} size="lg">
                    {isRunning ? "Running analysis..." : "Run Analysis"}
                </Button>
            </div>
        </div>
    );
};

/**
 * Analysis page - Statistical analysis and data profiling.
 *
 * Features (planned):
 * - Descriptive statistics
 * - Data profiling
 * - Histograms and distributions
 * - Correlation analysis
 * - Missing value analysis
 */
const AnalysisPage = () => {
    const { fileInfo, isFileLoaded } = useFileState();
    const processedData = useProcessedData();
    const { status, result, error, runAnalysis, loadCached, exportReport } =
        useAnalysis();
    const [exportMessage, setExportMessage] = useState<string | null>(null);
    const [exportStatus, setExportStatus] = useState<
        "success" | "error" | null
    >(null);

    const availableColumns = useMemo(() => {
        if (processedData.fileInfo) {
            return [
                ...processedData.fileInfo.columns,
                ...(fileInfo?.columns ?? []),
            ];
        }
        return fileInfo?.columns ?? [];
    }, [processedData.fileInfo, fileInfo?.columns]);

    const { isLoaded, uiState, setUIState } =
        useAnalysisUIState(availableColumns);

    useEffect(() => {
        if (!isLoaded || !isFileLoaded) {
            return;
        }

        loadCached(uiState.use_processed_data).catch(() => {
            // handled in hook
        });
    }, [isLoaded, isFileLoaded, uiState.use_processed_data, loadCached]);

    const activeFileInfo = uiState.use_processed_data
        ? processedData.fileInfo
        : fileInfo;
    const datasetLabel = uiState.use_processed_data ? "Processed" : "Original";

    const handleRun = () => {
        setExportMessage(null);
        setExportStatus(null);
        runAnalysis(uiState.use_processed_data).catch(() => {
            // handled in hook
        });
    };

    const handleExport = () => {
        exportReport(uiState.use_processed_data)
            .then((response) => {
                const fileName = response.report_path.split(/[\\/]/).pop();
                setExportStatus("success");
                setExportMessage(
                    fileName
                        ? `Report saved to ${fileName}`
                        : "Report exported",
                );
            })
            .catch((err) => {
                const message =
                    err instanceof Error ? err.message : String(err);
                setExportStatus("error");
                setExportMessage(message);
            });
    };

    const handleToggleDataset = (useProcessed: boolean) => {
        if (useProcessed && !processedData.hasProcessedData) {
            return;
        }
        setUIState({
            ...uiState,
            use_processed_data: useProcessed,
        });
    };

    useEffect(() => {
        if (!uiState.selected_column) {
            return;
        }
        const columns = uiState.use_processed_data
            ? processedData.fileInfo?.columns
            : fileInfo?.columns;
        if (!columns) {
            return;
        }
        const exists = columns.some(
            (column) => column.name === uiState.selected_column,
        );
        if (!exists) {
            setUIState({
                ...uiState,
                selected_column: null,
            });
        }
    }, [
        uiState.selected_column,
        uiState.use_processed_data,
        processedData.fileInfo?.columns,
        fileInfo?.columns,
        setUIState,
    ]);

    if (!isLoaded) {
        return (
            <AppShell sidebar={<div className="p-4" />}>
                <div className="text-muted-foreground flex h-full items-center justify-center">
                    Loading...
                </div>
            </AppShell>
        );
    }

    return (
        <AppShell
            sidebar={
                <AnalysisSidebar
                    datasetName={activeFileInfo?.name ?? null}
                    rows={activeFileInfo?.row_count ?? 0}
                    columns={activeFileInfo?.column_count ?? 0}
                    memoryBytes={result?.summary.memory_bytes ?? null}
                    datasetLabel={datasetLabel}
                    generatedAt={result?.generated_at ?? null}
                    durationMs={result?.duration_ms ?? null}
                    useProcessedData={uiState.use_processed_data}
                    hasProcessedData={processedData.hasProcessedData}
                    status={status}
                    hasResult={Boolean(result)}
                    exportMessage={exportMessage}
                    exportStatus={exportStatus}
                    onToggleDataset={handleToggleDataset}
                    onRun={handleRun}
                    onExport={handleExport}
                />
            }
        >
            {!isFileLoaded && <NoFileLoadedState />}
            {isFileLoaded && !result && (
                <NoAnalysisState
                    onRun={handleRun}
                    isRunning={status === "running"}
                    error={error}
                />
            )}
            {isFileLoaded && result && (
                <div className="flex flex-1 flex-col overflow-hidden">
                    <div className="relative flex flex-1 flex-col overflow-hidden p-3">
                        <div
                            className={cn(
                                "flex flex-1 flex-col overflow-hidden",
                                status === "running" && "opacity-60",
                            )}
                        >
                            {error && (
                                <div className="text-destructive mb-2 text-sm">
                                    {error}
                                </div>
                            )}
                            <AnalysisWorkspace
                                analysis={result}
                                activeTab={uiState.active_tab}
                                dataset={result.dataset}
                                onTabChange={(tab) =>
                                    setUIState({
                                        ...uiState,
                                        active_tab: tab,
                                    })
                                }
                                selectedColumn={uiState.selected_column}
                                onSelectColumn={(column) =>
                                    setUIState({
                                        ...uiState,
                                        selected_column: column,
                                    })
                                }
                            />
                        </div>
                        {status === "running" && (
                            <div className="bg-background/80 absolute inset-0 flex items-center justify-center">
                                <div className="text-muted-foreground text-sm">
                                    Running analysis...
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            )}
        </AppShell>
    );
};

export default AnalysisPage;
