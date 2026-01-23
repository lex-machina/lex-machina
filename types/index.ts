export interface ColumnInfo {
    name: string;
    dtype: string;
    null_count: number;
    width: number;
}

export interface FileInfo {
    path: string;
    name: string;
    size_bytes: number;
    row_count: number;
    column_count: number;
    columns: ColumnInfo[];
}

export type CellValue = string | number | boolean | null;

export type Row = CellValue[];

export interface RowsResponse {
    rows: CellValue[][];
    start: number;
    total_rows: number;
}

// ============================================================================
// EVENT TYPES (Mirrors Rust events.rs payloads)
// ============================================================================

/**
 * Event names emitted by Rust backend.
 * These must match the constants in src-tauri/src/events.rs
 */
export const RUST_EVENTS = {
    // File events
    FILE_LOADED: "file:loaded",
    FILE_CLOSED: "file:closed",

    // App events
    LOADING: "app:loading",
    ERROR: "app:error",

    // Preprocessing events
    PREPROCESSING_PROGRESS: "preprocessing:progress",
    PREPROCESSING_COMPLETE: "preprocessing:complete",
    PREPROCESSING_ERROR: "preprocessing:error",
    PREPROCESSING_CANCELLED: "preprocessing:cancelled",

    // Settings events
    THEME_CHANGED: "settings:theme-changed",

    // ML events
    ML_PROGRESS: "ml:progress",
    ML_COMPLETE: "ml:complete",
    ML_ERROR: "ml:error",
    ML_CANCELLED: "ml:cancelled",
    ML_KERNEL_STATUS: "ml:kernel-status",
} as const;

export type RustEventName = (typeof RUST_EVENTS)[keyof typeof RUST_EVENTS];

/**
 * Payload for the `file:loaded` event.
 * Mirrors: events.rs::FileLoadedPayload
 */
export interface FileLoadedPayload {
    file_info: FileInfo;
}

/**
 * Payload for the `app:loading` event.
 * Mirrors: events.rs::LoadingPayload
 */
export interface LoadingPayload {
    is_loading: boolean;
    message: string | null;
}

/**
 * Payload for the `app:error` event.
 * Mirrors: events.rs::ErrorPayload
 */
export interface ErrorPayload {
    code: string;
    message: string;
}

/**
 * Payload for the `preprocessing:error` event.
 * Mirrors: events.rs::PreprocessingErrorPayload
 */
export interface PreprocessingErrorPayload {
    code: string;
    message: string;
}

/**
 * Error codes from Rust backend.
 * Mirrors: events.rs::error_codes
 */
export const ERROR_CODES = {
    // File error codes
    FILE_NOT_FOUND: "FILE_NOT_FOUND",
    FILE_READ_ERROR: "FILE_READ_ERROR",
    FILE_PARSE_ERROR: "FILE_PARSE_ERROR",
    FILE_METADATA_ERROR: "FILE_METADATA_ERROR",
    UNKNOWN_ERROR: "UNKNOWN_ERROR",

    // Preprocessing error codes
    PREPROCESSING_CANCELLED: "CANCELLED",
    PREPROCESSING_NO_DATA: "NO_DATA_LOADED",
    PREPROCESSING_INVALID_CONFIG: "INVALID_CONFIG",
    PREPROCESSING_COLUMN_NOT_FOUND: "COLUMN_NOT_FOUND",
    PREPROCESSING_AI_ERROR: "AI_CLIENT_ERROR",
    PREPROCESSING_POLARS_ERROR: "POLARS_ERROR",
    PREPROCESSING_INTERNAL_ERROR: "INTERNAL_ERROR",

    // Settings error codes
    SETTINGS_INVALID_PROVIDER: "INVALID_PROVIDER",
    SETTINGS_INVALID_API_KEY: "INVALID_API_KEY",

    // ML error codes
    ML_NOT_INITIALIZED: "ML_NOT_INITIALIZED",
    ML_NO_DATA: "ML_NO_DATA",
    ML_TRAINING_IN_PROGRESS: "ML_TRAINING_IN_PROGRESS",
    ML_NO_MODEL: "ML_NO_MODEL",
    ML_INVALID_CONFIG: "ML_INVALID_CONFIG",
    ML_TRAINING_FAILED: "ML_TRAINING_FAILED",
    ML_CANCELLED: "ML_CANCELLED",
    ML_INFERENCE_ERROR: "ML_INFERENCE_ERROR",
    ML_RUNTIME_INIT_FAILED: "ML_RUNTIME_INIT_FAILED",
} as const;

export type ErrorCode = (typeof ERROR_CODES)[keyof typeof ERROR_CODES];

// ============================================================================
// UI STATE TYPES (Mirrors Rust state.rs)
// ============================================================================

/**
 * Grid scroll position.
 * Mirrors: state.rs::GridScrollPosition
 */
export interface GridScrollPosition {
    row_index: number;
    scroll_left: number;
}

/**
 * UI state persisted in Rust.
 * Mirrors: state.rs::UIState
 */
export interface UIState {
    sidebar_width: number;
    sidebar_collapsed: boolean;
    column_widths: number[];
    grid_scroll: GridScrollPosition;
}

// ============================================================================
// SETTINGS TYPES (Mirrors Rust state.rs)
// ============================================================================

/**
 * Supported AI provider types for preprocessing decisions.
 * Mirrors: state.rs::AIProviderType
 */
export type AIProviderType = "none" | "openrouter" | "gemini";

/**
 * Configuration for an AI provider.
 * Mirrors: state.rs::AIProviderConfig
 */
export interface AIProviderConfig {
    provider: AIProviderType;
    api_key: string;
}

/**
 * Application theme setting.
 * Mirrors: state.rs::Theme
 */
export type Theme = "system" | "light" | "dark";

/**
 * Navigation bar position setting.
 * Mirrors: state.rs::NavBarPosition
 */
export type NavBarPosition = "left" | "right" | "merged";

// ============================================================================
// PREPROCESSING CONFIGURATION TYPES
// ============================================================================

/**
 * Strategy for handling outliers.
 * Mirrors: lex-processing::OutlierStrategy
 */
export type OutlierStrategy = "cap" | "remove" | "median" | "keep";

/**
 * Method for imputing numeric missing values.
 * Mirrors: lex-processing::NumericImputation
 */
export type NumericImputation = "mean" | "median" | "knn" | "zero" | "drop";

/**
 * Method for imputing categorical missing values.
 * Mirrors: lex-processing::CategoricalImputation
 */
export type CategoricalImputation = "mode" | "constant" | "drop";

/**
 * Pipeline configuration from the frontend.
 * Mirrors: preprocessing.rs::PipelineConfigRequest
 */
export interface PipelineConfigRequest {
    /** Threshold for dropping columns with too many missing values (0.0-1.0) */
    missing_column_threshold: number;
    /** Threshold for dropping rows with too many missing values (0.0-1.0) */
    missing_row_threshold: number;
    /** Strategy for handling outliers */
    outlier_strategy: OutlierStrategy;
    /** Method for imputing numeric values */
    numeric_imputation: NumericImputation;
    /** Method for imputing categorical values */
    categorical_imputation: CategoricalImputation;
    /** Whether to enable automatic type correction */
    enable_type_correction: boolean;
    /** Whether to remove duplicate rows */
    remove_duplicates: boolean;
    /** Number of neighbors for KNN imputation */
    knn_neighbors: number;
    /** Whether to use AI for preprocessing decisions */
    use_ai_decisions: boolean;
    /** Optional target column for ML task detection */
    target_column: string | null;
}

/**
 * Row range for preprocessing a subset of data.
 */
export interface RowRange {
    start: number;
    end: number;
}

/**
 * Request to start preprocessing.
 * Mirrors: preprocessing.rs::PreprocessingRequest
 */
export interface PreprocessingRequest {
    /** Columns selected for preprocessing (empty = all columns) */
    selected_columns: string[];
    /** Optional row range to process (start, end indices) */
    row_range: [number, number] | null;
    /** Pipeline configuration options */
    config: PipelineConfigRequest;
}

// ============================================================================
// PREPROCESSING HISTORY TYPES
// ============================================================================

/**
 * A snapshot of preprocessing configuration for history entries.
 * Mirrors: state.rs::PreprocessingConfigSnapshot
 */
export interface PreprocessingConfigSnapshot {
    /** Columns that were selected for preprocessing */
    selected_columns: string[];
    /** Row range that was processed (start, end indices) */
    row_range: [number, number] | null;
    /** Threshold for dropping columns with too many missing values (0.0-1.0) */
    missing_column_threshold: number;
    /** Threshold for dropping rows with too many missing values (0.0-1.0) */
    missing_row_threshold: number;
    /** Strategy used for handling outliers */
    outlier_strategy: string;
    /** Method used for imputing numeric missing values */
    numeric_imputation: string;
    /** Method used for imputing categorical missing values */
    categorical_imputation: string;
    /** Whether type correction was enabled */
    enable_type_correction: boolean;
    /** Whether duplicate removal was enabled */
    remove_duplicates: boolean;
    /** Number of neighbors used for KNN imputation */
    knn_neighbors: number;
    /** Whether AI-guided decisions were used */
    use_ai_decisions: boolean;
    /** Target column if specified */
    target_column: string | null;
}

/**
 * An entry in the preprocessing history.
 * Mirrors: state.rs::PreprocessingHistoryEntry
 */
export interface PreprocessingHistoryEntry {
    /** Unique identifier for this history entry (UUID) */
    id: string;
    /** Unix timestamp when preprocessing was completed */
    timestamp: number;
    /** Configuration used for this preprocessing run */
    config: PreprocessingConfigSnapshot;
    /** Summary of what the preprocessing accomplished */
    summary: PreprocessingSummary;
}

// ============================================================================
// PREPROCESSING SUMMARY TYPES (Mirrors lex-processing types.rs)
// ============================================================================

/**
 * Types of actions that can be taken during preprocessing.
 * Mirrors: types.rs::ActionType
 */
export type ActionType =
    | "column_removed"
    | "rows_removed"
    | "type_corrected"
    | "value_imputed"
    | "outlier_handled"
    | "duplicates_removed"
    | "target_identified"
    | "problem_type_detected"
    | "column_renamed"
    | "value_cleaned"
    | "data_normalized"
    | "categories_encoded";

/**
 * A single action taken during preprocessing.
 * Mirrors: types.rs::PreprocessingAction
 */
export interface PreprocessingAction {
    /** Type of action performed */
    action_type: ActionType;
    /** Target of the action (column name or "dataset") */
    target: string;
    /** Human-readable description of the action */
    description: string;
    /** Additional details (e.g., values replaced, strategy used) */
    details?: string;
}

/**
 * Summary of changes made to a single column.
 * Mirrors: types.rs::ColumnSummary
 */
export interface ColumnSummary {
    /** Name of the column */
    name: string;
    /** Original data type (as string) */
    original_type: string;
    /** Final data type after preprocessing */
    final_type: string;
    /** Number of missing values before preprocessing */
    missing_before: number;
    /** Number of missing values after preprocessing */
    missing_after: number;
    /** Imputation method used, if any */
    imputation_method?: string;
    /** Number of outliers handled */
    outliers_handled: number;
    /** Number of type corrections made */
    type_corrections: number;
    /** Number of invalid values cleaned */
    values_cleaned: number;
    /** Whether the column was removed */
    was_removed: boolean;
    /** Reason for removal, if removed */
    removal_reason?: string;
}

/**
 * Human-readable summary of what the pipeline did.
 * Mirrors: types.rs::PreprocessingSummary
 */
export interface PreprocessingSummary {
    /** Total execution time in milliseconds */
    duration_ms: number;
    /** Number of rows before preprocessing */
    rows_before: number;
    /** Number of rows after preprocessing */
    rows_after: number;
    /** Number of rows removed during preprocessing */
    rows_removed: number;
    /** Number of columns before preprocessing */
    columns_before: number;
    /** Number of columns after preprocessing */
    columns_after: number;
    /** Number of columns removed during preprocessing */
    columns_removed: number;
    /** Number of data quality issues found */
    issues_found: number;
    /** Number of issues resolved by preprocessing */
    issues_resolved: number;
    /** Data quality score before preprocessing (0.0 - 1.0) */
    data_quality_score_before: number;
    /** Data quality score after preprocessing (0.0 - 1.0) */
    data_quality_score_after: number;
    /** List of actions taken during preprocessing */
    actions: PreprocessingAction[];
    /** Per-column summaries of changes */
    column_summaries: ColumnSummary[];
    /** Warnings and notes generated during preprocessing */
    warnings: string[];
}

// ============================================================================
// PREPROCESSING PROGRESS TYPES (Mirrors lex-processing progress.rs)
// ============================================================================

/**
 * Stages of the preprocessing pipeline.
 * Mirrors: progress.rs::PreprocessingStage
 */
export type PreprocessingStage =
    | "initializing"
    | "profiling"
    | "quality_analysis"
    | "type_correction"
    | "decision_making"
    | "cleaning"
    | "imputation"
    | "outlier_handling"
    | "report_generation"
    | "complete"
    | "cancelled"
    | "failed";

/**
 * Detailed progress update with sub-stage information.
 * Mirrors: progress.rs::ProgressUpdate
 */
export interface ProgressUpdate {
    /** Current pipeline stage */
    stage: PreprocessingStage;
    /** Optional sub-stage description (e.g., "Column: Age", "Row batch 1/10") */
    sub_stage?: string;
    /** Overall progress (0.0 - 1.0) */
    progress: number;
    /** Progress within current stage (0.0 - 1.0) */
    stage_progress: number;
    /** Human-readable message describing current activity */
    message: string;
    /** Number of items processed in current stage (for iterative operations) */
    items_processed?: number;
    /** Total items in current stage (for iterative operations) */
    items_total?: number;
}

// ============================================================================
// PREPROCESSING RESULT TYPES
// ============================================================================

/**
 * Result of running the preprocessing pipeline.
 * Mirrors: types.rs::PipelineResult (without DataFrame which can't be serialized)
 */
export interface PipelineResult {
    /** Whether preprocessing completed successfully */
    success: boolean;
    /** Path to the saved cleaned data file (if written to disk) */
    cleaned_data_path?: string;
    /** Target column if identified */
    target_column?: string;
    /** Problem type if detected (e.g., "binary_classification", "regression") */
    problem_type?: string;
    /** AI choices made during preprocessing */
    ai_choices: Record<string, string>;
    /** Path to the analysis report (if generated) */
    analysis_report?: string;
    /** List of processing steps performed */
    processing_steps: string[];
    /** List of cleaning actions taken */
    cleaning_actions: string[];
    /** Error message if preprocessing failed */
    error?: string;
    /** Detailed summary of preprocessing actions */
    summary?: PreprocessingSummary;
}

/**
 * Response containing rows from the processed DataFrame.
 * Mirrors: preprocessing.rs::ProcessedRowsResponse
 */
export interface ProcessedRowsResponse {
    rows: CellValue[][];
    start: number;
    total_rows: number;
}

/**
 * Result of exporting processed data.
 * Mirrors: preprocessing.rs::ExportResult
 */
export interface ExportResult {
    /** Path to the exported CSV file */
    csv_path: string;
    /** Path to the exported JSON report file */
    report_path: string;
}

// ============================================================================
// EVENT PAYLOAD TYPE ALIASES
// ============================================================================

/**
 * Payload for the `preprocessing:progress` event.
 * Same as ProgressUpdate but typed for event usage.
 */
export type PreprocessingProgressPayload = ProgressUpdate;

/**
 * Payload for the `preprocessing:complete` event.
 * Same as PreprocessingSummary but typed for event usage.
 */
export type PreprocessingCompletePayload = PreprocessingSummary;

/**
 * Payload for the `settings:theme-changed` event.
 * Contains the new theme value.
 */
export type ThemeChangedPayload = Theme;

// ============================================================================
// ML TYPES (Mirrors lex-learning + state.rs + events.rs)
// ============================================================================

export interface MLConfigRequest {
    smart_mode: boolean;
    target_column: string;
    problem_type: "classification" | "regression";
    excluded_columns: string[];
    use_processed_data: boolean;
    optimize_hyperparams?: boolean;
    n_trials?: number;
    cv_folds?: number;
    test_size?: number;
    enable_neural_networks?: boolean;
    enable_explainability?: boolean;
    top_k_algorithms?: number;
    algorithm?: string;
}

export interface TrainingResultResponse {
    success: boolean;
    best_model_name: string;
    metrics: Metrics;
    feature_importance: [string, number][];
    shap_plots: Record<string, string>;
    model_comparison: ModelComparison[];
    training_time_seconds: number;
    warnings: string[];
}

export interface Metrics {
    cv_score?: number;
    test_score?: number;
    train_score?: number;
    accuracy?: number;
    precision?: number;
    recall?: number;
    f1_score?: number;
    roc_auc?: number;
    mse?: number;
    rmse?: number;
    mae?: number;
    r2?: number;
}

export interface ModelComparison {
    name: string;
    test_score: number;
    train_score: number;
    cv_score: number;
    training_time_seconds: number;
    hyperparameters: Record<string, unknown>;
    overfitting_risk: "low" | "medium" | "high";
}

export interface PredictionResult {
    prediction: string | number | boolean | null;
    probabilities?: Record<string, number>;
    confidence?: number;
}

export interface BatchPredictionResult {
    predictions: (string | number | boolean | null)[];
    probabilities?: Record<string, number>[];
    row_count: number;
}

export interface ModelInfo {
    model_name: string;
    problem_type: string;
    target_column: string;
    feature_names: string[];
    class_labels?: string[];
    metrics: Metrics;
    hyperparameters: Record<string, unknown>;
}

export interface MLProgressUpdate {
    stage: string;
    progress: number;
    message: string;
    current_model?: string;
    models_completed?: [number, number];
}

export type MLKernelStatus =
    | "uninitialized"
    | "initializing"
    | "ready"
    | "error";

export type MLTrainingStatus =
    | "idle"
    | "training"
    | "completed"
    | "error"
    | "cancelled";

export interface MLUIState {
    smart_mode: boolean;
    target_column: string | null;
    problem_type: string;
    excluded_columns: string[];
    use_processed_data: boolean;
    config: MLConfigUIState;
    active_tab: string;
}

export interface MLConfigUIState {
    optimize_hyperparams: boolean;
    n_trials: number;
    cv_folds: number;
    test_size: number;
    enable_neural_networks: boolean;
    enable_explainability: boolean;
    top_k_algorithms: number;
    algorithm?: string;
}

export interface TrainingHistoryEntry {
    id: string;
    timestamp: number;
    config: MLConfigSnapshot;
    result_summary: TrainingResultSummary;
}

export interface MLConfigSnapshot {
    target_column: string;
    problem_type: string;
    excluded_columns: string[];
    use_processed_data: boolean;
    optimize_hyperparams: boolean;
    n_trials: number;
    cv_folds: number;
    enable_explainability: boolean;
    top_k_algorithms: number;
    algorithm?: string;
}

export interface TrainingResultSummary {
    best_model_name: string;
    test_score: number;
    training_time_seconds: number;
}

export interface MLCompletePayload {
    best_model_name: string;
    test_score: number;
    training_time_seconds: number;
}

export interface MLErrorPayload {
    code: string;
    message: string;
}

export interface MLKernelStatusPayload {
    status: MLKernelStatus;
    message?: string | null;
}

export type MLProgressPayload = MLProgressUpdate;

// ============================================================================
// ANALYSIS TYPES (Mirrors analysis.rs + state.rs)
// ============================================================================

export interface ColumnProfile {
    name: string;
    dtype: string;
    unique_count: number;
    null_count: number;
    null_percentage: number;
    sample_values: string[];
    inferred_type: string;
    inferred_role: string;
    characteristics: Record<string, unknown>;
}

export interface DatasetProfile {
    shape: [number, number];
    column_profiles: ColumnProfile[];
    target_candidates: string[];
    problem_type_candidates: string[];
    complexity_indicators: Record<string, unknown>;
    duplicate_count: number;
    duplicate_percentage: number;
}

export interface SolutionOption {
    option: string;
    description: string;
    pros?: string;
    cons?: string;
    best_for?: string;
}

export interface DataQualityIssue {
    issue_type: string;
    severity: string;
    affected_columns: string[];
    description: string;
    business_impact: string;
    detection_details: Record<string, unknown>;
    suggested_solutions: SolutionOption[];
}

export type AnalysisDataset = "original" | "processed";

export interface AnalysisUIState {
    use_processed_data: boolean;
    active_tab: string;
    selected_column: string | null;
}

export interface AnalysisSummary {
    rows: number;
    columns: number;
    memory_bytes: number;
    duplicate_count: number;
    duplicate_percentage: number;
    total_missing_cells: number;
    total_missing_percentage: number;
    type_distribution: TypeDistributionEntry[];
}

export interface TypeDistributionEntry {
    dtype: string;
    count: number;
    percentage: number;
}

export interface HistogramBin {
    start: number;
    end: number;
    count: number;
}

export interface BoxPlotSummary {
    min: number;
    q1: number;
    median: number;
    q3: number;
    max: number;
}

export interface CategoryCount {
    value: string;
    count: number;
    percentage: number;
}

export interface TimeBin {
    label: string;
    count: number;
}

export interface StatisticalTestResult {
    test: string;
    statistic: number;
    p_value: number;
    df?: number | null;
    effect_size?: number | null;
    notes?: string | null;
}

export interface NumericColumnStats {
    min: number;
    max: number;
    mean: number;
    median: number;
    std_dev: number;
    variance: number;
    iqr: number;
    skewness: number;
    kurtosis: number;
    outliers_iqr: number;
    outliers_robust_z: number;
    histogram: HistogramBin[];
    box_plot: BoxPlotSummary;
    normality_tests: StatisticalTestResult[];
}

export interface CategoricalColumnStats {
    cardinality: number;
    entropy: number;
    gini: number;
    imbalance_ratio: number;
    top_values: CategoryCount[];
}

export interface TextColumnStats {
    min_length: number;
    max_length: number;
    mean_length: number;
    median_length: number;
    empty_percentage: number;
    whitespace_percentage: number;
    unique_token_count: number;
    length_histogram: HistogramBin[];
}

export interface DateTimeColumnStats {
    min: string;
    max: string;
    range_days: number;
    granularity: string;
    time_bins: TimeBin[];
}

export interface AnalysisColumnStats {
    profile: ColumnProfile;
    numeric?: NumericColumnStats | null;
    categorical?: CategoricalColumnStats | null;
    text?: TextColumnStats | null;
    datetime?: DateTimeColumnStats | null;
}

export interface MissingnessColumn {
    column: string;
    missing_count: number;
    missing_percentage: number;
}

export interface HeatmapMatrix {
    x_labels: string[];
    y_labels: string[];
    values: number[][];
    p_values?: number[][] | null;
}

export interface MissingnessAnalysis {
    total_missing_cells: number;
    total_missing_percentage: number;
    per_column: MissingnessColumn[];
    co_missing_matrix: HeatmapMatrix;
}

export interface CorrelationPair {
    column_x: string;
    column_y: string;
    method: string;
    estimate: number;
    p_value: number;
}

export interface CorrelationAnalysis {
    numeric_columns: string[];
    pearson: HeatmapMatrix;
    spearman: HeatmapMatrix;
    top_pairs: CorrelationPair[];
}

export interface NumericCategoricalAssociation {
    numeric_column: string;
    categorical_column: string;
    anova?: StatisticalTestResult | null;
    variance_test?: StatisticalTestResult | null;
    kruskal?: StatisticalTestResult | null;
    t_test?: StatisticalTestResult | null;
    mann_whitney?: StatisticalTestResult | null;
}

export interface AssociationAnalysis {
    categorical_columns: string[];
    cramers_v: HeatmapMatrix;
    chi_square: HeatmapMatrix;
    numeric_categorical: NumericCategoricalAssociation[];
}

export interface AnalysisResult {
    dataset: AnalysisDataset;
    generated_at: string;
    duration_ms: number;
    summary: AnalysisSummary;
    dataset_profile: DatasetProfile;
    columns: AnalysisColumnStats[];
    missingness: MissingnessAnalysis;
    correlations: CorrelationAnalysis;
    associations: AssociationAnalysis;
    quality_issues: DataQualityIssue[];
}

export interface AnalysisExportResult {
    report_path: string;
}

// ============================================================================
// PREPROCESSING UI STATE (for navigation persistence)
// ============================================================================

/**
 * UI state for the preprocessing page.
 * Persisted in Rust across navigation.
 * Mirrors: state.rs::PreprocessingUIState
 */
export interface PreprocessingUIState {
    /** Selected column names for preprocessing */
    selected_columns: string[];
    /** Row range to process (start, end indices), or null for all rows */
    row_range: [number, number] | null;
    /** Pipeline configuration settings */
    config: PipelineConfigRequest;
    /** Active tab in the results panel ("results" or "history") */
    active_results_tab: "results" | "history";
}

// ============================================================================
// DEFAULT CONFIG VALUES
// ============================================================================

/**
 * Default values for pipeline configuration.
 * Use these when creating a new preprocessing request.
 */
export const DEFAULT_PIPELINE_CONFIG: PipelineConfigRequest = {
    missing_column_threshold: 0.7,
    missing_row_threshold: 0.5,
    outlier_strategy: "cap",
    numeric_imputation: "median",
    categorical_imputation: "mode",
    enable_type_correction: true,
    remove_duplicates: true,
    knn_neighbors: 5,
    use_ai_decisions: true, // AI-powered "Smart Mode" is ON by default for non-technical users
    target_column: null,
};
