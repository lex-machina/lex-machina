"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import * as Plot from "@observablehq/plot";

import { cn } from "@/lib/utils";

interface PlotFigureProps {
    options: Plot.PlotOptions | ((width: number) => Plot.PlotOptions);
    className?: string;
    autoHeight?: boolean;
}

const PlotFigure = ({
    options,
    className,
    autoHeight = false,
}: PlotFigureProps) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const [width, setWidth] = useState<number | null>(null);
    const [height, setHeight] = useState<number | null>(null);

    const resolvedOptions = useMemo(() => {
        if (width === null) {
            return null;
        }
        const baseOptions =
            typeof options === "function" ? options(width) : options;
        const resolvedHeight = autoHeight ? height : null;
        const styleOverrides = {
            "--plot-background": "var(--popover)",
            color: "var(--foreground)",
        };
        const mergedStyle =
            typeof baseOptions.style === "string"
                ? mergeStyleString(baseOptions.style, styleOverrides)
                : {
                      ...(baseOptions.style ?? {}),
                      ...styleOverrides,
                  };

        return {
            ...baseOptions,
            width,
            ...(resolvedHeight ? { height: resolvedHeight } : null),
            style: mergedStyle,
        } as Plot.PlotOptions;
    }, [options, width, height, autoHeight]);

    useEffect(() => {
        const container = containerRef.current;
        if (!container) {
            return;
        }

        const observer = new ResizeObserver((entries) => {
            const entry = entries[0];
            if (!entry) return;
            const nextWidth = Math.floor(entry.contentRect.width);
            if (nextWidth > 0) {
                setWidth(nextWidth);
            }
            if (autoHeight) {
                const nextHeight = Math.floor(entry.contentRect.height);
                if (nextHeight > 0) {
                    setHeight(nextHeight);
                }
            }
        });

        observer.observe(container);

        return () => {
            observer.disconnect();
        };
    }, [autoHeight]);

    useEffect(() => {
        const container = containerRef.current;
        if (!container || !resolvedOptions) {
            return;
        }

        container.innerHTML = "";
        const plot = Plot.plot(resolvedOptions);
        container.append(plot);

        return () => {
            plot.remove();
        };
    }, [resolvedOptions]);

    return <div ref={containerRef} className={cn("w-full", className)} />;
};

const mergeStyleString = (
    baseStyle: string,
    overrides: Record<string, string>,
) => {
    const normalized = baseStyle.trim();
    const suffix =
        normalized.length > 0 && !normalized.endsWith(";") ? ";" : "";
    const overrideString = Object.entries(overrides)
        .map(([key, value]) => `${key}: ${value}`)
        .join("; ");
    return `${normalized}${suffix} ${overrideString}`.trim();
};

export default PlotFigure;
