"use client";

import { useCallback, useRef, useState } from "react";
import { cn } from "@/lib/utils";

interface ScrollbarProps {
    direction: "vertical" | "horizontal";
    totalSize: number;
    currentPosition: number;
    visibleSize: number;
    onSeek: (position: number) => void;
    containerSize: number;
    hideWhenFits?: boolean;
}

const SCROLLBAR_SIZE = 10;
const SCROLLBAR_SIZE_HOVER = 12;
const MIN_THUMB_SIZE = 20;

const Scrollbar = ({
    direction,
    totalSize,
    currentPosition,
    visibleSize,
    onSeek,
    containerSize,
    hideWhenFits = false,
}: ScrollbarProps) => {
    const trackRef = useRef<HTMLDivElement>(null);
    const [isHovered, setIsHovered] = useState(false);
    const [isDragging, setIsDragging] = useState(false);

    const isVertical = direction === "vertical";

    // Calculate thumb size proportional to visible content
    const maxPosition = Math.max(0, totalSize - visibleSize);
    const thumbRatio = totalSize > 0 ? visibleSize / totalSize : 1;
    const thumbSize = Math.max(MIN_THUMB_SIZE, thumbRatio * containerSize);

    // Calculate thumb position
    const trackSpace = containerSize - thumbSize;
    const thumbPosition =
        maxPosition > 0 ? (currentPosition / maxPosition) * trackSpace : 0;

    // Convert mouse position to content position
    const positionFromMouse = useCallback(
        (clientX: number, clientY: number) => {
            const track = trackRef.current;
            if (!track) return 0;

            const rect = track.getBoundingClientRect();
            const mousePos = isVertical
                ? clientY - rect.top
                : clientX - rect.left;
            const pos = mousePos - thumbSize / 2;
            const ratio = Math.max(0, Math.min(1, pos / trackSpace));
            return Math.round(ratio * maxPosition);
        },
        [isVertical, thumbSize, trackSpace, maxPosition],
    );

    // Handle track click - jump directly to position
    const handleTrackClick = useCallback(
        (e: React.MouseEvent) => {
            if ((e.target as HTMLElement).dataset.thumb) return;
            const position = positionFromMouse(e.clientX, e.clientY);
            onSeek(position);
        },
        [positionFromMouse, onSeek],
    );

    // Handle thumb drag
    const handleThumbMouseDown = useCallback(
        (e: React.MouseEvent) => {
            e.preventDefault();
            e.stopPropagation();
            setIsDragging(true);

            const startPos = isVertical ? e.clientY : e.clientX;
            const startPosition = currentPosition;

            const handleMouseMove = (moveEvent: MouseEvent) => {
                const currentPos = isVertical
                    ? moveEvent.clientY
                    : moveEvent.clientX;
                const delta = currentPos - startPos;
                const deltaRatio = trackSpace > 0 ? delta / trackSpace : 0;
                const deltaPosition = Math.round(deltaRatio * maxPosition);
                const newPosition = Math.max(
                    0,
                    Math.min(maxPosition, startPosition + deltaPosition),
                );
                onSeek(newPosition);
            };

            const handleMouseUp = () => {
                setIsDragging(false);
                document.removeEventListener("mousemove", handleMouseMove);
                document.removeEventListener("mouseup", handleMouseUp);
            };

            document.addEventListener("mousemove", handleMouseMove);
            document.addEventListener("mouseup", handleMouseUp);
        },
        [isVertical, currentPosition, trackSpace, maxPosition, onSeek],
    );

    // Hide if content fits and hideWhenFits is true
    if (hideWhenFits && totalSize <= visibleSize) {
        return null;
    }

    const size =
        isHovered || isDragging ? SCROLLBAR_SIZE_HOVER : SCROLLBAR_SIZE;

    return (
        <div
            ref={trackRef}
            className={cn(
                "bg-muted/50 absolute z-10 cursor-pointer transition-all duration-150",
                isDragging && "bg-muted",
                isVertical ? "top-0 right-0" : "bottom-0 left-0",
            )}
            style={
                isVertical
                    ? { width: size, height: containerSize }
                    : { height: size, width: containerSize }
            }
            onClick={handleTrackClick}
            onMouseEnter={() => setIsHovered(true)}
            onMouseLeave={() => setIsHovered(false)}
        >
            {/* Thumb */}
            <div
                data-thumb="true"
                className={cn(
                    "absolute cursor-grab rounded-full transition-colors duration-150",
                    "bg-muted-foreground/50 hover:bg-muted-foreground/70",
                    isDragging && "bg-muted-foreground/80 cursor-grabbing",
                    isVertical ? "right-0 left-0" : "top-0 bottom-0",
                )}
                style={
                    isVertical
                        ? { height: thumbSize, top: thumbPosition }
                        : { width: thumbSize, left: thumbPosition }
                }
                onMouseDown={handleThumbMouseDown}
            />
        </div>
    );
};

export default Scrollbar;
