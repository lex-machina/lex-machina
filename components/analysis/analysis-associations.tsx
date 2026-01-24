"use client";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { useAnalysisHeatmap } from "@/lib/hooks/use-analysis-heatmap";
import type {
    AnalysisDataset,
    AssociationAnalysis,
    StatisticalTestResult,
} from "@/types";

import AnalysisHeatmap from "./analysis-heatmap";

interface AnalysisAssociationsProps {
    dataset: AnalysisDataset;
    associations: AssociationAnalysis;
}

const AnalysisAssociations = ({
    dataset,
    associations,
}: AnalysisAssociationsProps) => {
    const useProcessedData = dataset === "processed";
    const cramers = useAnalysisHeatmap(useProcessedData, "cramers_v");
    const chiSquare = useAnalysisHeatmap(useProcessedData, "chi_square");

    return (
        <div className="flex h-full min-h-0 gap-3">
            <Card className="h-full min-h-0 flex-[1_1_0%]">
                <CardHeader title="Numeric vs Categorical Tests" />
                <CardContent scrollable>
                    {associations.numeric_categorical.length === 0 ? (
                        <div className="text-muted-foreground py-2 text-xs">
                            No numeric/categorical pairs available for testing.
                        </div>
                    ) : (
                        <div className="divide-border flex flex-col divide-y text-sm">
                            {associations.numeric_categorical.map((entry) => (
                                <div
                                    key={`${entry.numeric_column}-${entry.categorical_column}`}
                                    className="flex flex-col gap-1 py-2"
                                >
                                    <div className="font-medium">
                                        {entry.numeric_column} ×{" "}
                                        {entry.categorical_column}
                                    </div>
                                    <div className="text-muted-foreground flex flex-wrap gap-3 text-xs">
                                        {renderTest(entry.variance_test)}
                                        {renderTest(entry.anova)}
                                        {renderTest(entry.kruskal)}
                                        {renderTest(entry.t_test)}
                                        {renderTest(entry.mann_whitney)}
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </CardContent>
            </Card>
            <div className="flex min-h-0 flex-[2_1_0%] flex-col gap-3">
                <div className="min-h-0 flex-1">
                    <AnalysisHeatmap
                        title="Cramér's V"
                        variant="sequential"
                        view={cramers.view}
                        isLoading={cramers.isLoading}
                        error={cramers.error}
                        hasData={associations.categorical_columns.length >= 2}
                        fillHeight
                    />
                </div>
                <div className="min-h-0 flex-1">
                    <AnalysisHeatmap
                        title="Chi-Square Statistic"
                        variant="sequential"
                        view={chiSquare.view}
                        isLoading={chiSquare.isLoading}
                        error={chiSquare.error}
                        hasData={associations.categorical_columns.length >= 2}
                        fillHeight
                    />
                </div>
            </div>
        </div>
    );
};

const renderTest = (test?: StatisticalTestResult | null) => {
    if (!test) {
        return null;
    }
    return (
        <span key={test.test}>
            {test.test}: p={test.p_value.toFixed(4)}
        </span>
    );
};

export default AnalysisAssociations;
