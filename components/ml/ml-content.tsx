"use client";

import { useEffect, useState, type ReactNode } from "react";
import {
    Brain,
    AlertTriangle,
    Database,
    BarChart3,
    Layers,
    Sparkles,
    Image,
} from "lucide-react";

import type {
    ColumnInfo,
    MLProgressUpdate,
    TrainingHistoryEntry,
    TrainingResultResponse,
    ModelComparison,
    Metrics,
} from "@/types";
import { useFileState } from "@/lib/hooks/use-file-state";
import { useML } from "@/lib/hooks/use-ml";
import { formatNumber, formatPercent } from "@/lib/utils";
import { Card, CardHeader, CardContent } from "@/components/ui/card";
import { ProgressBar } from "@/components/ui/progress-bar";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { PredictionPanel } from "@/components/ml/prediction-panel";
import { TrainingHistory } from "@/components/ml/training-history";

interface MLContentProps {
    kernelStatus: "uninitialized" | "initializing" | "ready" | "error";
    progress: MLProgressUpdate | null;
    result: TrainingResultResponse | null;
    error: string | null;
    activeTab: string;
    onTabChange: (tab: string) => void;
    availableColumns: ColumnInfo[];
    onSelectHistory?: (entry: TrainingHistoryEntry) => void;
}

function EmptyPanel({
    icon,
    title,
    message,
}: {
    icon: ReactNode;
    title: string;
    message: string;
}) {
    return (
        <div className="flex h-full flex-col items-center justify-center gap-3 text-center">
            <div className="bg-muted flex h-12 w-12 items-center justify-center rounded-full">
                {icon}
            </div>
            <div>
                <p className="text-sm font-medium">{title}</p>
                <p className="text-muted-foreground text-xs">{message}</p>
            </div>
        </div>
    );
}

function metricLabel(key: keyof Metrics): string {
    const labels: Record<keyof Metrics, string> = {
        cv_score: "CV score",
        test_score: "Test score",
        train_score: "Train score",
        accuracy: "Accuracy",
        precision: "Precision",
        recall: "Recall",
        f1_score: "F1 score",
        roc_auc: "ROC AUC",
        mse: "MSE",
        rmse: "RMSE",
        mae: "MAE",
        r2: "R2",
    };
    return labels[key];
}

function formatMetricValue(key: keyof Metrics, value: number): string {
    if (key === "mse" || key === "rmse" || key === "mae") {
        return value.toFixed(3);
    }
    if (key === "r2") {
        return value.toFixed(3);
    }
    return formatPercent(value);
}

function MetricsCard({ metrics }: { metrics: Metrics }) {
    const entries = (
        Object.entries(metrics) as [keyof Metrics, number | null | undefined][]
    ).filter(([, value]) => value !== null && value !== undefined);

    if (entries.length === 0) {
        return (
            <div className="text-muted-foreground text-xs">
                No metrics available yet.
            </div>
        );
    }

    return (
        <div className="grid gap-2 md:grid-cols-2">
            {entries.map(([key, value]) => (
                <div
                    key={key}
                    className="bg-muted/30 flex items-center justify-between rounded-md border px-3 py-2"
                >
                    <span className="text-muted-foreground text-xs">
                        {metricLabel(key)}
                    </span>
                    <span className="text-sm font-semibold">
                        {value !== undefined && value !== null
                            ? formatMetricValue(key, value)
                            : "-"}
                    </span>
                </div>
            ))}
        </div>
    );
}

function FeatureImportanceChart({ items }: { items: [string, number][] }) {
    if (items.length === 0) {
        return (
            <div className="text-muted-foreground text-xs">
                Feature importance not available.
            </div>
        );
    }

    const maxValue = Math.max(...items.map(([, value]) => value));

    return (
        <div className="flex flex-col gap-2">
            {items.map(([name, value]) => (
                <div key={name} className="flex items-center gap-2">
                    <span
                        className="text-muted-foreground w-24 truncate text-xs"
                        title={name}
                    >
                        {name}
                    </span>
                    <div className="bg-muted flex h-2 flex-1 overflow-hidden rounded-full">
                        <div
                            className="bg-foreground h-full"
                            style={{
                                width: `${Math.max(
                                    6,
                                    (value / maxValue) * 100,
                                )}%`,
                            }}
                        />
                    </div>
                    <span className="text-muted-foreground text-xs tabular-nums">
                        {formatPercent(value)}
                    </span>
                </div>
            ))}
        </div>
    );
}

function ModelComparisonTable({ models }: { models: ModelComparison[] }) {
    if (models.length === 0) {
        return (
            <div className="text-muted-foreground text-xs">
                No model comparison data available.
            </div>
        );
    }

    return (
        <div className="border-border overflow-hidden rounded-md border">
            <div className="bg-muted/30 grid grid-cols-[1.5fr_1fr_1fr_1fr_1fr] gap-2 px-3 py-2 text-xs font-semibold tracking-wide uppercase">
                <span>Model</span>
                <span>Test</span>
                <span>Train</span>
                <span>CV</span>
                <span>Risk</span>
            </div>
            <div className="divide-border divide-y text-xs">
                {models.map((model) => (
                    <div
                        key={model.name}
                        className="grid grid-cols-[1.5fr_1fr_1fr_1fr_1fr] gap-2 px-3 py-2"
                    >
                        <span
                            className="truncate font-medium"
                            title={model.name}
                        >
                            {model.name}
                        </span>
                        <span>{formatPercent(model.test_score)}</span>
                        <span>{formatPercent(model.train_score)}</span>
                        <span>{formatPercent(model.cv_score)}</span>
                        <span className="text-muted-foreground">
                            {model.overfitting_risk}
                        </span>
                    </div>
                ))}
            </div>
        </div>
    );
}

/* eslint-disable @next/next/no-img-element, jsx-a11y/alt-text */
function ShapViewer({ shapPlots }: { shapPlots: Record<string, string> }) {
    const entries = Object.entries(shapPlots ?? {});

    if (entries.length === 0) {
        return (
            <div className="text-muted-foreground text-xs">
                No SHAP plots available.
            </div>
        );
    }

    return (
        <div className="grid gap-3 md:grid-cols-2">
            {entries.map(([name, data]) => (
                <div
                    key={name}
                    className="border-border bg-muted/20 flex flex-col gap-2 rounded-md border p-2"
                >
                    <span className="text-muted-foreground text-xs uppercase">
                        {name}
                    </span>
                    <div className="bg-background flex items-center justify-center rounded-md border">
                        <img
                            src={`data:image/png;base64,${data}`}
                            alt=""
                            loading="lazy"
                            className="max-h-48 w-full object-contain"
                        />
                    </div>
                </div>
            ))}
        </div>
    );
}

export function MLContent({
    kernelStatus,
    progress,
    result,
    error,
    activeTab,
    onTabChange,
    availableColumns,
    onSelectHistory,
}: MLContentProps) {
    const { fileInfo } = useFileState();
    const {
        trainingStatus,
        predictSingle,
        predictBatch,
        getHistory,
        clearHistory,
    } = useML();

    const [history, setHistory] = useState<TrainingHistoryEntry[]>([]);
    const isTraining = trainingStatus === "training";

    useEffect(() => {
        getHistory().then((entries) => setHistory(entries));
    }, [getHistory]);

    const handleHistoryRefresh = async () => {
        const entries = await getHistory();
        setHistory(entries);
        return entries;
    };

    const handleHistoryClear = async () => {
        await clearHistory();
        setHistory([]);
    };

    const showNoData = !fileInfo;
    const showKernelStart = kernelStatus !== "ready";
    const showTrainingArea = !showNoData && !showKernelStart;

    const metrics = result?.metrics;
    const testScore = metrics?.test_score ?? metrics?.accuracy ?? metrics?.r2;
    const progressPercent = Math.round((progress?.progress ?? 0) * 100);

    return (
        <div className="flex min-h-0 flex-1 flex-col gap-4 p-4">
            <div className="grid min-h-0 flex-1 grid-cols-[1.4fr_1fr_1fr] gap-4">
                <Card className="min-h-0">
                    <CardHeader title="Training" />
                    <CardContent className="flex min-h-0 flex-1 flex-col gap-4">
                        {showNoData && (
                            <EmptyPanel
                                icon={
                                    <Database className="text-muted-foreground h-5 w-5" />
                                }
                                title="No dataset loaded"
                                message="Load a dataset to start training."
                            />
                        )}
                        {!showNoData && showKernelStart && (
                            <EmptyPanel
                                icon={
                                    <Brain className="text-muted-foreground h-5 w-5" />
                                }
                                title="Kernel not ready"
                                message="Initialize the ML kernel to enable training."
                            />
                        )}
                        {showTrainingArea && (
                            <div className="flex flex-col gap-3">
                                <div className="flex items-center justify-between">
                                    <div>
                                        <p className="text-sm font-medium">
                                            Training status
                                        </p>
                                        <p className="text-muted-foreground text-xs">
                                            {isTraining
                                                ? (progress?.message ??
                                                  "Training in progress")
                                                : result
                                                  ? "Training completed"
                                                  : "Ready"}
                                        </p>
                                    </div>
                                    <div className="text-muted-foreground text-xs">
                                        {isTraining
                                            ? `${progressPercent}%`
                                            : ""}
                                    </div>
                                </div>
                                <ProgressBar value={progressPercent} />
                                {result && (
                                    <div className="grid gap-3 md:grid-cols-2">
                                        <div className="bg-muted/30 rounded-md border p-3">
                                            <div className="text-muted-foreground text-xs">
                                                Best model
                                            </div>
                                            <div className="text-sm font-semibold">
                                                {result.best_model_name}
                                            </div>
                                        </div>
                                        <div className="bg-muted/30 rounded-md border p-3">
                                            <div className="text-muted-foreground text-xs">
                                                Test score
                                            </div>
                                            <div className="text-sm font-semibold">
                                                {testScore !== undefined
                                                    ? formatPercent(testScore)
                                                    : "-"}
                                            </div>
                                        </div>
                                    </div>
                                )}
                                {error && (
                                    <div className="border-muted bg-muted/30 flex items-start gap-2 rounded-md border p-3">
                                        <AlertTriangle className="text-muted-foreground h-4 w-4" />
                                        <p className="text-muted-foreground text-xs">
                                            {error}
                                        </p>
                                    </div>
                                )}
                            </div>
                        )}
                    </CardContent>
                </Card>

                <Card className="min-h-0">
                    <CardHeader
                        title="Metrics"
                        actions={
                            result?.metrics ? (
                                <BarChart3 className="text-muted-foreground h-4 w-4" />
                            ) : undefined
                        }
                    />
                    <CardContent className="flex min-h-0 flex-1 flex-col gap-3">
                        {result?.metrics ? (
                            <MetricsCard metrics={result.metrics} />
                        ) : (
                            <EmptyPanel
                                icon={
                                    <Brain className="text-muted-foreground h-5 w-5" />
                                }
                                title="No metrics yet"
                                message="Train a model to see performance metrics."
                            />
                        )}
                    </CardContent>
                </Card>

                <Card className="min-h-0">
                    <CardHeader
                        title="Model summary"
                        actions={
                            result ? (
                                <Layers className="text-muted-foreground h-4 w-4" />
                            ) : undefined
                        }
                    />
                    <CardContent className="flex min-h-0 flex-1 flex-col gap-4">
                        {result ? (
                            <div className="flex flex-col gap-3">
                                <div className="bg-muted/30 rounded-md border p-3">
                                    <div className="text-muted-foreground text-xs">
                                        Training time
                                    </div>
                                    <div className="text-sm font-semibold">
                                        {formatNumber(
                                            result.training_time_seconds,
                                        )}
                                        s
                                    </div>
                                </div>
                                <div className="bg-muted/30 rounded-md border p-3">
                                    <div className="text-muted-foreground text-xs">
                                        Models compared
                                    </div>
                                    <div className="text-sm font-semibold">
                                        {result.model_comparison.length}
                                    </div>
                                </div>
                                <div className="bg-muted/30 rounded-md border p-3">
                                    <div className="text-muted-foreground text-xs">
                                        Feature importance
                                    </div>
                                    <div className="text-sm font-semibold">
                                        {result.feature_importance.length}{" "}
                                        features
                                    </div>
                                </div>
                            </div>
                        ) : (
                            <EmptyPanel
                                icon={
                                    <Brain className="text-muted-foreground h-5 w-5" />
                                }
                                title="No results yet"
                                message="Train a model to see insights."
                            />
                        )}
                    </CardContent>
                </Card>
            </div>

            <Card className="min-h-0">
                <Tabs value={activeTab} onValueChange={onTabChange}>
                    <div className="flex items-center justify-between border-b px-3 py-2">
                        <TabsList>
                            <TabsTrigger value="results">Results</TabsTrigger>
                            <TabsTrigger value="comparison">
                                Comparison
                            </TabsTrigger>
                            <TabsTrigger value="shap">SHAP</TabsTrigger>
                            <TabsTrigger value="prediction">
                                Prediction
                            </TabsTrigger>
                            <TabsTrigger value="history">History</TabsTrigger>
                        </TabsList>
                        {result?.warnings?.length ? (
                            <div className="text-muted-foreground flex items-center gap-2 text-xs">
                                <Sparkles className="h-3.5 w-3.5" />
                                {result.warnings.length} warning
                                {result.warnings.length > 1 ? "s" : ""}
                            </div>
                        ) : null}
                    </div>
                    <CardContent className="min-h-[260px]">
                        <TabsContent value="results" className="h-full">
                            {result ? (
                                <div className="grid gap-4 md:grid-cols-[1.2fr_1fr]">
                                    <div className="border-border rounded-md border p-3">
                                        <div className="text-muted-foreground mb-2 flex items-center gap-2 text-xs uppercase">
                                            <BarChart3 className="h-3.5 w-3.5" />
                                            Metrics
                                        </div>
                                        <MetricsCard metrics={result.metrics} />
                                    </div>
                                    <div className="border-border rounded-md border p-3">
                                        <div className="text-muted-foreground mb-2 flex items-center gap-2 text-xs uppercase">
                                            <Sparkles className="h-3.5 w-3.5" />
                                            Feature importance
                                        </div>
                                        <FeatureImportanceChart
                                            items={result.feature_importance.slice(
                                                0,
                                                8,
                                            )}
                                        />
                                    </div>
                                </div>
                            ) : (
                                <EmptyPanel
                                    icon={
                                        <Brain className="text-muted-foreground h-5 w-5" />
                                    }
                                    title="No training results"
                                    message="Complete a training run to see metrics."
                                />
                            )}
                        </TabsContent>
                        <TabsContent value="comparison" className="h-full">
                            {result ? (
                                <div className="flex flex-col gap-3">
                                    <div className="text-muted-foreground flex items-center gap-2 text-xs uppercase">
                                        <Layers className="h-3.5 w-3.5" />
                                        Model comparison
                                    </div>
                                    <ModelComparisonTable
                                        models={result.model_comparison}
                                    />
                                </div>
                            ) : (
                                <EmptyPanel
                                    icon={
                                        <Layers className="text-muted-foreground h-5 w-5" />
                                    }
                                    title="No comparison data"
                                    message="Train models to compare performance."
                                />
                            )}
                        </TabsContent>
                        <TabsContent value="shap" className="h-full">
                            {result ? (
                                <div className="flex flex-col gap-3">
                                    <div className="text-muted-foreground flex items-center gap-2 text-xs uppercase">
                                        <Image className="h-3.5 w-3.5" />
                                        SHAP explainability
                                    </div>
                                    <ShapViewer shapPlots={result.shap_plots} />
                                </div>
                            ) : (
                                <EmptyPanel
                                    icon={
                                        <Image className="text-muted-foreground h-5 w-5" />
                                    }
                                    title="No SHAP plots"
                                    message="Enable explainability to generate SHAP plots."
                                />
                            )}
                        </TabsContent>
                        <TabsContent value="prediction" className="h-full">
                            <PredictionPanel
                                columns={availableColumns}
                                onPredictSingle={predictSingle}
                                onPredictBatch={predictBatch}
                                disabled={
                                    isTraining || availableColumns.length === 0
                                }
                            />
                        </TabsContent>
                        <TabsContent value="history" className="h-full">
                            <TrainingHistory
                                entries={history}
                                onRefresh={handleHistoryRefresh}
                                onClear={handleHistoryClear}
                                onSelect={(entry) => {
                                    onSelectHistory?.(entry);
                                }}
                                disabled={isTraining}
                            />
                        </TabsContent>
                    </CardContent>
                </Tabs>
            </Card>
        </div>
    );
}
