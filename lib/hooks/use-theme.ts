"use client";

import { useEffect, useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRustEvent } from "./use-rust-event";
import { RUST_EVENTS, type Theme, type ThemeChangedPayload } from "@/types";

// ============================================================================
// TYPES
// ============================================================================

/**
 * The resolved theme after considering system preference.
 * This is always "light" or "dark", never "system".
 */
export type ResolvedTheme = "light" | "dark";

/**
 * State returned by the useTheme hook.
 */
export interface ThemeState {
  /** The current theme setting (may be "system") */
  theme: Theme;
  /** The actual theme being applied ("light" or "dark") */
  resolvedTheme: ResolvedTheme;
  /** Whether the system prefers dark mode */
  systemPrefersDark: boolean;
}

/**
 * Actions returned by the useTheme hook.
 */
export interface ThemeActions {
  /**
   * Sets the application theme.
   *
   * @param theme - The theme to apply ("system", "light", or "dark")
   */
  setTheme: (theme: Theme) => Promise<void>;

  /**
   * Toggles between light and dark themes.
   * If current theme is "system", it will switch to the opposite of the current resolved theme.
   */
  toggleTheme: () => Promise<void>;
}

/**
 * Return type of the useTheme hook.
 */
export type UseThemeReturn = ThemeState & ThemeActions;

// ============================================================================
// HELPERS
// ============================================================================

/**
 * Gets the system's preferred color scheme.
 */
function getSystemPreference(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

/**
 * Resolves the actual theme to apply based on the setting and system preference.
 */
function resolveTheme(theme: Theme, systemPrefersDark: boolean): ResolvedTheme {
  if (theme === "system") {
    return systemPrefersDark ? "dark" : "light";
  }
  return theme;
}

/**
 * Applies the theme to the document element.
 * This adds/removes the "dark" class on <html>.
 */
function applyThemeToDOM(resolvedTheme: ResolvedTheme): void {
  if (typeof document === "undefined") {
    return;
  }

  const root = document.documentElement;

  if (resolvedTheme === "dark") {
    root.classList.add("dark");
  } else {
    root.classList.remove("dark");
  }

  // Also set the color-scheme CSS property for native elements
  root.style.colorScheme = resolvedTheme;
}

// ============================================================================
// HOOK IMPLEMENTATION
// ============================================================================

/**
 * Hook for managing and applying the application theme.
 *
 * This hook:
 * - Fetches the theme setting from Rust on mount
 * - Listens for theme change events from Rust
 * - Monitors system preference changes
 * - Applies the resolved theme to the DOM
 *
 * @returns State and actions for theme management
 *
 * @example
 * ```tsx
 * // In your root layout
 * function RootLayout({ children }: { children: React.ReactNode }) {
 *   // This will apply the theme to the DOM automatically
 *   useTheme();
 *
 *   return (
 *     <html lang="en">
 *       <body>{children}</body>
 *     </html>
 *   );
 * }
 * ```
 *
 * @example
 * ```tsx
 * // In a theme toggle component
 * function ThemeToggle() {
 *   const { theme, resolvedTheme, setTheme, toggleTheme } = useTheme();
 *
 *   return (
 *     <div>
 *       <span>Current: {resolvedTheme}</span>
 *       <button onClick={toggleTheme}>Toggle</button>
 *       <select value={theme} onChange={(e) => setTheme(e.target.value as Theme)}>
 *         <option value="system">System</option>
 *         <option value="light">Light</option>
 *         <option value="dark">Dark</option>
 *       </select>
 *     </div>
 *   );
 * }
 * ```
 *
 * @remarks
 * The theme setting is stored in Rust (AppState). This hook syncs with Rust
 * and applies the theme to the DOM via CSS classes.
 *
 * The "dark" class is added to <html> when dark mode is active,
 * which works with Tailwind's dark mode class strategy.
 */
export function useTheme(): UseThemeReturn {
  // State
  const [theme, setThemeState] = useState<Theme>("system");
  const [systemPrefersDark, setSystemPrefersDark] = useState(false);

  // Derived state
  const resolvedTheme = resolveTheme(theme, systemPrefersDark);

  // ============================================================================
  // SYSTEM PREFERENCE MONITORING
  // ============================================================================

  /**
   * Initialize system preference and set up listener.
   */
  useEffect(() => {
    // Get initial system preference
    // eslint-disable-next-line react-hooks/set-state-in-effect -- Initial state sync on mount
    setSystemPrefersDark(getSystemPreference());

    // Listen for system preference changes
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    const handleChange = (e: MediaQueryListEvent) => {
      setSystemPrefersDark(e.matches);
    };

    // Modern browsers
    mediaQuery.addEventListener("change", handleChange);

    return () => {
      mediaQuery.removeEventListener("change", handleChange);
    };
  }, []);

  // ============================================================================
  // THEME APPLICATION
  // ============================================================================

  /**
   * Apply theme to DOM whenever resolved theme changes.
   */
  useEffect(() => {
    applyThemeToDOM(resolvedTheme);
  }, [resolvedTheme]);

  // ============================================================================
  // RUST SYNCHRONIZATION
  // ============================================================================

  /**
   * Fetch theme from Rust on mount.
   */
  useEffect(() => {
    const fetchTheme = async () => {
      try {
        const rustTheme = await invoke<Theme>("get_theme");
        setThemeState(rustTheme);
      } catch (err) {
        console.error("Failed to fetch theme from Rust:", err);
        // Fall back to system theme
        setThemeState("system");
      }
    };

    fetchTheme();
  }, []);

  /**
   * Handle theme changed event from Rust.
   */
  const handleThemeChanged = useCallback((newTheme: ThemeChangedPayload) => {
    setThemeState(newTheme);
  }, []);

  useRustEvent<ThemeChangedPayload>(
    RUST_EVENTS.THEME_CHANGED,
    handleThemeChanged
  );

  // ============================================================================
  // ACTIONS
  // ============================================================================

  /**
   * Sets the application theme.
   */
  const setTheme = useCallback(async (newTheme: Theme) => {
    try {
      await invoke("set_theme", { theme: newTheme });
      // Local state will be updated via the event handler
    } catch (err) {
      console.error("Failed to set theme:", err);
      // Optimistically update local state anyway
      setThemeState(newTheme);
    }
  }, []);

  /**
   * Toggles between light and dark themes.
   */
  const toggleTheme = useCallback(async () => {
    // Toggle to the opposite of the current resolved theme
    const newTheme: Theme = resolvedTheme === "dark" ? "light" : "dark";
    await setTheme(newTheme);
  }, [resolvedTheme, setTheme]);

  // ============================================================================
  // RETURN
  // ============================================================================

  return {
    // State
    theme,
    resolvedTheme,
    systemPrefersDark,

    // Actions
    setTheme,
    toggleTheme,
  };
}

/**
 * Initializes the theme on page load before React hydrates.
 *
 * Call this function in a script tag in your HTML head to prevent
 * flash of wrong theme (FOWT) on page load.
 *
 * @example
 * ```html
 * <script>
 *   // Inline script to prevent flash of wrong theme
 *   (function() {
 *     const theme = localStorage.getItem('theme') || 'system';
 *     const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
 *     const dark = theme === 'dark' || (theme === 'system' && systemDark);
 *     if (dark) document.documentElement.classList.add('dark');
 *   })();
 * </script>
 * ```
 *
 * @remarks
 * Note: In our Tauri app, the theme is stored in Rust AppState, not localStorage.
 * The above script is a fallback pattern. The actual theme will be applied
 * once React hydrates and fetches the theme from Rust.
 *
 * For a seamless experience, consider:
 * 1. Defaulting to "system" theme
 * 2. Using CSS that handles both themes gracefully during the brief loading period
 */
export function getThemeInitScript(): string {
  return `
    (function() {
      try {
        var systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        if (systemDark) {
          document.documentElement.classList.add('dark');
          document.documentElement.style.colorScheme = 'dark';
        }
      } catch (e) {}
    })();
  `;
}
