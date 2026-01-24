"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import AppShell from "@/components/layout/app-shell";
import {
    MLSetupPanel,
    MLResultsPanel,
    MLPredictionPanel,
    MLOperationsSidebar,
} from "@/components/ml";
import { useFileState } from "@/lib/hooks/use-file-state";
import { useProcessedData } from "@/lib/hooks/use-processed-data";
import { useML } from "@/lib/hooks/use-ml";
import { useMLUIState } from "@/lib/hooks/use-ml-ui-state";
import type {
    ColumnInfo,
    TrainingHistoryEntry,
    MLConfigRequest,
} from "@/types";

// ============================================================================
// ML PAGE
// ============================================================================

/**
 * ML page - Automated Machine Learning workspace.
 *
 * Features:
 * - Target variable selection
 * - Feature selection
 * - AutoML training and evaluation
 * - Prediction and training history
 */
export default function MLPage() {
    const { fileInfo } = useFileState();
    const processedData = useProcessedData();
    const { isLoaded, uiState, setUIState } = useMLUIState(fileInfo);
    const {
        kernelStatus,
        trainingStatus,
        progress,
        result,
        error,
        initializeKernel,
        startTraining,
        cancelTraining,
        saveModel,
        loadModel,
        getHistory,
        clearHistory,
        predictSingle,
        predictBatchFromCsv,
        refreshKernelStatus,
    } = useML();

    const [historyEntries, setHistoryEntries] = useState<
        TrainingHistoryEntry[]
    >([]);

    const availableColumns = useMemo<ColumnInfo[]>(() => {
        if (
            uiState.use_processed_data &&
            processedData.hasProcessedData &&
            processedData.fileInfo
        ) {
            return processedData.fileInfo.columns;
        }
        return fileInfo?.columns ?? [];
    }, [
        uiState.use_processed_data,
        processedData.hasProcessedData,
        processedData.fileInfo,
        fileInfo?.columns,
    ]);

    const predictionColumns = useMemo(
        () =>
            uiState.target_column
                ? availableColumns.filter(
                      (column) => column.name !== uiState.target_column,
                  )
                : availableColumns,
        [availableColumns, uiState.target_column],
    );

    useEffect(() => {
        if (!uiState.target_column) {
            return;
        }

        const targetExists = availableColumns.some(
            (col) => col.name === uiState.target_column,
        );

        if (!targetExists) {
            setUIState({
                ...uiState,
                target_column: null,
                excluded_columns: uiState.excluded_columns.filter(
                    (column) => column !== uiState.target_column,
                ),
            });
        }
    }, [availableColumns, uiState, setUIState]);

    useEffect(() => {
        refreshKernelStatus().catch(() => {
            // ignore
        });
    }, [refreshKernelStatus]);

    const handleTabChange = (tab: string) => {
        setUIState({
            ...uiState,
            active_tab: tab,
        });
    };

    const previousTrainingStatus = useRef(trainingStatus);

    useEffect(() => {
        if (
            previousTrainingStatus.current !== "completed" &&
            trainingStatus === "completed" &&
            uiState.active_tab !== "overview"
        ) {
            setUIState({
                ...uiState,
                active_tab: "overview",
            });
        }
        previousTrainingStatus.current = trainingStatus;
    }, [trainingStatus, uiState, setUIState]);

    const handleHistorySelect = (entry: TrainingHistoryEntry) => {
        setUIState({
            ...uiState,
            target_column: entry.config.target_column,
            problem_type: entry.config.problem_type,
            excluded_columns: entry.config.excluded_columns,
            use_processed_data: entry.config.use_processed_data,
            config: {
                ...uiState.config,
                optimize_hyperparams: entry.config.optimize_hyperparams,
                n_trials: entry.config.n_trials,
                cv_folds: entry.config.cv_folds,
                enable_explainability: entry.config.enable_explainability,
                top_k_algorithms: entry.config.top_k_algorithms,
                algorithm: entry.config.algorithm,
            },
            active_tab: "history",
        });
    };

    const handleHistoryRefresh = useCallback(async () => {
        const entries = await getHistory();
        setHistoryEntries(entries);
        return entries;
    }, [getHistory]);

    const handleHistoryClear = useCallback(async () => {
        await clearHistory();
        setHistoryEntries([]);
    }, [clearHistory]);

    const handleStartTraining = useCallback(async () => {
        const problemType = (uiState.problem_type || "classification") as
            | "classification"
            | "regression";

        const request: MLConfigRequest = {
            smart_mode: uiState.smart_mode,
            target_column: uiState.target_column ?? "",
            problem_type: problemType,
            excluded_columns: uiState.excluded_columns,
            use_processed_data: uiState.use_processed_data,
            optimize_hyperparams: uiState.smart_mode
                ? undefined
                : uiState.config.optimize_hyperparams,
            n_trials: uiState.smart_mode ? undefined : uiState.config.n_trials,
            cv_folds: uiState.smart_mode ? undefined : uiState.config.cv_folds,
            test_size: uiState.smart_mode
                ? undefined
                : uiState.config.test_size,
            enable_neural_networks: uiState.smart_mode
                ? undefined
                : uiState.config.enable_neural_networks,
            enable_explainability: uiState.smart_mode
                ? undefined
                : uiState.config.enable_explainability,
            top_k_algorithms: uiState.smart_mode
                ? undefined
                : uiState.config.top_k_algorithms,
            algorithm: uiState.smart_mode
                ? undefined
                : uiState.config.algorithm || undefined,
        };
        await startTraining(request);
    }, [uiState, startTraining]);

    if (!isLoaded) {
        return (
            <AppShell sidebar={<div className="p-4" />}>
                <div className="text-muted-foreground flex h-full items-center justify-center">
                    Loading...
                </div>
            </AppShell>
        );
    }

    const totalRows = uiState.use_processed_data
        ? (processedData.fileInfo?.row_count ?? 0)
        : (fileInfo?.row_count ?? 0);

    const datasetName = uiState.use_processed_data
        ? (processedData.fileInfo?.name ?? fileInfo?.name)
        : fileInfo?.name;

    const hasModel = Boolean(result);

    return (
        <AppShell
            sidebar={
                <MLOperationsSidebar
                    datasetName={datasetName ?? null}
                    totalRows={totalRows}
                    useProcessedData={uiState.use_processed_data}
                    hasProcessedData={processedData.hasProcessedData}
                    onToggleProcessedData={(value) => {
                        if (!processedData.hasProcessedData && value) {
                            return;
                        }
                        setUIState({
                            ...uiState,
                            use_processed_data: value,
                        });
                    }}
                    kernelStatus={kernelStatus}
                    onInitializeKernel={initializeKernel}
                    trainingStatus={trainingStatus}
                    onStartTraining={handleStartTraining}
                    onCancelTraining={cancelTraining}
                    onSaveModel={saveModel}
                    onLoadModel={loadModel}
                    canStartTraining={Boolean(uiState.target_column)}
                />
            }
        >
            <div className="grid h-full min-h-0 flex-1 grid-cols-3 gap-4 p-4">
                <div className="min-h-0">
                    <MLSetupPanel
                        uiState={uiState}
                        setUIState={setUIState}
                        availableColumns={availableColumns}
                        trainingStatus={trainingStatus}
                    />
                </div>
                <div className="min-h-0">
                    <MLResultsPanel
                        progress={progress}
                        result={result}
                        error={error}
                        activeTab={uiState.active_tab}
                        onTabChange={handleTabChange}
                        trainingStatus={trainingStatus}
                        onSelectHistory={handleHistorySelect}
                        onRefreshHistory={handleHistoryRefresh}
                        onClearHistory={handleHistoryClear}
                        historyEntries={historyEntries}
                    />
                </div>
                <div className="min-h-0">
                    <MLPredictionPanel
                        columns={predictionColumns}
                        onPredictSingle={predictSingle}
                        onPredictBatchFromCsv={predictBatchFromCsv}
                        disabled={!hasModel || trainingStatus === "training"}
                    />
                </div>
            </div>
        </AppShell>
    );
}
