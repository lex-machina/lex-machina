"use client";

import { ProgressPanel } from "./progress-panel";
import { ResultsPanel } from "./results-panel";
import { useProcessingContext } from "./context";

/**
 * Right panel showing either progress (when running) or results/history (when idle).
 */
export function ResultsPanelWrapper() {
    const {
        status,
        progress,
        summary,
        error,
        cancelProcessing,
        reset,
        viewProcessedData,
        getHistory,
        clearHistory,
        loadHistoryEntry,
        activeResultsTab,
        setActiveResultsTab,
        isProcessing,
    } = useProcessingContext();

    const showProgress =
        status === "running" || status === "error" || status === "cancelled";

    return (
        <div className="flex h-full min-h-0 flex-col">
            {/* Progress Panel - Show when running or error/cancelled */}
            {showProgress ? (
                <ProgressPanel
                    status={status}
                    progress={progress}
                    onCancel={cancelProcessing}
                    onReset={reset}
                    error={error}
                />
            ) : (
                /* Results Panel with tabs (Results | History) - Show when idle or completed */
                <ResultsPanel
                    summary={summary}
                    onViewData={viewProcessedData}
                    getHistory={getHistory}
                    onSelectHistoryEntry={loadHistoryEntry}
                    onClearHistory={clearHistory}
                    disabled={isProcessing}
                    className="flex-1"
                    activeTab={activeResultsTab}
                    onActiveTabChange={setActiveResultsTab}
                />
            )}
        </div>
    );
}
