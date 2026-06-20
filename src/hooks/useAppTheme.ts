import { useCallback, useEffect, useState } from "react";
import { getSettings } from "../lib/settings";
import type { ThemePreference } from "../lib/settings";

function resolveTheme(preference: ThemePreference): "dark" | "light" {
  if (preference === "dark") return "dark";
  if (preference === "light") return "light";
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

export function applyDocumentTheme(
  personalityId: string,
  themePreference: ThemePreference,
): void {
  const root = document.documentElement;
  root.dataset.personality = personalityId || "luna";
  root.dataset.theme = themePreference;
  root.dataset.resolvedTheme = resolveTheme(themePreference);
}

export function useAppTheme() {
  const [personalityId, setPersonalityId] = useState("luna");
  const [themePreference, setThemePreference] = useState<ThemePreference>("system");

  const refreshTheme = useCallback(async () => {
    try {
      const settings = await getSettings();
      const pid = settings.personalityId || "luna";
      const theme = (settings.themePreference || "system") as ThemePreference;
      setPersonalityId(pid);
      setThemePreference(theme);
      applyDocumentTheme(pid, theme);
    } catch {
      applyDocumentTheme("luna", "system");
    }
  }, []);

  useEffect(() => {
    void refreshTheme();
  }, [refreshTheme]);

  useEffect(() => {
    if (themePreference !== "system") return;

    const media = window.matchMedia("(prefers-color-scheme: light)");
    const handler = () => applyDocumentTheme(personalityId, "system");
    media.addEventListener("change", handler);
    return () => media.removeEventListener("change", handler);
  }, [themePreference, personalityId]);

  return { personalityId, themePreference, refreshTheme };
}
