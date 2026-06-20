import { useCallback, useEffect, useState } from "react";
import { getSettings } from "../lib/settings";
import { normalizeLocale, type UiLocale } from "../lib/i18n";

export function useAppLocale() {
  const [locale, setLocale] = useState<UiLocale>("en");

  const refreshLocale = useCallback(async () => {
    try {
      const settings = await getSettings();
      setLocale(normalizeLocale(settings.uiLocale));
    } catch {
      setLocale("en");
    }
  }, []);

  useEffect(() => {
    void refreshLocale();
  }, [refreshLocale]);

  return { locale, setLocale, refreshLocale };
}
