"use client";

import { useMemo, useState } from "react";
import { Search } from "lucide-react";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import Input from "@/components/ui/input";
import Select from "@/components/ui/select";
import { useAnalysisColumnsView } from "@/lib/hooks/use-analysis-columns-view";
import { cn, formatNumber } from "@/lib/utils";
import type { AnalysisColumnFilter, AnalysisColumnListItem } from "@/types";

interface AnalysisColumnsProps {
    useProcessedData: boolean;
    selectedColumn: string | null;
    onSelect: (column: string) => void;
}

const TYPE_OPTIONS = [
    { value: "all", label: "All types" },
    { value: "numeric", label: "Numeric" },
    { value: "categorical", label: "Categorical" },
    { value: "text", label: "Text" },
    { value: "datetime", label: "Datetime" },
];

const SORT_OPTIONS = [
    { value: "name", label: "Name" },
    { value: "nulls", label: "Missing %" },
    { value: "cardinality", label: "Cardinality" },
    { value: "type", label: "Type" },
];

const SORT_DIR_OPTIONS = [
    { value: "asc", label: "Ascending" },
    { value: "desc", label: "Descending" },
];

const AnalysisColumns = ({
    useProcessedData,
    selectedColumn,
    onSelect,
}: AnalysisColumnsProps) => {
    const [search, setSearch] = useState("");
    const [typeFilter, setTypeFilter] = useState("all");
    const [sortBy, setSortBy] =
        useState<AnalysisColumnFilter["sort_by"]>("name");
    const [sortDirection, setSortDirection] =
        useState<AnalysisColumnFilter["sort_direction"]>("asc");

    const filter = useMemo<AnalysisColumnFilter>(
        () => ({
            search: search.trim() ? search : null,
            inferred_types: resolveTypeFilters(typeFilter),
            sort_by: sortBy,
            sort_direction: sortDirection,
        }),
        [search, typeFilter, sortBy, sortDirection],
    );

    const { response, isLoading, error } = useAnalysisColumnsView(
        useProcessedData,
        filter,
    );

    const columns = response?.columns ?? [];

    return (
        <Card className="h-full min-h-0">
            <CardHeader
                title="Columns"
                actions={
                    response && (
                        <span className="text-muted-foreground text-xs">
                            {response.filtered}/{response.total}
                        </span>
                    )
                }
            />
            <CardContent className="flex flex-col gap-3" padded>
                <Input
                    value={search}
                    onChange={(event) => setSearch(event.target.value)}
                    placeholder="Search columns"
                    size="sm"
                    leftAddon={<Search className="h-3.5 w-3.5" />}
                />
                <div className="grid grid-cols-3 gap-2">
                    <Select
                        value={typeFilter}
                        onValueChange={setTypeFilter}
                        options={TYPE_OPTIONS}
                    />
                    <Select
                        value={sortBy}
                        onValueChange={(value) =>
                            setSortBy(value as AnalysisColumnFilter["sort_by"])
                        }
                        options={SORT_OPTIONS}
                    />
                    <Select
                        value={sortDirection}
                        onValueChange={(value) =>
                            setSortDirection(
                                value as AnalysisColumnFilter["sort_direction"],
                            )
                        }
                        options={SORT_DIR_OPTIONS}
                    />
                </div>
                <div className="divide-border flex flex-1 flex-col divide-y overflow-y-auto text-sm">
                    {isLoading && (
                        <div className="text-muted-foreground px-2 py-3 text-xs">
                            Loading columns...
                        </div>
                    )}
                    {error && (
                        <div className="text-destructive px-2 py-3 text-xs">
                            {error}
                        </div>
                    )}
                    {!isLoading && columns.length === 0 && (
                        <div className="text-muted-foreground px-2 py-3 text-xs">
                            No columns match this filter.
                        </div>
                    )}
                    {columns.map((column) => (
                        <ColumnRow
                            key={column.name}
                            column={column}
                            isSelected={column.name === selectedColumn}
                            onSelect={onSelect}
                        />
                    ))}
                </div>
            </CardContent>
        </Card>
    );
};

const ColumnRow = ({
    column,
    isSelected,
    onSelect,
}: {
    column: AnalysisColumnListItem;
    isSelected: boolean;
    onSelect: (column: string) => void;
}) => (
    <button
        type="button"
        onClick={() => onSelect(column.name)}
        className={cn(
            "flex w-full items-start justify-between gap-3 px-2 py-2 text-left",
            "hover:bg-muted/40",
            isSelected && "bg-muted/50",
        )}
    >
        <div className="flex flex-col">
            <span className="font-medium">{column.name}</span>
            <span className="text-muted-foreground text-xs">
                {column.inferred_type} · {column.inferred_role} · {column.dtype}
            </span>
        </div>
        <div className="text-muted-foreground text-right text-xs">
            <div>{column.null_percentage.toFixed(1)}% null</div>
            <div>{formatNumber(column.unique_count)} unique</div>
        </div>
    </button>
);

const resolveTypeFilters = (typeFilter: string) => {
    switch (typeFilter) {
        case "numeric":
            return ["numeric"];
        case "categorical":
            return ["categorical", "boolean", "binary"];
        case "text":
            return ["string"];
        case "datetime":
            return ["datetime"];
        default:
            return ["all"];
    }
};

export default AnalysisColumns;
