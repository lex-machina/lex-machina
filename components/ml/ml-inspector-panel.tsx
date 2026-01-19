"use client";
"use client";

import { useCallback, useState, type ChangeEvent } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";

import type {
    MLTrainingStatus,
    MLUIState,
    TrainingHistoryEntry,
} from "@/types";
import { cn } from "@/lib/utils";
import { useML } from "@/lib/hooks/use-ml";
import { Toggle } from "@/components/ui/toggle";
import { Select, type SelectOption } from "@/components/ui/select";
import { Slider } from "@/components/ui/slider";
import { Input } from "@/components/ui/input";
import { Card, CardHeader, CardContent } from "@/components/ui/card";
import { TrainingHistory } from "@/components/ml/training-history";

interface MLInspectorPanelProps {
    uiState: MLUIState;
    setUIState: (state: MLUIState) => void;
    trainingStatus: MLTrainingStatus;
    onSelectHistory: (entry: TrainingHistoryEntry) => void;
}

const algorithmOptions: SelectOption[] = [
    { value: "", label: "Auto-select" },
    { value: "random_forest", label: "Random Forest" },
    { value: "xgboost", label: "XGBoost" },
    { value: "lightgbm", label: "LightGBM" },
    { value: "catboost", label: "CatBoost" },
    { value: "linear", label: "Linear Model" },
    { value: "svm", label: "SVM" },
    { value: "neural_network", label: "Neural Network" },
];

function updateConfig(state: MLUIState, next: Partial<MLUIState>): MLUIState {
    return {
        ...state,
        ...next,
        config: {
            ...state.config,
            ...(next.config ?? {}),
        },
    };
}

export function MLInspectorPanel({
    uiState,
    setUIState,
    trainingStatus,
    onSelectHistory,
}: MLInspectorPanelProps) {
    const { getHistory, clearHistory } = useML();
    const [advancedOpen, setAdvancedOpen] = useState(true);
    const [history, setHistory] = useState<TrainingHistoryEntry[]>([]);

    const isTraining = trainingStatus === "training";
    const manualDisabled = isTraining || uiState.smart_mode;

    const handleHistoryRefresh = useCallback(async () => {
        const entries = await getHistory();
        setHistory(entries);
        return entries;
    }, [getHistory]);

    const handleHistoryClear = useCallback(async () => {
        await clearHistory();
        setHistory([]);
    }, [clearHistory]);

    return (
        <div className="flex h-full min-h-0 flex-col border-l">
            <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto p-4">
                <Card className={cn(manualDisabled && "opacity-60")}>
                    <CardHeader
                        title="Advanced"
                        actions={
                            <button
                                type="button"
                                onClick={() => setAdvancedOpen((open) => !open)}
                                className="text-muted-foreground hover:text-foreground flex items-center gap-1 text-xs"
                            >
                                {advancedOpen ? "Hide" : "Show"}
                                {advancedOpen ? (
                                    <ChevronDown className="h-3.5 w-3.5" />
                                ) : (
                                    <ChevronRight className="h-3.5 w-3.5" />
                                )}
                            </button>
                        }
                    />
                    {advancedOpen && (
                        <CardContent className="space-y-3 p-3">
                            <Select
                                label="Algorithm"
                                value={uiState.config.algorithm ?? ""}
                                options={algorithmOptions}
                                onValueChange={(value) =>
                                    setUIState(
                                        updateConfig(uiState, {
                                            config: {
                                                ...uiState.config,
                                                algorithm: value || undefined,
                                            },
                                        }),
                                    )
                                }
                                disabled={manualDisabled}
                            />
                            <Toggle
                                pressed={uiState.config.optimize_hyperparams}
                                onPressedChange={(value) =>
                                    setUIState(
                                        updateConfig(uiState, {
                                            config: {
                                                ...uiState.config,
                                                optimize_hyperparams: value,
                                            },
                                        }),
                                    )
                                }
                                label="Optimize hyperparameters"
                                disabled={manualDisabled}
                            />
                            <Slider
                                label="Trials"
                                min={5}
                                max={50}
                                step={1}
                                value={uiState.config.n_trials}
                                onValueChange={(value: number) =>
                                    setUIState(
                                        updateConfig(uiState, {
                                            config: {
                                                ...uiState.config,
                                                n_trials: value,
                                            },
                                        }),
                                    )
                                }
                                disabled={manualDisabled}
                                showValue
                            />
                            <Slider
                                label="CV folds"
                                min={3}
                                max={10}
                                step={1}
                                value={uiState.config.cv_folds}
                                onValueChange={(value: number) =>
                                    setUIState(
                                        updateConfig(uiState, {
                                            config: {
                                                ...uiState.config,
                                                cv_folds: value,
                                            },
                                        }),
                                    )
                                }
                                disabled={manualDisabled}
                                showValue
                            />
                            <Input
                                label="Test size"
                                type="number"
                                value={uiState.config.test_size}
                                onChange={(
                                    event: ChangeEvent<HTMLInputElement>,
                                ) =>
                                    setUIState(
                                        updateConfig(uiState, {
                                            config: {
                                                ...uiState.config,
                                                test_size: Number(
                                                    event.target.value,
                                                ),
                                            },
                                        }),
                                    )
                                }
                                disabled={manualDisabled}
                                step={0.05}
                                min={0.1}
                                max={0.5}
                            />
                            <Toggle
                                pressed={uiState.config.enable_neural_networks}
                                onPressedChange={(value) =>
                                    setUIState(
                                        updateConfig(uiState, {
                                            config: {
                                                ...uiState.config,
                                                enable_neural_networks: value,
                                            },
                                        }),
                                    )
                                }
                                label="Enable neural networks"
                                disabled={manualDisabled}
                            />
                            <Toggle
                                pressed={uiState.config.enable_explainability}
                                onPressedChange={(value) =>
                                    setUIState(
                                        updateConfig(uiState, {
                                            config: {
                                                ...uiState.config,
                                                enable_explainability: value,
                                            },
                                        }),
                                    )
                                }
                                label="Enable explainability"
                                description="Generate SHAP plots and feature attributions"
                                disabled={manualDisabled}
                            />
                            <Slider
                                label="Top algorithms"
                                min={1}
                                max={10}
                                step={1}
                                value={uiState.config.top_k_algorithms}
                                onValueChange={(value: number) =>
                                    setUIState(
                                        updateConfig(uiState, {
                                            config: {
                                                ...uiState.config,
                                                top_k_algorithms: value,
                                            },
                                        }),
                                    )
                                }
                                disabled={manualDisabled}
                                showValue
                            />
                        </CardContent>
                    )}
                </Card>

                <Card>
                    <CardHeader title="History" />
                    <CardContent className="p-0">
                        <TrainingHistory
                            entries={history}
                            onRefresh={handleHistoryRefresh}
                            onClear={handleHistoryClear}
                            onSelect={onSelectHistory}
                            disabled={isTraining}
                        />
                    </CardContent>
                </Card>
            </div>
        </div>
    );
}
