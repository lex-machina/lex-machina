"use client";

import * as Plot from "@observablehq/plot";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import type { HeatmapMatrix } from "@/types";

import PlotFigure from "./plot-figure";

interface AnalysisHeatmapProps {
    title: string;
    matrix: HeatmapMatrix;
    height?: number;
}

const AnalysisHeatmap = ({
    title,
    matrix,
    height = 220,
}: AnalysisHeatmapProps) => {
    const cells = matrix.y_labels.flatMap((rowLabel, rowIndex) =>
        matrix.x_labels.map((colLabel, colIndex) => ({
            x: colLabel,
            y: rowLabel,
            value: matrix.values[rowIndex]?.[colIndex] ?? 0,
        })),
    );

    const options: Plot.PlotOptions = {
        height,
        marginLeft: 80,
        marginBottom: 60,
        color: { scheme: "greys" },
        x: { label: null, padding: 0.2 },
        y: { label: null, padding: 0.2 },
        marks: [
            Plot.cell(cells, { x: "x", y: "y", fill: "value" }),
            Plot.frame(),
        ],
    };

    return (
        <Card className="min-h-0">
            <CardHeader title={title} />
            <CardContent padded>
                <PlotFigure options={options} />
            </CardContent>
        </Card>
    );
};

export default AnalysisHeatmap;
