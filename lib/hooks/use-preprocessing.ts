"use client";

import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRustEvent } from "./use-rust-event";
import {
  RUST_EVENTS,
  DEFAULT_PIPELINE_CONFIG,
  type PreprocessingRequest,
  type PipelineConfigRequest,
  type PipelineResult,
  type ProgressUpdate,
  type PreprocessingSummary,
  type PreprocessingErrorPayload,
  type PreprocessingHistoryEntry,
} from "@/types";

// ============================================================================
// TYPES
// ============================================================================

/**
 * State of the preprocessing operation.
 */
export type PreprocessingStatus =
  | "idle"
  | "running"
  | "completed"
  | "cancelled"
  | "error";

/**
 * State returned by the usePreprocessing hook.
 */
export interface PreprocessingState {
  /** Current status of preprocessing */
  status: PreprocessingStatus;
  /** Whether preprocessing is currently running */
  isProcessing: boolean;
  /** Current progress update (null when idle) */
  progress: ProgressUpdate | null;
  /** Result from the last completed preprocessing run */
  result: PipelineResult | null;
  /** Summary from the last completed preprocessing run */
  summary: PreprocessingSummary | null;
  /** Error message if preprocessing failed */
  error: string | null;
  /** Error code if preprocessing failed */
  errorCode: string | null;
}

/**
 * Actions returned by the usePreprocessing hook.
 */
export interface PreprocessingActions {
  /**
   * Starts the preprocessing pipeline.
   *
   * @param selectedColumns - Columns to process (empty = all columns)
   * @param rowRange - Optional row range [start, end] to process
   * @param config - Pipeline configuration (uses defaults if not provided)
   * @returns Promise that resolves to the PipelineResult or rejects with error
   *
   * @example
   * ```tsx
   * // Process all columns with default config
   * await startPreprocessing([]);
   *
   * // Process specific columns
   * await startPreprocessing(["age", "income", "education"]);
   *
   * // Process with custom config
   * await startPreprocessing([], null, {
   *   ...DEFAULT_PIPELINE_CONFIG,
   *   outlier_strategy: "remove",
   *   use_ai_decisions: true,
   * });
   * ```
   */
  startPreprocessing: (
    selectedColumns: string[],
    rowRange?: [number, number] | null,
    config?: PipelineConfigRequest
  ) => Promise<PipelineResult>;

  /**
   * Cancels the currently running preprocessing pipeline.
   *
   * Cancellation is not immediate - the pipeline will stop at the next
   * checkpoint. The status will change to "cancelled" when it actually stops.
   */
  cancelPreprocessing: () => Promise<void>;

  /**
   * Resets the preprocessing state to idle.
   *
   * Call this to clear error states or prepare for a new run.
   */
  reset: () => void;

  /**
   * Gets the preprocessing history.
   *
   * @returns Promise that resolves to array of history entries (newest first)
   */
  getHistory: () => Promise<PreprocessingHistoryEntry[]>;

  /**
   * Clears all preprocessing history.
   */
  clearHistory: () => Promise<void>;
}

/**
 * Return type of the usePreprocessing hook.
 */
export type UsePreprocessingReturn = PreprocessingState & PreprocessingActions;

// ============================================================================
// HOOK IMPLEMENTATION
// ============================================================================

/**
 * Hook for managing preprocessing operations.
 *
 * This hook provides a complete interface for:
 * - Starting preprocessing with configuration
 * - Tracking real-time progress via events
 * - Cancelling running operations
 * - Accessing results and history
 *
 * @returns State and actions for preprocessing operations
 *
 * @example
 * ```tsx
 * function PreprocessingPanel() {
 *   const {
 *     status,
 *     isProcessing,
 *     progress,
 *     result,
 *     error,
 *     startPreprocessing,
 *     cancelPreprocessing,
 *     reset,
 *   } = usePreprocessing();
 *
 *   const handleStart = async () => {
 *     try {
 *       const result = await startPreprocessing(["col1", "col2"]);
 *       console.log("Preprocessing complete:", result.summary);
 *     } catch (err) {
 *       console.error("Preprocessing failed:", err);
 *     }
 *   };
 *
 *   return (
 *     <div>
 *       {isProcessing && (
 *         <div>
 *           <p>{progress?.message}</p>
 *           <progress value={progress?.progress} max={1} />
 *           <button onClick={cancelPreprocessing}>Cancel</button>
 *         </div>
 *       )}
 *       {status === "error" && <p className="error">{error}</p>}
 *       {status === "completed" && <p>Done! Quality improved by X%</p>}
 *       <button onClick={handleStart} disabled={isProcessing}>
 *         Start Processing
 *       </button>
 *     </div>
 *   );
 * }
 * ```
 *
 * @remarks
 * Following "Rust Supremacy", all actual preprocessing happens in Rust.
 * This hook only manages the IPC communication and local UI state.
 */
export function usePreprocessing(): UsePreprocessingReturn {
  // State
  const [status, setStatus] = useState<PreprocessingStatus>("idle");
  const [progress, setProgress] = useState<ProgressUpdate | null>(null);
  const [result, setResult] = useState<PipelineResult | null>(null);
  const [summary, setSummary] = useState<PreprocessingSummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [errorCode, setErrorCode] = useState<string | null>(null);

  // Track if we're processing to avoid state updates after unmount
  const isProcessingRef = useRef(false);

  // Derived state
  const isProcessing = status === "running";

  // ============================================================================
  // EVENT HANDLERS
  // ============================================================================

  /**
   * Handle progress updates from Rust.
   */
  const handleProgress = useCallback((update: ProgressUpdate) => {
    if (isProcessingRef.current) {
      setProgress(update);

      // Update status based on stage
      if (update.stage === "complete") {
        setStatus("completed");
        isProcessingRef.current = false;
      } else if (update.stage === "cancelled") {
        setStatus("cancelled");
        isProcessingRef.current = false;
      } else if (update.stage === "failed") {
        setStatus("error");
        isProcessingRef.current = false;
      }
    }
  }, []);

  /**
   * Handle preprocessing completion event.
   */
  const handleComplete = useCallback((completeSummary: PreprocessingSummary) => {
    setSummary(completeSummary);
    setStatus("completed");
    isProcessingRef.current = false;
  }, []);

  /**
   * Handle preprocessing error event.
   */
  const handleError = useCallback((payload: PreprocessingErrorPayload) => {
    setError(payload.message);
    setErrorCode(payload.code);
    setStatus("error");
    isProcessingRef.current = false;
  }, []);

  /**
   * Handle preprocessing cancelled event.
   */
  const handleCancelled = useCallback(() => {
    setStatus("cancelled");
    setError("Preprocessing was cancelled");
    isProcessingRef.current = false;
  }, []);

  // ============================================================================
  // EVENT SUBSCRIPTIONS
  // ============================================================================

  useRustEvent<ProgressUpdate>(
    RUST_EVENTS.PREPROCESSING_PROGRESS,
    handleProgress
  );

  useRustEvent<PreprocessingSummary>(
    RUST_EVENTS.PREPROCESSING_COMPLETE,
    handleComplete
  );

  useRustEvent<PreprocessingErrorPayload>(
    RUST_EVENTS.PREPROCESSING_ERROR,
    handleError
  );

  useRustEvent<null>(RUST_EVENTS.PREPROCESSING_CANCELLED, handleCancelled);

  // ============================================================================
  // ACTIONS
  // ============================================================================

  /**
   * Starts the preprocessing pipeline.
   */
  const startPreprocessing = useCallback(
    async (
      selectedColumns: string[],
      rowRange?: [number, number] | null,
      config?: PipelineConfigRequest
    ): Promise<PipelineResult> => {
      // Prevent starting if already running
      if (isProcessingRef.current) {
        throw new Error("Preprocessing is already running");
      }

      // Reset state
      setStatus("running");
      setProgress(null);
      setResult(null);
      setSummary(null);
      setError(null);
      setErrorCode(null);
      isProcessingRef.current = true;

      // Build request
      const request: PreprocessingRequest = {
        selected_columns: selectedColumns,
        row_range: rowRange ?? null,
        config: config ?? DEFAULT_PIPELINE_CONFIG,
      };

      try {
        // Call Rust command
        const pipelineResult = await invoke<PipelineResult>(
          "start_preprocessing",
          { request }
        );

        // Store result
        setResult(pipelineResult);

        // Summary is set via event, but also available in result
        if (pipelineResult.summary) {
          setSummary(pipelineResult.summary);
        }

        // Mark as completed (event might have already done this)
        if (pipelineResult.success) {
          setStatus("completed");
        } else {
          setStatus("error");
          setError(pipelineResult.error ?? "Unknown error");
        }

        isProcessingRef.current = false;
        return pipelineResult;
      } catch (err) {
        // Handle error from invoke (e.g., command not found, serialization error)
        const message = err instanceof Error ? err.message : String(err);
        setError(message);
        setStatus("error");
        isProcessingRef.current = false;
        throw err;
      }
    },
    []
  );

  /**
   * Cancels the currently running preprocessing.
   */
  const cancelPreprocessing = useCallback(async () => {
    if (!isProcessingRef.current) {
      return; // Nothing to cancel
    }

    try {
      await invoke("cancel_preprocessing");
      // Status will be updated by the cancelled event
    } catch (err) {
      console.error("Failed to cancel preprocessing:", err);
    }
  }, []);

  /**
   * Resets the preprocessing state to idle.
   */
  const reset = useCallback(() => {
    setStatus("idle");
    setProgress(null);
    setResult(null);
    setSummary(null);
    setError(null);
    setErrorCode(null);
    isProcessingRef.current = false;
  }, []);

  /**
   * Gets the preprocessing history.
   */
  const getHistory = useCallback(async (): Promise<
    PreprocessingHistoryEntry[]
  > => {
    try {
      return await invoke<PreprocessingHistoryEntry[]>(
        "get_preprocessing_history"
      );
    } catch (err) {
      console.error("Failed to get preprocessing history:", err);
      return [];
    }
  }, []);

  /**
   * Clears all preprocessing history.
   */
  const clearHistory = useCallback(async () => {
    try {
      await invoke("clear_preprocessing_history");
    } catch (err) {
      console.error("Failed to clear preprocessing history:", err);
    }
  }, []);

  // ============================================================================
  // CLEANUP
  // ============================================================================

  // Cancel preprocessing on unmount if still running
  useEffect(() => {
    return () => {
      if (isProcessingRef.current) {
        invoke("cancel_preprocessing").catch(() => {
          // Ignore errors on cleanup
        });
      }
    };
  }, []);

  // ============================================================================
  // RETURN
  // ============================================================================

  return {
    // State
    status,
    isProcessing,
    progress,
    result,
    summary,
    error,
    errorCode,

    // Actions
    startPreprocessing,
    cancelPreprocessing,
    reset,
    getHistory,
    clearHistory,
  };
}
