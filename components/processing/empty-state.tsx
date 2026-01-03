"use client";

import Link from "next/link";
import { Cog, Sparkles, Eraser, GitBranch, Table2 } from "lucide-react";
import { Button } from "@/components/ui/button";

/**
 * Empty state component shown when no file is loaded.
 */
export function EmptyState() {
    return (
        <div className="flex flex-1 items-center justify-center p-8">
            <div className="max-w-md text-center">
                {/* Icon */}
                <div className="bg-muted mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full">
                    <Cog className="text-muted-foreground h-8 w-8" />
                </div>

                {/* Title and description */}
                <h2 className="mb-2 text-xl font-semibold">
                    Data Preprocessing
                </h2>
                <p className="text-muted-foreground mb-6">
                    Import a dataset to clean, transform, and prepare your data
                    for analysis and ML.
                </p>

                {/* Features */}
                <ul className="text-muted-foreground mb-8 space-y-2 text-left text-sm">
                    <li className="flex items-center gap-3">
                        <Eraser className="h-4 w-4 shrink-0" />
                        <span>Missing value imputation (KNN, statistical)</span>
                    </li>
                    <li className="flex items-center gap-3">
                        <GitBranch className="h-4 w-4 shrink-0" />
                        <span>Outlier detection and handling</span>
                    </li>
                    <li className="flex items-center gap-3">
                        <Sparkles className="h-4 w-4 shrink-0" />
                        <span>AI-guided preprocessing decisions</span>
                    </li>
                    <li className="flex items-center gap-3">
                        <Cog className="h-4 w-4 shrink-0" />
                        <span>Type correction and data cleaning</span>
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
