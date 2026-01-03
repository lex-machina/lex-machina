"use client";

import { Card, CardHeader, CardContent } from "@/components/ui/card";
import { ConfigPanel } from "./config-panel";
import { useProcessingContext } from "./context";

/**
 * Center panel wrapper for the ConfigPanel component.
 */
export function ConfigPanelWrapper() {
    const {
        config,
        setConfig,
        columns,
        selectedColumns,
        hasAIProvider,
        isProcessing,
    } = useProcessingContext();

    return (
        <Card className="h-full min-h-0">
            <CardHeader title="Configuration" />
            <CardContent scrollable>
                <ConfigPanel
                    config={config}
                    onConfigChange={setConfig}
                    columns={columns}
                    selectedColumns={selectedColumns}
                    hasAIProvider={hasAIProvider}
                    disabled={isProcessing}
                />
            </CardContent>
        </Card>
    );
}
