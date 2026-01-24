"use client";

import type { VisualizationChart, VisualizationChartKind } from "@/types";

import VisualizationCard from "./visualization-card";

interface VisualizationsGridProps {
    charts: VisualizationChart[];
    chartKinds: Record<string, VisualizationChartKind>;
    onChartKindChange: (column: string, kind: VisualizationChartKind) => void;
}

const VisualizationsGrid = ({
    charts,
    chartKinds,
    onChartKindChange,
}: VisualizationsGridProps) => {
    return (
        <div className="grid w-full grid-cols-3 gap-3">
            {charts.map((chart, index) => (
                <VisualizationCard
                    key={`${chart.column}-${chart.kind}-${index}`}
                    chart={chart}
                    accentIndex={index}
                    selectedKind={chartKinds[chart.column]}
                    onChartKindChange={onChartKindChange}
                />
            ))}
        </div>
    );
};

export default VisualizationsGrid;
