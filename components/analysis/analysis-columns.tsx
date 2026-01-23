"use client";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type { AnalysisColumnStats } from "@/types";

interface AnalysisColumnsProps {
    columns: AnalysisColumnStats[];
    selectedColumn: string | null;
    onSelect: (column: string) => void;
}

const AnalysisColumns = ({
    columns,
    selectedColumn,
    onSelect,
}: AnalysisColumnsProps) => {
    return (
        <Card className="min-h-0">
            <CardHeader title="Columns" />
            <CardContent scrollable>
                <div className="divide-border flex flex-col divide-y text-sm">
                    {columns.map((column) => {
                        const profile = column.profile;
                        const isSelected = profile.name === selectedColumn;
                        return (
                            <button
                                key={profile.name}
                                type="button"
                                onClick={() => onSelect(profile.name)}
                                className={cn(
                                    "flex w-full items-start justify-between gap-3 px-3 py-2 text-left",
                                    "hover:bg-muted/40",
                                    isSelected && "bg-muted/50",
                                )}
                            >
                                <div className="flex flex-col">
                                    <span className="font-medium">
                                        {profile.name}
                                    </span>
                                    <span className="text-muted-foreground text-xs">
                                        {profile.inferred_type} ·{" "}
                                        {profile.inferred_role}
                                    </span>
                                </div>
                                <span className="text-muted-foreground text-xs">
                                    {profile.null_percentage.toFixed(1)}% null
                                </span>
                            </button>
                        );
                    })}
                </div>
            </CardContent>
        </Card>
    );
};

export default AnalysisColumns;
