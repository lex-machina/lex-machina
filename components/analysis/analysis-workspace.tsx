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
    dataset: AnalysisResult["dataset"];
    onTabChange: (tab: string) => void;
    selectedColumn: string | null;
    onSelectColumn: (column: string) => void;
}

const AnalysisWorkspace = ({
    analysis,
    activeTab,
    dataset,
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
            <TabsList className="grid w-full grid-cols-6 gap-1 px-4">
                <TabsTrigger value="overview" className="w-full">
                    Overview
                </TabsTrigger>
                <TabsTrigger value="columns" className="w-full">
                    Columns
                </TabsTrigger>
                <TabsTrigger value="missingness" className="w-full">
                    Missingness
                </TabsTrigger>
                <TabsTrigger value="correlations" className="w-full">
                    Correlations
                </TabsTrigger>
                <TabsTrigger value="associations" className="w-full">
                    Associations
                </TabsTrigger>
                <TabsTrigger value="quality" className="w-full">
                    Quality
                </TabsTrigger>
            </TabsList>

            <TabsContent value="overview" className="h-full min-h-0 flex-1">
                <AnalysisOverview analysis={analysis} />
            </TabsContent>

            <TabsContent value="columns" className="h-full min-h-0 flex-1">
                <div className="flex h-full min-h-0 gap-3">
                    <div className="h-full min-h-0 flex-[1_1_0%]">
                        <AnalysisColumns
                            useProcessedData={dataset === "processed"}
                            selectedColumn={activeColumn?.profile.name ?? null}
                            onSelect={onSelectColumn}
                        />
                    </div>
                    <div className="h-full min-h-0 flex-[2_1_0%]">
                        <AnalysisColumnDetail column={activeColumn} />
                    </div>
                </div>
            </TabsContent>

            <TabsContent value="missingness" className="h-full min-h-0 flex-1">
                <AnalysisMissingness
                    dataset={dataset}
                    missingness={analysis.missingness}
                />
            </TabsContent>

            <TabsContent value="correlations" className="h-full min-h-0 flex-1">
                <AnalysisCorrelations
                    dataset={dataset}
                    correlations={analysis.correlations}
                />
            </TabsContent>

            <TabsContent value="associations" className="h-full min-h-0 flex-1">
                <AnalysisAssociations
                    dataset={dataset}
                    associations={analysis.associations}
                />
            </TabsContent>

            <TabsContent value="quality" className="h-full min-h-0 flex-1">
                <AnalysisQuality issues={analysis.quality_issues} />
            </TabsContent>
        </Tabs>
    );
};

export default AnalysisWorkspace;
