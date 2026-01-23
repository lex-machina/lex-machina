"use client";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { formatNumber } from "@/lib/utils";
import type { MissingnessAnalysis } from "@/types";

import AnalysisHeatmap from "./analysis-heatmap";

interface AnalysisMissingnessProps {
    missingness: MissingnessAnalysis;
}

const AnalysisMissingness = ({ missingness }: AnalysisMissingnessProps) => {
    return (
        <div className="grid h-full grid-cols-[360px_1fr] gap-3">
            <Card className="min-h-0">
                <CardHeader title="Missingness" />
                <CardContent scrollable>
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

            <AnalysisHeatmap
                title="Co-missing Heatmap"
                matrix={missingness.co_missing_matrix}
            />
        </div>
    );
};

export default AnalysisMissingness;
