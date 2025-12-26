"use client";

import "./globals.css";
import { useTheme } from "@/lib/hooks/use-theme";

/**
 * Theme Provider Component
 *
 * This component uses the useTheme hook to ensure the theme is applied
 * to the DOM. It renders its children without any wrapper elements.
 */
function ThemeProvider({ children }: { children: React.ReactNode }) {
  // useTheme hook handles:
  // 1. Fetching theme from Rust on mount
  // 2. Listening for theme change events
  // 3. Applying the resolved theme to the DOM (dark class on <html>)
  useTheme();
  return <>{children}</>;
}

/**
 * Root Layout
 *
 * The top-level layout for the entire application.
 * Applies global styles and provides theme management.
 *
 * Theme is managed by Rust (AppState) and applied via useTheme hook.
 * The "dark" class is added/removed from <html> by the hook.
 */
const RootLayout = ({ children }: Readonly<{ children: React.ReactNode }>) => {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <ThemeProvider>{children}</ThemeProvider>
      </body>
    </html>
  );
};

export default RootLayout;
