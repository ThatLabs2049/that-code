import { useCallback, useEffect, useState } from "react";
import { getSettings } from "../lib/settings";
import type { ThemePreference } from "../lib/settings";

function resolveTheme(preference: ThemePreference): "dark" | "light" {
  if (preference === "dark") return "dark";
  if (preference === "light") return "light";
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

export function applyDocumentTheme(themePreference: ThemePreference): void {
  const root = document.documentElement;
  root.dataset.theme = themePreference;
  root.dataset.resolvedTheme = resolveTheme(themePreference);
}

export function useAppTheme() {
  const [themePreference, setThemePreference] = useState<ThemePreference>("system");

  const refreshTheme = useCallback(async () => {
    try {
      const settings = await getSettings();
      const theme = (settings.themePreference || "system") as ThemePreference;
      setThemePreference(theme);
      applyDocumentTheme(theme);
    } catch {
      applyDocumentTheme("system");
    }
  }, []);

  useEffect(() => {
    void refreshTheme();
  }, [refreshTheme]);

  useEffect(() => {
    if (themePreference !== "system") return;

    const media = window.matchMedia("(prefers-color-scheme: light)");
    const handler = () => applyDocumentTheme("system");
    media.addEventListener("change", handler);
    return () => media.removeEventListener("change", handler);
  }, [themePreference]);

  return { themePreference, refreshTheme };
}
