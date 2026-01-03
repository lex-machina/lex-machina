"use client";

import { ReactNode, MouseEvent } from "react";
import { openExternalUrl, cn } from "@/lib/utils";

// ============================================================================
// TYPES
// ============================================================================

interface ExternalLinkProps {
    /** The URL to open in the system browser */
    href: string;
    /** Content to render inside the link */
    children: ReactNode;
    /** Additional CSS classes */
    className?: string;
}

// ============================================================================
// EXTERNAL LINK COMPONENT
// ============================================================================

/**
 * A button styled as a link that opens URLs in the system browser.
 *
 * Use this instead of `<a>` tags for external links in Tauri.
 * Standard `<a href="..." target="_blank">` doesn't work in Tauri's WebView.
 *
 * @example
 * ```tsx
 * <ExternalLink href="https://github.com/user/repo" className="text-blue-500">
 *   View on GitHub
 * </ExternalLink>
 * ```
 */
const ExternalLink = ({ href, children, className }: ExternalLinkProps) => {
    const handleClick = async (e: MouseEvent) => {
        e.preventDefault();
        try {
            await openExternalUrl(href);
        } catch (error) {
            console.error("Failed to open external URL:", error);
        }
    };

    return (
        <button type="button" onClick={handleClick} className={cn(className)}>
            {children}
        </button>
    );
};

export default ExternalLink;
