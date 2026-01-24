"use client";

/**
 * SidebarNav Component
 *
 * Navigation icons component that supports both horizontal and vertical layouts.
 * Used within the unified Sidebar component.
 *
 * ## Layouts
 *
 * - **Horizontal (expanded):** Icons in a row at top of sidebar
 * - **Vertical (collapsed):** Icons in a column, full height
 *
 * ## Design Principles
 *
 * - Single component handles both layouts (no duplication)
 * - Active route highlighting with `bg-primary text-primary-foreground`
 * - Tooltips on hover (vertical mode only - horizontal has labels)
 * - Muted colors for inactive items
 */

import { usePathname } from "next/navigation";
import Link from "next/link";
import { cn } from "@/lib/utils";
import {
    Home,
    Table2,
    Cog,
    BarChart3,
    LineChart,
    Brain,
    Settings,
    type LucideIcon,
} from "lucide-react";

// ============================================================================
// CONSTANTS
// ============================================================================

/** Width of the nav strip when in vertical (collapsed) mode */
export const NAV_STRIP_WIDTH = 56;

/** Size of each nav icon button (h-8 w-8 = 32px) */
const NAV_ICON_SIZE = 32;

/** Gap between nav items (gap-1 = 4px) */
const NAV_GAP = 4;

/** Horizontal padding on nav container (px-2 = 8px each side) */
const NAV_PADDING = 8 * 2;

/** Separator width including margins (w-px + mx-0.5 = 1 + 4px) */
const NAV_SEPARATOR_WIDTH = 1 + 4;

/** Number of main nav items */
const MAIN_NAV_COUNT = 6;

/** Number of bottom nav items (settings) */
const BOTTOM_NAV_COUNT = 1;

/** Total number of nav items */
const TOTAL_NAV_COUNT = MAIN_NAV_COUNT + BOTTOM_NAV_COUNT;

/**
 * Minimum width for horizontal nav (merged sidebar).
 * Calculated as: icons + gaps + separator + padding
 */
export const MIN_HORIZONTAL_NAV_WIDTH =
    TOTAL_NAV_COUNT * NAV_ICON_SIZE +
    (TOTAL_NAV_COUNT - 1) * NAV_GAP +
    NAV_SEPARATOR_WIDTH +
    NAV_PADDING;

// ============================================================================
// TYPES
// ============================================================================

/**
 * Navigation item configuration.
 */
interface NavItem {
    id: string;
    label: string;
    href: string;
    icon: LucideIcon;
}

interface SidebarNavProps {
    /**
     * Layout orientation.
     * - `horizontal`: Icons in a row (for expanded sidebar)
     * - `vertical`: Icons in a column (for collapsed sidebar or standalone nav)
     */
    orientation: "horizontal" | "vertical";

    /**
     * Which side of the screen the nav is on.
     * Affects tooltip positioning and border side.
     * - `left`: Nav on left side, tooltips appear on right, border on right
     * - `right`: Nav on right side, tooltips appear on left, border on left
     * @default "right"
     */
    side?: "left" | "right";
}

// ============================================================================
// NAVIGATION ITEMS
// ============================================================================

/**
 * Main navigation items.
 * Order: Home, Data, Processing, Analysis, Visualizations, ML
 */
const MAIN_NAV_ITEMS: NavItem[] = [
    {
        id: "home",
        label: "Home",
        href: "/home",
        icon: Home,
    },
    {
        id: "data",
        label: "Data",
        href: "/data",
        icon: Table2,
    },
    {
        id: "processing",
        label: "Processing",
        href: "/processing",
        icon: Cog,
    },
    {
        id: "analysis",
        label: "Analysis",
        href: "/analysis",
        icon: BarChart3,
    },
    {
        id: "visualizations",
        label: "Visualizations",
        href: "/visualizations",
        icon: LineChart,
    },
    {
        id: "ml",
        label: "ML",
        href: "/ml",
        icon: Brain,
    },
];

/**
 * Bottom navigation items (settings).
 * Separated from main nav with a divider in vertical mode.
 */
const BOTTOM_NAV_ITEMS: NavItem[] = [
    {
        id: "settings",
        label: "Settings",
        href: "/settings",
        icon: Settings,
    },
];

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

/**
 * Single navigation item for vertical layout.
 * Shows icon only with tooltip on hover.
 */
function VerticalNavItem({
    item,
    isActive,
    side = "right",
}: {
    item: NavItem;
    isActive: boolean;
    /** Which side of the screen - affects tooltip positioning */
    side?: "left" | "right";
}) {
    const Icon = item.icon;

    // Tooltip positioning based on which side the nav is on
    const tooltipPositionClasses =
        side === "left"
            ? "left-full ml-2" // Nav on left, tooltip on right
            : "right-full mr-2"; // Nav on right, tooltip on left

    return (
        <div className="group relative">
            <Link
                href={item.href}
                className={cn(
                    "my-1 flex h-10 w-10 items-center justify-center rounded-lg",
                    "transition-colors duration-150",
                    isActive
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground hover:bg-muted hover:text-foreground",
                )}
                title={item.label}
            >
                <Icon size={20} />
            </Link>

            {/* Tooltip - appears on hover */}
            <div
                className={cn(
                    "absolute rounded px-2 py-1 text-xs font-medium",
                    "bg-popover text-popover-foreground border shadow-md",
                    "opacity-0 transition-opacity duration-150 group-hover:opacity-100",
                    "pointer-events-none z-50 whitespace-nowrap",
                    tooltipPositionClasses,
                )}
                style={{ top: "50%", transform: "translateY(-50%)" }}
            >
                {item.label}
            </div>
        </div>
    );
}

/**
 * Single navigation item for horizontal layout.
 * Shows icon only with tooltip on hover.
 */
function HorizontalNavItem({
    item,
    isActive,
}: {
    item: NavItem;
    isActive: boolean;
}) {
    const Icon = item.icon;

    return (
        <div className="group relative shrink-0">
            <Link
                href={item.href}
                className={cn(
                    "flex h-8 w-8 items-center justify-center rounded-md",
                    "transition-colors duration-150",
                    isActive
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground hover:bg-muted hover:text-foreground",
                )}
                title={item.label}
            >
                <Icon size={16} />
            </Link>

            {/* Tooltip - appears on hover, below the icon */}
            <div
                className={cn(
                    "absolute top-full left-1/2 mt-1 -translate-x-1/2 rounded px-2 py-1 text-xs font-medium",
                    "bg-popover text-popover-foreground border shadow-md",
                    "opacity-0 transition-opacity duration-150 group-hover:opacity-100",
                    "pointer-events-none z-50 whitespace-nowrap",
                )}
            >
                {item.label}
            </div>
        </div>
    );
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

/**
 * SidebarNav Component
 *
 * Renders navigation items in horizontal or vertical layout.
 *
 * @example
 * ```tsx
 * // Horizontal (in expanded sidebar)
 * <SidebarNav orientation="horizontal" />
 *
 * // Vertical on right side (in collapsed sidebar)
 * <SidebarNav orientation="vertical" side="right" />
 *
 * // Vertical on left side (standalone left nav bar)
 * <SidebarNav orientation="vertical" side="left" />
 * ```
 */
export function SidebarNav({ orientation, side = "right" }: SidebarNavProps) {
    const pathname = usePathname();

    /**
     * Check if a nav item is active based on current pathname.
     */
    const isActive = (item: NavItem): boolean => {
        // Handle root path - treat as home
        if (item.id === "home" && pathname === "/") {
            return true;
        }
        return pathname === item.href;
    };

    // Border class based on which side the nav is on
    const borderClass = side === "left" ? "border-r" : "border-l";

    // ========================================================================
    // VERTICAL LAYOUT (collapsed mode or standalone nav bar)
    // ========================================================================

    if (orientation === "vertical") {
        return (
            <nav
                className={cn(
                    "bg-background flex h-full shrink-0 flex-col items-center py-2",
                    borderClass,
                )}
                style={{ width: NAV_STRIP_WIDTH }}
            >
                {/* Main navigation items */}
                <div className="flex flex-col items-center">
                    {MAIN_NAV_ITEMS.map((item) => (
                        <VerticalNavItem
                            key={item.id}
                            item={item}
                            isActive={isActive(item)}
                            side={side}
                        />
                    ))}
                </div>

                {/* Spacer */}
                <div className="flex-1" />

                {/* Separator */}
                <div className="bg-border my-2 h-px w-8" />

                {/* Bottom navigation items (Settings) */}
                <div className="flex flex-col items-center">
                    {BOTTOM_NAV_ITEMS.map((item) => (
                        <VerticalNavItem
                            key={item.id}
                            item={item}
                            isActive={isActive(item)}
                            side={side}
                        />
                    ))}
                </div>
            </nav>
        );
    }

    // ========================================================================
    // HORIZONTAL LAYOUT (expanded mode)
    // ========================================================================

    return (
        <nav
            className="flex shrink-0 items-center gap-1 border-b px-2 py-2"
            style={{ minWidth: MIN_HORIZONTAL_NAV_WIDTH }}
        >
            {/* All navigation items */}
            {MAIN_NAV_ITEMS.map((item) => (
                <HorizontalNavItem
                    key={item.id}
                    item={item}
                    isActive={isActive(item)}
                />
            ))}

            {/* Separator */}
            <div className="bg-border mx-0.5 h-4 w-px shrink-0" />

            {/* Bottom items (Settings) */}
            {BOTTOM_NAV_ITEMS.map((item) => (
                <HorizontalNavItem
                    key={item.id}
                    item={item}
                    isActive={isActive(item)}
                />
            ))}
        </nav>
    );
}

export default SidebarNav;
