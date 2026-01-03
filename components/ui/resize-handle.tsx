"use client";

import { useCallback, useRef, useState } from "react";
import { cn } from "@/lib/utils";

interface ResizeHandleProps {
    direction: "horizontal" | "vertical";
    onResize: (delta: number) => void;
    onResizeEnd?: () => void;
    className?: string;
}

const ResizeHandle = ({
    direction,
    onResize,
    onResizeEnd,
    className,
}: ResizeHandleProps) => {
    const [isDragging, setIsDragging] = useState<boolean>(false);
    const startPos = useRef(0);

    const handleMouseDown = useCallback(
        (e: React.MouseEvent) => {
            e.preventDefault();
            setIsDragging(true);

            startPos.current =
                direction === "horizontal" ? e.clientX : e.clientY;

            const handleMouseMove = (moveEvent: MouseEvent) => {
                const currentPos =
                    direction === "horizontal"
                        ? moveEvent.clientX
                        : moveEvent.clientY;
                const delta = currentPos - startPos.current;
                startPos.current = currentPos;
                onResize(delta);
            };
            const handleMouseUp = () => {
                setIsDragging(false);

                document.removeEventListener("mousemove", handleMouseMove);
                document.removeEventListener("mouseup", handleMouseUp);

                onResizeEnd?.();
            };
            document.addEventListener("mousemove", handleMouseMove);
            document.addEventListener("mouseup", handleMouseUp);
        },
        [direction, onResize, onResizeEnd],
    );

    return (
        <div
            className={cn(
                "bg-border shrink-0 transition-colors",
                "hover:bg-primary/50",
                direction === "horizontal"
                    ? "h-full w-1 cursor-col-resize"
                    : "h-1 w-full cursor-row-resize",
                isDragging && "bg-primary",
                className,
            )}
            onMouseDown={handleMouseDown}
        />
    );
};

export default ResizeHandle;
