"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { VisualizationsUIState } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface UseVisualizationsUIStateReturn {
    isLoaded: boolean;
    uiState: VisualizationsUIState;
    setUIState: (state: VisualizationsUIState) => void;
}

export const defaultVisualizationsUIState: VisualizationsUIState = {
    use_processed_data: false,
    chart_overrides: {},
};

// ============================================================================
// HOOK
// ============================================================================

export function useVisualizationsUIState(): UseVisualizationsUIStateReturn {
    const [isLoaded, setIsLoaded] = useState(false);
    const [uiState, setUIState] = useState<VisualizationsUIState>(
        defaultVisualizationsUIState,
    );
    const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    useEffect(() => {
        if (isLoaded) {
            return;
        }

        async function loadPersistedState() {
            try {
                const savedState = await invoke<VisualizationsUIState>(
                    "get_visualizations_ui_state",
                );
                if (savedState) {
                    setUIState({
                        ...defaultVisualizationsUIState,
                        ...savedState,
                        chart_overrides: savedState.chart_overrides ?? {},
                    });
                }
                setIsLoaded(true);
            } catch (err) {
                console.warn("Failed to load visualizations UI state:", err);
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
            invoke("set_visualizations_ui_state", { uiState }).catch((err) => {
                console.warn("Failed to save visualizations UI state:", err);
            });
        }, 300);

        return () => {
            if (saveTimeoutRef.current) {
                clearTimeout(saveTimeoutRef.current);
            }
        };
    }, [uiState, isLoaded]);

    const setState = useCallback((state: VisualizationsUIState) => {
        setUIState(state);
    }, []);

    return {
        isLoaded,
        uiState,
        setUIState: setState,
    };
}
