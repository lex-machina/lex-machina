"use client";

import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRustEvent } from "./use-rust-event";
import { RUST_EVENTS, type FileInfo, type FileLoadedPayload } from "@/types";

/**
 * State returned by the useFileState hook.
 */
export interface FileState {
    /** Currently loaded file info, or null if no file is loaded */
    fileInfo: FileInfo | null;
    /** Whether a file is currently loaded */
    isFileLoaded: boolean;
    /** Column widths derived from file info */
    columnWidths: number[];
}

/**
 * Hook for tracking file state from Rust backend.
 *
 * This hook:
 * 1. Queries Rust for current file state on mount (handles navigation/missed events)
 * 2. Subscribes to `file:loaded` and `file:closed` events for subsequent updates
 *
 * Components using this hook will automatically re-render when file state changes.
 *
 * @returns Current file state including fileInfo, isFileLoaded, and columnWidths
 *
 * @example
 * ```tsx
 * function FileInfoPanel() {
 *   const { fileInfo, isFileLoaded, columnWidths } = useFileState();
 *
 *   if (!isFileLoaded) {
 *     return <div>No file loaded</div>;
 *   }
 *
 *   return (
 *     <div>
 *       <h2>{fileInfo.name}</h2>
 *       <p>{fileInfo.row_count} rows</p>
 *     </div>
 *   );
 * }
 * ```
 *
 * @remarks
 * Following "Rust Supremacy", this hook only reads state from Rust.
 * To load/close files, use Tauri commands directly (invoke("load_file"), invoke("close_file")).
 */
export function useFileState(): FileState {
    const [fileInfo, setFileInfo] = useState<FileInfo | null>(null);

    // Query Rust for current file state on mount
    // This ensures we sync with Rust state when component mounts
    // (e.g., after navigation or if event was missed)
    useEffect(() => {
        const fetchInitialState = async () => {
            try {
                const info = await invoke<FileInfo | null>("get_file_info");
                if (info) {
                    setFileInfo(info);
                }
            } catch (err) {
                console.error("Failed to get initial file state:", err);
            }
        };
        fetchInitialState();
    }, []);

    // Handle file loaded event
    const handleFileLoaded = useCallback((payload: FileLoadedPayload) => {
        setFileInfo(payload.file_info);
    }, []);

    // Handle file closed event
    const handleFileClosed = useCallback(() => {
        setFileInfo(null);
    }, []);

    // Subscribe to file events
    useRustEvent<FileLoadedPayload>(RUST_EVENTS.FILE_LOADED, handleFileLoaded);
    useRustEvent<null>(RUST_EVENTS.FILE_CLOSED, handleFileClosed);

    // Derive computed values
    const isFileLoaded = fileInfo !== null;
    const columnWidths = fileInfo?.columns.map((col) => col.width) ?? [];

    return {
        fileInfo,
        isFileLoaded,
        columnWidths,
    };
}
