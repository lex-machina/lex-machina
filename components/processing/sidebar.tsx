"use client";

import { Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useProcessingContext } from "./context";
import { formatNumber } from "@/lib/utils";

/**
 * Sidebar for the processing page.
 *
 * Contains:
 * - Start Processing button (moved from toolbar)
 * - Quick stats about selection
 */
export function ProcessingSidebar() {
    const {
        fileInfo,
        selectedColumns,
        rowRange,
        canStart,
        isProcessing,
        startProcessing,
        columns,
    } = useProcessingContext();

    const totalRows = fileInfo?.row_count ?? 0;
    const selectedRowCount = rowRange
        ? rowRange.end - rowRange.start + 1
        : totalRows;
    const rangeStart = rowRange ? rowRange.start + 1 : 1;
    const rangeEnd = rowRange ? rowRange.end + 1 : totalRows;

    return (
        <div className="flex h-full flex-col">
            {/* Start Processing Button */}
            <div className="border-b p-4">
                <Button
                    variant="default"
                    size="sm"
                    onClick={startProcessing}
                    disabled={!canStart || isProcessing}
                    className="w-full"
                >
                    <Play className="mr-1.5 h-3.5 w-3.5" />
                    {isProcessing ? "Processing..." : "Start Processing"}
                </Button>
            </div>

            {/* Selection Summary */}
            <div className="flex-1 space-y-5 overflow-y-auto p-4">
                <section>
                    <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                        Selection Summary
                    </h2>
                    <dl className="space-y-2 text-sm">
                        <div className="flex items-center justify-between">
                            <dt className="text-muted-foreground">Columns</dt>
                            <dd className="font-medium">
                                {selectedColumns.length} / {columns.length}
                            </dd>
                        </div>
                        <div className="flex items-center justify-between">
                            <dt className="text-muted-foreground">Rows</dt>
                            <dd className="font-medium">
                                {formatNumber(selectedRowCount)} /{" "}
                                {formatNumber(totalRows)}
                            </dd>
                        </div>
                        <div className="flex items-center justify-between">
                            <dt className="text-muted-foreground">Row Range</dt>
                            <dd className="font-mono text-xs">
                                {formatNumber(rangeStart)} -{" "}
                                {formatNumber(rangeEnd)}
                            </dd>
                        </div>
                    </dl>
                </section>

                {/* Status */}
                <section>
                    <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                        Status
                    </h2>
                    <p className="text-sm">
                        {!canStart ? (
                            <span className="text-muted-foreground">
                                Select columns to start
                            </span>
                        ) : isProcessing ? (
                            <span className="text-primary">Processing...</span>
                        ) : (
                            <span className="text-muted-foreground">
                                Ready to process
                            </span>
                        )}
                    </p>
                </section>
            </div>
        </div>
    );
}
