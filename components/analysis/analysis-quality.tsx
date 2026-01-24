"use client";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import type { DataQualityIssue } from "@/types";

interface AnalysisQualityProps {
    issues: DataQualityIssue[];
}

const AnalysisQuality = ({ issues }: AnalysisQualityProps) => {
    return (
        <Card className="h-full">
            <CardHeader title="Quality Issues" />
            <CardContent scrollable padded>
                {issues.length === 0 ? (
                    <div className="text-muted-foreground py-3 text-xs">
                        No critical issues detected.
                    </div>
                ) : (
                    <div className="divide-border flex flex-col divide-y text-sm">
                        {issues.map((issue, index) => (
                            <div
                                key={`${issue.issue_type}-${index}`}
                                className="py-3"
                            >
                                <div className="flex items-center justify-between">
                                    <span className="font-medium">
                                        {issue.issue_type}
                                    </span>
                                    <span className="text-muted-foreground text-xs">
                                        {issue.severity}
                                    </span>
                                </div>
                                <p className="text-muted-foreground mt-1 text-xs">
                                    {issue.description}
                                </p>
                                <p className="text-muted-foreground mt-1 text-xs">
                                    Impact: {issue.business_impact}
                                </p>
                                {issue.suggested_solutions.length > 0 && (
                                    <ul className="mt-2 space-y-1 text-xs">
                                        {issue.suggested_solutions.map(
                                            (solution) => (
                                                <li key={solution.option}>
                                                    <span className="font-medium">
                                                        {solution.option}:
                                                    </span>{" "}
                                                    <span className="text-muted-foreground">
                                                        {solution.description}
                                                    </span>
                                                </li>
                                            ),
                                        )}
                                    </ul>
                                )}
                            </div>
                        ))}
                    </div>
                )}
            </CardContent>
        </Card>
    );
};

export default AnalysisQuality;
