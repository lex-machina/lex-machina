import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

const formatBytes = (bytes: number): string => {
    if (bytes === 0) {
        return "0 B";
    }
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
};

const formatNumber = (num: number): string => {
    return num.toLocaleString();
};

/**
 * Format a percentage (0-1 scale) as a human-readable string.
 */
const formatPercent = (value: number): string => {
    return `${Math.round(value * 100)}%`;
};

/**
 * Format duration in human-readable format.
 */
const formatDuration = (ms: number): string => {
    if (ms < 1000) return `${ms}ms`;
    const seconds = ms / 1000;
    if (seconds < 60) return `${seconds.toFixed(1)}s`;
    const minutes = Math.floor(seconds / 60);
    return `${minutes}m ${Math.round(seconds % 60)}s`;
};

/**
 * Opens a URL in the system's default browser.
 * Use this for external links in the Tauri desktop app.
 *
 * @param url - The URL to open (must be http://, https://, mailto:, or tel:)
 */
const openExternalUrl = async (url: string): Promise<void> => {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await openUrl(url);
};

export {
    formatBytes,
    formatNumber,
    formatPercent,
    formatDuration,
    openExternalUrl,
};
