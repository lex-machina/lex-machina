"use client";

import { useCallback } from "react";
import { AlertTriangle } from "lucide-react";

import {
    Card,
    CardHeader,
    CardContent,
    CardFooter,
} from "@/components/ui/card";
import { ColumnSelector, ColumnSelectorHeader } from "./column-selector";
import { RowRangeSelector } from "./row-range-selector";
import { useProcessingContext } from "./context";

/**
 * Left panel containing column selector and row range selector.
 */
export function ColumnsPanel() {
    const {
        fileInfo,
        columns,
        selectedColumns,
        setSelectedColumns,
        rowRange,
        setRowRange,
        isProcessing,
        hasColumnsSelected,
    } = useProcessingContext();

    // Selection handlers for the header
    const handleSelectAll = useCallback(() => {
        setSelectedColumns(columns.map((col) => col.name));
    }, [columns, setSelectedColumns]);

    const handleDeselectAll = useCallback(() => {
        setSelectedColumns([]);
    }, [setSelectedColumns]);

    const totalRows = fileInfo?.row_count ?? 0;
    const noColumnsSelected = !hasColumnsSelected && columns.length > 0;

    return (
        <div className="flex h-full min-h-0 flex-col">
            {/* Column Selector - takes remaining space with internal scroll */}
            <Card className="min-h-0 flex-1">
                <CardHeader
                    title="Columns"
                    actions={
                        <ColumnSelectorHeader
                            totalCount={columns.length}
                            selectedCount={selectedColumns.length}
                            onSelectAll={handleSelectAll}
                            onDeselectAll={handleDeselectAll}
                            disabled={isProcessing}
                        />
                    }
                />
                <CardContent className="overflow-hidden">
                    <ColumnSelector
                        columns={columns}
                        selectedColumns={selectedColumns}
                        onSelectionChange={setSelectedColumns}
                        disabled={isProcessing}
                        hideHeader={true}
                        className="h-full"
                    />
                </CardContent>
                {/* Warning when no columns selected */}
                {noColumnsSelected && (
                    <CardFooter className="text-muted-foreground flex items-center gap-2 text-xs">
                        <AlertTriangle className="size-3.5 shrink-0" />
                        <span>Select at least one column to process</span>
                    </CardFooter>
                )}
            </Card>

            {/* Row Range Selector - fixed at bottom */}
            <Card className="mt-3 shrink-0">
                <CardHeader title="Row Range" />
                <CardContent padded>
                    <RowRangeSelector
                        totalRows={totalRows}
                        rowRange={rowRange}
                        onRangeChange={setRowRange}
                        disabled={isProcessing}
                    />
                </CardContent>
            </Card>
        </div>
    );
}
