"use client";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import type { AssociationAnalysis, StatisticalTestResult } from "@/types";

import AnalysisHeatmap from "./analysis-heatmap";

interface AnalysisAssociationsProps {
    associations: AssociationAnalysis;
}

const AnalysisAssociations = ({ associations }: AnalysisAssociationsProps) => {
    return (
        <div className="grid h-full grid-cols-2 gap-3">
            <AnalysisHeatmap
                title="Cramér's V"
                matrix={associations.cramers_v}
            />
            <AnalysisHeatmap
                title="Chi-Square Statistic"
                matrix={associations.chi_square}
            />

            <Card className="col-span-2 min-h-0">
                <CardHeader title="Numeric vs Categorical Tests" />
                <CardContent scrollable>
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
                </CardContent>
            </Card>
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
