"use client";

import { useId, type SelectHTMLAttributes, type ReactNode } from "react";
import { cn } from "@/lib/utils";

// ============================================================================
// TYPES
// ============================================================================

export interface SelectOption {
  /** The value to be submitted */
  value: string;
  /** Display label for the option */
  label: string;
  /** Whether this option is disabled */
  disabled?: boolean;
}

export interface SelectProps
  extends Omit<SelectHTMLAttributes<HTMLSelectElement>, "onChange"> {
  /** The controlled value */
  value?: string;
  /** Default value (uncontrolled) */
  defaultValue?: string;
  /** Callback when selection changes */
  onValueChange?: (value: string) => void;
  /** Options to display */
  options: SelectOption[];
  /** Placeholder text when no value is selected */
  placeholder?: string;
  /** Label text above the select */
  label?: string;
  /** Additional class names */
  className?: string;
}

// ============================================================================
// SELECT COMPONENT
// ============================================================================

/**
 * A native select dropdown with styling consistent with the app design.
 *
 * Uses native `<select>` for best desktop compatibility and accessibility.
 *
 * @example
 * ```tsx
 * <Select
 *   label="Outlier Strategy"
 *   value={strategy}
 *   onValueChange={setStrategy}
 *   options={[
 *     { value: "cap", label: "Cap values" },
 *     { value: "remove", label: "Remove outliers" },
 *     { value: "median", label: "Replace with median" },
 *     { value: "keep", label: "Keep as-is" },
 *   ]}
 * />
 * ```
 */
export function Select({
  value,
  defaultValue,
  onValueChange,
  options,
  placeholder,
  label,
  className,
  disabled,
  id: providedId,
  ...props
}: SelectProps) {
  const generatedId = useId();
  const id = providedId ?? generatedId;

  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    onValueChange?.(e.target.value);
  };

  return (
    <div className={cn("flex flex-col gap-1.5", className)} data-slot="select">
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
      <select
        id={id}
        value={value}
        defaultValue={defaultValue}
        disabled={disabled}
        onChange={handleChange}
        className={cn(
          // Base styles
          "h-9 w-full rounded-md border border-input bg-background px-3 py-1",
          "text-sm",
          // Appearance
          "appearance-none cursor-pointer",
          // Background arrow indicator
          "bg-[url('data:image/svg+xml;charset=utf-8,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%2212%22%20height%3D%2212%22%20viewBox%3D%220%200%2012%2012%22%3E%3Cpath%20fill%3D%22%236b7280%22%20d%3D%22M2.22%204.47a.75.75%200%200%201%201.06%200L6%207.19l2.72-2.72a.75.75%200%201%201%201.06%201.06l-3.25%203.25a.75.75%200%200%201-1.06%200L2.22%205.53a.75.75%200%200%201%200-1.06z%22%2F%3E%3C%2Fsvg%3E')]",
          "bg-[length:12px_12px] bg-[right_0.5rem_center] bg-no-repeat",
          "pr-8",
          // Focus styles
          "focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background",
          // Disabled styles
          "disabled:cursor-not-allowed disabled:opacity-50",
          // Hover
          "hover:border-ring/50"
        )}
        {...props}
      >
        {placeholder && (
          <option value="" disabled>
            {placeholder}
          </option>
        )}
        {options.map((option) => (
          <option
            key={option.value}
            value={option.value}
            disabled={option.disabled}
          >
            {option.label}
          </option>
        ))}
      </select>
    </div>
  );
}

// ============================================================================
// SELECT GROUP (for organizing options)
// ============================================================================

export interface SelectGroupProps {
  /** Group label */
  label: string;
  /** Select options in this group */
  children: ReactNode;
}

/**
 * Groups options within a Select for better organization.
 *
 * @example
 * ```tsx
 * <select>
 *   <SelectGroup label="Numeric">
 *     <option value="mean">Mean</option>
 *     <option value="median">Median</option>
 *   </SelectGroup>
 * </select>
 * ```
 */
export function SelectGroup({ label, children }: SelectGroupProps) {
  return <optgroup label={label}>{children}</optgroup>;
}

export default Select;
