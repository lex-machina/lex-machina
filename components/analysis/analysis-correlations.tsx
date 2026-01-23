"use client";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import type { CorrelationAnalysis } from "@/types";

import AnalysisHeatmap from "./analysis-heatmap";

interface AnalysisCorrelationsProps {
    correlations: CorrelationAnalysis;
}

const AnalysisCorrelations = ({ correlations }: AnalysisCorrelationsProps) => {
    return (
        <div className="grid h-full grid-cols-2 gap-3">
            <AnalysisHeatmap
                title="Pearson Correlation"
                matrix={correlations.pearson}
            />
            <AnalysisHeatmap
                title="Spearman Correlation"
                matrix={correlations.spearman}
            />

            <Card className="col-span-2 min-h-0">
                <CardHeader title="Top Correlations" />
                <CardContent scrollable>
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
                                    {pair.method} · {pair.estimate.toFixed(3)}{" "}
                                    (p=
                                    {pair.p_value.toFixed(4)})
                                </span>
                            </div>
                        ))}
                    </div>
                </CardContent>
            </Card>
        </div>
    );
};

export default AnalysisCorrelations;
