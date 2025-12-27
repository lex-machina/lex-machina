"use client";

import { useCallback, useEffect } from "react";
import Link from "next/link";
import { cn } from "@/lib/utils";
import { Select, type SelectOption } from "@/components/ui/select";
import { Slider } from "@/components/ui/slider";
import { Toggle } from "@/components/ui/toggle";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type {
  PipelineConfigRequest,
  OutlierStrategy,
  NumericImputation,
  CategoricalImputation,
  ColumnInfo,
} from "@/types";
import { DEFAULT_PIPELINE_CONFIG } from "@/types";

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
  /** Names of currently selected columns (only these appear in target dropdown) */
  selectedColumns?: string[];
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
 * Features two modes:
 * - **Smart Mode (default):** AI automatically selects optimal preprocessing strategies.
 *   All manual settings are visible but grayed out.
 * - **Manual Mode:** User configures all options manually.
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
  selectedColumns,
  hasAIProvider = false,
  disabled = false,
  className,
}: ConfigPanelProps) {
  // Smart mode = AI decisions enabled
  const isSmartMode = config.use_ai_decisions;
  const currentMode = isSmartMode ? "smart" : "manual";

  // In Smart mode, all manual settings are disabled (grayed out)
  const settingsDisabled = disabled || isSmartMode;

  // Helper to update a single config field
  const updateConfig = useCallback(
    <K extends keyof PipelineConfigRequest>(key: K, value: PipelineConfigRequest[K]) => {
      onConfigChange({ ...config, [key]: value });
    },
    [config, onConfigChange]
  );

  // Handle mode change from tabs
  const handleModeChange = useCallback(
    (mode: string) => {
      updateConfig("use_ai_decisions", mode === "smart");
    },
    [updateConfig]
  );

  // Handle reset to defaults
  const handleReset = useCallback(() => {
    onConfigChange(DEFAULT_PIPELINE_CONFIG);
  }, [onConfigChange]);

  // Clear target column if it's no longer in the selected columns
  useEffect(() => {
    if (
      config.target_column &&
      selectedColumns &&
      !selectedColumns.includes(config.target_column)
    ) {
      onConfigChange({ ...config, target_column: null });
    }
  }, [selectedColumns, config, onConfigChange]);

  // Build target column options from selected columns only
  // If selectedColumns is provided, filter to only those columns
  const availableColumns = selectedColumns
    ? columns.filter((col) => selectedColumns.includes(col.name))
    : columns;

  const targetColumnOptions: SelectOption[] = [
    { value: "", label: "None (auto-detect)" },
    ...availableColumns.map((col) => ({
      value: col.name,
      label: `${col.name} (${col.dtype})`,
    })),
  ];

  return (
    <div
      className={cn(
        "flex flex-col gap-4 p-4",
        disabled && "opacity-50 pointer-events-none",
        className
      )}
      data-slot="config-panel"
    >
      {/* Mode Selector using Tabs */}
      <div className="flex flex-col gap-3">
        <Tabs value={currentMode} onValueChange={handleModeChange}>
          <TabsList className="w-full">
            <TabsTrigger value="smart" className="flex-1" disabled={disabled}>
              Smart
            </TabsTrigger>
            <TabsTrigger value="manual" className="flex-1" disabled={disabled}>
              Manual
            </TabsTrigger>
          </TabsList>
        </Tabs>

        {/* Mode Description */}
        <p className="text-xs text-muted-foreground">
          {isSmartMode
            ? "AI analyzes your data and automatically selects the best preprocessing strategies."
            : "Manually configure all preprocessing options."}
        </p>

        {/* AI Provider Warning (only in Smart mode) */}
        {isSmartMode && !hasAIProvider && (
          <div className="flex flex-col gap-2 p-3 rounded-md bg-muted/50 border border-border">
            <p className="text-xs text-muted-foreground">
              No AI provider configured. Smart mode requires an AI provider to analyze your data.
            </p>
            <Button variant="outline" size="sm" asChild className="w-fit">
              <Link href="/settings">
                Configure AI Provider
              </Link>
            </Button>
          </div>
        )}
      </div>

      {/* All settings below - grayed out in Smart mode */}
      <div className={cn(
        "flex flex-col gap-6 transition-opacity",
        isSmartMode && "opacity-50 pointer-events-none"
      )}>
        {/* Target Column */}
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
            disabled={settingsDisabled}
          />
          <p className="text-xs text-muted-foreground">
            {config.target_column
              ? `Target: "${config.target_column}" - this column will be preserved and used for ML task detection`
              : "Leave empty to let the system auto-detect the target column"}
          </p>
        </ConfigSection>

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
            disabled={settingsDisabled}
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
            disabled={settingsDisabled}
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
            disabled={settingsDisabled}
          />

          <Select
            label="Categorical columns"
            value={config.categorical_imputation}
            onValueChange={(v) => updateConfig("categorical_imputation", v as CategoricalImputation)}
            options={CATEGORICAL_IMPUTATION_OPTIONS}
            disabled={settingsDisabled}
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
              disabled={settingsDisabled}
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
            disabled={settingsDisabled}
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
            disabled={settingsDisabled}
          />

          <Toggle
            pressed={config.remove_duplicates}
            onPressedChange={(v) => updateConfig("remove_duplicates", v)}
            label="Remove duplicates"
            description="Remove duplicate rows from the dataset"
            disabled={settingsDisabled}
          />
        </ConfigSection>

        {/* Reset Button */}
        <Button
          variant="outline"
          size="sm"
          onClick={handleReset}
          disabled={settingsDisabled}
          className="w-fit"
        >
          Reset to Defaults
        </Button>
      </div>
    </div>
  );
}

export default ConfigPanel;
