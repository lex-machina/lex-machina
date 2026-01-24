"use client";

import Link from "next/link";
import { useEffect, useMemo } from "react";
import { BarChart3, PieChart, Sparkles } from "lucide-react";

import AppShell from "@/components/layout/app-shell";
import VisualizationsGrid from "@/components/visualizations/visualizations-grid";
import VisualizationsSidebar from "@/components/visualizations/visualizations-sidebar";
import { Button } from "@/components/ui/button";
import { useFileState } from "@/lib/hooks/use-file-state";
import { useProcessedData } from "@/lib/hooks/use-processed-data";
import { useVisualizations } from "@/lib/hooks/use-visualizations";
import { useVisualizationsUIState } from "@/lib/hooks/use-visualizations-ui-state";
import type { VisualizationChartKind } from "@/types";

function VisualizationHeader({
    datasetLabel,
    datasetName,
    chartCount,
    generatedAt,
}: {
    datasetLabel: string;
    datasetName: string | null;
    chartCount: number;
    generatedAt: string | null;
}) {
    return (
        <div className="text-muted-foreground border-b px-4 py-2 text-xs">
            <div className="flex flex-wrap items-center gap-3">
                <span className="text-foreground font-medium">
                    {datasetLabel} dataset
                </span>
                {datasetName && <span className="truncate">{datasetName}</span>}
                <span>{chartCount} charts</span>
                {generatedAt && (
                    <span>
                        Last generated {new Date(generatedAt).toLocaleString()}
                    </span>
                )}
            </div>
        </div>
    );
}

function NoFileLoadedState() {
    return (
        <div className="flex flex-1 items-center justify-center p-8">
            <div className="max-w-md text-center">
                <div className="bg-muted mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full">
                    <BarChart3 className="text-muted-foreground h-8 w-8" />
                </div>
                <h2 className="mb-2 text-xl font-semibold">Visualizations</h2>
                <p className="text-muted-foreground mb-6">
                    Load a dataset to auto-generate a dashboard of charts for
                    every column.
                </p>
                <ul className="text-muted-foreground mb-8 space-y-2 text-left text-sm">
                    <li className="flex items-center gap-3">
                        <Sparkles className="h-4 w-4 shrink-0" />
                        <span>Smart chart selection by data type</span>
                    </li>
                    <li className="flex items-center gap-3">
                        <PieChart className="h-4 w-4 shrink-0" />
                        <span>Bars, pies, and distributions per column</span>
                    </li>
                    <li className="flex items-center gap-3">
                        <BarChart3 className="h-4 w-4 shrink-0" />
                        <span>
                            Switch chart styles with per-column controls
                        </span>
                    </li>
                </ul>
                <Button asChild size="lg">
                    <Link href="/data">Go to Data</Link>
                </Button>
            </div>
        </div>
    );
}

function NoVisualizationsState({
    isRunning,
    error,
    onGenerate,
}: {
    isRunning: boolean;
    error: string | null;
    onGenerate: () => void;
}) {
    return (
        <div className="flex flex-1 items-center justify-center p-8">
            <div className="max-w-md text-center">
                <div className="bg-muted mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full">
                    <Sparkles className="text-muted-foreground h-8 w-8" />
                </div>
                <h2 className="mb-2 text-xl font-semibold">
                    No visualizations yet
                </h2>
                <p className="text-muted-foreground mb-6">
                    {isRunning
                        ? "Generating charts from your dataset..."
                        : "Generate a dashboard with charts for every column."}
                </p>
                <ul className="text-muted-foreground mb-8 space-y-2 text-left text-sm">
                    <li className="flex items-center gap-3">
                        <BarChart3 className="h-4 w-4 shrink-0" />
                        <span>Automatic chart type selection</span>
                    </li>
                    <li className="flex items-center gap-3">
                        <PieChart className="h-4 w-4 shrink-0" />
                        <span>Built-in alternatives per column</span>
                    </li>
                </ul>
                <Button onClick={onGenerate} size="lg" disabled={isRunning}>
                    {isRunning ? "Generating..." : "Generate Visualizations"}
                </Button>
                {error && (
                    <p className="text-destructive mt-4 text-sm">{error}</p>
                )}
            </div>
        </div>
    );
}

export default function VisualizationsPage() {
    const { fileInfo, isFileLoaded } = useFileState();
    const processedData = useProcessedData();
    const { status, result, error, runVisualizations, loadCached } =
        useVisualizations();
    const { isLoaded, uiState, setUIState } = useVisualizationsUIState();

    useEffect(() => {
        if (!isLoaded) {
            return;
        }
        loadCached(uiState.use_processed_data).catch(() => {
            // handled in hook
        });
    }, [isLoaded, uiState.use_processed_data, loadCached]);

    const handleRun = () => {
        runVisualizations(uiState.use_processed_data).catch(() => {
            // handled in hook
        });
    };

    const handleToggleDataset = (useProcessed: boolean) => {
        if (useProcessed && !processedData.hasProcessedData) {
            return;
        }
        setUIState({
            use_processed_data: useProcessed,
            chart_overrides: uiState.chart_overrides,
        });
    };

    const activeFileInfo = uiState.use_processed_data
        ? (processedData.fileInfo ?? fileInfo)
        : fileInfo;
    const datasetLabel = uiState.use_processed_data ? "Processed" : "Original";
    const chartKinds = useMemo(() => {
        if (!result) {
            return {} as Record<string, VisualizationChartKind>;
        }
        const next: Record<string, VisualizationChartKind> = {};
        for (const chart of result.charts) {
            const override = uiState.chart_overrides[chart.column];
            if (override && chart.available_kinds.includes(override)) {
                next[chart.column] = override;
            } else {
                next[chart.column] = chart.kind;
            }
        }
        return next;
    }, [result, uiState.chart_overrides]);

    const handleChartKindChange = (
        column: string,
        kind: VisualizationChartKind,
    ) => {
        setUIState({
            ...uiState,
            chart_overrides: {
                ...uiState.chart_overrides,
                [column]: kind,
            },
        });
    };

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
                <VisualizationsSidebar
                    datasetName={activeFileInfo?.name ?? null}
                    rows={activeFileInfo?.row_count ?? 0}
                    columns={activeFileInfo?.column_count ?? 0}
                    memoryBytes={activeFileInfo?.size_bytes ?? null}
                    datasetLabel={datasetLabel}
                    generatedAt={result?.generated_at ?? null}
                    useProcessedData={uiState.use_processed_data}
                    hasProcessedData={processedData.hasProcessedData}
                    status={status}
                    hasResult={Boolean(result)}
                    onToggleDataset={handleToggleDataset}
                    onRun={handleRun}
                />
            }
        >
            {!isFileLoaded && <NoFileLoadedState />}
            {isFileLoaded && !result && (
                <NoVisualizationsState
                    isRunning={status === "running"}
                    error={error}
                    onGenerate={handleRun}
                />
            )}
            {isFileLoaded && result && (
                <div className="flex h-full min-h-0 w-full flex-col overflow-hidden">
                    <VisualizationHeader
                        datasetLabel={datasetLabel}
                        datasetName={activeFileInfo?.name ?? null}
                        chartCount={result.charts.length}
                        generatedAt={result.generated_at}
                    />
                    <div className="flex-1 overflow-y-auto p-3">
                        <VisualizationsGrid
                            charts={result.charts}
                            chartKinds={chartKinds}
                            onChartKindChange={handleChartKindChange}
                        />
                    </div>
                </div>
            )}
        </AppShell>
    );
}
