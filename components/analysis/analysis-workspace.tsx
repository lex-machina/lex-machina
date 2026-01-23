"use client";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { AnalysisResult } from "@/types";

import AnalysisAssociations from "./analysis-associations";
import AnalysisColumnDetail from "./analysis-column-detail";
import AnalysisColumns from "./analysis-columns";
import AnalysisCorrelations from "./analysis-correlations";
import AnalysisMissingness from "./analysis-missingness";
import AnalysisOverview from "./analysis-overview";
import AnalysisQuality from "./analysis-quality";

interface AnalysisWorkspaceProps {
    analysis: AnalysisResult;
    activeTab: string;
    onTabChange: (tab: string) => void;
    selectedColumn: string | null;
    onSelectColumn: (column: string) => void;
}

const AnalysisWorkspace = ({
    analysis,
    activeTab,
    onTabChange,
    selectedColumn,
    onSelectColumn,
}: AnalysisWorkspaceProps) => {
    const activeColumn =
        analysis.columns.find((col) => col.profile.name === selectedColumn) ??
        analysis.columns[0] ??
        null;

    return (
        <Tabs value={activeTab} onValueChange={onTabChange} className="h-full">
            <TabsList className="px-4">
                <TabsTrigger value="overview">Overview</TabsTrigger>
                <TabsTrigger value="columns">Columns</TabsTrigger>
                <TabsTrigger value="missingness">Missingness</TabsTrigger>
                <TabsTrigger value="correlations">Correlations</TabsTrigger>
                <TabsTrigger value="associations">Associations</TabsTrigger>
                <TabsTrigger value="quality">Quality</TabsTrigger>
            </TabsList>

            <TabsContent value="overview" className="flex-1">
                <AnalysisOverview analysis={analysis} />
            </TabsContent>

            <TabsContent value="columns" className="flex-1">
                <div className="grid h-full grid-cols-[280px_1fr] gap-3">
                    <AnalysisColumns
                        columns={analysis.columns}
                        selectedColumn={activeColumn?.profile.name ?? null}
                        onSelect={onSelectColumn}
                    />
                    <AnalysisColumnDetail column={activeColumn} />
                </div>
            </TabsContent>

            <TabsContent value="missingness" className="flex-1">
                <AnalysisMissingness missingness={analysis.missingness} />
            </TabsContent>

            <TabsContent value="correlations" className="flex-1">
                <AnalysisCorrelations correlations={analysis.correlations} />
            </TabsContent>

            <TabsContent value="associations" className="flex-1">
                <AnalysisAssociations associations={analysis.associations} />
            </TabsContent>

            <TabsContent value="quality" className="flex-1">
                <AnalysisQuality issues={analysis.quality_issues} />
            </TabsContent>
        </Tabs>
    );
};

export default AnalysisWorkspace;
