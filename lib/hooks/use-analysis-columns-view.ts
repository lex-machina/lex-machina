"use client";

import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { AnalysisColumnFilter, AnalysisColumnListResponse } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface UseAnalysisColumnsViewReturn {
    isLoading: boolean;
    response: AnalysisColumnListResponse | null;
    error: string | null;
}

// ============================================================================
// HOOK
// ============================================================================

export function useAnalysisColumnsView(
    useProcessedData: boolean,
    filter: AnalysisColumnFilter,
): UseAnalysisColumnsViewReturn {
    const [response, setResponse] = useState<AnalysisColumnListResponse | null>(
        null,
    );
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const filterKey = useMemo(() => JSON.stringify(filter), [filter]);

    useEffect(() => {
        let isActive = true;
        setIsLoading(true);
        setError(null);

        const timeout = setTimeout(
            () => {
                invoke<AnalysisColumnListResponse | null>(
                    "get_analysis_columns_view",
                    {
                        useProcessedData,
                        filter,
                    },
                )
                    .then((data) => {
                        if (!isActive) return;
                        setResponse(data ?? null);
                    })
                    .catch((err) => {
                        if (!isActive) return;
                        setError(
                            err instanceof Error ? err.message : String(err),
                        );
                    })
                    .finally(() => {
                        if (!isActive) return;
                        setIsLoading(false);
                    });
            },
            filter.search ? 200 : 0,
        );

        return () => {
            isActive = false;
            clearTimeout(timeout);
        };
    }, [useProcessedData, filterKey]);

    return {
        isLoading,
        response,
        error,
    };
}
