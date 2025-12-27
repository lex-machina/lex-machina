"use client";

import Link from "next/link";
import {
  BarChart3,
  PieChart,
  TrendingUp,
  FileSearch,
  Table2,
} from "lucide-react";

import { useFileState } from "@/lib/hooks/use-file-state";
import AppShell from "@/components/layout/app-shell";
import ContextSidebar from "@/components/layout/context-sidebar";
import { Button } from "@/components/ui/button";
import { formatNumber } from "@/lib/utils";

/**
 * Empty state component shown when no file is loaded.
 */
function NoFileLoadedState() {
  return (
    <div className="flex-1 flex items-center justify-center p-8">
      <div className="text-center max-w-md">
        {/* Icon */}
        <div className="mx-auto w-16 h-16 rounded-full bg-muted flex items-center justify-center mb-6">
          <BarChart3 className="w-8 h-8 text-muted-foreground" />
        </div>

        {/* Title and description */}
        <h2 className="text-xl font-semibold mb-2">Statistical Analysis</h2>
        <p className="text-muted-foreground mb-6">
          Import a dataset to explore statistical insights and visualize your data distributions.
        </p>

        {/* Features */}
        <ul className="text-sm text-muted-foreground space-y-2 mb-8 text-left">
          <li className="flex items-center gap-3">
            <TrendingUp className="w-4 h-4 shrink-0" />
            <span>Descriptive statistics (mean, median, std)</span>
          </li>
          <li className="flex items-center gap-3">
            <PieChart className="w-4 h-4 shrink-0" />
            <span>Distribution histograms and charts</span>
          </li>
          <li className="flex items-center gap-3">
            <Table2 className="w-4 h-4 shrink-0" />
            <span>Correlation matrix analysis</span>
          </li>
          <li className="flex items-center gap-3">
            <FileSearch className="w-4 h-4 shrink-0" />
            <span>Missing value detection and profiling</span>
          </li>
        </ul>

        {/* Action button */}
        <Button asChild size="lg">
          <Link href="/data">
            <Table2 className="w-4 h-4 mr-2" />
            Go to Data
          </Link>
        </Button>
      </div>
    </div>
  );
}

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
    return <NoFileLoadedState />;
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
