"use client";

import { useId, type InputHTMLAttributes } from "react";
import { cn } from "@/lib/utils";

// ============================================================================
// TYPES
// ============================================================================

export interface CheckboxProps extends Omit<
    InputHTMLAttributes<HTMLInputElement>,
    "type" | "onChange"
> {
    /** Whether the checkbox is checked */
    checked?: boolean;
    /** Default checked state (uncontrolled) */
    defaultChecked?: boolean;
    /** Callback when checked state changes */
    onCheckedChange?: (checked: boolean) => void;
    /** Label text to display next to the checkbox */
    label?: string;
    /** Additional description text below the label */
    description?: string;
    /** Additional class names for the container */
    className?: string;
}

// ============================================================================
// CHECKBOX COMPONENT
// ============================================================================

/**
 * A checkbox input with optional label and description.
 *
 * Follows desktop conventions with keyboard support and accessibility.
 *
 * @example
 * ```tsx
 * // Simple checkbox
 * <Checkbox
 *   checked={isEnabled}
 *   onCheckedChange={setIsEnabled}
 *   label="Enable feature"
 * />
 *
 * // With description
 * <Checkbox
 *   checked={useAI}
 *   onCheckedChange={setUseAI}
 *   label="Use AI decisions"
 *   description="Let AI determine the best preprocessing strategies"
 * />
 * ```
 */
export function Checkbox({
    checked,
    defaultChecked,
    onCheckedChange,
    label,
    description,
    className,
    disabled,
    id: providedId,
    ...props
}: CheckboxProps) {
    const generatedId = useId();
    const id = providedId ?? generatedId;
    const descriptionId = description ? `${id}-description` : undefined;

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        onCheckedChange?.(e.target.checked);
    };

    return (
        <div
            className={cn("flex items-start gap-3", className)}
            data-slot="checkbox"
        >
            <input
                type="checkbox"
                id={id}
                checked={checked}
                defaultChecked={defaultChecked}
                disabled={disabled}
                onChange={handleChange}
                aria-describedby={descriptionId}
                className={cn(
                    // Base styles
                    "border-input h-4 w-4 shrink-0 cursor-pointer rounded border",
                    // Background
                    "bg-background",
                    // Checked state
                    "checked:bg-primary checked:border-primary",
                    // Focus styles
                    "focus:ring-ring focus:ring-offset-background focus:ring-2 focus:ring-offset-2 focus:outline-none",
                    // Disabled styles
                    "disabled:cursor-not-allowed disabled:opacity-50",
                    // Accent color for the checkmark
                    "accent-primary",
                )}
                {...props}
            />
            {(label || description) && (
                <div className="flex flex-col gap-0.5">
                    {label && (
                        <label
                            htmlFor={id}
                            className={cn(
                                "cursor-pointer text-sm leading-none font-medium select-none",
                                "peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
                                disabled && "cursor-not-allowed opacity-70",
                            )}
                        >
                            {label}
                        </label>
                    )}
                    {description && (
                        <p
                            id={descriptionId}
                            className={cn(
                                "text-muted-foreground text-xs",
                                disabled && "opacity-70",
                            )}
                        >
                            {description}
                        </p>
                    )}
                </div>
            )}
        </div>
    );
}

export default Checkbox;
