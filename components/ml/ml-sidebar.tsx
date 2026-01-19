"use client";

import { useCallback, useMemo, useState } from "react";
import {
    Play,
    Square,
    Save,
    FolderOpen,
    Zap,
    ChevronDown,
    ChevronRight,
} from "lucide-react";

import { useFileState } from "@/lib/hooks/use-file-state";
import { useProcessedData } from "@/lib/hooks/use-processed-data";
import type {
    ColumnInfo,
    MLConfigRequest,
    MLTrainingStatus,
    MLUIState,
    ModelInfo,
    TrainingResultResponse,
} from "@/types";
import { cn, formatNumber } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Toggle } from "@/components/ui/toggle";
import { Select, type SelectOption } from "@/components/ui/select";
import { Checkbox } from "@/components/ui/checkbox";
import { Slider } from "@/components/ui/slider";
import { Input } from "@/components/ui/input";
import {
    Card,
    CardHeader,
    CardContent,
    CardFooter,
} from "@/components/ui/card";

interface MLSidebarProps {
    uiState: MLUIState;
    setUIState: (state: MLUIState) => void;
    availableColumns: ColumnInfo[];
    trainingStatus: MLTrainingStatus;
    kernelStatus: "uninitialized" | "initializing" | "ready" | "error";
    onInitializeKernel: () => Promise<void>;
    onStartTraining: (
        request: MLConfigRequest,
    ) => Promise<TrainingResultResponse>;
    onCancelTraining: () => Promise<void>;
    onSaveModel: () => Promise<string>;
    onLoadModel: () => Promise<ModelInfo>;
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

function buildRequest(state: MLUIState): MLConfigRequest {
    return {
        smart_mode: state.smart_mode,
        target_column: state.target_column ?? "",
        problem_type: state.problem_type as "classification" | "regression",
        excluded_columns: state.excluded_columns,
        use_processed_data: state.use_processed_data,
        optimize_hyperparams: state.smart_mode
            ? undefined
            : state.config.optimize_hyperparams,
        n_trials: state.smart_mode ? undefined : state.config.n_trials,
        cv_folds: state.smart_mode ? undefined : state.config.cv_folds,
        test_size: state.smart_mode ? undefined : state.config.test_size,
        enable_neural_networks: state.smart_mode
            ? undefined
            : state.config.enable_neural_networks,
        enable_explainability: state.smart_mode
            ? undefined
            : state.config.enable_explainability,
        top_k_algorithms: state.smart_mode
            ? undefined
            : state.config.top_k_algorithms,
        algorithm: state.smart_mode
            ? undefined
            : state.config.algorithm || undefined,
    };
}

export function MLSidebar({
    uiState,
    setUIState,
    availableColumns,
    trainingStatus,
    kernelStatus,
    onInitializeKernel,
    onStartTraining,
    onCancelTraining,
    onSaveModel,
    onLoadModel,
}: MLSidebarProps) {
    const { fileInfo } = useFileState();
    const processedData = useProcessedData();

    const [advancedOpen, setAdvancedOpen] = useState(true);
    const [featuresOpen, setFeaturesOpen] = useState(true);

    const hasProcessedData = processedData.hasProcessedData;

    const isTraining = trainingStatus === "training";
    const hasColumns = availableColumns.length > 0;

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

    const totalRows = useMemo(() => {
        if (uiState.use_processed_data && processedData.fileInfo) {
            return processedData.fileInfo.row_count;
        }
        return fileInfo?.row_count ?? 0;
    }, [uiState.use_processed_data, processedData.fileInfo, fileInfo]);

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

    const handleDataSource = useCallback(
        (processed: boolean) => {
            if (processed && !hasProcessedData) {
                setUIState(
                    updateConfig(uiState, {
                        use_processed_data: false,
                    }),
                );
                return;
            }
            setUIState(
                updateConfig(uiState, {
                    use_processed_data: processed,
                }),
            );
        },
        [uiState, setUIState, hasProcessedData],
    );

    const handleTargetChange = useCallback(
        (value: string) => {
            setUIState(
                updateConfig(uiState, {
                    target_column: value || null,
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

    const handleTraining = useCallback(async () => {
        if (!uiState.problem_type) {
            setUIState(
                updateConfig(uiState, {
                    problem_type: "classification",
                }),
            );
        }
        const request = buildRequest({
            ...uiState,
            problem_type: uiState.problem_type || "classification",
        });
        await onStartTraining(request);
    }, [uiState, onStartTraining, setUIState]);

    const handleSave = useCallback(async () => {
        await onSaveModel();
    }, [onSaveModel]);

    const handleLoad = useCallback(async () => {
        await onLoadModel();
    }, [onLoadModel]);

    const isKernelReady = kernelStatus === "ready";
    const disabledControls = !hasColumns || !isKernelReady;
    const manualDisabled = disabledControls || uiState.smart_mode;
    const targetDisabled =
        disabledControls || (uiState.use_processed_data && !hasProcessedData);

    return (
        <div className="flex h-full min-h-0 flex-col">
            <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto p-4">
                <Card>
                    <CardHeader title="Dataset" />
                    <CardContent padded>
                        <dl className="space-y-2 text-sm">
                            <div>
                                <dt className="text-muted-foreground">File</dt>
                                <dd className="truncate font-medium">
                                    {fileInfo?.name ?? "No file"}
                                </dd>
                            </div>
                            <div className="flex items-center justify-between">
                                <dt className="text-muted-foreground">Rows</dt>
                                <dd className="font-medium">
                                    {formatNumber(totalRows)}
                                </dd>
                            </div>
                            <div className="flex items-center justify-between">
                                <dt className="text-muted-foreground">
                                    Columns
                                </dt>
                                <dd className="font-medium">
                                    {formatNumber(availableColumns.length)}
                                </dd>
                            </div>
                        </dl>
                        <div className="text-muted-foreground mt-3 flex items-center gap-2 text-xs">
                            <Zap className="h-3.5 w-3.5" />
                            {uiState.use_processed_data
                                ? "Using processed data"
                                : "Using original data"}
                        </div>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader title="Kernel" />
                    <CardContent padded>
                        <div className="flex items-center justify-between text-sm">
                            <span className="text-muted-foreground">
                                Status
                            </span>
                            <span
                                className={cn(
                                    isKernelReady
                                        ? "text-foreground"
                                        : "text-muted-foreground",
                                )}
                            >
                                {kernelStatus === "initializing"
                                    ? "Starting"
                                    : kernelStatus === "error"
                                      ? "Error"
                                      : kernelStatus === "ready"
                                        ? "Ready"
                                        : "Not ready"}
                            </span>
                        </div>
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={onInitializeKernel}
                            disabled={
                                isKernelReady || kernelStatus === "initializing"
                            }
                            className="mt-3 w-full"
                        >
                            {kernelStatus === "initializing"
                                ? "Initializing..."
                                : "Initialize kernel"}
                        </Button>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader title="Mode" />
                    <CardContent padded>
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
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader title="Data Source" />
                    <CardContent padded>
                        <Toggle
                            pressed={uiState.use_processed_data}
                            onPressedChange={handleDataSource}
                            label={
                                uiState.use_processed_data
                                    ? "Processed data"
                                    : "Original data"
                            }
                            description={
                                uiState.use_processed_data && !hasProcessedData
                                    ? "No processed data available"
                                    : "Switch between raw and processed data"
                            }
                        />
                    </CardContent>
                </Card>

                <Card className={cn(disabledControls && "opacity-60")}>
                    <CardHeader title="Target" />
                    <CardContent padded>
                        <Select
                            label="Target column"
                            value={uiState.target_column ?? ""}
                            options={[
                                { value: "", label: "Select target" },
                                ...columnOptions,
                            ]}
                            onValueChange={handleTargetChange}
                            disabled={targetDisabled}
                        />
                        <div className="mt-3">
                            <Select
                                label="Problem type"
                                value={uiState.problem_type}
                                options={problemTypeOptions}
                                onValueChange={handleProblemType}
                                disabled={targetDisabled}
                            />
                        </div>
                    </CardContent>
                </Card>

                <Card className={cn(disabledControls && "opacity-60")}>
                    <CardHeader
                        title="Features"
                        actions={
                            <button
                                type="button"
                                onClick={() => setFeaturesOpen((open) => !open)}
                                className="text-muted-foreground hover:text-foreground flex items-center gap-1 text-xs"
                            >
                                {featuresOpen ? "Hide" : "Show"}
                                {featuresOpen ? (
                                    <ChevronDown className="h-3.5 w-3.5" />
                                ) : (
                                    <ChevronRight className="h-3.5 w-3.5" />
                                )}
                            </button>
                        }
                    />
                    {featuresOpen && (
                        <CardContent className="max-h-56 overflow-y-auto">
                            <div className="flex flex-col gap-2 p-3">
                                {featureColumns.map((col) => (
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
                                        disabled={targetDisabled}
                                    />
                                ))}
                            </div>
                        </CardContent>
                    )}
                    <CardFooter className="text-muted-foreground text-xs">
                        {featuresOpen
                            ? `${featureColumns.length} selectable features`
                            : "Feature list hidden"}
                    </CardFooter>
                </Card>

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
                                onValueChange={(value) =>
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
                                onValueChange={(value) =>
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
                                onValueChange={(value) =>
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
            </div>

            <div className="border-border grid gap-2 border-t p-4">
                <Button
                    size="sm"
                    onClick={handleTraining}
                    disabled={disabledControls || isTraining}
                    className="w-full"
                >
                    <Play className="h-4 w-4" />
                    {isTraining ? "Training..." : "Train model"}
                </Button>
                <div className="grid grid-cols-2 gap-2">
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={handleSave}
                        disabled={isTraining}
                    >
                        <Save className="h-4 w-4" />
                        Save
                    </Button>
                    <Button variant="outline" size="sm" onClick={handleLoad}>
                        <FolderOpen className="h-4 w-4" />
                        Load
                    </Button>
                </div>
                <Button
                    variant="destructive"
                    size="sm"
                    onClick={onCancelTraining}
                    disabled={!isTraining}
                >
                    <Square className="h-4 w-4" />
                    Cancel training
                </Button>
            </div>
        </div>
    );
}
