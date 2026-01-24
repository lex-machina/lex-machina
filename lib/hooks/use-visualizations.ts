"use client";

import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { VisualizationDashboard } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export type VisualizationsStatus = "idle" | "running" | "completed" | "error";

export interface VisualizationsState {
    status: VisualizationsStatus;
    result: VisualizationDashboard | null;
    error: string | null;
}

export interface VisualizationsActions {
    runVisualizations: (
        useProcessedData: boolean,
    ) => Promise<VisualizationDashboard>;
    loadCached: (
        useProcessedData: boolean,
    ) => Promise<VisualizationDashboard | null>;
    clear: () => Promise<void>;
}

export type UseVisualizationsReturn = VisualizationsState &
    VisualizationsActions;

// ============================================================================
// HOOK
// ============================================================================

export function useVisualizations(): UseVisualizationsReturn {
    const [status, setStatus] = useState<VisualizationsStatus>("idle");
    const [result, setResult] = useState<VisualizationDashboard | null>(null);
    const [error, setError] = useState<string | null>(null);

    const runVisualizations = useCallback(async (useProcessedData: boolean) => {
        setStatus("running");
        setError(null);
        try {
            const dashboard = await invoke<VisualizationDashboard>(
                "run_visualizations",
                {
                    useProcessedData,
                },
            );
            setResult(dashboard);
            setStatus("completed");
            return dashboard;
        } catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            setError(message);
            setStatus("error");
            throw err;
        }
    }, []);

    const loadCached = useCallback(async (useProcessedData: boolean) => {
        try {
            const cached = await invoke<VisualizationDashboard | null>(
                "get_visualizations_result",
                {
                    useProcessedData,
                },
            );
            setResult(cached);
            setStatus(cached ? "completed" : "idle");
            return cached;
        } catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            setError(message);
            setStatus("error");
            return null;
        }
    }, []);

    const clear = useCallback(async () => {
        await invoke("clear_visualizations_results");
        setResult(null);
        setStatus("idle");
        setError(null);
    }, []);

    return {
        status,
        result,
        error,
        runVisualizations,
        loadCached,
        clear,
    };
}
