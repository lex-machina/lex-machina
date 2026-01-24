"use client";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import type { AnalysisResult } from "@/types";

interface AnalysisOverviewProps {
    analysis: AnalysisResult;
}

const AnalysisOverview = ({ analysis }: AnalysisOverviewProps) => {
    const { summary_view, dataset_profile } = analysis;

    return (
        <div className="grid h-full grid-cols-2 gap-3">
            <Card className="min-h-0">
                <CardHeader title="Summary" />
                <CardContent padded>
                    <dl className="grid grid-cols-2 gap-x-4 gap-y-3 text-sm">
                        <div>
                            <dt className="text-muted-foreground">Rows</dt>
                            <dd className="font-medium">{summary_view.rows}</dd>
                        </div>
                        <div>
                            <dt className="text-muted-foreground">Columns</dt>
                            <dd className="font-medium">
                                {summary_view.columns}
                            </dd>
                        </div>
                        <div>
                            <dt className="text-muted-foreground">Memory</dt>
                            <dd className="font-medium">
                                {summary_view.memory}
                            </dd>
                        </div>
                        <div>
                            <dt className="text-muted-foreground">
                                Duplicates
                            </dt>
                            <dd className="font-medium">
                                {summary_view.duplicates}
                            </dd>
                        </div>
                        <div>
                            <dt className="text-muted-foreground">
                                Missing Cells
                            </dt>
                            <dd className="font-medium">
                                {summary_view.missing_cells}
                            </dd>
                        </div>
                        <div>
                            <dt className="text-muted-foreground">Generated</dt>
                            <dd className="font-medium">
                                {new Date(
                                    analysis.generated_at,
                                ).toLocaleString()}
                            </dd>
                        </div>
                    </dl>
                </CardContent>
            </Card>

            <Card className="min-h-0">
                <CardHeader title="Type Distribution" />
                <CardContent padded>
                    <div className="space-y-2 text-sm">
                        {summary_view.type_distribution.map((entry) => (
                            <div
                                key={entry.dtype}
                                className="flex items-center justify-between"
                            >
                                <span className="text-muted-foreground">
                                    {entry.dtype}
                                </span>
                                <span className="font-medium">
                                    {entry.count} ({entry.percentage})
                                </span>
                            </div>
                        ))}
                    </div>
                </CardContent>
            </Card>

            <Card className="min-h-0">
                <CardHeader title="Target Discovery" />
                <CardContent padded>
                    <div className="space-y-3 text-sm">
                        <div>
                            <div className="text-muted-foreground mb-1">
                                Target candidates
                            </div>
                            <div className="font-medium">
                                {dataset_profile.target_candidates.length
                                    ? dataset_profile.target_candidates.join(
                                          ", ",
                                      )
                                    : "None detected"}
                            </div>
                        </div>
                        <div>
                            <div className="text-muted-foreground mb-1">
                                Problem type candidates
                            </div>
                            <div className="font-medium">
                                {dataset_profile.problem_type_candidates.length
                                    ? dataset_profile.problem_type_candidates.join(
                                          ", ",
                                      )
                                    : "None detected"}
                            </div>
                        </div>
                    </div>
                </CardContent>
            </Card>

            <Card className="min-h-0">
                <CardHeader title="Complexity Indicators" />
                <CardContent padded>
                    <dl className="space-y-2 text-sm">
                        {Object.entries(
                            dataset_profile.complexity_indicators,
                        ).map(([key, value]) => (
                            <div
                                key={key}
                                className="flex items-center justify-between"
                            >
                                <dt className="text-muted-foreground">
                                    {formatComplexityLabel(key)}
                                </dt>
                                <dd className="font-medium">
                                    {formatComplexityValue(value)}
                                </dd>
                            </div>
                        ))}
                    </dl>
                </CardContent>
            </Card>
        </div>
    );
};

const formatComplexityLabel = (label: string) =>
    label
        .split("_")
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join(" ");

const formatComplexityValue = (value: unknown) => {
    if (value === null || value === undefined) {
        return "—";
    }
    if (typeof value === "boolean") {
        return value ? "Yes" : "No";
    }
    if (typeof value === "number") {
        return value.toFixed(2);
    }
    if (typeof value === "string") {
        return value;
    }
    return JSON.stringify(value);
};

export default AnalysisOverview;
