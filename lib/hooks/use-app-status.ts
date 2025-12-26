"use client";

import { useState, useCallback, useRef, useEffect } from "react";
import { useRustEvent } from "./use-rust-event";
import {
  RUST_EVENTS,
  type LoadingPayload,
  type ErrorPayload,
} from "@/types";

/**
 * Error state with timestamp for display purposes.
 */
export interface AppError {
  /** Error code for programmatic handling */
  code: string;
  /** Human-readable error message */
  message: string;
  /** Timestamp when the error occurred */
  timestamp: number;
}

/**
 * State returned by the useAppStatus hook.
 */
export interface AppStatus {
  /** Whether a loading operation is in progress */
  isLoading: boolean;
  /** Optional loading message describing the current operation */
  loadingMessage: string | null;
  /** Most recent error, or null if no error */
  error: AppError | null;
  /** Clear the current error */
  clearError: () => void;
}

/**
 * Hook for tracking application status from Rust backend events.
 *
 * This hook subscribes to `app:loading` and `app:error` events from Rust
 * and maintains the current loading and error state. Components using this
 * hook can display loading indicators and error notifications.
 *
 * @param options - Optional configuration
 * @param options.autoClearErrorMs - Auto-clear error after this many milliseconds (default: no auto-clear)
 *
 * @returns Current app status including loading state, error, and clearError function
 *
 * @example
 * ```tsx
 * function StatusIndicator() {
 *   const { isLoading, loadingMessage, error, clearError } = useAppStatus({
 *     autoClearErrorMs: 5000, // Auto-clear errors after 5 seconds
 *   });
 *
 *   return (
 *     <div>
 *       {isLoading && <Spinner message={loadingMessage} />}
 *       {error && (
 *         <ErrorBanner
 *           message={error.message}
 *           onDismiss={clearError}
 *         />
 *       )}
 *     </div>
 *   );
 * }
 * ```
 *
 * @remarks
 * Following "Rust Supremacy", this hook is purely reactive - it only listens
 * to events from Rust and never initiates state changes. Errors and loading
 * states are pushed from Rust commands.
 */
export function useAppStatus(options?: {
  autoClearErrorMs?: number;
}): AppStatus {
  const [isLoading, setIsLoading] = useState(false);
  const [loadingMessage, setLoadingMessage] = useState<string | null>(null);
  const [error, setError] = useState<AppError | null>(null);

  // Track auto-clear timeout
  const errorTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Handle loading state change
  const handleLoading = useCallback((payload: LoadingPayload) => {
    setIsLoading(payload.is_loading);
    setLoadingMessage(payload.message);
  }, []);

  // Extract the auto-clear value to satisfy React Compiler's dependency inference
  const autoClearErrorMs = options?.autoClearErrorMs;

  // Handle error event
  const handleError = useCallback(
    (payload: ErrorPayload) => {
      const appError: AppError = {
        code: payload.code,
        message: payload.message,
        timestamp: Date.now(),
      };
      setError(appError);

      // Auto-clear error if configured
      if (autoClearErrorMs) {
        // Clear any existing timeout
        if (errorTimeoutRef.current) {
          clearTimeout(errorTimeoutRef.current);
        }
        errorTimeoutRef.current = setTimeout(() => {
          setError(null);
        }, autoClearErrorMs);
      }
    },
    [autoClearErrorMs]
  );

  // Clear error manually
  const clearError = useCallback(() => {
    setError(null);
    if (errorTimeoutRef.current) {
      clearTimeout(errorTimeoutRef.current);
      errorTimeoutRef.current = null;
    }
  }, []);

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (errorTimeoutRef.current) {
        clearTimeout(errorTimeoutRef.current);
      }
    };
  }, []);

  // Subscribe to events
  useRustEvent<LoadingPayload>(RUST_EVENTS.LOADING, handleLoading);
  useRustEvent<ErrorPayload>(RUST_EVENTS.ERROR, handleError);

  return {
    isLoading,
    loadingMessage,
    error,
    clearError,
  };
}
