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

      startPos.current = direction === "horizontal" ? e.clientX : e.clientY;

      const handleMouseMove = (moveEvent: MouseEvent) => {
        const currentPos =
          direction === "horizontal" ? moveEvent.clientX : moveEvent.clientY;
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
        "shrink-0 bg-border transition-colors",
        "hover:bg-primary/50",
        direction === "horizontal"
          ? "w-1 cursor-col-resize h-full"
          : "h-1 cursor-row-resize w-full",
        isDragging && "bg-primary",
        className,
      )}
      onMouseDown={handleMouseDown}
    />
  );
};

export default ResizeHandle;
