"use client";

import { useCallback } from "react";
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
    icon: <SystemIcon />,
  },
  {
    value: "light",
    label: "Light",
    description: "Light background with dark text",
    icon: <SunIcon />,
  },
  {
    value: "dark",
    label: "Dark",
    description: "Dark background with light text",
    icon: <MoonIcon />,
  },
];

// ============================================================================
// ICONS
// ============================================================================

function SystemIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0"
    >
      <rect x="2" y="3" width="20" height="14" rx="2" />
      <line x1="8" x2="16" y1="21" y2="21" />
      <line x1="12" x2="12" y1="17" y2="21" />
    </svg>
  );
}

function SunIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0"
    >
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2v2" />
      <path d="M12 20v2" />
      <path d="m4.93 4.93 1.41 1.41" />
      <path d="m17.66 17.66 1.41 1.41" />
      <path d="M2 12h2" />
      <path d="M20 12h2" />
      <path d="m6.34 17.66-1.41 1.41" />
      <path d="m19.07 4.93-1.41 1.41" />
    </svg>
  );
}

function MoonIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0"
    >
      <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0"
    >
      <path d="M20 6 9 17l-5-5" />
    </svg>
  );
}

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
        "relative flex items-start gap-3 p-3 rounded-md text-left",
        "border transition-all duration-150",
        // Focus styles
        "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
        // Selected state
        isSelected
          ? "border-primary bg-primary/5"
          : "border-border hover:border-muted-foreground/50 hover:bg-muted/50",
        // Disabled state
        disabled && "cursor-not-allowed opacity-50"
      )}
    >
      {/* Icon */}
      <div
        className={cn(
          "flex items-center justify-center w-10 h-10 rounded-md",
          isSelected
            ? "bg-primary text-primary-foreground"
            : "bg-muted text-muted-foreground"
        )}
      >
        {option.icon}
      </div>

      {/* Content */}
      <div className="flex flex-col gap-0.5 min-w-0 flex-1">
        <span
          className={cn(
            "text-sm font-medium",
            isSelected ? "text-foreground" : "text-foreground"
          )}
        >
          {option.label}
        </span>
        <span className="text-xs text-muted-foreground">
          {option.description}
        </span>
      </div>

      {/* Check indicator */}
      {isSelected && (
        <div className="absolute top-3 right-3 text-primary">
          <CheckIcon />
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
    [disabled, value, onChange]
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
