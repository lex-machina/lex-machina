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

interface MLPredictionPanelProps {
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

function PanelTab({
    label,
    isActive,
    onClick,
}: {
    label: string;
    isActive: boolean;
    onClick: () => void;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={cn(
                "text-center text-xs font-semibold tracking-wider uppercase transition-colors",
                isActive
                    ? "text-foreground"
                    : "text-muted-foreground hover:text-foreground",
            )}
        >
            {label}
        </button>
    );
}

export function MLPredictionPanel({
    columns,
    onPredictSingle,
    onPredictBatch,
    disabled = false,
}: MLPredictionPanelProps) {
    const [mode, setMode] = useState<"single" | "batch">("single");
    const [inputMode, setInputMode] = useState<"form" | "json">("form");
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
        const payload =
            inputMode === "json" ? JSON.parse(jsonValue) : formValues;
        const result = await onPredictSingle(payload);
        setSingleResult(result);
    };

    const handlePredictBatch = async () => {
        const result = await onPredictBatch();
        setBatchResult(result);
    };

    return (
        <div className="flex h-full min-h-0 flex-col overflow-hidden rounded-lg border">
            <div className="bg-muted/30 shrink-0 border-b px-3 py-2">
                <div className="grid grid-cols-2">
                    <PanelTab
                        label="Single"
                        isActive={mode === "single"}
                        onClick={() => setMode("single")}
                    />
                    <PanelTab
                        label="Batch"
                        isActive={mode === "batch"}
                        onClick={() => setMode("batch")}
                    />
                </div>
            </div>

            {mode === "single" ? (
                <div className="min-h-0 flex-1 overflow-y-auto p-3">
                    <div className="flex flex-col gap-3">
                        <div className="text-muted-foreground flex items-center gap-2 text-xs">
                            <Wand2 className="h-3.5 w-3.5" />
                            Single prediction
                        </div>
                        <div className="flex gap-2">
                            <Button
                                variant={
                                    inputMode === "form" ? "default" : "outline"
                                }
                                size="sm"
                                onClick={() => setInputMode("form")}
                                disabled={disabled}
                            >
                                Form
                            </Button>
                            <Button
                                variant={
                                    inputMode === "json" ? "default" : "outline"
                                }
                                size="sm"
                                onClick={() => setInputMode("json")}
                                disabled={disabled}
                            >
                                JSON
                            </Button>
                        </div>
                        {inputMode === "form" && (
                            <div className="flex flex-col gap-2">
                                {columnNames.map((name) => (
                                    <Input
                                        key={name}
                                        label={name}
                                        value={String(formValues[name] ?? "")}
                                        onChange={(event) =>
                                            handleFormChange(
                                                name,
                                                event.target.value,
                                            )
                                        }
                                        disabled={disabled}
                                    />
                                ))}
                            </div>
                        )}
                        {inputMode === "json" && (
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
                                <div className="text-muted-foreground flex items-center gap-2 text-xs">
                                    <Braces className="h-3.5 w-3.5" />
                                    JSON must include feature fields in the
                                    training order.
                                </div>
                            </div>
                        )}
                        <Button
                            size="sm"
                            onClick={handlePredictSingle}
                            disabled={disabled}
                        >
                            Predict single
                        </Button>
                        {singleResult && (
                            <div className="bg-muted/30 rounded-md border p-3 text-xs">
                                <div className="text-muted-foreground">
                                    Single prediction
                                </div>
                                <div className="text-sm font-semibold">
                                    {String(singleResult.prediction)}
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            ) : null}

            {mode === "batch" ? (
                <div className="min-h-0 flex-1 overflow-y-auto p-3">
                    <div className="flex flex-col gap-3">
                        <div className="text-muted-foreground flex items-center gap-2 text-xs">
                            <Wand2 className="h-3.5 w-3.5" />
                            Batch prediction
                        </div>
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={handlePredictBatch}
                            disabled={disabled}
                        >
                            Predict batch
                        </Button>
                        {batchResult && (
                            <div className="bg-muted/30 rounded-md border p-3 text-xs">
                                <div className="text-muted-foreground">
                                    Batch predictions ({batchResult.row_count}{" "}
                                    rows)
                                </div>
                                <div className="text-sm font-semibold">
                                    {batchResult.predictions
                                        .slice(0, 3)
                                        .map((value) => String(value))
                                        .join(", ")}
                                    {batchResult.predictions.length > 3 &&
                                        "..."}
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            ) : null}
        </div>
    );
}
