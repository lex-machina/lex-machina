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
    <div className={cn("flex flex-col items-center select-none", className)}>
      {/* LM Initials - Large bold monospace */}
      <div
        className="font-mono font-bold text-9xl tracking-tighter text-foreground"
        aria-label="Lex Machina"
      >
        LM
      </div>
      {/* Full name - smaller, muted */}
      <div className="text-base text-muted-foreground mt-2">Lex Machina</div>
    </div>
  );
};

export default Logo;
