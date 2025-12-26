"use client";

import { useCallback } from "react";
import { cn } from "@/lib/utils";
import { Select, type SelectOption } from "@/components/ui/select";
import { Slider } from "@/components/ui/slider";
import { Toggle } from "@/components/ui/toggle";
import { Input } from "@/components/ui/input";
import type {
  PipelineConfigRequest,
  OutlierStrategy,
  NumericImputation,
  CategoricalImputation,
  ColumnInfo,
} from "@/types";

// ============================================================================
// TYPES
// ============================================================================

export interface ConfigPanelProps {
  /** Current pipeline configuration */
  config: PipelineConfigRequest;
  /** Callback when configuration changes */
  onConfigChange: (config: PipelineConfigRequest) => void;
  /** Available columns for target column selection */
  columns?: ColumnInfo[];
  /** Whether AI provider is configured */
  hasAIProvider?: boolean;
  /** Whether the panel is disabled */
  disabled?: boolean;
  /** Additional class names */
  className?: string;
}

// ============================================================================
// OPTIONS
// ============================================================================

const OUTLIER_STRATEGY_OPTIONS: SelectOption[] = [
  { value: "cap", label: "Cap at bounds (Winsorize)" },
  { value: "remove", label: "Remove outlier rows" },
  { value: "median", label: "Replace with median" },
  { value: "keep", label: "Keep as-is" },
];

const NUMERIC_IMPUTATION_OPTIONS: SelectOption[] = [
  { value: "median", label: "Median (recommended)" },
  { value: "mean", label: "Mean" },
  { value: "knn", label: "K-Nearest Neighbors" },
  { value: "zero", label: "Fill with zero" },
  { value: "drop", label: "Drop rows with missing" },
];

const CATEGORICAL_IMPUTATION_OPTIONS: SelectOption[] = [
  { value: "mode", label: "Mode (most frequent)" },
  { value: "constant", label: "Fill with 'Unknown'" },
  { value: "drop", label: "Drop rows with missing" },
];

// ============================================================================
// SECTION COMPONENT
// ============================================================================

interface ConfigSectionProps {
  title: string;
  description?: string;
  children: React.ReactNode;
  className?: string;
}

function ConfigSection({ title, description, children, className }: ConfigSectionProps) {
  return (
    <div className={cn("flex flex-col gap-3", className)}>
      <div className="flex flex-col gap-0.5">
        <h3 className="text-sm font-medium">{title}</h3>
        {description && (
          <p className="text-xs text-muted-foreground">{description}</p>
        )}
      </div>
      <div className="flex flex-col gap-3">
        {children}
      </div>
    </div>
  );
}

// ============================================================================
// CONFIG PANEL COMPONENT
// ============================================================================

/**
 * Configuration panel for preprocessing pipeline options.
 *
 * Provides controls for all pipeline configuration options including
 * thresholds, imputation methods, outlier handling, and AI decisions.
 *
 * @example
 * ```tsx
 * const [config, setConfig] = useState<PipelineConfigRequest>(DEFAULT_PIPELINE_CONFIG);
 * const { fileInfo } = useFileState();
 * const { aiConfig } = useSettings();
 *
 * <ConfigPanel
 *   config={config}
 *   onConfigChange={setConfig}
 *   columns={fileInfo?.columns}
 *   hasAIProvider={!!aiConfig}
 * />
 * ```
 */
export function ConfigPanel({
  config,
  onConfigChange,
  columns = [],
  hasAIProvider = false,
  disabled = false,
  className,
}: ConfigPanelProps) {
  // Helper to update a single config field
  const updateConfig = useCallback(
    <K extends keyof PipelineConfigRequest>(key: K, value: PipelineConfigRequest[K]) => {
      onConfigChange({ ...config, [key]: value });
    },
    [config, onConfigChange]
  );

  // Build target column options from available columns
  const targetColumnOptions: SelectOption[] = [
    { value: "", label: "None (auto-detect)" },
    ...columns.map((col) => ({
      value: col.name,
      label: `${col.name} (${col.dtype})`,
    })),
  ];

  return (
    <div
      className={cn(
        "flex flex-col gap-6 p-4",
        disabled && "opacity-50 pointer-events-none",
        className
      )}
      data-slot="config-panel"
    >
      {/* Missing Value Handling */}
      <ConfigSection
        title="Missing Value Handling"
        description="Configure how missing values are detected and handled"
      >
        <Slider
          label="Column drop threshold"
          value={config.missing_column_threshold}
          onValueChange={(v) => updateConfig("missing_column_threshold", v)}
          min={0}
          max={1}
          step={0.05}
          showValue
          formatValue={(v) => `${Math.round(v * 100)}%`}
          disabled={disabled}
        />
        <p className="text-xs text-muted-foreground -mt-1">
          Drop columns with more than this percentage of missing values
        </p>

        <Slider
          label="Row drop threshold"
          value={config.missing_row_threshold}
          onValueChange={(v) => updateConfig("missing_row_threshold", v)}
          min={0}
          max={1}
          step={0.05}
          showValue
          formatValue={(v) => `${Math.round(v * 100)}%`}
          disabled={disabled}
        />
        <p className="text-xs text-muted-foreground -mt-1">
          Drop rows with more than this percentage of missing values
        </p>
      </ConfigSection>

      {/* Imputation Methods */}
      <ConfigSection
        title="Imputation Methods"
        description="How to fill remaining missing values"
      >
        <Select
          label="Numeric columns"
          value={config.numeric_imputation}
          onValueChange={(v) => updateConfig("numeric_imputation", v as NumericImputation)}
          options={NUMERIC_IMPUTATION_OPTIONS}
          disabled={disabled}
        />

        <Select
          label="Categorical columns"
          value={config.categorical_imputation}
          onValueChange={(v) => updateConfig("categorical_imputation", v as CategoricalImputation)}
          options={CATEGORICAL_IMPUTATION_OPTIONS}
          disabled={disabled}
        />

        {/* KNN neighbors - only show when KNN is selected */}
        {config.numeric_imputation === "knn" && (
          <Input
            label="KNN neighbors"
            type="number"
            min={1}
            max={50}
            value={config.knn_neighbors}
            onChange={(e) => {
              const val = parseInt(e.target.value, 10);
              if (!isNaN(val) && val >= 1) {
                updateConfig("knn_neighbors", val);
              }
            }}
            helperText="Number of neighbors to use for KNN imputation (1-50)"
            disabled={disabled}
          />
        )}
      </ConfigSection>

      {/* Outlier Handling */}
      <ConfigSection
        title="Outlier Handling"
        description="How to detect and handle statistical outliers"
      >
        <Select
          label="Outlier strategy"
          value={config.outlier_strategy}
          onValueChange={(v) => updateConfig("outlier_strategy", v as OutlierStrategy)}
          options={OUTLIER_STRATEGY_OPTIONS}
          disabled={disabled}
        />
      </ConfigSection>

      {/* Data Cleaning Options */}
      <ConfigSection
        title="Data Cleaning"
        description="Additional cleaning operations"
      >
        <Toggle
          pressed={config.enable_type_correction}
          onPressedChange={(v) => updateConfig("enable_type_correction", v)}
          label="Type correction"
          description="Automatically fix mistyped values (e.g., '123' as string to number)"
          disabled={disabled}
        />

        <Toggle
          pressed={config.remove_duplicates}
          onPressedChange={(v) => updateConfig("remove_duplicates", v)}
          label="Remove duplicates"
          description="Remove duplicate rows from the dataset"
          disabled={disabled}
        />
      </ConfigSection>

      {/* AI Decision Making */}
      <ConfigSection
        title="AI-Guided Decisions"
        description="Use AI to determine optimal preprocessing strategies"
      >
        <Toggle
          pressed={config.use_ai_decisions}
          onPressedChange={(v) => updateConfig("use_ai_decisions", v)}
          label="Enable AI decisions"
          description={
            hasAIProvider
              ? "Let AI analyze your data and suggest the best preprocessing approach"
              : "Configure an AI provider in Settings to enable this feature"
          }
          disabled={disabled || !hasAIProvider}
        />
        {config.use_ai_decisions && !hasAIProvider && (
          <p className="text-xs text-amber-500">
            No AI provider configured. Go to Settings to add one.
          </p>
        )}
      </ConfigSection>

      {/* Target Column Selection */}
      <ConfigSection
        title="Target Column"
        description="Specify the target column for ML task detection"
      >
        <Select
          label="Target column"
          value={config.target_column ?? ""}
          onValueChange={(v) => updateConfig("target_column", v || null)}
          options={targetColumnOptions}
          placeholder="Select target column..."
          disabled={disabled}
        />
        <p className="text-xs text-muted-foreground">
          {config.target_column
            ? `Target: "${config.target_column}" - this column will be preserved and used for ML task detection`
            : "Leave empty to let the system auto-detect the target column"}
        </p>
      </ConfigSection>
    </div>
  );
}

export default ConfigPanel;
