"use client";

import { cn } from "@/lib/utils";

interface LogoProps {
    className?: string;
}

/**
 * Lex Machina logo component.
 *
 * Displays the "LM" initials in bold monospace font as a text-based logo,
 * with the full name "Lex Machina" below.
 *
 * @example
 * ```tsx
 * <Logo />
 * <Logo className="mb-8" />
 * ```
 */
const Logo = ({ className }: LogoProps) => {
    return (
        <div
            className={cn("flex flex-col items-center select-none", className)}
        >
            {/* LM Initials - Large bold monospace */}
            <div
                className="text-foreground font-mono text-9xl font-bold tracking-tighter"
                aria-label="Lex Machina"
            >
                LM
            </div>
            {/* Full name - smaller, muted */}
            <div className="text-muted-foreground mt-2 text-base">
                Lex Machina
            </div>
        </div>
    );
};

export default Logo;
