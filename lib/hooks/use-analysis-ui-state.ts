"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { AnalysisUIState, ColumnInfo } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface UseAnalysisUIStateReturn {
    isLoaded: boolean;
    uiState: AnalysisUIState;
    setUIState: (state: AnalysisUIState) => void;
}

export const defaultAnalysisUIState: AnalysisUIState = {
    use_processed_data: false,
    active_tab: "overview",
    selected_column: null,
};

// ============================================================================
// HOOK
// ============================================================================

export function useAnalysisUIState(
    availableColumns: ColumnInfo[] = [],
): UseAnalysisUIStateReturn {
    const [isLoaded, setIsLoaded] = useState(false);
    const [uiState, setUIState] = useState<AnalysisUIState>(
        defaultAnalysisUIState,
    );
    const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    useEffect(() => {
        if (isLoaded) {
            return;
        }

        async function loadPersistedState() {
            try {
                const savedState = await invoke<AnalysisUIState>(
                    "get_analysis_ui_state",
                );
                if (savedState) {
                    setUIState(savedState);
                }
                setIsLoaded(true);
            } catch (err) {
                console.warn("Failed to load analysis UI state:", err);
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
            invoke("set_analysis_ui_state", { uiState }).catch((err) => {
                console.warn("Failed to save analysis UI state:", err);
            });
        }, 300);

        return () => {
            if (saveTimeoutRef.current) {
                clearTimeout(saveTimeoutRef.current);
            }
        };
    }, [uiState, isLoaded]);

    useEffect(() => {
        if (!uiState.selected_column) {
            return;
        }

        const exists = availableColumns.some(
            (column) => column.name === uiState.selected_column,
        );
        if (!exists) {
            setUIState((current) => ({
                ...current,
                selected_column: null,
            }));
        }
    }, [availableColumns, uiState.selected_column]);

    const setState = useCallback((state: AnalysisUIState) => {
        setUIState(state);
    }, []);

    return {
        isLoaded,
        uiState,
        setUIState: setState,
    };
}
