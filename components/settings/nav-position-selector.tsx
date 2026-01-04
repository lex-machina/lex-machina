"use client";

import { useCallback } from "react";
import { PanelLeft, PanelRight, Columns2, Check } from "lucide-react";
import { cn } from "@/lib/utils";
import type { NavBarPosition } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface NavPositionSelectorProps {
    /** Current nav bar position value */
    value: NavBarPosition;
    /** Callback when position changes */
    onChange: (position: NavBarPosition) => void;
    /** Whether the selector is disabled */
    disabled?: boolean;
    /** Additional class names */
    className?: string;
}

// ============================================================================
// POSITION OPTIONS
// ============================================================================

interface PositionOption {
    value: NavBarPosition;
    label: string;
    description: string;
    icon: React.ReactNode;
}

const POSITION_OPTIONS: PositionOption[] = [
    {
        value: "merged",
        label: "Merged",
        description: "Navigation merged with context sidebar (default)",
        icon: <Columns2 className="h-5 w-5 shrink-0" />,
    },
    {
        value: "left",
        label: "Left",
        description: "Navigation bar on the left side",
        icon: <PanelLeft className="h-5 w-5 shrink-0" />,
    },
    {
        value: "right",
        label: "Right",
        description: "Navigation bar on the right side",
        icon: <PanelRight className="h-5 w-5 shrink-0" />,
    },
];

// ============================================================================
// POSITION OPTION BUTTON COMPONENT
// ============================================================================

interface PositionOptionButtonProps {
    option: PositionOption;
    isSelected: boolean;
    onClick: () => void;
    disabled?: boolean;
}

function PositionOptionButton({
    option,
    isSelected,
    onClick,
    disabled,
}: PositionOptionButtonProps) {
    return (
        <button
            type="button"
            role="radio"
            aria-checked={isSelected}
            onClick={onClick}
            disabled={disabled}
            className={cn(
                // Base styles
                "relative flex items-start gap-3 rounded-md p-3 text-left",
                "border transition-all duration-150",
                // Focus styles
                "focus-visible:ring-ring focus-visible:ring-offset-background focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2",
                // Selected state
                isSelected
                    ? "border-primary bg-primary/5"
                    : "border-border hover:border-muted-foreground/50 hover:bg-muted/50",
                // Disabled state
                disabled && "cursor-not-allowed opacity-50",
            )}
        >
            {/* Icon */}
            <div
                className={cn(
                    "flex h-10 w-10 items-center justify-center rounded-md",
                    isSelected
                        ? "bg-primary text-primary-foreground"
                        : "bg-muted text-muted-foreground",
                )}
            >
                {option.icon}
            </div>

            {/* Content */}
            <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                <span
                    className={cn(
                        "text-sm font-medium",
                        isSelected ? "text-foreground" : "text-foreground",
                    )}
                >
                    {option.label}
                </span>
                <span className="text-muted-foreground text-xs">
                    {option.description}
                </span>
            </div>

            {/* Check indicator */}
            {isSelected && (
                <div className="text-primary absolute top-3 right-3">
                    <Check className="h-4 w-4 shrink-0" />
                </div>
            )}
        </button>
    );
}

// ============================================================================
// NAV POSITION SELECTOR COMPONENT
// ============================================================================

/**
 * Navigation position selector component for choosing nav bar placement.
 *
 * Displays three radio-style buttons with icons and descriptions.
 * Follows desktop application patterns with immediate selection feedback.
 *
 * Options:
 * - **Merged**: Navigation icons appear at the top of the right sidebar (default)
 * - **Left**: Traditional left navigation bar (always visible vertical strip)
 * - **Right**: Right navigation bar (always visible vertical strip)
 *
 * Note: Only "merged" mode is fully implemented. Left/Right modes are planned
 * for a future phase but the UI is ready.
 *
 * @example
 * ```tsx
 * const { navBarPosition, setNavBarPosition } = useSidebar();
 *
 * <NavPositionSelector
 *   value={navBarPosition}
 *   onChange={(newPosition) => setNavBarPosition(newPosition)}
 * />
 * ```
 */
export function NavPositionSelector({
    value,
    onChange,
    disabled = false,
    className,
}: NavPositionSelectorProps) {
    const handleSelect = useCallback(
        (position: NavBarPosition) => {
            if (!disabled && position !== value) {
                onChange(position);
            }
        },
        [disabled, value, onChange],
    );

    return (
        <div
            role="radiogroup"
            aria-label="Navigation position selection"
            className={cn("flex flex-col gap-2", className)}
            data-slot="nav-position-selector"
        >
            {POSITION_OPTIONS.map((option) => (
                <PositionOptionButton
                    key={option.value}
                    option={option}
                    isSelected={value === option.value}
                    onClick={() => handleSelect(option.value)}
                    disabled={disabled}
                />
            ))}
        </div>
    );
}

export default NavPositionSelector;
