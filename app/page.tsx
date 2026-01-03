"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";

/**
 * Root page - redirects to /home.
 *
 * This page serves as the entry point and will later be used
 * to handle authentication state and redirect accordingly
 * (e.g., to /login if not authenticated, or /home if authenticated).
 */
export default function RootPage() {
  const router = useRouter();

  useEffect(() => {
    router.replace("/home");
  }, [router]);

  // Return null while redirecting - this is a client-side desktop app
  // so there's no flash of content to worry about
  return null;
}
