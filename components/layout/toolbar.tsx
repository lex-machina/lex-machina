"use client";

import type { ReactNode } from "react";

interface ToolbarProps {
    /** Content to render on the left side of the toolbar */
    children?: ReactNode;
}

/**
 * Top toolbar component with slots for page-specific actions.
 *
 * This is a generic toolbar that provides the layout structure.
 * Page-specific buttons/actions are passed as children.
 *
 * The app title "Lex Machina" is always displayed on the right.
 *
 * @example
 * ```tsx
 * // In a page component
 * <Toolbar>
 *   <Button onClick={handleImport}>Import File</Button>
 *   <Button onClick={handleClear}>Clear</Button>
 * </Toolbar>
 * ```
 */
const Toolbar = ({ children }: ToolbarProps) => {
    return (
        <header className="bg-background flex h-12 items-center justify-between border-b px-5">
            <div className="flex items-center gap-2">{children}</div>
            <h1 className="text-muted-foreground text-lg font-bold">
                Lex Machina
            </h1>
        </header>
    );
};

export default Toolbar;
