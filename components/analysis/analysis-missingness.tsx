"use client";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { useAnalysisHeatmap } from "@/lib/hooks/use-analysis-heatmap";
import { formatNumber } from "@/lib/utils";
import type { AnalysisDataset, MissingnessAnalysis } from "@/types";

import AnalysisHeatmap from "./analysis-heatmap";

interface AnalysisMissingnessProps {
    dataset: AnalysisDataset;
    missingness: MissingnessAnalysis;
}

const AnalysisMissingness = ({
    dataset,
    missingness,
}: AnalysisMissingnessProps) => {
    const useProcessedData = dataset === "processed";
    const heatmap = useAnalysisHeatmap(useProcessedData, "missingness");

    return (
        <div className="flex h-full min-h-0 gap-3">
            <Card className="h-full min-h-0 flex-[1_1_0%]">
                <CardHeader
                    title="Missingness"
                    actions={
                        <span className="text-muted-foreground text-xs">
                            {missingness.total_missing_percentage.toFixed(1)}%
                            overall
                        </span>
                    }
                />
                <CardContent scrollable padded>
                    <div className="space-y-2 text-sm">
                        {missingness.per_column.map((entry) => (
                            <div
                                key={entry.column}
                                className="flex items-center justify-between"
                            >
                                <span className="truncate">{entry.column}</span>
                                <span className="text-muted-foreground">
                                    {formatNumber(entry.missing_count)} (
                                    {entry.missing_percentage.toFixed(1)}%)
                                </span>
                            </div>
                        ))}
                    </div>
                </CardContent>
            </Card>

            <div className="h-full min-h-0 flex-[2_1_0%]">
                <AnalysisHeatmap
                    title="Co-missing Heatmap"
                    variant="sequential"
                    view={heatmap.view}
                    isLoading={heatmap.isLoading}
                    error={heatmap.error}
                    hasData={missingness.per_column.length > 0}
                    fillHeight
                />
            </div>
        </div>
    );
};

export default AnalysisMissingness;
