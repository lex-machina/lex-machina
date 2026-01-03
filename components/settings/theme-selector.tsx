"use client";

import { useCallback } from "react";
import { Monitor, Sun, Moon, Check } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Theme } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface ThemeSelectorProps {
    /** Current theme value */
    value: Theme;
    /** Callback when theme changes */
    onChange: (theme: Theme) => void;
    /** Whether the selector is disabled */
    disabled?: boolean;
    /** Additional class names */
    className?: string;
}

// ============================================================================
// THEME OPTIONS
// ============================================================================

interface ThemeOption {
    value: Theme;
    label: string;
    description: string;
    icon: React.ReactNode;
}

const THEME_OPTIONS: ThemeOption[] = [
    {
        value: "system",
        label: "System",
        description: "Follow your operating system preference",
        icon: <Monitor className="h-5 w-5 shrink-0" />,
    },
    {
        value: "light",
        label: "Light",
        description: "Light background with dark text",
        icon: <Sun className="h-5 w-5 shrink-0" />,
    },
    {
        value: "dark",
        label: "Dark",
        description: "Dark background with light text",
        icon: <Moon className="h-5 w-5 shrink-0" />,
    },
];

// ============================================================================
// THEME OPTION BUTTON COMPONENT
// ============================================================================

interface ThemeOptionButtonProps {
    option: ThemeOption;
    isSelected: boolean;
    onClick: () => void;
    disabled?: boolean;
}

function ThemeOptionButton({
    option,
    isSelected,
    onClick,
    disabled,
}: ThemeOptionButtonProps) {
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
// THEME SELECTOR COMPONENT
// ============================================================================

/**
 * Theme selector component for choosing between System, Light, and Dark themes.
 *
 * Displays three radio-style buttons with icons and descriptions.
 * Follows desktop application patterns with immediate selection feedback.
 *
 * @example
 * ```tsx
 * const { theme, setTheme } = useSettings();
 *
 * <ThemeSelector
 *   value={theme}
 *   onChange={(newTheme) => setTheme(newTheme)}
 * />
 * ```
 *
 * @example
 * ```tsx
 * // With useTheme hook
 * const { theme, setTheme } = useTheme();
 *
 * <ThemeSelector
 *   value={theme}
 *   onChange={setTheme}
 * />
 * ```
 */
export function ThemeSelector({
    value,
    onChange,
    disabled = false,
    className,
}: ThemeSelectorProps) {
    const handleSelect = useCallback(
        (theme: Theme) => {
            if (!disabled && theme !== value) {
                onChange(theme);
            }
        },
        [disabled, value, onChange],
    );

    return (
        <div
            role="radiogroup"
            aria-label="Theme selection"
            className={cn("flex flex-col gap-2", className)}
            data-slot="theme-selector"
        >
            {THEME_OPTIONS.map((option) => (
                <ThemeOptionButton
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

export default ThemeSelector;
