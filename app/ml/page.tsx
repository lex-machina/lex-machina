"use client";

import Link from "next/link";
import {
  Brain,
  Sparkles,
  Eye,
  Target,
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
          <Brain className="w-8 h-8 text-muted-foreground" />
        </div>

        {/* Title and description */}
        <h2 className="text-xl font-semibold mb-2">Machine Learning</h2>
        <p className="text-muted-foreground mb-6">
          Import a dataset to train ML models with automated hyperparameter tuning and explainability.
        </p>

        {/* Features */}
        <ul className="text-sm text-muted-foreground space-y-2 mb-8 text-left">
          <li className="flex items-center gap-3">
            <Sparkles className="w-4 h-4 shrink-0" />
            <span>AutoML with hyperparameter optimization</span>
          </li>
          <li className="flex items-center gap-3">
            <Target className="w-4 h-4 shrink-0" />
            <span>Classification and regression tasks</span>
          </li>
          <li className="flex items-center gap-3">
            <Eye className="w-4 h-4 shrink-0" />
            <span>SHAP and LIME explainability</span>
          </li>
          <li className="flex items-center gap-3">
            <Brain className="w-4 h-4 shrink-0" />
            <span>Model comparison and selection</span>
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
 * ML Toolbar - Actions for the ML page.
 */
const MLToolbar = () => {
  return (
    <span className="text-sm text-muted-foreground">
      Machine Learning
    </span>
  );
};

/**
 * ML Sidebar - Model configuration and training options.
 */
const MLSidebar = () => {
  const { fileInfo } = useFileState();

  if (!fileInfo) {
    return (
      <div className="p-5">
        <p className="text-sm text-muted-foreground">
          Load a file to train models
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
          Model Configuration
        </h2>
        <p className="text-sm text-muted-foreground">Coming soon...</p>
      </section>
    </div>
  );
};

/**
 * ML Content - Main ML workspace.
 */
const MLContent = () => {
  const { isFileLoaded } = useFileState();

  if (!isFileLoaded) {
    return <NoFileLoadedState />;
  }

  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="text-center">
        <h2 className="text-xl font-semibold mb-2">AutoML Coming Soon</h2>
        <p className="text-muted-foreground max-w-md">
          This page will provide automated machine learning with
          hyperparameter tuning, model comparison, and explainable AI
          (SHAP/LIME).
        </p>
      </div>
    </div>
  );
};

/**
 * ML page - Automated Machine Learning workspace.
 *
 * Features (planned):
 * - Target variable selection
 * - Feature engineering
 * - AutoML with Optuna
 * - Model training and evaluation
 * - SHAP/LIME explanations
 */
const MLPage = () => {
  return (
    <AppShell
      toolbar={<MLToolbar />}
      sidebar={
        <ContextSidebar visible={true}>
          <MLSidebar />
        </ContextSidebar>
      }
    >
      <MLContent />
    </AppShell>
  );
};

export default MLPage;
