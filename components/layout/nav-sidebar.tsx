"use client";

import { usePathname } from "next/navigation";
import Link from "next/link";
import { cn } from "@/lib/utils";
import {
  Home,
  Table2,
  Cog,
  BarChart3,
  Brain,
  Settings,
  type LucideIcon,
} from "lucide-react";

/**
 * Navigation item configuration.
 */
interface NavItem {
  id: string;
  label: string;
  href: string;
  icon: LucideIcon;
}

/**
 * Main navigation items.
 * All items are always clickable - pages show appropriate empty states when needed.
 */
const NAV_ITEMS: NavItem[] = [
  {
    id: "home",
    label: "Home",
    href: "/",
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
    id: "ml",
    label: "ML",
    href: "/ml",
    icon: Brain,
  },
];

/**
 * Bottom navigation items (settings, etc.)
 * These are shown at the bottom of the sidebar, separated from main nav.
 */
const BOTTOM_NAV_ITEMS: NavItem[] = [
  {
    id: "settings",
    label: "Settings",
    href: "/settings",
    icon: Settings,
  },
];

const NAV_WIDTH = 56;

/**
 * Renders a single navigation item as a Link.
 * All items are always clickable - no disabled state.
 */
function NavItemButton({
  item,
  isActive,
}: {
  item: NavItem;
  isActive: boolean;
}) {
  const Icon = item.icon;

  return (
    <div className="relative group">
      <Link
        href={item.href}
        className={cn(
          "flex items-center justify-center w-10 h-10 rounded-lg my-1",
          "transition-colors duration-150",
          isActive
            ? "bg-primary text-primary-foreground"
            : "text-muted-foreground hover:bg-muted hover:text-foreground"
        )}
        title={item.label}
      >
        <Icon size={20} />
      </Link>

      {/* Tooltip - appears on hover */}
      <div
        className={cn(
          "absolute left-full ml-2 px-2 py-1 rounded text-xs font-medium",
          "bg-popover text-popover-foreground shadow-md border",
          "opacity-0 group-hover:opacity-100 transition-opacity duration-150",
          "pointer-events-none whitespace-nowrap z-50"
        )}
        style={{ top: "50%", transform: "translateY(-50%)" }}
      >
        {item.label}
      </div>
    </div>
  );
}

/**
 * Left navigation sidebar with icon-based navigation.
 *
 * Features:
 * - Icon-only design with tooltips (hover to see label)
 * - Active state based on current route
 * - All items always clickable (pages show empty states when needed)
 * - Fixed width of 56px
 * - Uses Lucide icons for consistency
 *
 * @example
 * ```tsx
 * // In app-shell.tsx
 * <div className="flex h-screen">
 *   <NavSidebar />
 *   <main className="flex-1">{children}</main>
 * </div>
 * ```
 */
const NavSidebar = () => {
  const pathname = usePathname();

  return (
    <nav
      className="shrink-0 flex flex-col items-center py-2 border-r bg-background"
      style={{ width: NAV_WIDTH }}
    >
      {/* Main navigation items */}
      <div className="flex flex-col items-center">
        {NAV_ITEMS.map((item) => (
          <NavItemButton
            key={item.id}
            item={item}
            isActive={pathname === item.href}
          />
        ))}
      </div>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Separator */}
      <div className="w-8 h-px bg-border my-2" />

      {/* Bottom navigation items (Settings) */}
      <div className="flex flex-col items-center">
        {BOTTOM_NAV_ITEMS.map((item) => (
          <NavItemButton
            key={item.id}
            item={item}
            isActive={pathname === item.href}
          />
        ))}
      </div>
    </nav>
  );
};

export default NavSidebar;
export { NAV_WIDTH };
