"use client";

import { useMemo } from "react";

import type { MLTrainingStatus, ModelInfo } from "@/types";
import { Button } from "@/components/ui/button";
import { Toggle } from "@/components/ui/toggle";
import { cn, formatNumber } from "@/lib/utils";

interface MLOperationsSidebarProps {
    datasetName: string | null;
    totalRows: number;
    useProcessedData: boolean;
    hasProcessedData: boolean;
    onToggleProcessedData: (value: boolean) => void;
    kernelStatus: "uninitialized" | "initializing" | "ready" | "error";
    onInitializeKernel: () => Promise<void>;
    trainingStatus: MLTrainingStatus;
    onStartTraining: () => Promise<void>;
    onCancelTraining: () => Promise<void>;
    onSaveModel: () => Promise<string>;
    onLoadModel: () => Promise<ModelInfo>;
    canStartTraining: boolean;
}

export function MLOperationsSidebar({
    datasetName,
    totalRows,
    useProcessedData,
    hasProcessedData,
    onToggleProcessedData,
    kernelStatus,
    onInitializeKernel,
    trainingStatus,
    onStartTraining,
    onCancelTraining,
    onSaveModel,
    onLoadModel,
    canStartTraining,
}: MLOperationsSidebarProps) {
    const kernelLabel = useMemo(() => {
        if (kernelStatus === "initializing") return "Starting";
        if (kernelStatus === "ready") return "Ready";
        if (kernelStatus === "error") return "Error";
        return "Not ready";
    }, [kernelStatus]);

    const canInitialize =
        kernelStatus !== "initializing" && kernelStatus !== "ready";

    const isTraining = trainingStatus === "training";

    return (
        <div className="flex h-full flex-col">
            <div className="border-b p-4">
                <Button
                    size="sm"
                    onClick={onStartTraining}
                    disabled={
                        isTraining ||
                        kernelStatus !== "ready" ||
                        !canStartTraining
                    }
                    className="w-full"
                >
                    {isTraining ? "Training..." : "Start training"}
                </Button>
            </div>

            <div className="flex-1 space-y-6 overflow-y-auto p-4">
                <section>
                    <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                        Dataset
                    </h2>
                    <dl className="space-y-2 text-sm">
                        <div className="flex items-center justify-between gap-2">
                            <dt className="text-muted-foreground">Name</dt>
                            <dd className="truncate text-right font-medium">
                                {datasetName ?? "No dataset"}
                            </dd>
                        </div>
                        <div className="flex items-center justify-between">
                            <dt className="text-muted-foreground">Rows</dt>
                            <dd className="font-medium">
                                {formatNumber(totalRows)}
                            </dd>
                        </div>
                        <div className="flex items-center justify-between">
                            <dt className="text-muted-foreground">Source</dt>
                            <dd className="font-medium">
                                {useProcessedData ? "Processed" : "Original"}
                            </dd>
                        </div>
                    </dl>
                    <div className="mt-3">
                        <Toggle
                            pressed={useProcessedData}
                            onPressedChange={onToggleProcessedData}
                            label="Processed data"
                            disabled={!hasProcessedData}
                        />
                    </div>
                </section>

                <section>
                    <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                        Kernel
                    </h2>
                    <div className="flex items-center justify-between text-sm">
                        <span className="text-muted-foreground">Status</span>
                        <span
                            className={cn(
                                "font-medium",
                                kernelStatus === "ready"
                                    ? "text-foreground"
                                    : "text-muted-foreground",
                            )}
                        >
                            {kernelLabel}
                        </span>
                    </div>
                    <div className="mt-3">
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={onInitializeKernel}
                            disabled={!canInitialize}
                            className="w-full"
                        >
                            {kernelStatus === "initializing"
                                ? "Initializing..."
                                : "Initialize"}
                        </Button>
                    </div>
                </section>

                <section>
                    <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                        Training
                    </h2>
                    <div className="space-y-2">
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={onCancelTraining}
                            disabled={!isTraining}
                            className="w-full"
                        >
                            Cancel training
                        </Button>
                    </div>
                </section>

                <section>
                    <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                        Model
                    </h2>
                    <div className="space-y-2">
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={onSaveModel}
                            disabled={isTraining}
                            className="w-full"
                        >
                            Save model
                        </Button>
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={onLoadModel}
                            disabled={isTraining}
                            className="w-full"
                        >
                            Load model
                        </Button>
                    </div>
                </section>
            </div>
        </div>
    );
}
