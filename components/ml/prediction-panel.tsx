"use client";

import { useMemo, useState } from "react";
import { Braces, Wand2 } from "lucide-react";

import type {
    BatchPredictionResult,
    ColumnInfo,
    PredictionResult,
} from "@/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

interface PredictionPanelProps {
    columns: ColumnInfo[];
    onPredictSingle: (
        payload: Record<string, unknown>,
    ) => Promise<PredictionResult>;
    onPredictBatch: () => Promise<BatchPredictionResult>;
    disabled?: boolean;
}

function emptyPayload(columns: ColumnInfo[]): Record<string, unknown> {
    return columns.reduce<Record<string, unknown>>((acc, col) => {
        acc[col.name] = "";
        return acc;
    }, {});
}

export function PredictionPanel({
    columns,
    onPredictSingle,
    onPredictBatch,
    disabled = false,
}: PredictionPanelProps) {
    const [mode, setMode] = useState<"form" | "json">("form");
    const [formValues, setFormValues] = useState(() => emptyPayload(columns));
    const [jsonValue, setJsonValue] = useState("{}");
    const [singleResult, setSingleResult] = useState<PredictionResult | null>(
        null,
    );
    const [batchResult, setBatchResult] =
        useState<BatchPredictionResult | null>(null);

    const columnNames = useMemo(
        () => columns.map((col) => col.name),
        [columns],
    );

    const handleFormChange = (name: string, value: string) => {
        setFormValues((current) => ({
            ...current,
            [name]: value,
        }));
    };

    const handlePredictSingle = async () => {
        const payload = mode === "json" ? JSON.parse(jsonValue) : formValues;
        const result = await onPredictSingle(payload);
        setSingleResult(result);
    };

    const handlePredictBatch = async () => {
        const result = await onPredictBatch();
        setBatchResult(result);
    };

    return (
        <div className="flex h-full flex-col">
            <div className="flex items-center justify-between border-b px-3 py-2">
                <div className="text-muted-foreground flex items-center gap-2 text-xs">
                    <Wand2 className="h-3.5 w-3.5" />
                    Prediction
                </div>
                <div className="flex gap-2">
                    <Button
                        variant={mode === "form" ? "default" : "outline"}
                        size="sm"
                        onClick={() => setMode("form")}
                        disabled={disabled}
                    >
                        Form
                    </Button>
                    <Button
                        variant={mode === "json" ? "default" : "outline"}
                        size="sm"
                        onClick={() => setMode("json")}
                        disabled={disabled}
                    >
                        JSON
                    </Button>
                </div>
            </div>

            <div className="flex-1 overflow-y-auto p-3">
                {mode === "form" && (
                    <div className="flex flex-col gap-2">
                        {columnNames.map((name) => (
                            <Input
                                key={name}
                                label={name}
                                value={String(formValues[name] ?? "")}
                                onChange={(event) =>
                                    handleFormChange(name, event.target.value)
                                }
                                disabled={disabled}
                            />
                        ))}
                    </div>
                )}
                {mode === "json" && (
                    <div className="flex flex-col gap-2">
                        <label className="text-sm font-medium">
                            Input JSON
                        </label>
                        <textarea
                            className={cn(
                                "bg-background min-h-[140px] w-full rounded-md border px-3 py-2 text-sm",
                                "focus:ring-ring focus:ring-offset-background focus:ring-2 focus:ring-offset-2 focus:outline-none",
                                "disabled:opacity-60",
                            )}
                            value={jsonValue}
                            onChange={(event) =>
                                setJsonValue(event.target.value)
                            }
                            disabled={disabled}
                        />
                    </div>
                )}

                {(singleResult || batchResult) && (
                    <div className="bg-muted/30 mt-4 rounded-md border p-3 text-xs">
                        {singleResult && (
                            <div className="space-y-1">
                                <div className="text-muted-foreground">
                                    Single prediction
                                </div>
                                <div className="text-sm font-semibold">
                                    {String(singleResult.prediction)}
                                </div>
                            </div>
                        )}
                        {batchResult && (
                            <div className="mt-3 space-y-1">
                                <div className="text-muted-foreground">
                                    Batch predictions ({batchResult.row_count}{" "}
                                    rows)
                                </div>
                                <div className="text-sm font-semibold">
                                    {batchResult.predictions
                                        .slice(0, 3)
                                        .map((value) => String(value))
                                        .join(", ")}
                                    {batchResult.predictions.length > 3 && "…"}
                                </div>
                            </div>
                        )}
                    </div>
                )}
            </div>

            <div className="border-border grid gap-2 border-t p-3">
                <Button
                    size="sm"
                    onClick={handlePredictSingle}
                    disabled={disabled}
                >
                    Predict single
                </Button>
                <Button
                    variant="outline"
                    size="sm"
                    onClick={handlePredictBatch}
                    disabled={disabled}
                >
                    Predict batch
                </Button>
                {mode === "json" && (
                    <div className="text-muted-foreground flex items-center gap-2 text-xs">
                        <Braces className="h-3.5 w-3.5" />
                        JSON must include feature fields in the training order.
                    </div>
                )}
            </div>
        </div>
    );
}
