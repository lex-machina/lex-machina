"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { FileInfo, MLUIState } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface UseMLUIStateReturn {
    isLoaded: boolean;
    uiState: MLUIState;
    setUIState: (state: MLUIState) => void;
}

export const defaultMLUIState: MLUIState = {
    smart_mode: true,
    target_column: null,
    problem_type: "classification",
    excluded_columns: [],
    use_processed_data: false,
    config: {
        optimize_hyperparams: true,
        n_trials: 10,
        cv_folds: 5,
        test_size: 0.2,
        enable_neural_networks: false,
        enable_explainability: true,
        top_k_algorithms: 3,
        algorithm: undefined,
    },
    active_tab: "overview",
};

// ============================================================================
// HOOK
// ============================================================================

export function useMLUIState(fileInfo: FileInfo | null): UseMLUIStateReturn {
    const [isLoaded, setIsLoaded] = useState(false);
    const [uiState, setUIState] = useState<MLUIState>(defaultMLUIState);

    const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    useEffect(() => {
        if (isLoaded) {
            return;
        }

        async function loadPersistedState() {
            try {
                const savedState = await invoke<MLUIState>("get_ml_ui_state");
                if (savedState) {
                    setUIState({
                        ...savedState,
                        problem_type:
                            savedState.problem_type || "classification",
                    });
                }
                setIsLoaded(true);
            } catch (err) {
                console.warn("Failed to load ML UI state:", err);
                setIsLoaded(true);
            }
        }

        loadPersistedState();
    }, [isLoaded]);

    useEffect(() => {
        if (!isLoaded) {
            return;
        }

        if (saveTimeoutRef.current) {
            clearTimeout(saveTimeoutRef.current);
        }

        saveTimeoutRef.current = setTimeout(() => {
            invoke("set_ml_ui_state", { uiState }).catch((err) => {
                console.warn("Failed to save ML UI state:", err);
            });
        }, 300);

        return () => {
            if (saveTimeoutRef.current) {
                clearTimeout(saveTimeoutRef.current);
            }
        };
    }, [uiState, isLoaded]);

    const pruneExcludedColumns = useCallback(() => {
        if (!fileInfo) {
            return;
        }

        setUIState((current) => ({
            ...current,
            excluded_columns: current.excluded_columns.filter((column) =>
                fileInfo.columns.some((info) => info.name === column),
            ),
        }));
    }, [fileInfo]);

    useEffect(() => {
        if (!fileInfo) {
            return;
        }

        queueMicrotask(() => {
            pruneExcludedColumns();
        });
    }, [fileInfo, pruneExcludedColumns]);

    const setState = useCallback((state: MLUIState) => {
        setUIState(state);
    }, []);

    return {
        isLoaded,
        uiState,
        setUIState: setState,
    };
}
