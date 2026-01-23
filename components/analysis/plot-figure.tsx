"use client";

import { useEffect, useRef } from "react";
import * as Plot from "@observablehq/plot";

import { cn } from "@/lib/utils";

interface PlotFigureProps {
    options: Plot.PlotOptions;
    className?: string;
}

const PlotFigure = ({ options, className }: PlotFigureProps) => {
    const containerRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const container = containerRef.current;
        if (!container) {
            return;
        }

        container.innerHTML = "";
        const plot = Plot.plot(options);
        container.append(plot);

        return () => {
            plot.remove();
        };
    }, [options]);

    return <div ref={containerRef} className={cn("w-full", className)} />;
};

export default PlotFigure;
