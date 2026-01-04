"use client";

import { useFileState } from "@/lib/hooks/use-file-state";
import { useProcessingUIState } from "@/lib/hooks/use-processing-ui-state";
import AppShell from "@/components/layout/app-shell";
import {
    ProcessingProvider,
    ProcessingSidebar,
    ProcessingLayout,
} from "@/components/processing";

// ============================================================================
// PROCESSING PAGE
// ============================================================================

/**
 * Processing page - Configure and run data preprocessing.
 *
 * Features:
 * - Three-column desktop layout (Columns | Configuration | Progress/Results)
 * - Column selection with data type badges
 * - Row range selection
 * - Preprocessing configuration
 * - Real-time progress tracking
 * - Results summary
 * - Processing history
 *
 * Layout is designed for desktop use with dense information display
 * and no scrolling on the main page (panels scroll internally).
 *
 * Architecture:
 * - useProcessingUIState: Handles UI state persistence to Rust
 * - ProcessingProvider: Combines file state, preprocessing ops, and settings
 * - ProcessingLayout: Three-column layout with extracted panel components
 * - ProcessingSidebar: Start button + selection summary (in right sidebar)
 */
export default function ProcessingPage() {
    // File state for UI state persistence
    const { fileInfo } = useFileState();

    // UI state with Rust persistence
    const {
        selectedColumns,
        setSelectedColumns,
        rowRange,
        setRowRange,
        config,
        setConfig,
        activeResultsTab,
        setActiveResultsTab,
    } = useProcessingUIState(fileInfo);

    return (
        <ProcessingProvider
            selectedColumns={selectedColumns}
            setSelectedColumns={setSelectedColumns}
            rowRange={rowRange}
            setRowRange={setRowRange}
            config={config}
            setConfig={setConfig}
            activeResultsTab={activeResultsTab}
            setActiveResultsTab={setActiveResultsTab}
        >
            <AppShell sidebar={<ProcessingSidebar />}>
                <ProcessingLayout />
            </AppShell>
        </ProcessingProvider>
    );
}
