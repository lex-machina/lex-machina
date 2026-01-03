"use client";

import { Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useProcessingContext } from "./context";

/**
 * Toolbar for the processing page with the start button.
 */
export function ProcessingToolbar() {
    const { canStart, isProcessing, startProcessing } = useProcessingContext();

    return (
        <div className="flex items-center gap-4">
            <Button
                variant="default"
                size="sm"
                onClick={startProcessing}
                disabled={!canStart || isProcessing}
            >
                <Play className="mr-1.5 h-3.5 w-3.5" />
                {isProcessing ? "Processing..." : "Start Processing"}
            </Button>
        </div>
    );
}
