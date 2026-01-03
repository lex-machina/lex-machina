"use client";

import {
    createContext,
    useContext,
    useState,
    useCallback,
    useEffect,
    useId,
    type ReactNode,
    type KeyboardEvent,
} from "react";
import { cn } from "@/lib/utils";

// ============================================================================
// TYPES
// ============================================================================

/**
 * Context value for tabs state.
 */
interface TabsContextValue {
    /** Currently active tab value */
    activeTab: string;
    /** Set the active tab */
    setActiveTab: (value: string) => void;
    /** Base ID for accessibility attributes */
    baseId: string;
    /** List of tab values for keyboard navigation */
    tabValues: string[];
    /** Register a tab value */
    registerTab: (value: string) => void;
    /** Unregister a tab value */
    unregisterTab: (value: string) => void;
}

const TabsContext = createContext<TabsContextValue | null>(null);

function useTabsContext() {
    const context = useContext(TabsContext);
    if (!context) {
        throw new Error(
            "Tabs components must be used within a <Tabs> provider",
        );
    }
    return context;
}

// ============================================================================
// TABS ROOT
// ============================================================================

export interface TabsProps {
    /** The controlled value of the active tab */
    value?: string;
    /** The default active tab (uncontrolled) */
    defaultValue?: string;
    /** Callback when the active tab changes */
    onValueChange?: (value: string) => void;
    /** Additional class names */
    className?: string;
    /** Child components (TabsList, TabsContent) */
    children: ReactNode;
}

/**
 * Root component for a tabbed interface.
 *
 * Provides context for TabsList, TabsTrigger, and TabsContent components.
 *
 * @example
 * ```tsx
 * <Tabs defaultValue="tab1" onValueChange={(v) => console.log(v)}>
 *   <TabsList>
 *     <TabsTrigger value="tab1">Tab 1</TabsTrigger>
 *     <TabsTrigger value="tab2">Tab 2</TabsTrigger>
 *   </TabsList>
 *   <TabsContent value="tab1">Content 1</TabsContent>
 *   <TabsContent value="tab2">Content 2</TabsContent>
 * </Tabs>
 * ```
 */
export function Tabs({
    value,
    defaultValue = "",
    onValueChange,
    className,
    children,
}: TabsProps) {
    const [internalValue, setInternalValue] = useState(defaultValue);
    const [tabValues, setTabValues] = useState<string[]>([]);
    const baseId = useId();

    // Use controlled value if provided, otherwise use internal state
    const activeTab = value ?? internalValue;

    const setActiveTab = useCallback(
        (newValue: string) => {
            if (value === undefined) {
                setInternalValue(newValue);
            }
            onValueChange?.(newValue);
        },
        [value, onValueChange],
    );

    const registerTab = useCallback((tabValue: string) => {
        setTabValues((prev) => {
            if (prev.includes(tabValue)) return prev;
            return [...prev, tabValue];
        });
    }, []);

    const unregisterTab = useCallback((tabValue: string) => {
        setTabValues((prev) => prev.filter((v) => v !== tabValue));
    }, []);

    return (
        <TabsContext.Provider
            value={{
                activeTab,
                setActiveTab,
                baseId,
                tabValues,
                registerTab,
                unregisterTab,
            }}
        >
            <div className={cn("flex flex-col", className)} data-slot="tabs">
                {children}
            </div>
        </TabsContext.Provider>
    );
}

// ============================================================================
// TABS LIST
// ============================================================================

export interface TabsListProps {
    /** Additional class names */
    className?: string;
    /** TabsTrigger components */
    children: ReactNode;
}

/**
 * Container for tab triggers.
 *
 * Renders as a horizontal list with keyboard navigation support.
 */
export function TabsList({ className, children }: TabsListProps) {
    const { activeTab, setActiveTab, tabValues } = useTabsContext();

    const handleKeyDown = useCallback(
        (e: KeyboardEvent<HTMLDivElement>) => {
            const currentIndex = tabValues.indexOf(activeTab);
            if (currentIndex === -1) return;

            let nextIndex: number | null = null;

            switch (e.key) {
                case "ArrowLeft":
                    nextIndex =
                        currentIndex > 0
                            ? currentIndex - 1
                            : tabValues.length - 1;
                    break;
                case "ArrowRight":
                    nextIndex =
                        currentIndex < tabValues.length - 1
                            ? currentIndex + 1
                            : 0;
                    break;
                case "Home":
                    nextIndex = 0;
                    break;
                case "End":
                    nextIndex = tabValues.length - 1;
                    break;
                default:
                    return;
            }

            if (nextIndex !== null) {
                e.preventDefault();
                setActiveTab(tabValues[nextIndex]);
                // Focus the new tab trigger
                const nextTab = e.currentTarget.querySelector(
                    `[data-value="${tabValues[nextIndex]}"]`,
                ) as HTMLElement | null;
                nextTab?.focus();
            }
        },
        [activeTab, setActiveTab, tabValues],
    );

    return (
        <div
            role="tablist"
            aria-orientation="horizontal"
            className={cn(
                // Desktop-style tab bar
                "bg-muted text-muted-foreground inline-flex h-9 items-center gap-1 rounded-md p-1",
                className,
            )}
            onKeyDown={handleKeyDown}
            data-slot="tabs-list"
        >
            {children}
        </div>
    );
}

// ============================================================================
// TABS TRIGGER
// ============================================================================

export interface TabsTriggerProps {
    /** Unique value for this tab */
    value: string;
    /** Whether this tab is disabled */
    disabled?: boolean;
    /** Additional class names */
    className?: string;
    /** Tab label content */
    children: ReactNode;
}

/**
 * A clickable tab trigger button.
 *
 * Must be used within a TabsList.
 */
export function TabsTrigger({
    value,
    disabled = false,
    className,
    children,
}: TabsTriggerProps) {
    const { activeTab, setActiveTab, baseId, registerTab, unregisterTab } =
        useTabsContext();

    const isActive = activeTab === value;
    const triggerId = `${baseId}-trigger-${value}`;
    const panelId = `${baseId}-panel-${value}`;

    // Register this tab on mount
    useEffect(() => {
        registerTab(value);
        return () => unregisterTab(value);
    }, [value, registerTab, unregisterTab]);

    const handleClick = useCallback(() => {
        if (!disabled) {
            setActiveTab(value);
        }
    }, [disabled, setActiveTab, value]);

    const handleKeyDown = useCallback(
        (e: KeyboardEvent<HTMLButtonElement>) => {
            if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                if (!disabled) {
                    setActiveTab(value);
                }
            }
        },
        [disabled, setActiveTab, value],
    );

    return (
        <button
            type="button"
            role="tab"
            id={triggerId}
            aria-selected={isActive}
            aria-controls={panelId}
            aria-disabled={disabled}
            tabIndex={isActive ? 0 : -1}
            data-state={isActive ? "active" : "inactive"}
            data-value={value}
            disabled={disabled}
            onClick={handleClick}
            onKeyDown={handleKeyDown}
            className={cn(
                // Base styles
                "inline-flex items-center justify-center rounded-sm px-3 py-1.5 whitespace-nowrap",
                "ring-offset-background text-sm font-medium transition-all",
                // Focus styles
                "focus-visible:ring-ring focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none",
                // Disabled styles
                "disabled:pointer-events-none disabled:opacity-50",
                // Active state
                "data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm",
                // Hover state (only when not active)
                "data-[state=inactive]:hover:bg-background/50 data-[state=inactive]:hover:text-foreground/80",
                className,
            )}
            data-slot="tabs-trigger"
        >
            {children}
        </button>
    );
}

// ============================================================================
// TABS CONTENT
// ============================================================================

export interface TabsContentProps {
    /** Value that matches the corresponding TabsTrigger */
    value: string;
    /** Whether to force mount the content (keep in DOM when hidden) */
    forceMount?: boolean;
    /** Additional class names */
    className?: string;
    /** Panel content */
    children: ReactNode;
}

/**
 * Content panel for a tab.
 *
 * Only renders when the corresponding tab is active (unless forceMount is true).
 */
export function TabsContent({
    value,
    forceMount = false,
    className,
    children,
}: TabsContentProps) {
    const { activeTab, baseId } = useTabsContext();

    const isActive = activeTab === value;
    const triggerId = `${baseId}-trigger-${value}`;
    const panelId = `${baseId}-panel-${value}`;

    // Don't render if not active and not force mounted
    if (!isActive && !forceMount) {
        return null;
    }

    return (
        <div
            role="tabpanel"
            id={panelId}
            aria-labelledby={triggerId}
            tabIndex={0}
            hidden={!isActive}
            data-state={isActive ? "active" : "inactive"}
            className={cn(
                // Base styles
                "ring-offset-background mt-2",
                // Focus styles
                "focus-visible:ring-ring focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none",
                // Hidden state (when forceMount but not active)
                "data-[state=inactive]:hidden",
                className,
            )}
            data-slot="tabs-content"
        >
            {children}
        </div>
    );
}

// ============================================================================
// CONVENIENCE EXPORTS
// ============================================================================

export default Tabs;
