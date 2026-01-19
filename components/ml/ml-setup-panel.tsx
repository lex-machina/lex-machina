"use client";

import { useCallback, useMemo } from "react";

import type { ColumnInfo, MLTrainingStatus, MLUIState } from "@/types";
import { cn } from "@/lib/utils";
import { Toggle } from "@/components/ui/toggle";
import { Select, type SelectOption } from "@/components/ui/select";
import { Checkbox } from "@/components/ui/checkbox";
import { Slider } from "@/components/ui/slider";
import { Input } from "@/components/ui/input";
import { Card, CardHeader, CardContent } from "@/components/ui/card";

interface MLSetupPanelProps {
    uiState: MLUIState;
    setUIState: (state: MLUIState) => void;
    availableColumns: ColumnInfo[];
    trainingStatus: MLTrainingStatus;
}

const problemTypeOptions: SelectOption[] = [
    { value: "classification", label: "Classification" },
    { value: "regression", label: "Regression" },
];

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

export function MLSetupPanel({
    uiState,
    setUIState,
    availableColumns,
    trainingStatus,
}: MLSetupPanelProps) {
    const isTraining = trainingStatus === "training";
    const hasColumns = availableColumns.length > 0;

    const baseDisabled = !hasColumns || isTraining;
    const advancedDisabled = uiState.smart_mode || isTraining;

    const columnOptions = useMemo<SelectOption[]>(() => {
        return availableColumns.map((col) => ({
            value: col.name,
            label: col.name,
        }));
    }, [availableColumns]);

    const featureColumns = useMemo(() => {
        return availableColumns.filter(
            (col) => col.name !== uiState.target_column,
        );
    }, [availableColumns, uiState.target_column]);

    const excludedSet = useMemo(
        () => new Set(uiState.excluded_columns),
        [uiState.excluded_columns],
    );

    const handleToggleMode = useCallback(
        (smart: boolean) => {
            setUIState(
                updateConfig(uiState, {
                    smart_mode: smart,
                }),
            );
        },
        [uiState, setUIState],
    );

    const handleTargetChange = useCallback(
        (value: string) => {
            setUIState(
                updateConfig(uiState, {
                    target_column: value || null,
                    problem_type: value
                        ? uiState.problem_type || "classification"
                        : uiState.problem_type,
                    excluded_columns: uiState.excluded_columns.filter(
                        (name) => name !== value,
                    ),
                }),
            );
        },
        [uiState, setUIState],
    );

    const handleProblemType = useCallback(
        (value: string) => {
            setUIState(
                updateConfig(uiState, {
                    problem_type: value,
                }),
            );
        },
        [uiState, setUIState],
    );

    const handleFeatureToggle = useCallback(
        (column: string, enabled: boolean) => {
            const next = new Set(uiState.excluded_columns);
            if (enabled) {
                next.delete(column);
            } else {
                next.add(column);
            }
            setUIState(updateConfig(uiState, { excluded_columns: [...next] }));
        },
        [uiState, setUIState],
    );

    return (
        <div className="flex h-full min-h-0 flex-col">
            <Card className="h-full min-h-0">
                <CardHeader title="Configuration" />
                <CardContent scrollable padded>
                    <div className="flex flex-col gap-4">
                        <section>
                            <div className="text-muted-foreground text-xs font-semibold uppercase">
                                Mode
                            </div>
                            <div className="mt-2">
                                <Toggle
                                    pressed={uiState.smart_mode}
                                    onPressedChange={handleToggleMode}
                                    label={
                                        uiState.smart_mode
                                            ? "Smart mode"
                                            : "Manual mode"
                                    }
                                    description={
                                        uiState.smart_mode
                                            ? "Auto-selects algorithms and settings"
                                            : "Customize algorithms and tuning"
                                    }
                                />
                            </div>
                        </section>

                        <section className={cn(baseDisabled && "opacity-60")}>
                            <div className="text-muted-foreground text-xs font-semibold uppercase">
                                Target
                            </div>
                            <div className="mt-2">
                                <Select
                                    label="Target column"
                                    value={uiState.target_column ?? ""}
                                    options={[
                                        { value: "", label: "Select target" },
                                        ...columnOptions,
                                    ]}
                                    onValueChange={handleTargetChange}
                                    disabled={baseDisabled}
                                />
                                <div className="mt-3">
                                    <Select
                                        label="Problem type"
                                        value={
                                            uiState.problem_type ||
                                            "classification"
                                        }
                                        options={problemTypeOptions}
                                        onValueChange={handleProblemType}
                                        disabled={baseDisabled}
                                    />
                                </div>
                            </div>
                        </section>

                        <section className={cn(baseDisabled && "opacity-60")}>
                            <div className="text-muted-foreground text-xs font-semibold uppercase">
                                Features
                            </div>
                            <div className="mt-2 space-y-2">
                                {featureColumns.length === 0 ? (
                                    <div className="text-muted-foreground text-xs">
                                        No features available.
                                    </div>
                                ) : (
                                    featureColumns.map((col) => (
                                        <Checkbox
                                            key={col.name}
                                            checked={!excludedSet.has(col.name)}
                                            onCheckedChange={(checked) =>
                                                handleFeatureToggle(
                                                    col.name,
                                                    checked,
                                                )
                                            }
                                            label={col.name}
                                            disabled={baseDisabled}
                                        />
                                    ))
                                )}
                            </div>
                            <div className="text-muted-foreground mt-2 text-xs">
                                {featureColumns.length} selectable features
                            </div>
                        </section>

                        <section
                            className={cn(advancedDisabled && "opacity-60")}
                        >
                            <div className="text-muted-foreground text-xs font-semibold uppercase">
                                Advanced
                            </div>
                            <div className="mt-2 space-y-3">
                                <Select
                                    label="Algorithm"
                                    value={uiState.config.algorithm ?? ""}
                                    options={algorithmOptions}
                                    onValueChange={(value) =>
                                        setUIState(
                                            updateConfig(uiState, {
                                                config: {
                                                    ...uiState.config,
                                                    algorithm:
                                                        value || undefined,
                                                },
                                            }),
                                        )
                                    }
                                    disabled={advancedDisabled}
                                />
                                <Toggle
                                    pressed={
                                        uiState.config.optimize_hyperparams
                                    }
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
                                    disabled={advancedDisabled}
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
                                    disabled={advancedDisabled}
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
                                    disabled={advancedDisabled}
                                    showValue
                                />
                                <Input
                                    label="Test size"
                                    type="number"
                                    value={uiState.config.test_size}
                                    onChange={(event) =>
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
                                    disabled={advancedDisabled}
                                    step={0.05}
                                    min={0.1}
                                    max={0.5}
                                />
                                <Toggle
                                    pressed={
                                        uiState.config.enable_neural_networks
                                    }
                                    onPressedChange={(value) =>
                                        setUIState(
                                            updateConfig(uiState, {
                                                config: {
                                                    ...uiState.config,
                                                    enable_neural_networks:
                                                        value,
                                                },
                                            }),
                                        )
                                    }
                                    label="Enable neural networks"
                                    disabled={advancedDisabled}
                                />
                                <Toggle
                                    pressed={
                                        uiState.config.enable_explainability
                                    }
                                    onPressedChange={(value) =>
                                        setUIState(
                                            updateConfig(uiState, {
                                                config: {
                                                    ...uiState.config,
                                                    enable_explainability:
                                                        value,
                                                },
                                            }),
                                        )
                                    }
                                    label="Enable explainability"
                                    description="Generate SHAP plots and feature attributions"
                                    disabled={advancedDisabled}
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
                                    disabled={advancedDisabled}
                                    showValue
                                />
                            </div>
                        </section>
                    </div>
                </CardContent>
            </Card>
        </div>
    );
}
