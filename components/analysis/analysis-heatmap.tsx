"use client";

import { useMemo } from "react";
import * as Plot from "@observablehq/plot";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import type { HeatmapMatrixView } from "@/types";

import PlotFigure from "./plot-figure";

interface AnalysisHeatmapProps {
    title: string;
    view: HeatmapMatrixView | null;
    variant: "correlation" | "sequential";
    isLoading: boolean;
    error?: string | null;
    hasData: boolean;
    height?: number;
    fillHeight?: boolean;
}

const AnalysisHeatmap = ({
    title,
    view,
    variant,
    isLoading,
    error,
    hasData,
    height = 220,
    fillHeight = false,
}: AnalysisHeatmapProps) => {
    const plotHeight = useMemo(() => height, [height]);
    if (!hasData) {
        return (
            <Card className="min-h-0">
                <CardHeader title={title} />
                <CardContent className="flex h-full items-center justify-center text-xs">
                    <span className="text-muted-foreground">
                        Not enough columns to render this heatmap.
                    </span>
                </CardContent>
            </Card>
        );
    }

    if (error) {
        return (
            <Card className="min-h-0">
                <CardHeader title={title} />
                <CardContent className="flex h-full items-center justify-center text-xs">
                    <span className="text-destructive">{error}</span>
                </CardContent>
            </Card>
        );
    }

    if (isLoading || !view) {
        return (
            <Card className="min-h-0">
                <CardHeader title={title} />
                <CardContent className="flex h-full items-center justify-center text-xs">
                    <span className="text-muted-foreground">
                        {isLoading ? "Loading heatmap..." : "No data available"}
                    </span>
                </CardContent>
            </Card>
        );
    }

    const cells = view.y_labels.flatMap((rowLabel, rowIndex) =>
        view.x_labels.map((colLabel, colIndex) => ({
            x: colLabel,
            y: rowLabel,
            value: view.values[rowIndex]?.[colIndex] ?? 0,
            p_value: view.p_values?.[rowIndex]?.[colIndex] ?? null,
        })),
    );

    const labelFormatter = (label: string) =>
        label.length > 14 ? `${label.slice(0, 12)}…` : label;

    const colorScheme = variant === "correlation" ? "rdylbu" : "blues";

    const options: Plot.PlotOptions = {
        height: plotHeight,
        marginLeft: 90,
        marginBottom: 70,
        marginRight: 40,
        style: { overflow: "visible" },
        color: {
            scheme: colorScheme,
            domain: [view.min, view.max],
            legend: true,
        },
        x: {
            label: null,
            padding: 0.2,
            tickRotate: 30,
            tickFormat: labelFormatter,
        },
        y: {
            label: null,
            padding: 0.2,
            tickFormat: labelFormatter,
        },
        marks: [
            Plot.cell(cells, {
                x: "x",
                y: "y",
                fill: "value",
                stroke: "var(--border)",
                strokeWidth: 1,
                channels: { value: "value", p_value: "p_value" },
                tip: {
                    format: {
                        x: (value) => value,
                        y: (value) => value,
                        value: (value) =>
                            typeof value === "number" ? value.toFixed(3) : "—",
                        p_value: (value) =>
                            typeof value === "number" ? value.toFixed(4) : "—",
                    },
                },
            }),
            Plot.frame(),
        ],
    };

    const contentClassName = fillHeight
        ? "flex min-h-0 flex-1 flex-col h-full"
        : undefined;
    const containerClassName = fillHeight
        ? "flex min-h-0 flex-1 h-full"
        : "h-full";

    return (
        <Card className={fillHeight ? "h-full min-h-0" : "min-h-0"}>
            <CardHeader
                title={title}
                actions={
                    view.truncated ? (
                        <span className="text-muted-foreground text-xs">
                            Showing {view.x_labels.length} of{" "}
                            {view.total_columns}
                        </span>
                    ) : undefined
                }
            />
            <CardContent padded className={contentClassName}>
                <div className={containerClassName}>
                    <PlotFigure
                        options={options}
                        className="h-full"
                        autoHeight={fillHeight}
                    />
                </div>
            </CardContent>
        </Card>
    );
};

export default AnalysisHeatmap;
