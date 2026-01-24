"use client";

import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { AnalysisHeatmapKind, HeatmapMatrixView } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface UseAnalysisHeatmapReturn {
    view: HeatmapMatrixView | null;
    isLoading: boolean;
    error: string | null;
}

// ============================================================================
// HOOK
// ============================================================================

export function useAnalysisHeatmap(
    useProcessedData: boolean,
    kind: AnalysisHeatmapKind,
    maxColumns?: number,
): UseAnalysisHeatmapReturn {
    const [view, setView] = useState<HeatmapMatrixView | null>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const key = useMemo(
        () => `${kind}:${maxColumns ?? "default"}`,
        [kind, maxColumns],
    );

    useEffect(() => {
        let isActive = true;
        setIsLoading(true);
        setError(null);

        invoke<HeatmapMatrixView | null>("get_analysis_heatmap_view", {
            useProcessedData,
            kind,
            maxColumns,
        })
            .then((data) => {
                if (!isActive) return;
                setView(data ?? null);
            })
            .catch((err) => {
                if (!isActive) return;
                setError(err instanceof Error ? err.message : String(err));
            })
            .finally(() => {
                if (!isActive) return;
                setIsLoading(false);
            });

        return () => {
            isActive = false;
        };
    }, [useProcessedData, key, kind, maxColumns]);

    return {
        view,
        isLoading,
        error,
    };
}
