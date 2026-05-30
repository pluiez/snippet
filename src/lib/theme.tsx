// ThemeApplier: pure-effect component that syncs the `<html>` class to
// the user's theme preference. Mount once per window — each window
// independently listens for settings-changed so all windows switch in
// lock-step.

import { useEffect } from "react";
import type { ThemePreference } from "./bindings/ThemePreference";

/**
 * Standalone applier for windows that don't have SettingsProvider (e.g.
 * onboarding). Reads theme from the value you pass; defaults to "system"
 * when null.
 */
export function ThemeApplier({ theme }: { theme: ThemePreference | null }) {
  useEffect(() => {
    const pref = theme ?? "system";

    const apply = (dark: boolean) => {
      document.documentElement.classList.toggle("dark", dark);
    };

    if (pref === "dark") {
      apply(true);
      return;
    }
    if (pref === "light") {
      apply(false);
      return;
    }

    // "system" — match OS preference and listen for changes.
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    apply(mq.matches);
    const handler = (e: MediaQueryListEvent) => apply(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [theme]);

  return null;
}
