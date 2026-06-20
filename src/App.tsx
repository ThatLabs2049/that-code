import { useEffect, useRef, useState } from "react";
import { ChatScreen, type SettingsTab } from "./components/ChatScreen";
import { LocaleProvider } from "./context/LocaleContext";
import { useAppLocale } from "./hooks/useAppLocale";
import { useAppTheme } from "./hooks/useAppTheme";
import { getSettings } from "./lib/settings";
import "./styles/global.css";

function App() {
  const { locale, setLocale, refreshLocale } = useAppLocale();
  const { refreshTheme } = useAppTheme();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [settingsTab, setSettingsTab] = useState<SettingsTab | undefined>();
  const [onboardingOpen, setOnboardingOpen] = useState(false);
  const [bootstrapping, setBootstrapping] = useState(true);
  const settingsTriggerRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    void getSettings()
      .then((settings) => {
        if (!settings.onboardingCompleted) {
          setOnboardingOpen(true);
        }
      })
      .catch(() => {
        // Non-blocking if settings fail to load
      })
      .finally(() => {
        setBootstrapping(false);
      });
  }, []);

  function closeSettings() {
    setSettingsOpen(false);
    setSettingsTab(undefined);
    settingsTriggerRef.current?.focus();
  }

  function openSettings(tab?: SettingsTab) {
    setSettingsTab(tab);
    setSettingsOpen(true);
  }

  function handleSettingsChanged() {
    void refreshLocale();
    void refreshTheme();
  }

  if (bootstrapping) {
    return (
      <div className="app-bootstrap" role="status" aria-live="polite">
        …
      </div>
    );
  }

  return (
    <LocaleProvider locale={locale} setLocale={setLocale}>
      <ChatScreen
        settingsOpen={settingsOpen}
        settingsInitialTab={settingsTab}
        settingsTriggerRef={settingsTriggerRef}
        onboardingOpen={onboardingOpen}
        onOpenSettings={openSettings}
        onCloseSettings={closeSettings}
        onSettingsChanged={handleSettingsChanged}
        onOnboardingComplete={() => {
          setOnboardingOpen(false);
          void refreshLocale();
          void refreshTheme();
        }}
        onOnboardingSkip={() => {
          setOnboardingOpen(false);
          void refreshLocale();
          void refreshTheme();
        }}
      />
    </LocaleProvider>
  );
}

export default App;
