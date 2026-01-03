"use client";

import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

// ============================================================================
// CARD HEADER
// ============================================================================

interface CardHeaderProps {
  /** The title text displayed in the header */
  title: string;
  /** Optional actions to display on the right side of the header */
  actions?: ReactNode;
  /** Additional class names */
  className?: string;
}

/**
 * Card header with uppercase title and optional actions.
 *
 * @example
 * ```tsx
 * <CardHeader title="Columns" />
 * <CardHeader title="Settings" actions={<Button size="sm">Reset</Button>} />
 * ```
 */
const CardHeader = ({ title, actions, className }: CardHeaderProps) => {
  return (
    <div
      className={cn(
        "flex items-center justify-between px-3 py-2 border-b bg-muted/30 shrink-0",
        className
      )}
    >
      <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
        {title}
      </h3>
      {actions && <div className="flex items-center gap-2">{actions}</div>}
    </div>
  );
};

// ============================================================================
// CARD CONTENT
// ============================================================================

interface CardContentProps {
  /** Content to render inside the card body */
  children: ReactNode;
  /** Whether the content should scroll when it overflows */
  scrollable?: boolean;
  /** Whether to add default padding */
  padded?: boolean;
  /** Additional class names */
  className?: string;
}

/**
 * Card content area with optional scrolling and padding.
 *
 * @example
 * ```tsx
 * <CardContent>Static content</CardContent>
 * <CardContent scrollable>Long scrollable content</CardContent>
 * <CardContent padded>Content with padding</CardContent>
 * ```
 */
const CardContent = ({
  children,
  scrollable = false,
  padded = false,
  className,
}: CardContentProps) => {
  return (
    <div
      className={cn(
        "flex-1 min-h-0",
        scrollable && "overflow-y-auto",
        padded && "p-3",
        className
      )}
    >
      {children}
    </div>
  );
};

// ============================================================================
// CARD FOOTER
// ============================================================================

interface CardFooterProps {
  /** Content to render in the footer */
  children: ReactNode;
  /** Additional class names */
  className?: string;
}

/**
 * Card footer with top border.
 *
 * @example
 * ```tsx
 * <CardFooter>
 *   <Button>Save</Button>
 * </CardFooter>
 * ```
 */
const CardFooter = ({ children, className }: CardFooterProps) => {
  return (
    <div
      className={cn(
        "px-3 py-2 border-t bg-muted/30 shrink-0",
        className
      )}
    >
      {children}
    </div>
  );
};

// ============================================================================
// CARD
// ============================================================================

interface CardProps {
  /** Content to render inside the card */
  children: ReactNode;
  /** Additional class names */
  className?: string;
}

/**
 * A card container with border and rounded corners.
 *
 * Use with CardHeader, CardContent, and CardFooter for structured layouts.
 * Designed for desktop applications with dense information displays.
 *
 * @example
 * ```tsx
 * <Card>
 *   <CardHeader title="Configuration" />
 *   <CardContent scrollable>
 *     <ConfigForm />
 *   </CardContent>
 * </Card>
 *
 * <Card>
 *   <CardHeader title="Columns" actions={<SelectAllButton />} />
 *   <CardContent scrollable>
 *     <ColumnList />
 *   </CardContent>
 *   <CardFooter>
 *     <WarningMessage />
 *   </CardFooter>
 * </Card>
 * ```
 */
const Card = ({ children, className }: CardProps) => {
  return (
    <div
      className={cn(
        "border rounded-lg overflow-hidden flex flex-col bg-background",
        className
      )}
    >
      {children}
    </div>
  );
};

export { Card, CardHeader, CardContent, CardFooter };
export default Card;
