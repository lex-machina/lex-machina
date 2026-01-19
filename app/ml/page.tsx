"use client";
"use client";

import { useEffect, useMemo } from "react";

import AppShell from "@/components/layout/app-shell";
import { MLContent, MLSidebar } from "@/components/ml";
import { useFileState } from "@/lib/hooks/use-file-state";
import { useProcessedData } from "@/lib/hooks/use-processed-data";
import { useML } from "@/lib/hooks/use-ml";
import { useMLUIState } from "@/lib/hooks/use-ml-ui-state";
import type { ColumnInfo, TrainingHistoryEntry } from "@/types";

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
    } = useML();

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

    const handleTabChange = (tab: string) => {
        setUIState({
            ...uiState,
            active_tab: tab,
        });
    };

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
            active_tab: "results",
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
                <MLSidebar
                    uiState={uiState}
                    setUIState={setUIState}
                    availableColumns={availableColumns}
                    trainingStatus={trainingStatus}
                    kernelStatus={kernelStatus}
                    onInitializeKernel={initializeKernel}
                    onStartTraining={startTraining}
                    onCancelTraining={cancelTraining}
                    onSaveModel={saveModel}
                    onLoadModel={loadModel}
                />
            }
        >
            <MLContent
                kernelStatus={kernelStatus}
                progress={progress}
                result={result}
                error={error}
                activeTab={uiState.active_tab}
                onTabChange={handleTabChange}
                availableColumns={availableColumns}
                onSelectHistory={handleHistorySelect}
            />
        </AppShell>
    );
}
