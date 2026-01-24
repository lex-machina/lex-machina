"use client";

import * as Plot from "@observablehq/plot";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { formatNumber } from "@/lib/utils";
import type { AnalysisColumnStats, CategoryCount } from "@/types";

import PlotFigure from "./plot-figure";

interface AnalysisColumnDetailProps {
    column: AnalysisColumnStats | null;
}

const AnalysisColumnDetail = ({ column }: AnalysisColumnDetailProps) => {
    if (!column) {
        return (
            <Card className="h-full">
                <CardHeader title="Column Detail" />
                <CardContent className="flex h-full items-center justify-center text-sm">
                    <span className="text-muted-foreground">
                        Select a column to explore detailed statistics.
                    </span>
                </CardContent>
            </Card>
        );
    }

    const { profile } = column;

    return (
        <Card className="h-full">
            <CardHeader title={`Column: ${profile.name}`} />
            <CardContent className="flex h-full flex-col gap-4" padded>
                <div className="flex flex-wrap gap-2 text-xs">
                    <span className="bg-muted text-muted-foreground rounded-md px-2 py-1">
                        {profile.inferred_type}
                    </span>
                    <span className="bg-muted text-muted-foreground rounded-md px-2 py-1">
                        {profile.inferred_role}
                    </span>
                    <span className="bg-muted text-muted-foreground rounded-md px-2 py-1">
                        {profile.null_percentage.toFixed(1)}% null
                    </span>
                </div>
                <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                        <div className="text-muted-foreground">Type</div>
                        <div className="font-medium">{profile.dtype}</div>
                    </div>
                    <div>
                        <div className="text-muted-foreground">Inferred</div>
                        <div className="font-medium">
                            {profile.inferred_type}
                        </div>
                    </div>
                    <div>
                        <div className="text-muted-foreground">Role</div>
                        <div className="font-medium">
                            {profile.inferred_role}
                        </div>
                    </div>
                    <div>
                        <div className="text-muted-foreground">Unique</div>
                        <div className="font-medium">
                            {formatNumber(profile.unique_count)}
                        </div>
                    </div>
                    <div>
                        <div className="text-muted-foreground">Nulls</div>
                        <div className="font-medium">
                            {formatNumber(profile.null_count)} (
                            {profile.null_percentage.toFixed(1)}%)
                        </div>
                    </div>
                </div>

                {profile.sample_values.length > 0 && (
                    <section className="space-y-2">
                        <h3 className="text-muted-foreground text-xs font-semibold uppercase">
                            Sample Values
                        </h3>
                        <div className="flex flex-wrap gap-2 text-xs">
                            {profile.sample_values
                                .slice(0, 12)
                                .map((value, index) => (
                                    <span
                                        key={`${value}-${index}`}
                                        className="bg-muted text-foreground rounded-md px-2 py-1"
                                    >
                                        {value}
                                    </span>
                                ))}
                        </div>
                    </section>
                )}

                {Object.keys(profile.characteristics).length > 0 && (
                    <section className="space-y-2">
                        <h3 className="text-muted-foreground text-xs font-semibold uppercase">
                            Characteristics
                        </h3>
                        <div className="space-y-1 text-sm">
                            {Object.entries(profile.characteristics).map(
                                ([key, value]) => (
                                    <div
                                        key={key}
                                        className="flex items-center justify-between"
                                    >
                                        <span className="text-muted-foreground">
                                            {key}
                                        </span>
                                        <span className="font-medium">
                                            {formatCharacteristicValue(value)}
                                        </span>
                                    </div>
                                ),
                            )}
                        </div>
                    </section>
                )}

                {column.numeric && <NumericDetail stats={column.numeric} />}
                {column.categorical && (
                    <CategoricalDetail stats={column.categorical} />
                )}
                {column.text && <TextDetail stats={column.text} />}
                {column.datetime && <DatetimeDetail stats={column.datetime} />}
            </CardContent>
        </Card>
    );
};

const NumericDetail = ({
    stats,
}: {
    stats: AnalysisColumnStats["numeric"];
}) => {
    if (!stats) {
        return null;
    }

    const histogramData = stats.histogram.map((bin) => ({
        start: bin.start,
        end: bin.end,
        count: bin.count,
        range: formatRange(bin.start, bin.end),
    }));

    const histogramOptions: Plot.PlotOptions = {
        height: 160,
        marginLeft: 48,
        x: { label: null },
        y: { label: null, grid: true },
        color: { scheme: "greys" },
        marks: [
            Plot.rectY(histogramData, {
                x1: "start",
                x2: "end",
                y: "count",
                fill: "count",
                channels: { range: "range", count: "count" },
                tip: {
                    format: {
                        range: (value) => value,
                        count: (value) => formatCount(value),
                    },
                },
            }),
        ],
    };

    const boxPlotOptions: Plot.PlotOptions = {
        height: 90,
        marginLeft: 48,
        x: { label: null },
        y: { label: null },
        marks: [
            Plot.ruleX([stats.min, stats.max], { y: 0, stroke: "#9ca3af" }),
            Plot.rect(
                [
                    {
                        x1: stats.box_plot.q1,
                        x2: stats.box_plot.q3,
                        y1: -0.35,
                        y2: 0.35,
                    },
                ],
                {
                    x1: "x1",
                    x2: "x2",
                    y1: "y1",
                    y2: "y2",
                    fill: "#cbd5e1",
                    stroke: "#94a3b8",
                },
            ),
            Plot.ruleX([stats.box_plot.median], {
                stroke: "#475569",
                strokeWidth: 2,
            }),
        ],
    };

    return (
        <section className="space-y-3">
            <h3 className="text-muted-foreground text-xs font-semibold uppercase">
                Numeric Distribution
            </h3>
            <div className="grid grid-cols-2 gap-3 text-sm">
                <Metric label="Min" value={stats.min.toFixed(4)} />
                <Metric label="Max" value={stats.max.toFixed(4)} />
                <Metric label="Mean" value={stats.mean.toFixed(4)} />
                <Metric label="Median" value={stats.median.toFixed(4)} />
                <Metric label="Std Dev" value={stats.std_dev.toFixed(4)} />
                <Metric label="IQR" value={stats.iqr.toFixed(4)} />
                <Metric label="Skew" value={stats.skewness.toFixed(3)} />
                <Metric label="Kurtosis" value={stats.kurtosis.toFixed(3)} />
                <Metric label="Outliers (IQR)" value={stats.outliers_iqr} />
                <Metric
                    label="Outliers (Robust Z)"
                    value={stats.outliers_robust_z}
                />
            </div>
            <PlotFigure options={histogramOptions} />
            <PlotFigure options={boxPlotOptions} />

            {stats.normality_tests.length > 0 && (
                <div className="space-y-2 text-sm">
                    <div className="text-muted-foreground text-xs font-semibold uppercase">
                        Normality Tests
                    </div>
                    <div className="space-y-1">
                        {stats.normality_tests.map((test) => (
                            <div
                                key={test.test}
                                className="flex items-center justify-between"
                            >
                                <span>{test.test}</span>
                                <span className="text-muted-foreground">
                                    p={test.p_value.toFixed(4)}
                                </span>
                            </div>
                        ))}
                    </div>
                </div>
            )}
        </section>
    );
};

const CategoricalDetail = ({
    stats,
}: {
    stats: AnalysisColumnStats["categorical"];
}) => {
    if (!stats) {
        return null;
    }

    const barData = stats.top_values.map((value) => ({
        label: value.value,
        count: value.count,
        percentage: value.percentage,
    }));

    const barOptions: Plot.PlotOptions = {
        height: 160,
        marginLeft: 100,
        x: { label: null },
        y: { label: null },
        marks: [
            Plot.barX(barData, {
                x: "count",
                y: "label",
                fill: "#9ca3af",
                channels: { percentage: "percentage" },
                tip: {
                    format: {
                        label: (value) => value,
                        count: (value) => formatCount(value),
                        percentage: (value) => formatPercentValue(value, 1),
                    },
                },
            }),
        ],
    };

    return (
        <section className="space-y-3">
            <h3 className="text-muted-foreground text-xs font-semibold uppercase">
                Categorical Distribution
            </h3>
            <div className="grid grid-cols-2 gap-3 text-sm">
                <Metric label="Cardinality" value={stats.cardinality} />
                <Metric label="Entropy" value={stats.entropy.toFixed(3)} />
                <Metric label="Gini" value={stats.gini.toFixed(3)} />
                <Metric
                    label="Imbalance"
                    value={stats.imbalance_ratio.toFixed(2)}
                />
            </div>
            <PlotFigure options={barOptions} />
            <CategoryTable entries={stats.top_values} />
        </section>
    );
};

const TextDetail = ({ stats }: { stats: AnalysisColumnStats["text"] }) => {
    if (!stats) {
        return null;
    }

    const lengthHistogramData = stats.length_histogram.map((bin) => ({
        start: bin.start,
        end: bin.end,
        count: bin.count,
        range: formatRange(bin.start, bin.end),
    }));

    const histogramOptions: Plot.PlotOptions = {
        height: 140,
        marginLeft: 48,
        x: { label: null },
        y: { label: null, grid: true },
        marks: [
            Plot.rectY(lengthHistogramData, {
                x1: "start",
                x2: "end",
                y: "count",
                fill: "#9ca3af",
                channels: { range: "range", count: "count" },
                tip: {
                    format: {
                        range: (value) => value,
                        count: (value) => formatCount(value),
                    },
                },
            }),
        ],
    };

    return (
        <section className="space-y-3">
            <h3 className="text-muted-foreground text-xs font-semibold uppercase">
                Text Characteristics
            </h3>
            <div className="grid grid-cols-2 gap-3 text-sm">
                <Metric label="Min Length" value={stats.min_length} />
                <Metric label="Max Length" value={stats.max_length} />
                <Metric
                    label="Mean Length"
                    value={stats.mean_length.toFixed(2)}
                />
                <Metric
                    label="Median Length"
                    value={stats.median_length.toFixed(2)}
                />
                <Metric
                    label="Empty %"
                    value={`${stats.empty_percentage.toFixed(2)}%`}
                />
                <Metric
                    label="Whitespace %"
                    value={`${stats.whitespace_percentage.toFixed(2)}%`}
                />
                <Metric
                    label="Unique Tokens"
                    value={formatNumber(stats.unique_token_count)}
                />
            </div>
            <PlotFigure options={histogramOptions} />
        </section>
    );
};

const DatetimeDetail = ({
    stats,
}: {
    stats: AnalysisColumnStats["datetime"];
}) => {
    if (!stats) {
        return null;
    }

    const timeOptions: Plot.PlotOptions = {
        height: 140,
        marginLeft: 48,
        x: { label: null },
        y: { label: null, grid: true },
        marks: [
            Plot.barY(stats.time_bins, {
                x: "label",
                y: "count",
                fill: "#94a3b8",
                tip: {
                    format: {
                        label: (value) => value,
                        count: (value) => formatCount(value),
                    },
                },
            }),
        ],
    };

    return (
        <section className="space-y-3">
            <h3 className="text-muted-foreground text-xs font-semibold uppercase">
                Date/Time Coverage
            </h3>
            <div className="grid grid-cols-2 gap-3 text-sm">
                <Metric label="Min" value={stats.min} />
                <Metric label="Max" value={stats.max} />
                <Metric
                    label="Range (days)"
                    value={stats.range_days.toFixed(2)}
                />
                <Metric label="Granularity" value={stats.granularity} />
            </div>
            <PlotFigure options={timeOptions} />
        </section>
    );
};

const Metric = ({
    label,
    value,
}: {
    label: string;
    value: string | number;
}) => (
    <div>
        <div className="text-muted-foreground text-xs">{label}</div>
        <div className="font-medium">{value}</div>
    </div>
);

const CategoryTable = ({ entries }: { entries: CategoryCount[] }) => (
    <div className="space-y-1 text-sm">
        {entries.map((entry) => (
            <div
                key={entry.value}
                className="flex items-center justify-between"
            >
                <span className="truncate">{entry.value}</span>
                <span className="text-muted-foreground">
                    {formatNumber(entry.count)} ({entry.percentage.toFixed(1)}%)
                </span>
            </div>
        ))}
    </div>
);

const formatCharacteristicValue = (value: unknown) => {
    if (value === null || value === undefined) {
        return "—";
    }
    if (typeof value === "string") {
        return value;
    }
    if (typeof value === "number") {
        return value.toFixed(3);
    }
    if (typeof value === "boolean") {
        return value ? "true" : "false";
    }
    return JSON.stringify(value);
};

const formatCount = (value: unknown) =>
    typeof value === "number" ? formatNumber(value) : "—";

const formatPercentValue = (value: unknown, decimals = 1) =>
    typeof value === "number" ? `${value.toFixed(decimals)}%` : "—";

const formatNumericValue = (value: number) => {
    if (!Number.isFinite(value)) {
        return "—";
    }
    return Number.isInteger(value) ? formatNumber(value) : value.toFixed(2);
};

const formatRange = (start: number, end: number) =>
    `${formatNumericValue(start)} - ${formatNumericValue(end)}`;

export default AnalysisColumnDetail;
