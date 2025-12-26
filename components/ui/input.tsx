"use client";

import {
  forwardRef,
  useId,
  type InputHTMLAttributes,
  type ReactNode,
} from "react";
import { cn } from "@/lib/utils";

// ============================================================================
// TYPES
// ============================================================================

export interface InputProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, "size"> {
  /** Label text above the input */
  label?: string;
  /** Helper text below the input */
  helperText?: string;
  /** Error message (replaces helper text when present) */
  error?: string;
  /** Left addon (icon or text) */
  leftAddon?: ReactNode;
  /** Right addon (icon or text) */
  rightAddon?: ReactNode;
  /** Size variant */
  size?: "sm" | "default" | "lg";
  /** Additional class names for the container */
  containerClassName?: string;
}

// ============================================================================
// INPUT COMPONENT
// ============================================================================

/**
 * A text input with optional label, helper text, and addons.
 *
 * @example
 * ```tsx
 * // Simple input
 * <Input
 *   label="API Key"
 *   type="password"
 *   placeholder="Enter your API key"
 *   value={apiKey}
 *   onChange={(e) => setApiKey(e.target.value)}
 * />
 *
 * // With error
 * <Input
 *   label="Email"
 *   type="email"
 *   error="Please enter a valid email"
 * />
 *
 * // With addon
 * <Input
 *   label="Search"
 *   leftAddon={<SearchIcon className="h-4 w-4" />}
 *   placeholder="Search..."
 * />
 * ```
 */
export const Input = forwardRef<HTMLInputElement, InputProps>(
  (
    {
      label,
      helperText,
      error,
      leftAddon,
      rightAddon,
      size = "default",
      className,
      containerClassName,
      disabled,
      id: providedId,
      ...props
    },
    ref
  ) => {
    const generatedId = useId();
    const id = providedId ?? generatedId;
    const helperId = `${id}-helper`;
    const hasError = !!error;

    const sizeClasses = {
      sm: "h-8 text-xs px-2",
      default: "h-9 text-sm px-3",
      lg: "h-10 text-base px-4",
    };

    return (
      <div
        className={cn("flex flex-col gap-1.5", containerClassName)}
        data-slot="input"
      >
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
        <div className="relative flex items-center">
          {leftAddon && (
            <div
              className={cn(
                "absolute left-3 flex items-center text-muted-foreground",
                disabled && "opacity-50"
              )}
            >
              {leftAddon}
            </div>
          )}
          <input
            ref={ref}
            id={id}
            disabled={disabled}
            aria-invalid={hasError}
            aria-describedby={helperText || error ? helperId : undefined}
            className={cn(
              // Base styles
              "w-full rounded-md border bg-background",
              "placeholder:text-muted-foreground",
              // Size
              sizeClasses[size],
              // Left addon padding
              leftAddon && "pl-9",
              // Right addon padding
              rightAddon && "pr-9",
              // Border color
              hasError
                ? "border-destructive focus:ring-destructive/20"
                : "border-input",
              // Focus styles
              "focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background",
              // Hover
              "hover:border-ring/50",
              // Disabled styles
              "disabled:cursor-not-allowed disabled:opacity-50 disabled:bg-muted",
              className
            )}
            {...props}
          />
          {rightAddon && (
            <div
              className={cn(
                "absolute right-3 flex items-center text-muted-foreground",
                disabled && "opacity-50"
              )}
            >
              {rightAddon}
            </div>
          )}
        </div>
        {(helperText || error) && (
          <p
            id={helperId}
            className={cn(
              "text-xs",
              hasError ? "text-destructive" : "text-muted-foreground",
              disabled && "opacity-70"
            )}
          >
            {error || helperText}
          </p>
        )}
      </div>
    );
  }
);

Input.displayName = "Input";

export default Input;
