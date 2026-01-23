"use client";

import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { AnalysisExportResult, AnalysisResult } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export type AnalysisStatus = "idle" | "running" | "completed" | "error";

export interface AnalysisState {
    status: AnalysisStatus;
    result: AnalysisResult | null;
    error: string | null;
}

export interface AnalysisActions {
    runAnalysis: (useProcessedData: boolean) => Promise<AnalysisResult>;
    loadCached: (useProcessedData: boolean) => Promise<AnalysisResult | null>;
    exportReport: (useProcessedData: boolean) => Promise<AnalysisExportResult>;
    clear: () => Promise<void>;
}

export type UseAnalysisReturn = AnalysisState & AnalysisActions;

// ============================================================================
// HOOK
// ============================================================================

export function useAnalysis(): UseAnalysisReturn {
    const [status, setStatus] = useState<AnalysisStatus>("idle");
    const [result, setResult] = useState<AnalysisResult | null>(null);
    const [error, setError] = useState<string | null>(null);

    const runAnalysis = useCallback(async (useProcessedData: boolean) => {
        setStatus("running");
        setError(null);
        try {
            const analysis = await invoke<AnalysisResult>("run_analysis", {
                useProcessedData,
            });
            setResult(analysis);
            setStatus("completed");
            return analysis;
        } catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            setError(message);
            setStatus("error");
            throw err;
        }
    }, []);

    const loadCached = useCallback(async (useProcessedData: boolean) => {
        try {
            const cached = await invoke<AnalysisResult | null>(
                "get_analysis_result",
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

    const exportReport = useCallback(async (useProcessedData: boolean) => {
        return invoke<AnalysisExportResult>("export_analysis_report", {
            useProcessedData,
        });
    }, []);

    const clear = useCallback(async () => {
        await invoke("clear_analysis_results");
        setResult(null);
        setStatus("idle");
        setError(null);
    }, []);

    return {
        status,
        result,
        error,
        runAnalysis,
        loadCached,
        exportReport,
        clear,
    };
}
