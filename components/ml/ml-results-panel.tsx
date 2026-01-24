"use client";

import { type ReactNode, useState } from "react";

import {
    Brain,
    AlertTriangle,
    BarChart3,
    Layers,
    Sparkles,
    Image,
} from "lucide-react";

import type {
    MLProgressUpdate,
    TrainingResultResponse,
    ModelComparison,
    Metrics,
    TrainingHistoryEntry,
    MLTrainingStatus,
} from "@/types";
import { formatNumber, formatPercent } from "@/lib/utils";
import { ProgressBar } from "@/components/ui/progress-bar";
import { TrainingHistory } from "@/components/ml/training-history";
import { cn } from "@/lib/utils";

interface MLResultsPanelProps {
    progress: MLProgressUpdate | null;
    result: TrainingResultResponse | null;
    error: string | null;
    activeTab: string;
    onTabChange: (tab: string) => void;
    trainingStatus: MLTrainingStatus;
    onSelectHistory: (entry: TrainingHistoryEntry) => void;
    onRefreshHistory: () => Promise<TrainingHistoryEntry[]>;
    onClearHistory: () => Promise<void>;
    historyEntries: TrainingHistoryEntry[];
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
    if (key === "mse" || key === "rmse" || key === "mae" || key === "r2") {
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
        <div className="flex flex-col gap-3">
            {items.map(([name, value]) => (
                <div key={name} className="flex flex-col gap-2">
                    <span
                        className="text-muted-foreground truncate text-xs"
                        title={name}
                    >
                        {name}
                    </span>
                    <div className="flex items-center gap-2">
                        <div className="bg-muted flex h-2 flex-1 overflow-hidden rounded-full">
                            <div
                                className="bg-foreground h-full"
                                style={{
                                    width: `${Math.max(6, (value / maxValue) * 100)}%`,
                                }}
                            />
                        </div>
                        <span className="text-muted-foreground text-xs tabular-nums">
                            {formatPercent(value)}
                        </span>
                    </div>
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
    const [activePlot, setActivePlot] = useState<{
        title: string;
        data: string;
    } | null>(null);

    if (entries.length === 0) {
        return (
            <div className="text-muted-foreground text-xs">
                No SHAP plots available.
            </div>
        );
    }

    const globalPlots = buildPlotGroup(entries, [
        ["summary", "Summary (global impact)"],
        ["beeswarm", "Beeswarm (feature effects)"],
        ["feature_importance", "Feature importance"],
    ]);
    const localPlots = buildPlotGroup(entries, [
        ["waterfall", "Waterfall (single prediction)"],
        ["decision", "Decision (model path)"],
    ]);
    const dependencePlots = entries
        .filter(([name]) => name.startsWith("dependence:"))
        .map(([name, data]) => ({
            name,
            label: `Dependence: ${name.replace("dependence:", "")}`,
            data,
        }));

    return (
        <div className="flex flex-col gap-4">
            {globalPlots.length > 0 && (
                <PlotSection
                    title="Global explanations"
                    plots={globalPlots}
                    onSelect={setActivePlot}
                />
            )}
            {localPlots.length > 0 && (
                <PlotSection
                    title="Local explanations"
                    plots={localPlots}
                    onSelect={setActivePlot}
                />
            )}
            {dependencePlots.length > 0 && (
                <PlotSection
                    title="Feature dependence"
                    plots={dependencePlots}
                    onSelect={setActivePlot}
                />
            )}
            {activePlot && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6">
                    <button
                        type="button"
                        className="absolute inset-0"
                        onClick={() => setActivePlot(null)}
                    />
                    <div className="bg-background border-border relative w-full max-w-5xl rounded-lg border p-4">
                        <div className="mb-3 flex items-center justify-between">
                            <span className="text-sm font-semibold">
                                {activePlot.title}
                            </span>
                            <button
                                type="button"
                                className="text-muted-foreground hover:text-foreground text-xs"
                                onClick={() => setActivePlot(null)}
                            >
                                Close
                            </button>
                        </div>
                        <div className="max-h-[80vh] overflow-auto">
                            <img
                                src={`data:image/png;base64,${activePlot.data}`}
                                alt=""
                                className="h-auto w-full object-contain"
                            />
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}

function buildPlotGroup(
    entries: [string, string][],
    configs: Array<[string, string]>,
) {
    return configs
        .map(([name, label]) => {
            const match = entries.find(([key]) => key === name);
            if (!match) {
                return null;
            }
            return {
                name,
                label,
                data: match[1],
            };
        })
        .filter((plot): plot is { name: string; label: string; data: string } =>
            Boolean(plot),
        );
}

function PlotSection({
    title,
    plots,
    onSelect,
}: {
    title: string;
    plots: { name: string; label: string; data: string }[];
    onSelect: (plot: { title: string; data: string }) => void;
}) {
    return (
        <div>
            <div className="text-muted-foreground mb-2 text-xs uppercase">
                {title}
            </div>
            <div className="grid gap-3 md:grid-cols-2">
                {plots.map((plot) => (
                    <button
                        key={plot.name}
                        type="button"
                        className="border-border bg-muted/20 hover:border-muted-foreground/40 flex flex-col gap-2 rounded-md border p-2 text-left transition"
                        onClick={() =>
                            onSelect({ title: plot.label, data: plot.data })
                        }
                    >
                        <span className="text-muted-foreground text-xs">
                            {plot.label}
                        </span>
                        <div className="bg-background flex items-center justify-center rounded-md border p-2">
                            <img
                                src={`data:image/png;base64,${plot.data}`}
                                alt=""
                                loading="lazy"
                                className="h-auto w-full object-contain"
                            />
                        </div>
                    </button>
                ))}
            </div>
        </div>
    );
}

function PanelTab({
    label,
    isActive,
    onClick,
}: {
    label: string;
    isActive: boolean;
    onClick: () => void;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={cn(
                "w-full text-center text-xs font-semibold tracking-wider uppercase transition-colors",
                isActive
                    ? "text-foreground"
                    : "text-muted-foreground hover:text-foreground",
            )}
        >
            {label}
        </button>
    );
}

export function MLResultsPanel({
    progress,
    result,
    error,
    activeTab,
    onTabChange,
    trainingStatus,
    onSelectHistory,
    onRefreshHistory,
    onClearHistory,
    historyEntries,
}: MLResultsPanelProps) {
    const isTraining = trainingStatus === "training";
    const metrics = result?.metrics;
    const testScore = metrics?.test_score ?? metrics?.accuracy ?? metrics?.r2;
    const progressPercent = Math.round((progress?.progress ?? 0) * 100);
    const hasData = Boolean(result || progress || error);

    return (
        <div className="flex h-full min-h-0 flex-col overflow-hidden rounded-lg border">
            <div className="bg-muted/30 shrink-0 border-b px-3 py-2">
                <div className="grid grid-cols-4">
                    <PanelTab
                        label="Overview"
                        isActive={activeTab === "overview"}
                        onClick={() => onTabChange("overview")}
                    />
                    <PanelTab
                        label="Importance"
                        isActive={
                            activeTab === "importance" ||
                            activeTab === "comparison"
                        }
                        onClick={() => onTabChange("importance")}
                    />
                    <PanelTab
                        label="SHAP"
                        isActive={activeTab === "shap"}
                        onClick={() => onTabChange("shap")}
                    />
                    <PanelTab
                        label="History"
                        isActive={activeTab === "history"}
                        onClick={() => onTabChange("history")}
                    />
                </div>
                {result?.warnings?.length ? (
                    <div className="text-muted-foreground mt-2 flex items-center gap-2 text-xs">
                        <Sparkles className="h-3.5 w-3.5" />
                        {result.warnings.length} warning
                        {result.warnings.length > 1 ? "s" : ""}
                    </div>
                ) : null}
            </div>

            {activeTab === "overview" ? (
                <div className="min-h-0 flex-1 overflow-y-auto p-3">
                    {hasData ? (
                        <div className="flex flex-col gap-4">
                            <div>
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
                                <div className="mt-3">
                                    <ProgressBar value={progressPercent} />
                                </div>
                                {result && (
                                    <div className="mt-3 grid gap-3 md:grid-cols-2">
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
                                    <div className="border-muted bg-muted/30 mt-3 flex items-start gap-2 rounded-md border p-3">
                                        <AlertTriangle className="text-muted-foreground h-4 w-4" />
                                        <p className="text-muted-foreground text-xs">
                                            {error}
                                        </p>
                                    </div>
                                )}
                            </div>

                            <div>
                                <div className="text-muted-foreground mb-2 flex items-center gap-2 text-xs uppercase">
                                    <BarChart3 className="h-3.5 w-3.5" />
                                    Metrics
                                </div>
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
                            </div>

                            <div>
                                <div className="text-muted-foreground mb-2 flex items-center gap-2 text-xs uppercase">
                                    <Layers className="h-3.5 w-3.5" />
                                    Model summary
                                </div>
                                {result ? (
                                    <div className="grid gap-3 md:grid-cols-3">
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
                                                {
                                                    result.feature_importance
                                                        .length
                                                }{" "}
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
                            </div>

                            <div>
                                <div className="text-muted-foreground mb-2 flex items-center gap-2 text-xs uppercase">
                                    <Layers className="h-3.5 w-3.5" />
                                    Model comparison
                                </div>
                                {result ? (
                                    <ModelComparisonTable
                                        models={result.model_comparison}
                                    />
                                ) : (
                                    <EmptyPanel
                                        icon={
                                            <Layers className="text-muted-foreground h-5 w-5" />
                                        }
                                        title="No comparison data"
                                        message="Train models to compare performance."
                                    />
                                )}
                            </div>
                        </div>
                    ) : (
                        <EmptyPanel
                            icon={
                                <Brain className="text-muted-foreground h-5 w-5" />
                            }
                            title="No training data"
                            message="Initialize the kernel and start training to see results."
                        />
                    )}
                </div>
            ) : null}

            {activeTab === "importance" || activeTab === "comparison" ? (
                <div className="min-h-0 flex-1 overflow-y-auto p-3">
                    {result ? (
                        <div className="flex flex-col gap-4">
                            <div>
                                <div className="text-muted-foreground flex items-center gap-2 text-xs uppercase">
                                    <BarChart3 className="h-3.5 w-3.5" />
                                    Feature importance
                                </div>
                                <div className="mt-2">
                                    <FeatureImportanceChart
                                        items={result.feature_importance.slice(
                                            0,
                                            12,
                                        )}
                                    />
                                </div>
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
                </div>
            ) : null}

            {activeTab === "shap" ? (
                <div className="min-h-0 flex-1 overflow-y-auto p-3">
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
                </div>
            ) : null}

            {activeTab === "history" ? (
                <div className="min-h-0 flex-1 overflow-y-auto p-3">
                    <TrainingHistory
                        entries={historyEntries}
                        onRefresh={onRefreshHistory}
                        onClear={onClearHistory}
                        onSelect={onSelectHistory}
                        disabled={isTraining}
                    />
                </div>
            ) : null}
        </div>
    );
}
