"use client";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { useAnalysisHeatmap } from "@/lib/hooks/use-analysis-heatmap";
import type { AnalysisDataset, CorrelationAnalysis } from "@/types";

import AnalysisHeatmap from "./analysis-heatmap";

interface AnalysisCorrelationsProps {
    dataset: AnalysisDataset;
    correlations: CorrelationAnalysis;
}

const AnalysisCorrelations = ({
    dataset,
    correlations,
}: AnalysisCorrelationsProps) => {
    const useProcessedData = dataset === "processed";
    const pearson = useAnalysisHeatmap(useProcessedData, "pearson");
    const spearman = useAnalysisHeatmap(useProcessedData, "spearman");
    const hasNumeric = correlations.numeric_columns.length >= 2;

    return (
        <div className="flex h-full min-h-0 gap-3">
            <Card className="h-full min-h-0 flex-[1_1_0%]">
                <CardHeader title="Top Correlations" />
                <CardContent scrollable padded>
                    {correlations.top_pairs.length === 0 ? (
                        <div className="text-muted-foreground py-2 text-xs">
                            Not enough numeric columns to compute correlations.
                        </div>
                    ) : (
                        <div className="divide-border flex flex-col divide-y text-sm">
                            {correlations.top_pairs.map((pair) => (
                                <div
                                    key={`${pair.column_x}-${pair.column_y}-${pair.method}`}
                                    className="flex items-center justify-between py-2"
                                >
                                    <span>
                                        {pair.column_x} × {pair.column_y}
                                    </span>
                                    <span className="text-muted-foreground">
                                        {pair.method} ·{" "}
                                        {pair.estimate.toFixed(3)} (p=
                                        {pair.p_value.toFixed(4)})
                                    </span>
                                </div>
                            ))}
                        </div>
                    )}
                </CardContent>
            </Card>
            <div className="flex min-h-0 flex-[2_1_0%] flex-col gap-3">
                <div className="min-h-0 flex-1">
                    <AnalysisHeatmap
                        title="Pearson Correlation"
                        variant="correlation"
                        view={pearson.view}
                        isLoading={pearson.isLoading}
                        error={pearson.error}
                        hasData={hasNumeric}
                        fillHeight
                    />
                </div>
                <div className="min-h-0 flex-1">
                    <AnalysisHeatmap
                        title="Spearman Correlation"
                        variant="correlation"
                        view={spearman.view}
                        isLoading={spearman.isLoading}
                        error={spearman.error}
                        hasData={hasNumeric}
                        fillHeight
                    />
                </div>
            </div>
        </div>
    );
};

export default AnalysisCorrelations;
