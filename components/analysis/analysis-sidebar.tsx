"use client";

import { Button } from "@/components/ui/button";
import Toggle from "@/components/ui/toggle";
import { formatBytes, formatNumber } from "@/lib/utils";

import type { AnalysisStatus } from "@/lib/hooks/use-analysis";

interface AnalysisSidebarProps {
    datasetName: string | null;
    rows: number;
    columns: number;
    memoryBytes: number | null;
    useProcessedData: boolean;
    hasProcessedData: boolean;
    status: AnalysisStatus;
    hasResult: boolean;
    onToggleDataset: (useProcessed: boolean) => void;
    onRun: () => void;
    onExport: () => void;
}

const AnalysisSidebar = ({
    datasetName,
    rows,
    columns,
    memoryBytes,
    useProcessedData,
    hasProcessedData,
    status,
    hasResult,
    onToggleDataset,
    onRun,
    onExport,
}: AnalysisSidebarProps) => {
    const isRunning = status === "running";

    return (
        <div className="flex h-full flex-col gap-6 p-4">
            <section>
                <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                    Dataset
                </h2>
                {datasetName ? (
                    <dl className="space-y-2 text-sm">
                        <div>
                            <dt className="text-muted-foreground">Name</dt>
                            <dd className="truncate font-medium">
                                {datasetName}
                            </dd>
                        </div>
                        <div>
                            <dt className="text-muted-foreground">Rows</dt>
                            <dd className="font-medium">
                                {formatNumber(rows)}
                            </dd>
                        </div>
                        <div>
                            <dt className="text-muted-foreground">Columns</dt>
                            <dd className="font-medium">
                                {formatNumber(columns)}
                            </dd>
                        </div>
                        <div>
                            <dt className="text-muted-foreground">Memory</dt>
                            <dd className="font-medium">
                                {memoryBytes ? formatBytes(memoryBytes) : "—"}
                            </dd>
                        </div>
                    </dl>
                ) : (
                    <p className="text-muted-foreground text-sm">
                        Load a dataset to begin analysis.
                    </p>
                )}
            </section>

            <section>
                <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                    Analysis Scope
                </h2>
                <Toggle
                    pressed={useProcessedData}
                    onPressedChange={onToggleDataset}
                    label="Use processed data"
                    description={
                        hasProcessedData
                            ? "Analyze the cleaned dataset"
                            : "Run preprocessing to enable"
                    }
                    disabled={!hasProcessedData}
                />
            </section>

            <section className="flex flex-col gap-2">
                <Button onClick={onRun} disabled={isRunning || !datasetName}>
                    {isRunning ? "Running analysis..." : "Run Analysis"}
                </Button>
                <Button
                    variant="outline"
                    onClick={onExport}
                    disabled={!hasResult}
                >
                    Export Report
                </Button>
            </section>

            <section className="mt-auto">
                <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                    Status
                </h2>
                <div className="text-sm font-medium">
                    {status === "running" && "Running"}
                    {status === "completed" && "Complete"}
                    {status === "error" && "Error"}
                    {status === "idle" && (hasResult ? "Cached" : "Idle")}
                </div>
            </section>
        </div>
    );
};

export default AnalysisSidebar;
