"use client";

import Link from "next/link";
import { Brain, Sparkles, Eye, Target, Table2 } from "lucide-react";

import { useFileState } from "@/lib/hooks/use-file-state";
import AppShell from "@/components/layout/app-shell";
import { Button } from "@/components/ui/button";
import { formatNumber } from "@/lib/utils";

/**
 * Empty state component shown when no file is loaded.
 */
function NoFileLoadedState() {
    return (
        <div className="flex flex-1 items-center justify-center p-8">
            <div className="max-w-md text-center">
                {/* Icon */}
                <div className="bg-muted mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full">
                    <Brain className="text-muted-foreground h-8 w-8" />
                </div>

                {/* Title and description */}
                <h2 className="mb-2 text-xl font-semibold">Machine Learning</h2>
                <p className="text-muted-foreground mb-6">
                    Import a dataset to train ML models with automated
                    hyperparameter tuning and explainability.
                </p>

                {/* Features */}
                <ul className="text-muted-foreground mb-8 space-y-2 text-left text-sm">
                    <li className="flex items-center gap-3">
                        <Sparkles className="h-4 w-4 shrink-0" />
                        <span>AutoML with hyperparameter optimization</span>
                    </li>
                    <li className="flex items-center gap-3">
                        <Target className="h-4 w-4 shrink-0" />
                        <span>Classification and regression tasks</span>
                    </li>
                    <li className="flex items-center gap-3">
                        <Eye className="h-4 w-4 shrink-0" />
                        <span>SHAP and LIME explainability</span>
                    </li>
                    <li className="flex items-center gap-3">
                        <Brain className="h-4 w-4 shrink-0" />
                        <span>Model comparison and selection</span>
                    </li>
                </ul>

                {/* Action button */}
                <Button asChild size="lg">
                    <Link href="/data">
                        <Table2 className="mr-2 h-4 w-4" />
                        Go to Data
                    </Link>
                </Button>
            </div>
        </div>
    );
}

/**
 * ML Sidebar - Model configuration and training options.
 */
const MLSidebar = () => {
    const { fileInfo } = useFileState();

    if (!fileInfo) {
        return (
            <div className="p-4">
                <p className="text-muted-foreground text-sm">
                    Load a file to train models
                </p>
            </div>
        );
    }

    return (
        <div className="space-y-5 p-4">
            <section>
                <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                    Dataset
                </h2>
                <dl className="space-y-2 text-sm">
                    <div>
                        <dt className="text-muted-foreground">File</dt>
                        <dd className="truncate font-medium">
                            {fileInfo.name}
                        </dd>
                    </div>
                    <div>
                        <dt className="text-muted-foreground">Rows</dt>
                        <dd className="font-medium">
                            {formatNumber(fileInfo.row_count)}
                        </dd>
                    </div>
                    <div>
                        <dt className="text-muted-foreground">Columns</dt>
                        <dd className="font-medium">
                            {formatNumber(fileInfo.column_count)}
                        </dd>
                    </div>
                </dl>
            </section>

            <section>
                <h2 className="text-muted-foreground mb-3 text-xs font-semibold uppercase">
                    Model Configuration
                </h2>
                <p className="text-muted-foreground text-sm">Coming soon...</p>
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
        <div className="flex flex-1 items-center justify-center">
            <div className="text-center">
                <h2 className="mb-2 text-xl font-semibold">
                    AutoML Coming Soon
                </h2>
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
        <AppShell sidebar={<MLSidebar />}>
            <MLContent />
        </AppShell>
    );
};

export default MLPage;
