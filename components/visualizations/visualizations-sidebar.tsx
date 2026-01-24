"use client";

import { Button } from "@/components/ui/button";
import Toggle from "@/components/ui/toggle";
import { formatBytes, formatNumber } from "@/lib/utils";

import type { VisualizationsStatus } from "@/lib/hooks/use-visualizations";

interface VisualizationsSidebarProps {
    datasetName: string | null;
    rows: number;
    columns: number;
    memoryBytes: number | null;
    datasetLabel: string;
    generatedAt: string | null;
    useProcessedData: boolean;
    hasProcessedData: boolean;
    status: VisualizationsStatus;
    hasResult: boolean;
    onToggleDataset: (useProcessed: boolean) => void;
    onRun: () => void;
}

const VisualizationsSidebar = ({
    datasetName,
    rows,
    columns,
    memoryBytes,
    datasetLabel,
    generatedAt,
    useProcessedData,
    hasProcessedData,
    status,
    hasResult,
    onToggleDataset,
    onRun,
}: VisualizationsSidebarProps) => {
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
                        <div>
                            <dt className="text-muted-foreground">Dataset</dt>
                            <dd className="font-medium">{datasetLabel}</dd>
                        </div>
                    </dl>
                ) : (
                    <p className="text-muted-foreground text-sm">
                        Load a dataset to begin visualizations.
                    </p>
                )}
            </section>

            <section>
                <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                    Visualization Scope
                </h2>
                <Toggle
                    pressed={useProcessedData}
                    onPressedChange={onToggleDataset}
                    label="Use processed data"
                    description={
                        hasProcessedData
                            ? "Build charts from the cleaned dataset"
                            : "Run preprocessing to enable"
                    }
                    disabled={!hasProcessedData}
                />
            </section>

            <section className="flex flex-col gap-2">
                <Button onClick={onRun} disabled={isRunning || !datasetName}>
                    {isRunning
                        ? "Generating visualizations..."
                        : "Generate Visualizations"}
                </Button>
            </section>

            <section className="mt-auto">
                <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                    Status
                </h2>
                <div className="space-y-1 text-sm font-medium">
                    <div>
                        {status === "running" && "Running"}
                        {status === "completed" && "Complete"}
                        {status === "error" && "Error"}
                        {status === "idle" && (hasResult ? "Cached" : "Idle")}
                    </div>
                    {generatedAt && (
                        <div className="text-muted-foreground text-xs font-normal">
                            {new Date(generatedAt).toLocaleString()}
                        </div>
                    )}
                </div>
            </section>
        </div>
    );
};

export default VisualizationsSidebar;
