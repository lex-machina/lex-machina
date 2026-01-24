"use client";

import { useMemo } from "react";
import * as Plot from "@observablehq/plot";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Select } from "@/components/ui/select";
import { formatNumber } from "@/lib/utils";
import type {
    VisualizationChart,
    VisualizationChartKind,
    VisualizationPieSlice,
} from "@/types";

import PlotFigure from "@/components/analysis/plot-figure";

interface VisualizationCardProps {
    chart: VisualizationChart;
    accentIndex: number;
    selectedKind?: VisualizationChartKind;
    onChartKindChange: (column: string, kind: VisualizationChartKind) => void;
}

const VisualizationCard = ({
    chart,
    accentIndex,
    selectedKind,
    onChartKindChange,
}: VisualizationCardProps) => {
    const resolvedKind = selectedKind ?? chart.kind;
    const {
        options,
        hasData,
        legendLabel,
        accentColor,
        renderMode,
        pieSlices,
        palette,
    } = useMemo(() => {
        const palette = rotatePalette(CHART_PALETTE, accentIndex);
        const gradientRange = [
            palette[2] ?? palette[0],
            palette[0],
            palette[1] ?? palette[0],
        ];
        switch (resolvedKind) {
            case "histogram": {
                const bins = chart.histogram ?? [];
                const data = bins.map((bin) => ({
                    start: bin.start,
                    end: bin.end,
                    count: bin.count,
                    range: formatRange(bin.start, bin.end),
                }));
                return {
                    hasData: data.length > 0,
                    legendLabel: `${chart.inferred_type} · Bins ${data.length}`,
                    accentColor: palette[0],
                    renderMode: "plot",
                    palette,
                    options: {
                        height: 220,
                        marginLeft: 48,
                        marginBottom: 42,
                        x: { label: null },
                        y: { label: null, grid: true },
                        color: { range: gradientRange, legend: false },
                        marks: [
                            Plot.rectY(data, {
                                x1: "start",
                                x2: "end",
                                y: "count",
                                fill: "count",
                                fillOpacity: 0.9,
                                stroke: "var(--border)",
                                strokeWidth: 0.5,
                                channels: {
                                    range: "range",
                                    count: "count",
                                },
                                tip: {
                                    format: {
                                        range: (value) => value,
                                        count: (value) => formatCount(value),
                                    },
                                },
                            }),
                        ],
                    } satisfies Plot.PlotOptions,
                };
            }
            case "time": {
                const bins = chart.time_bins ?? [];
                const data = bins.map((bin) => ({
                    label: bin.label,
                    count: bin.count,
                }));
                return {
                    hasData: data.length > 0,
                    legendLabel: `${chart.inferred_type} · Buckets ${data.length}`,
                    accentColor: palette[0],
                    renderMode: "plot",
                    palette,
                    options: {
                        height: 220,
                        marginLeft: 48,
                        marginBottom: 52,
                        x: { label: null, tickRotate: 30 },
                        y: { label: null, grid: true },
                        color: { range: gradientRange, legend: false },
                        marks: [
                            Plot.barY(data, {
                                x: "label",
                                y: "count",
                                fill: "count",
                                fillOpacity: 0.9,
                                stroke: "var(--border)",
                                strokeWidth: 0.5,
                                tip: {
                                    format: {
                                        label: (value) => value,
                                        count: (value) => formatCount(value),
                                    },
                                },
                            }),
                        ],
                    } satisfies Plot.PlotOptions,
                };
            }
            case "line": {
                const histogramBins = chart.histogram ?? [];
                const timeBins = chart.time_bins ?? [];
                if (histogramBins.length > 0) {
                    const data = histogramBins.map((bin) => ({
                        center: (bin.start + bin.end) / 2,
                        count: bin.count,
                    }));
                    return {
                        hasData: data.length > 0,
                        legendLabel: `${chart.inferred_type} · Bins ${data.length}`,
                        accentColor: palette[1] ?? palette[0],
                        renderMode: "plot",
                        palette,
                        options: {
                            height: 220,
                            marginLeft: 48,
                            marginBottom: 42,
                            x: { label: null },
                            y: { label: null, grid: true },
                            marks: [
                                Plot.lineY(data, {
                                    x: "center",
                                    y: "count",
                                    stroke: palette[1] ?? palette[0],
                                    strokeWidth: 2,
                                }),
                                Plot.dot(data, {
                                    x: "center",
                                    y: "count",
                                    fill: palette[1] ?? palette[0],
                                    r: 2.5,
                                }),
                            ],
                        } satisfies Plot.PlotOptions,
                    };
                }

                const data = timeBins.map((bin) => ({
                    label: bin.label,
                    count: bin.count,
                }));
                return {
                    hasData: data.length > 0,
                    legendLabel: `${chart.inferred_type} · Buckets ${data.length}`,
                    accentColor: palette[1] ?? palette[0],
                    renderMode: "plot",
                    palette,
                    options: {
                        height: 220,
                        marginLeft: 48,
                        marginBottom: 52,
                        x: { label: null, tickRotate: 30 },
                        y: { label: null, grid: true },
                        marks: [
                            Plot.lineY(data, {
                                x: "label",
                                y: "count",
                                stroke: palette[1] ?? palette[0],
                                strokeWidth: 2,
                            }),
                            Plot.dot(data, {
                                x: "label",
                                y: "count",
                                fill: palette[1] ?? palette[0],
                                r: 2.5,
                            }),
                        ],
                    } satisfies Plot.PlotOptions,
                };
            }
            case "pie": {
                const slices = chart.pie_slices ?? [];
                return {
                    hasData: slices.length > 0,
                    legendLabel: `${chart.inferred_type} · ${slices.length} slices`,
                    accentColor: palette[0],
                    renderMode: "pie",
                    pieSlices: slices,
                    palette,
                    options: { height: 220 } satisfies Plot.PlotOptions,
                };
            }
            case "column":
            case "bar":
            default: {
                const categories = chart.categories ?? [];
                const data = categories.map((entry) => ({
                    label: entry.value,
                    count: entry.count,
                    percentage: entry.percentage,
                }));
                const isColumn = resolvedKind === "column";
                return {
                    hasData: data.length > 0,
                    legendLabel: `${chart.inferred_type} · Top ${data.length}`,
                    accentColor: palette[0],
                    renderMode: "plot",
                    palette,
                    options: {
                        height: 220,
                        marginLeft: isColumn ? 48 : 80,
                        marginBottom: isColumn ? 52 : 32,
                        x: { label: null, tickRotate: isColumn ? 30 : 0 },
                        y: { label: null },
                        color: { range: palette, legend: false },
                        marks: [
                            isColumn
                                ? Plot.barY(data, {
                                      x: "label",
                                      y: "count",
                                      fill: "label",
                                      fillOpacity: 0.85,
                                      stroke: "var(--border)",
                                      strokeWidth: 0.5,
                                      tip: {
                                          format: {
                                              label: (value) => value,
                                              count: (value) =>
                                                  formatCount(value),
                                          },
                                      },
                                  })
                                : Plot.barX(data, {
                                      x: "count",
                                      y: "label",
                                      fill: "label",
                                      fillOpacity: 0.85,
                                      stroke: "var(--border)",
                                      strokeWidth: 0.5,
                                      channels: {
                                          percentage: "percentage",
                                      },
                                      tip: {
                                          format: {
                                              label: (value) => value,
                                              count: (value) =>
                                                  formatCount(value),
                                              percentage: (value) =>
                                                  formatPercentValue(value, 1),
                                          },
                                      },
                                  }),
                        ],
                    } satisfies Plot.PlotOptions,
                };
            }
        }
    }, [chart, accentIndex, resolvedKind]);

    const chartKindOptions = chart.available_kinds.map((kind) => ({
        value: kind,
        label: CHART_KIND_LABELS[kind] ?? kind,
    }));
    const resolvedPalette = palette ?? CHART_PALETTE;
    const pieLegendItems =
        renderMode === "pie" ? (pieSlices ?? []).slice(0, 6) : [];
    const pieExtraCount =
        renderMode === "pie"
            ? (pieSlices?.length ?? 0) - pieLegendItems.length
            : 0;

    return (
        <Card className="min-h-0">
            <CardHeader
                title={chart.title}
                actions={
                    <Select
                        value={resolvedKind}
                        onValueChange={(value) =>
                            onChartKindChange(
                                chart.column,
                                value as VisualizationChartKind,
                            )
                        }
                        options={chartKindOptions}
                        className="w-36"
                    />
                }
            />
            <CardContent padded className="flex flex-col gap-3">
                <div className="text-muted-foreground text-xs">
                    <span>{legendLabel}</span>
                </div>
                {renderMode === "pie" && pieLegendItems.length > 0 && (
                    <div className="text-muted-foreground flex flex-wrap gap-x-3 gap-y-1 text-xs">
                        {pieLegendItems.map((slice, index) => (
                            <span
                                key={`${slice.label}-${index}`}
                                className="flex items-center gap-1"
                            >
                                <span
                                    className="h-2 w-2 rounded-full"
                                    style={{
                                        backgroundColor:
                                            resolvedPalette[
                                                slice.color_index %
                                                    resolvedPalette.length
                                            ],
                                    }}
                                />
                                <span className="max-w-[120px] truncate">
                                    {slice.label}
                                </span>
                                <span>{slice.percentage.toFixed(1)}%</span>
                            </span>
                        ))}
                        {pieExtraCount > 0 && (
                            <span className="text-muted-foreground">
                                +{pieExtraCount} more
                            </span>
                        )}
                    </div>
                )}
                {hasData ? (
                    <div className="w-full" style={{ aspectRatio: "1 / 1" }}>
                        {renderMode === "pie" ? (
                            <PieChart
                                slices={pieSlices ?? []}
                                palette={palette ?? CHART_PALETTE}
                            />
                        ) : (
                            <PlotFigure
                                options={options}
                                autoHeight
                                className="h-full w-full"
                            />
                        )}
                    </div>
                ) : (
                    <div className="text-muted-foreground flex h-full items-center justify-center text-xs">
                        No data available for this chart.
                    </div>
                )}
                <div className="text-muted-foreground flex flex-wrap gap-3 text-xs">
                    <span>Unique {formatNumber(chart.unique_count)}</span>
                    <span>Nulls {chart.null_percentage.toFixed(1)}%</span>
                </div>
            </CardContent>
        </Card>
    );
};

const PieChart = ({
    slices,
    palette,
}: {
    slices: VisualizationPieSlice[];
    palette: string[];
}) => {
    const size = 200;
    const radius = 80;
    const center = size / 2;

    const toPoint = (angle: number) => {
        const adjusted = angle - Math.PI / 2;
        return {
            x: center + radius * Math.cos(adjusted),
            y: center + radius * Math.sin(adjusted),
        };
    };

    const arcPath = (start: number, end: number) => {
        const startPoint = toPoint(start);
        const endPoint = toPoint(end);
        const largeArc = end - start > Math.PI ? 1 : 0;
        return `M ${center} ${center} L ${startPoint.x} ${startPoint.y} A ${radius} ${radius} 0 ${largeArc} 1 ${endPoint.x} ${endPoint.y} Z`;
    };

    return (
        <svg viewBox={`0 0 ${size} ${size}`} className="h-full w-full">
            {slices.map((slice, index) => (
                <path
                    key={`${slice.label}-${index}`}
                    d={arcPath(slice.start_angle, slice.end_angle)}
                    fill={palette[slice.color_index % palette.length]}
                    stroke="var(--border)"
                    strokeWidth={0.6}
                >
                    <title>
                        {`${slice.label}: ${slice.value} (${slice.percentage.toFixed(1)}%)`}
                    </title>
                </path>
            ))}
        </svg>
    );
};

const formatCount = (value: unknown) =>
    typeof value === "number" ? formatNumber(value) : "—";

const formatPercentValue = (value: unknown, decimals = 1) =>
    typeof value === "number" ? `${value.toFixed(decimals)}%` : "—";

const formatRange = (start: number, end: number) =>
    `${start.toFixed(2)} - ${end.toFixed(2)}`;

const CHART_PALETTE = [
    "var(--chart-1)",
    "var(--chart-2)",
    "var(--chart-3)",
    "var(--chart-4)",
    "var(--chart-5)",
];

const CHART_KIND_LABELS: Record<VisualizationChart["kind"], string> = {
    histogram: "Histogram",
    bar: "Bar",
    time: "Time Bars",
    line: "Line",
    column: "Columns",
    pie: "Pie",
};

const rotatePalette = (palette: string[], offset: number) => {
    if (palette.length === 0) {
        return palette;
    }
    const shift = ((offset % palette.length) + palette.length) % palette.length;
    return palette.slice(shift).concat(palette.slice(0, shift));
};

export default VisualizationCard;
