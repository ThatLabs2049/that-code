import { createContext, useContext, type ReactNode } from "react";
import {
  applyDocumentLocale,
  t,
  type MessageKey,
  type UiLocale,
} from "../lib/i18n";

interface LocaleContextValue {
  locale: UiLocale;
  setLocale: (locale: UiLocale) => void;
  translate: (key: MessageKey) => string;
}

const LocaleContext = createContext<LocaleContextValue | null>(null);

export function LocaleProvider({
  locale,
  setLocale,
  children,
}: {
  locale: UiLocale;
  setLocale: (locale: UiLocale) => void;
  children: ReactNode;
}) {
  applyDocumentLocale(locale);

  const value: LocaleContextValue = {
    locale,
    setLocale,
    translate: (key) => t(locale, key),
  };

  return <LocaleContext.Provider value={value}>{children}</LocaleContext.Provider>;
}

export function useLocale() {
  const context = useContext(LocaleContext);
  if (!context) {
    throw new Error("useLocale must be used within LocaleProvider");
  }
  return context;
}
