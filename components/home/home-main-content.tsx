"use client";

import Link from "next/link";
import { Table2, Cog, BarChart3, Brain, Github } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import ExternalLink from "@/components/ui/external-link";

// ============================================================================
// TYPES
// ============================================================================

interface LinkItem {
    id: string;
    label: string;
    href: string;
    icon: LucideIcon;
    external?: boolean;
}

// ============================================================================
// LINK DATA
// ============================================================================

const WORKFLOW_LINKS: LinkItem[] = [
    { id: "data", label: "View Data", href: "/data", icon: Table2 },
    { id: "processing", label: "Process Data", href: "/processing", icon: Cog },
    { id: "analysis", label: "Analyze", href: "/analysis", icon: BarChart3 },
    { id: "ml", label: "Train Model", href: "/ml", icon: Brain },
];

const EXTERNAL_LINKS: LinkItem[] = [
    {
        id: "github",
        label: "GitHub",
        href: "https://github.com/sshussh/lex-machina",
        icon: Github,
        external: true,
    },
];

// ============================================================================
// LINK ITEM COMPONENT
// ============================================================================

/**
 * A single link item displayed as a clickable row.
 */
const LinkItemButton = ({ item }: { item: LinkItem }) => {
    const Icon = item.icon;

    const linkClasses = cn(
        "flex items-center gap-2 py-1.5 px-2 -mx-2 rounded",
        "text-sm text-muted-foreground hover:text-foreground",
        "hover:bg-muted/50 transition-colors",
    );

    if (item.external) {
        return (
            <ExternalLink href={item.href} className={linkClasses}>
                <Icon className="h-4 w-4" />
                {item.label}
            </ExternalLink>
        );
    }

    return (
        <Link href={item.href} className={linkClasses}>
            <Icon className="h-4 w-4" />
            {item.label}
        </Link>
    );
};

// ============================================================================
// LINK SECTION COMPONENT
// ============================================================================

interface LinkSectionProps {
    title: string;
    items: LinkItem[];
}

/**
 * A section of links with a title.
 */
const LinkSection = ({ title, items }: LinkSectionProps) => {
    return (
        <div>
            <h3 className="text-muted-foreground mb-2 text-xs font-semibold tracking-wider uppercase">
                {title}
            </h3>
            <div className="space-y-0.5">
                {items.map((item) => (
                    <LinkItemButton key={item.id} item={item} />
                ))}
            </div>
        </div>
    );
};

// ============================================================================
// MAIN CONTENT COMPONENT
// ============================================================================

interface HomeMainContentProps {
    className?: string;
}

/**
 * Main content area for the home page.
 *
 * Shows workflow links and external links in a two-column layout below the logo.
 */
const HomeMainContent = ({ className }: HomeMainContentProps) => {
    return (
        <div className={cn("grid max-w-md grid-cols-2 gap-8", className)}>
            <LinkSection title="Workflow" items={WORKFLOW_LINKS} />
            <LinkSection title="Links" items={EXTERNAL_LINKS} />
        </div>
    );
};

export default HomeMainContent;
