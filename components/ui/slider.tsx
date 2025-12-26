"use client";

import { useId, useState, useCallback, type InputHTMLAttributes } from "react";
import { cn } from "@/lib/utils";

// ============================================================================
// TYPES
// ============================================================================

export interface SliderProps
  extends Omit<
    InputHTMLAttributes<HTMLInputElement>,
    "type" | "onChange" | "value" | "defaultValue"
  > {
  /** The controlled value */
  value?: number;
  /** Default value (uncontrolled) */
  defaultValue?: number;
  /** Callback when value changes */
  onValueChange?: (value: number) => void;
  /** Minimum value */
  min?: number;
  /** Maximum value */
  max?: number;
  /** Step increment */
  step?: number;
  /** Label text above the slider */
  label?: string;
  /** Whether to show the current value */
  showValue?: boolean;
  /** Custom value formatter */
  formatValue?: (value: number) => string;
  /** Additional class names */
  className?: string;
}

// ============================================================================
// SLIDER COMPONENT
// ============================================================================

/**
 * A range slider input with optional label and value display.
 *
 * @example
 * ```tsx
 * // Simple slider
 * <Slider
 *   value={threshold}
 *   onValueChange={setThreshold}
 *   min={0}
 *   max={1}
 *   step={0.1}
 *   label="Missing Value Threshold"
 *   showValue
 * />
 *
 * // With custom formatting
 * <Slider
 *   value={neighbors}
 *   onValueChange={setNeighbors}
 *   min={1}
 *   max={20}
 *   step={1}
 *   label="KNN Neighbors"
 *   showValue
 *   formatValue={(v) => `${v} neighbors`}
 * />
 * ```
 */
export function Slider({
  value,
  defaultValue = 0,
  onValueChange,
  min = 0,
  max = 100,
  step = 1,
  label,
  showValue = false,
  formatValue,
  className,
  disabled,
  id: providedId,
  ...props
}: SliderProps) {
  const generatedId = useId();
  const id = providedId ?? generatedId;

  // Internal state for uncontrolled mode
  const [internalValue, setInternalValue] = useState(defaultValue);
  const currentValue = value ?? internalValue;

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = parseFloat(e.target.value);
      if (value === undefined) {
        setInternalValue(newValue);
      }
      onValueChange?.(newValue);
    },
    [value, onValueChange]
  );

  // Calculate percentage for styling
  const percentage = ((currentValue - min) / (max - min)) * 100;

  // Format the display value
  const displayValue = formatValue
    ? formatValue(currentValue)
    : currentValue.toString();

  return (
    <div className={cn("flex flex-col gap-2", className)} data-slot="slider">
      {(label || showValue) && (
        <div className="flex items-center justify-between">
          {label && (
            <label
              htmlFor={id}
              className={cn(
                "text-sm font-medium leading-none",
                disabled && "opacity-70"
              )}
            >
              {label}
            </label>
          )}
          {showValue && (
            <span
              className={cn(
                "text-sm text-muted-foreground tabular-nums",
                disabled && "opacity-70"
              )}
            >
              {displayValue}
            </span>
          )}
        </div>
      )}
      <input
        type="range"
        id={id}
        value={currentValue}
        min={min}
        max={max}
        step={step}
        disabled={disabled}
        onChange={handleChange}
        className={cn(
          // Base styles
          "h-2 w-full cursor-pointer appearance-none rounded-full bg-secondary",
          // Track fill (using CSS gradient)
          "[&::-webkit-slider-runnable-track]:rounded-full",
          "[&::-webkit-slider-runnable-track]:bg-secondary",
          // Thumb styles
          "[&::-webkit-slider-thumb]:appearance-none",
          "[&::-webkit-slider-thumb]:h-4",
          "[&::-webkit-slider-thumb]:w-4",
          "[&::-webkit-slider-thumb]:rounded-full",
          "[&::-webkit-slider-thumb]:bg-primary",
          "[&::-webkit-slider-thumb]:border-2",
          "[&::-webkit-slider-thumb]:border-primary",
          "[&::-webkit-slider-thumb]:shadow-sm",
          "[&::-webkit-slider-thumb]:transition-all",
          "[&::-webkit-slider-thumb]:hover:scale-110",
          // Firefox thumb
          "[&::-moz-range-thumb]:h-4",
          "[&::-moz-range-thumb]:w-4",
          "[&::-moz-range-thumb]:rounded-full",
          "[&::-moz-range-thumb]:bg-primary",
          "[&::-moz-range-thumb]:border-2",
          "[&::-moz-range-thumb]:border-primary",
          "[&::-moz-range-thumb]:shadow-sm",
          // Focus styles
          "focus:outline-none",
          "[&::-webkit-slider-thumb]:focus:ring-2",
          "[&::-webkit-slider-thumb]:focus:ring-ring",
          "[&::-webkit-slider-thumb]:focus:ring-offset-2",
          // Disabled styles
          "disabled:cursor-not-allowed disabled:opacity-50"
        )}
        style={{
          // CSS custom property for the fill
          background: `linear-gradient(to right, hsl(var(--primary)) ${percentage}%, hsl(var(--secondary)) ${percentage}%)`,
        }}
        {...props}
      />
    </div>
  );
}

export default Slider;
