"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRustEvent } from "./use-rust-event";
import {
    RUST_EVENTS,
    type BatchPredictionResult,
    type MLErrorPayload,
    type MLKernelStatus,
    type MLKernelStatusPayload,
    type MLProgressUpdate,
    type MLTrainingStatus,
    type MLUIState,
    type MLConfigRequest,
    type MLCompletePayload,
    type ModelInfo,
    type PredictionResult,
    type TrainingHistoryEntry,
    type TrainingResultResponse,
} from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface MLState {
    kernelStatus: MLKernelStatus;
    trainingStatus: MLTrainingStatus;
    progress: MLProgressUpdate | null;
    result: TrainingResultResponse | null;
    error: string | null;
    errorCode: string | null;
    modelInfo: ModelInfo | null;
    history: TrainingHistoryEntry[];
}

export interface MLActions {
    initializeKernel: () => Promise<void>;
    startTraining: (config: MLConfigRequest) => Promise<TrainingResultResponse>;
    cancelTraining: () => Promise<void>;
    getTrainingResult: () => Promise<TrainingResultResponse | null>;
    getModelInfo: () => Promise<ModelInfo | null>;
    saveModel: () => Promise<string>;
    loadModel: () => Promise<ModelInfo>;
    getSHAPPlot: (name: string) => Promise<string>;
    predictSingle: (
        instance: Record<string, unknown>,
    ) => Promise<PredictionResult>;
    predictBatch: () => Promise<BatchPredictionResult>;
    predictBatchFromCsv: (path: string) => Promise<BatchPredictionResult>;
    getHistory: () => Promise<TrainingHistoryEntry[]>;
    clearHistory: () => Promise<void>;
    refreshKernelStatus: () => Promise<MLKernelStatus>;
    loadUIState: () => Promise<MLUIState>;
    saveUIState: (uiState: MLUIState) => Promise<void>;
}

export type UseMLReturn = MLState & MLActions;

// ============================================================================
// HOOK
// ============================================================================

export function useML(): UseMLReturn {
    const [kernelStatus, setKernelStatus] =
        useState<MLKernelStatus>("uninitialized");
    const [trainingStatus, setTrainingStatus] =
        useState<MLTrainingStatus>("idle");
    const [progress, setProgress] = useState<MLProgressUpdate | null>(null);
    const [result, setResult] = useState<TrainingResultResponse | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [errorCode, setErrorCode] = useState<string | null>(null);
    const [modelInfo, setModelInfo] = useState<ModelInfo | null>(null);
    const [history, setHistory] = useState<TrainingHistoryEntry[]>([]);

    const isTrainingRef = useRef(false);

    const handleProgress = useCallback((update: MLProgressUpdate) => {
        if (!isTrainingRef.current) {
            return;
        }

        setProgress(update);
    }, []);

    const handleComplete = useCallback((payload: MLCompletePayload) => {
        setTrainingStatus("completed");
        isTrainingRef.current = false;
        setResult((current) => {
            if (
                current &&
                current.best_model_name === payload.best_model_name
            ) {
                return current;
            }
            return current;
        });
        invoke<TrainingResultResponse>("get_training_result")
            .then((res) => {
                setResult(res);
            })
            .catch(() => {
                // ignore
            });
    }, []);

    const handleError = useCallback((payload: MLErrorPayload) => {
        setError(payload.message);
        setErrorCode(payload.code);
        setTrainingStatus("error");
        isTrainingRef.current = false;
    }, []);

    const handleCancelled = useCallback(() => {
        setError("Training was cancelled");
        setTrainingStatus("cancelled");
        isTrainingRef.current = false;
    }, []);

    const handleKernelStatus = useCallback((payload: MLKernelStatusPayload) => {
        setKernelStatus(payload.status);
        if (payload.status === "error" && payload.message) {
            setError(payload.message);
        }
    }, []);

    useRustEvent<MLProgressUpdate>(RUST_EVENTS.ML_PROGRESS, handleProgress);
    useRustEvent<MLCompletePayload>(RUST_EVENTS.ML_COMPLETE, handleComplete);
    useRustEvent<MLErrorPayload>(RUST_EVENTS.ML_ERROR, handleError);
    useRustEvent<null>(RUST_EVENTS.ML_CANCELLED, handleCancelled);
    useRustEvent<MLKernelStatusPayload>(
        RUST_EVENTS.ML_KERNEL_STATUS,
        handleKernelStatus,
    );

    const initializeKernel = useCallback(async () => {
        setKernelStatus("initializing");
        setError(null);
        setErrorCode(null);
        await invoke("initialize_ml");
        setKernelStatus("ready");
    }, []);

    const refreshKernelStatus = useCallback(async () => {
        const initialized = await invoke<boolean>("is_ml_initialized");
        const status: MLKernelStatus = initialized ? "ready" : "uninitialized";
        setKernelStatus(status);
        return status;
    }, []);

    const startTraining = useCallback(async (config: MLConfigRequest) => {
        if (isTrainingRef.current) {
            throw new Error("Training is already in progress");
        }

        setTrainingStatus("training");
        setProgress(null);
        setResult(null);
        setError(null);
        setErrorCode(null);
        isTrainingRef.current = true;

        try {
            const trainingResult = await invoke<TrainingResultResponse>(
                "start_training",
                {
                    request: config,
                },
            );
            setResult(trainingResult);
            setTrainingStatus("completed");
            isTrainingRef.current = false;
            return trainingResult;
        } catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            setError(message);
            setTrainingStatus("error");
            isTrainingRef.current = false;
            throw err;
        }
    }, []);

    const cancelTraining = useCallback(async () => {
        if (!isTrainingRef.current) {
            return;
        }

        await invoke("cancel_training");
    }, []);

    const getTrainingResult = useCallback(async () => {
        try {
            const trainingResult = await invoke<TrainingResultResponse>(
                "get_training_result",
            );
            setResult(trainingResult);
            return trainingResult;
        } catch {
            return null;
        }
    }, []);

    const getModelInfo = useCallback(async () => {
        try {
            const info = await invoke<ModelInfo>("get_model_info");
            setModelInfo(info);
            return info;
        } catch {
            return null;
        }
    }, []);

    const saveModel = useCallback(async () => {
        return await invoke<string>("save_model");
    }, []);

    const loadModel = useCallback(async () => {
        const info = await invoke<ModelInfo>("load_model");
        setModelInfo(info);
        setResult(null);
        return info;
    }, []);

    const getSHAPPlot = useCallback(async (name: string) => {
        return await invoke<string>("get_shap_plot", { name });
    }, []);

    const predictSingle = useCallback(
        async (instance: Record<string, unknown>) => {
            return await invoke<PredictionResult>("predict_single", {
                instance,
            });
        },
        [],
    );

    const predictBatch = useCallback(async () => {
        return await invoke<BatchPredictionResult>("predict_batch");
    }, []);

    const predictBatchFromCsv = useCallback(async (path: string) => {
        return await invoke<BatchPredictionResult>("predict_batch_from_csv", {
            path,
        });
    }, []);

    const getHistory = useCallback(async () => {
        const entries = await invoke<TrainingHistoryEntry[]>(
            "get_training_history",
        );
        setHistory(entries);
        return entries;
    }, []);

    const clearHistory = useCallback(async () => {
        await invoke("clear_training_history");
        setHistory([]);
    }, []);

    const loadUIState = useCallback(async () => {
        return await invoke<MLUIState>("get_ml_ui_state");
    }, []);

    const saveUIState = useCallback(async (uiState: MLUIState) => {
        await invoke("set_ml_ui_state", { uiState });
    }, []);

    useEffect(() => {
        queueMicrotask(() => {
            refreshKernelStatus().catch(() => {
                // ignore
            });
        });
    }, [refreshKernelStatus]);

    return {
        kernelStatus,
        trainingStatus,
        progress,
        result,
        error,
        errorCode,
        modelInfo,
        history,
        initializeKernel,
        startTraining,
        cancelTraining,
        getTrainingResult,
        getModelInfo,
        saveModel,
        loadModel,
        getSHAPPlot,
        predictSingle,
        predictBatch,
        predictBatchFromCsv,
        getHistory,
        clearHistory,
        refreshKernelStatus,
        loadUIState,
        saveUIState,
    };
}
