"use client";

import { useMemo } from "react";
import { cn } from "@/lib/utils";
import type { FileInfo, ColumnInfo } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface DatasetPreviewProps {
    /** File info from the loaded dataset */
    fileInfo: FileInfo | null;
    /** Additional class names */
    className?: string;
}

// ============================================================================
// HELPERS
// ============================================================================

/**
 * Format file size in human-readable format.
 */
function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024)
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

/**
 * Format a number with locale-aware formatting.
 */
function formatNumber(num: number): string {
    return num.toLocaleString();
}

/**
 * Get dtype category for grouping.
 */
function getDtypeCategory(
    dtype: string,
): "numeric" | "string" | "boolean" | "datetime" | "other" {
    const lower = dtype.toLowerCase();
    if (
        lower.includes("int") ||
        lower.includes("float") ||
        lower.includes("f64") ||
        lower.includes("f32")
    ) {
        return "numeric";
    }
    if (
        lower.includes("str") ||
        lower.includes("utf8") ||
        lower.includes("string")
    ) {
        return "string";
    }
    if (lower.includes("bool")) {
        return "boolean";
    }
    if (lower.includes("date") || lower.includes("time")) {
        return "datetime";
    }
    return "other";
}

/**
 * Get category color class.
 */
function getCategoryColorClass(
    category: "numeric" | "string" | "boolean" | "datetime" | "other",
): string {
    switch (category) {
        case "numeric":
            return "bg-blue-500/20 text-blue-400";
        case "string":
            return "bg-green-500/20 text-green-400";
        case "boolean":
            return "bg-purple-500/20 text-purple-400";
        case "datetime":
            return "bg-orange-500/20 text-orange-400";
        default:
            return "bg-muted text-muted-foreground";
    }
}

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

interface StatItemProps {
    label: string;
    value: string | number;
    subtext?: string;
}

function StatItem({ label, value, subtext }: StatItemProps) {
    return (
        <div className="flex flex-col gap-0.5">
            <span className="text-muted-foreground text-xs">{label}</span>
            <span className="text-lg font-semibold tabular-nums">
                {typeof value === "number" ? formatNumber(value) : value}
            </span>
            {subtext && (
                <span className="text-muted-foreground text-xs">{subtext}</span>
            )}
        </div>
    );
}

interface ColumnTypeSummaryProps {
    columns: ColumnInfo[];
}

function ColumnTypeSummary({ columns }: ColumnTypeSummaryProps) {
    const summary = useMemo(() => {
        const counts = {
            numeric: 0,
            string: 0,
            boolean: 0,
            datetime: 0,
            other: 0,
        };

        for (const col of columns) {
            const category = getDtypeCategory(col.dtype);
            counts[category]++;
        }

        return counts;
    }, [columns]);

    const categories = [
        { key: "numeric", label: "Numeric", count: summary.numeric },
        { key: "string", label: "String", count: summary.string },
        { key: "boolean", label: "Boolean", count: summary.boolean },
        { key: "datetime", label: "Date/Time", count: summary.datetime },
        { key: "other", label: "Other", count: summary.other },
    ].filter((c) => c.count > 0);

    return (
        <div className="flex flex-wrap gap-2">
            {categories.map((cat) => (
                <span
                    key={cat.key}
                    className={cn(
                        "rounded px-2 py-1 text-xs",
                        getCategoryColorClass(
                            cat.key as
                                | "numeric"
                                | "string"
                                | "boolean"
                                | "datetime"
                                | "other",
                        ),
                    )}
                >
                    {cat.count} {cat.label}
                </span>
            ))}
        </div>
    );
}

interface MissingValuesSummaryProps {
    columns: ColumnInfo[];
    totalRows: number;
}

function MissingValuesSummary({
    columns,
    totalRows,
}: MissingValuesSummaryProps) {
    const stats = useMemo(() => {
        let totalMissing = 0;
        let columnsWithMissing = 0;

        for (const col of columns) {
            if (col.null_count > 0) {
                totalMissing += col.null_count;
                columnsWithMissing++;
            }
        }

        const totalCells = totalRows * columns.length;
        const missingPercent =
            totalCells > 0 ? (totalMissing / totalCells) * 100 : 0;

        return {
            totalMissing,
            columnsWithMissing,
            missingPercent,
        };
    }, [columns, totalRows]);

    if (stats.totalMissing === 0) {
        return (
            <div className="flex items-center gap-2 text-sm text-green-500">
                <svg
                    className="h-4 w-4"
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                >
                    <path
                        fillRule="evenodd"
                        d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                        clipRule="evenodd"
                    />
                </svg>
                No missing values detected
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-2">
            <div className="flex items-center gap-2 text-sm text-yellow-500">
                <svg
                    className="h-4 w-4"
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                >
                    <path
                        fillRule="evenodd"
                        d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                        clipRule="evenodd"
                    />
                </svg>
                Missing values detected
            </div>
            <div className="grid grid-cols-3 gap-4 text-xs">
                <div className="flex flex-col">
                    <span className="text-muted-foreground">Total missing</span>
                    <span className="font-medium tabular-nums">
                        {formatNumber(stats.totalMissing)}
                    </span>
                </div>
                <div className="flex flex-col">
                    <span className="text-muted-foreground">
                        Columns affected
                    </span>
                    <span className="font-medium tabular-nums">
                        {stats.columnsWithMissing}
                    </span>
                </div>
                <div className="flex flex-col">
                    <span className="text-muted-foreground">
                        Dataset coverage
                    </span>
                    <span className="font-medium tabular-nums">
                        {stats.missingPercent.toFixed(1)}%
                    </span>
                </div>
            </div>
        </div>
    );
}

// ============================================================================
// DATASET PREVIEW COMPONENT
// ============================================================================

/**
 * A preview component showing dataset summary before preprocessing.
 *
 * Displays file info, row/column counts, column type distribution,
 * and missing value statistics to give users context.
 *
 * @example
 * ```tsx
 * const { fileInfo } = useFileState();
 *
 * <DatasetPreview fileInfo={fileInfo} />
 * ```
 */
export function DatasetPreview({ fileInfo, className }: DatasetPreviewProps) {
    // Empty state
    if (!fileInfo) {
        return (
            <div
                className={cn(
                    "flex flex-col items-center justify-center p-8",
                    "border-border rounded-lg border border-dashed",
                    "text-muted-foreground",
                    className,
                )}
            >
                <svg
                    className="mb-3 h-10 w-10 opacity-50"
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                >
                    <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={1.5}
                        d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                    />
                </svg>
                <p className="text-sm font-medium">No dataset loaded</p>
                <p className="mt-1 text-xs">
                    Load a CSV file to start preprocessing
                </p>
            </div>
        );
    }

    return (
        <div
            className={cn(
                "border-border bg-card flex flex-col gap-4 rounded-lg border p-4",
                className,
            )}
            data-slot="dataset-preview"
        >
            {/* File info header */}
            <div className="flex items-start justify-between">
                <div className="flex min-w-0 flex-col gap-1">
                    <h3
                        className="truncate text-sm font-medium"
                        title={fileInfo.name}
                    >
                        {fileInfo.name}
                    </h3>
                    <p
                        className="text-muted-foreground truncate text-xs"
                        title={fileInfo.path}
                    >
                        {fileInfo.path}
                    </p>
                </div>
                <span className="text-muted-foreground ml-2 shrink-0 text-xs">
                    {formatFileSize(fileInfo.size_bytes)}
                </span>
            </div>

            {/* Stats row */}
            <div className="border-border grid grid-cols-3 gap-4 border-y py-3">
                <StatItem label="Rows" value={fileInfo.row_count} />
                <StatItem label="Columns" value={fileInfo.column_count} />
                <StatItem
                    label="Cells"
                    value={fileInfo.row_count * fileInfo.column_count}
                />
            </div>

            {/* Column types */}
            <div className="flex flex-col gap-2">
                <span className="text-muted-foreground text-xs font-medium">
                    Column Types
                </span>
                <ColumnTypeSummary columns={fileInfo.columns} />
            </div>

            {/* Missing values */}
            <div className="flex flex-col gap-2">
                <span className="text-muted-foreground text-xs font-medium">
                    Data Quality
                </span>
                <MissingValuesSummary
                    columns={fileInfo.columns}
                    totalRows={fileInfo.row_count}
                />
            </div>

            {/* Top columns with most nulls */}
            {fileInfo.columns.some((c) => c.null_count > 0) && (
                <div className="flex flex-col gap-2">
                    <span className="text-muted-foreground text-xs font-medium">
                        Columns with Missing Values
                    </span>
                    <div className="flex max-h-[120px] flex-col gap-1 overflow-y-auto">
                        {fileInfo.columns
                            .filter((c) => c.null_count > 0)
                            .sort((a, b) => b.null_count - a.null_count)
                            .slice(0, 5)
                            .map((col) => {
                                const percent =
                                    (col.null_count / fileInfo.row_count) * 100;
                                return (
                                    <div
                                        key={col.name}
                                        className="flex items-center justify-between text-xs"
                                    >
                                        <span
                                            className="truncate font-mono"
                                            title={col.name}
                                        >
                                            {col.name}
                                        </span>
                                        <div className="ml-2 flex shrink-0 items-center gap-2">
                                            <div className="bg-secondary h-1.5 w-16 overflow-hidden rounded-full">
                                                <div
                                                    className="h-full bg-yellow-500"
                                                    style={{
                                                        width: `${Math.min(percent, 100)}%`,
                                                    }}
                                                />
                                            </div>
                                            <span className="text-muted-foreground w-12 text-right tabular-nums">
                                                {percent.toFixed(1)}%
                                            </span>
                                        </div>
                                    </div>
                                );
                            })}
                        {fileInfo.columns.filter((c) => c.null_count > 0)
                            .length > 5 && (
                            <span className="text-muted-foreground text-xs">
                                +
                                {fileInfo.columns.filter(
                                    (c) => c.null_count > 0,
                                ).length - 5}{" "}
                                more columns
                            </span>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}

export default DatasetPreview;
