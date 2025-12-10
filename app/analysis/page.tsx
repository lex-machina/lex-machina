"use client";

import { useFileState } from "@/lib/hooks/use-file-state";
import AppShell from "@/components/layout/app-shell";
import ContextSidebar from "@/components/layout/context-sidebar";
import { formatNumber } from "@/lib/utils";

/**
 * Analysis Toolbar - Actions for the analysis page.
 */
const AnalysisToolbar = () => {
  return (
    <span className="text-sm text-muted-foreground">
      Statistical Analysis
    </span>
  );
};

/**
 * Analysis Sidebar - Analysis options and results summary.
 */
const AnalysisSidebar = () => {
  const { fileInfo } = useFileState();

  if (!fileInfo) {
    return (
      <div className="p-5">
        <p className="text-sm text-muted-foreground">
          Load a file to run analysis
        </p>
      </div>
    );
  }

  return (
    <div className="p-5 space-y-5">
      <section>
        <h2 className="text-xs font-semibold uppercase text-muted-foreground mb-3">
          Dataset
        </h2>
        <dl className="space-y-2 text-sm">
          <div>
            <dt className="text-muted-foreground">File</dt>
            <dd className="font-medium truncate">{fileInfo.name}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Rows</dt>
            <dd className="font-medium">{formatNumber(fileInfo.row_count)}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Columns</dt>
            <dd className="font-medium">{formatNumber(fileInfo.column_count)}</dd>
          </div>
        </dl>
      </section>

      <section>
        <h2 className="text-xs font-semibold uppercase text-muted-foreground mb-3">
          Analysis Options
        </h2>
        <p className="text-sm text-muted-foreground">Coming soon...</p>
      </section>
    </div>
  );
};

/**
 * Analysis Content - Main analysis workspace.
 */
const AnalysisContent = () => {
  const { isFileLoaded } = useFileState();

  if (!isFileLoaded) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-center">
          <h2 className="text-xl font-semibold mb-2">No Data Loaded</h2>
          <p className="text-muted-foreground">
            Import a CSV file from the Data page to run analysis
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="text-center">
        <h2 className="text-xl font-semibold mb-2">Analysis Coming Soon</h2>
        <p className="text-muted-foreground max-w-md">
          This page will provide statistical analysis, data profiling,
          histograms, correlation matrices, and more.
        </p>
      </div>
    </div>
  );
};

/**
 * Analysis page - Statistical analysis and data profiling.
 *
 * Features (planned):
 * - Descriptive statistics
 * - Data profiling
 * - Histograms and distributions
 * - Correlation analysis
 * - Missing value analysis
 */
const AnalysisPage = () => {
  return (
    <AppShell
      toolbar={<AnalysisToolbar />}
      sidebar={
        <ContextSidebar visible={true}>
          <AnalysisSidebar />
        </ContextSidebar>
      }
    >
      <AnalysisContent />
    </AppShell>
  );
};

export default AnalysisPage;
