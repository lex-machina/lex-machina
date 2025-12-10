"use client";

import { usePathname } from "next/navigation";
import Link from "next/link";
import { cn } from "@/lib/utils";
import { useFileState } from "@/lib/hooks/use-file-state";

/**
 * Navigation items for the left sidebar.
 * Each item can optionally require a file to be loaded.
 */
interface NavItem {
  id: string;
  label: string;
  href: string;
  icon: React.ReactNode;
  /** If true, item is disabled when no file is loaded */
  requiresFile?: boolean;
}

/**
 * Simple icon components using CSS/text for now.
 * In production, replace with proper icon library (e.g., lucide-react).
 */
const HomeIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="20"
    height="20"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
    <polyline points="9 22 9 12 15 12 15 22" />
  </svg>
);

const DataIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="20"
    height="20"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
    <line x1="3" y1="9" x2="21" y2="9" />
    <line x1="3" y1="15" x2="21" y2="15" />
    <line x1="9" y1="3" x2="9" y2="21" />
    <line x1="15" y1="3" x2="15" y2="21" />
  </svg>
);

const AnalysisIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="20"
    height="20"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <line x1="18" y1="20" x2="18" y2="10" />
    <line x1="12" y1="20" x2="12" y2="4" />
    <line x1="6" y1="20" x2="6" y2="14" />
  </svg>
);

const MLIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="20"
    height="20"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M12 2a4 4 0 0 1 4 4c0 1.1-.4 2.1-1 3l7 7-3 3-7-7c-.9.6-1.9 1-3 1a4 4 0 0 1 0-8" />
    <path d="M12 12l-6 6" />
    <circle cx="7" cy="17" r="3" />
  </svg>
);

/**
 * Navigation configuration.
 * Add new pages here as the app grows.
 */
const NAV_ITEMS: NavItem[] = [
  {
    id: "home",
    label: "Home",
    href: "/",
    icon: <HomeIcon />,
  },
  {
    id: "data",
    label: "Data",
    href: "/data",
    icon: <DataIcon />,
    requiresFile: false, // Data page available to import files
  },
  {
    id: "analysis",
    label: "Analysis",
    href: "/analysis",
    icon: <AnalysisIcon />,
    requiresFile: true,
  },
  {
    id: "ml",
    label: "ML",
    href: "/ml",
    icon: <MLIcon />,
    requiresFile: true,
  },
];

const NAV_WIDTH = 56;

/**
 * Left navigation sidebar with icon-based navigation.
 *
 * Features:
 * - Icon-only design with tooltips (hover to see label)
 * - Active state based on current route
 * - Disabled state for items requiring a file to be loaded
 * - Fixed width of 56px
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
  const { isFileLoaded } = useFileState();

  return (
    <nav
      className="shrink-0 flex flex-col items-center py-2 border-r bg-background"
      style={{ width: NAV_WIDTH }}
    >
      {NAV_ITEMS.map((item) => {
        const isActive = pathname === item.href;
        const isDisabled = item.requiresFile && !isFileLoaded;

        return (
          <div key={item.id} className="relative group">
            {isDisabled ? (
              // Disabled state - not clickable
              <div
                className={cn(
                  "flex items-center justify-center w-10 h-10 rounded-lg my-1",
                  "text-muted-foreground/40 cursor-not-allowed"
                )}
                title={`${item.label} (requires file)`}
              >
                {item.icon}
              </div>
            ) : (
              // Active/normal state - clickable link
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
                {item.icon}
              </Link>
            )}

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
              {isDisabled && (
                <span className="text-muted-foreground ml-1">(no file)</span>
              )}
            </div>
          </div>
        );
      })}
    </nav>
  );
};

export default NavSidebar;
export { NAV_WIDTH };
