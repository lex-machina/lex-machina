"use client";

import { useId, type ButtonHTMLAttributes } from "react";
import { cn } from "@/lib/utils";

// ============================================================================
// TYPES
// ============================================================================

export interface ToggleProps
  extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "onChange"> {
  /** Whether the toggle is on */
  pressed?: boolean;
  /** Default pressed state (uncontrolled) */
  defaultPressed?: boolean;
  /** Callback when pressed state changes */
  onPressedChange?: (pressed: boolean) => void;
  /** Label text next to the toggle */
  label?: string;
  /** Position of the label */
  labelPosition?: "left" | "right";
  /** Additional description text */
  description?: string;
  /** Size variant */
  size?: "sm" | "default" | "lg";
  /** Additional class names */
  className?: string;
}

// ============================================================================
// TOGGLE COMPONENT
// ============================================================================

/**
 * A toggle switch for boolean settings.
 *
 * @example
 * ```tsx
 * // Simple toggle
 * <Toggle
 *   pressed={isEnabled}
 *   onPressedChange={setIsEnabled}
 *   label="Enable feature"
 * />
 *
 * // With description
 * <Toggle
 *   pressed={useAI}
 *   onPressedChange={setUseAI}
 *   label="Use AI decisions"
 *   description="Let AI determine preprocessing strategies"
 * />
 *
 * // Different sizes
 * <Toggle size="sm" pressed={value} onPressedChange={setValue} />
 * <Toggle size="lg" pressed={value} onPressedChange={setValue} />
 * ```
 */
export function Toggle({
  pressed,
  defaultPressed = false,
  onPressedChange,
  label,
  labelPosition = "right",
  description,
  size = "default",
  className,
  disabled,
  id: providedId,
  ...props
}: ToggleProps) {
  const generatedId = useId();
  const id = providedId ?? generatedId;
  const descriptionId = description ? `${id}-description` : undefined;

  // Use internal state if uncontrolled
  const isPressed = pressed ?? defaultPressed;

  const handleClick = () => {
    if (!disabled) {
      onPressedChange?.(!isPressed);
    }
  };

  const sizeConfig = {
    sm: {
      track: "h-4 w-7",
      thumb: "h-3 w-3",
      translate: "translate-x-3",
    },
    default: {
      track: "h-5 w-9",
      thumb: "h-4 w-4",
      translate: "translate-x-4",
    },
    lg: {
      track: "h-6 w-11",
      thumb: "h-5 w-5",
      translate: "translate-x-5",
    },
  };

  const config = sizeConfig[size];

  const toggleButton = (
    <button
      type="button"
      role="switch"
      id={id}
      aria-checked={isPressed}
      aria-describedby={descriptionId}
      disabled={disabled}
      onClick={handleClick}
      className={cn(
        // Base track styles
        "relative inline-flex shrink-0 cursor-pointer rounded-full border-2 border-transparent",
        "transition-colors duration-200 ease-in-out",
        // Track size
        config.track,
        // Track colors
        isPressed ? "bg-primary" : "bg-input",
        // Focus styles
        "focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background",
        // Disabled styles
        "disabled:cursor-not-allowed disabled:opacity-50"
      )}
      {...props}
    >
      <span
        aria-hidden="true"
        className={cn(
          // Base thumb styles
          "pointer-events-none inline-block rounded-full bg-background shadow-sm ring-0",
          "transition-transform duration-200 ease-in-out",
          // Thumb size
          config.thumb,
          // Position based on state
          isPressed ? config.translate : "translate-x-0"
        )}
      />
    </button>
  );

  // If no label, just return the toggle
  if (!label && !description) {
    return (
      <div className={className} data-slot="toggle">
        {toggleButton}
      </div>
    );
  }

  // With label/description
  return (
    <div
      className={cn(
        "flex items-start gap-3",
        labelPosition === "left" && "flex-row-reverse justify-end",
        className
      )}
      data-slot="toggle"
    >
      {toggleButton}
      <div className="flex flex-col gap-0.5">
        {label && (
          <label
            htmlFor={id}
            className={cn(
              "text-sm font-medium leading-none cursor-pointer select-none",
              disabled && "cursor-not-allowed opacity-70"
            )}
          >
            {label}
          </label>
        )}
        {description && (
          <p
            id={descriptionId}
            className={cn(
              "text-xs text-muted-foreground",
              disabled && "opacity-70"
            )}
          >
            {description}
          </p>
        )}
      </div>
    </div>
  );
}

export default Toggle;
