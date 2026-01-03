"use client";

import { EmptyState } from "./empty-state";
import { ColumnsPanel } from "./columns-panel";
import { ConfigPanelWrapper } from "./config-panel-wrapper";
import { ResultsPanelWrapper } from "./results-panel-wrapper";
import { useProcessingContext } from "./context";

/**
 * Three-column layout for the processing page.
 *
 * Shows empty state if no file is loaded, otherwise displays:
 * - Left: Column selector and row range
 * - Center: Configuration panel
 * - Right: Progress/Results/History
 */
export function ProcessingLayout() {
    const { isFileLoaded } = useProcessingContext();

    // Show empty state when no file is loaded
    if (!isFileLoaded) {
        return <EmptyState />;
    }

    return (
        <div className="grid min-h-0 flex-1 grid-cols-3 gap-4 p-4">
            {/* Left Panel - Columns & Row Range */}
            <div className="min-h-0">
                <ColumnsPanel />
            </div>

            {/* Center Panel - Configuration */}
            <div className="min-h-0">
                <ConfigPanelWrapper />
            </div>

            {/* Right Panel - Progress/Results/History */}
            <div className="min-h-0">
                <ResultsPanelWrapper />
            </div>
        </div>
    );
}
