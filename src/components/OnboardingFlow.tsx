import { useCallback, useEffect, useRef, useState } from "react";
import { PersonalityCards } from "./PersonalityCards";
import { getSettings, testAiConnection, updateSettings, type UpdateAiSettings } from "../lib/settings";
import { isLocalProvider } from "../lib/connection";
import { invokeErrorMessage } from "../lib/invokeError";
import { isUiLocale, type UiLocale } from "../lib/i18n";
import { useLocale } from "../context/LocaleContext";
import { useFocusTrap } from "../hooks/useFocusTrap";
import "./OnboardingFlow.css";

interface OnboardingFlowProps {
  onComplete: () => void;
  onSkip: () => void;
}

type Step = "welcome" | "connection" | "workspace" | "personality";

export function OnboardingFlow({ onComplete, onSkip }: OnboardingFlowProps) {
  const { locale, setLocale, translate } = useLocale();
  const dialogRef = useFocusTrap(true);
  const composerRef = useRef<HTMLTextAreaElement>(null);
  const [step, setStep] = useState<Step>("welcome");
  const [baseUrl, setBaseUrl] = useState("https://api.openai.com/v1");
  const [apiKey, setApiKey] = useState("");
  const [companionModel, setCompanionModel] = useState("gpt-4o-mini");
  const [workspacePath, setWorkspacePath] = useState("");
  const [personalityId, setPersonalityId] = useState("luna");
  const [uiLocale, setUiLocale] = useState<UiLocale>(locale);
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [testOk, setTestOk] = useState(false);

  useEffect(() => {
    void getSettings().then((settings) => {
      setBaseUrl(settings.baseUrl);
      setCompanionModel(settings.companionModel);
      setWorkspacePath(settings.workspacePath ?? "");
      setPersonalityId(settings.personalityId || "luna");
      setUiLocale(isUiLocale(settings.uiLocale) ? settings.uiLocale : "en");
      if (
        settings.apiKeyConfigured ||
        isLocalProvider(settings.baseUrl)
      ) {
        setTestOk(true);
      }
    });
  }, []);

  function markConnectionEdited() {
    setTestOk(false);
    setError(null);
  }

  const finish = useCallback(async () => {
    setSaving(true);
    setError(null);
    const update: UpdateAiSettings = {
      baseUrl,
      companionModel,
      personalityId,
      uiLocale,
      workspacePath: workspacePath.trim() || null,
      onboardingCompleted: true,
    };
    if (apiKey.trim()) {
      update.apiKey = apiKey.trim();
    }
    try {
      const saved = await updateSettings(update);
      setLocale(isUiLocale(saved.uiLocale) ? saved.uiLocale : "en");
      onComplete();
    } catch (err) {
      setError(invokeErrorMessage(err, translate("saveSettingsError")));
    } finally {
      setSaving(false);
    }
  }, [
    apiKey,
    baseUrl,
    companionModel,
    onComplete,
    personalityId,
    setLocale,
    translate,
    uiLocale,
    workspacePath,
  ]);

  async function handleTestConnection() {
    setTesting(true);
    setError(null);
    const probe: UpdateAiSettings = { baseUrl, companionModel };
    if (apiKey.trim()) probe.apiKey = apiKey.trim();
    try {
      await testAiConnection(probe);
      setTestOk(true);
    } catch (err) {
      setTestOk(false);
      setError(invokeErrorMessage(err, translate("testConnectionError")));
    } finally {
      setTesting(false);
    }
  }

  async function handleBrowseWorkspace() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        multiple: false,
        title: translate("workspacePath"),
      });
      if (typeof selected === "string") {
        setWorkspacePath(selected);
      }
    } catch {
      setError(translate("loadSettingsError"));
    }
  }

  async function handleSkip() {
    const update: UpdateAiSettings = {
      onboardingCompleted: true,
      uiLocale,
      personalityId,
    };
    if (baseUrl.trim()) update.baseUrl = baseUrl.trim();
    if (companionModel.trim()) update.companionModel = companionModel.trim();
    if (workspacePath.trim()) update.workspacePath = workspacePath.trim();
    if (apiKey.trim()) update.apiKey = apiKey.trim();
    try {
      await updateSettings(update);
    } catch {
      // Allow chat even if persist fails
    }
    onSkip();
  }

  async function handleContinueFromConnection() {
    if (isLocalProvider(baseUrl) || testOk) {
      setStep("workspace");
      return;
    }

    setTesting(true);
    setError(null);
    const probe: UpdateAiSettings = { baseUrl, companionModel };
    if (apiKey.trim()) probe.apiKey = apiKey.trim();

    try {
      await testAiConnection(probe);
      setTestOk(true);
      setStep("workspace");
    } catch (err) {
      setTestOk(false);
      setError(invokeErrorMessage(err, translate("testConnectionError")));
    } finally {
      setTesting(false);
    }
  }

  useEffect(() => {
    if (step === "personality") {
      composerRef.current?.focus();
    }
  }, [step]);

  return (
    <div className="onboarding-overlay" role="presentation">
      <div
        ref={dialogRef}
        className="onboarding"
        role="dialog"
        aria-modal="true"
        aria-labelledby="onboarding-title"
      >
        <header className="onboarding__header">
          <h2 id="onboarding-title" className="onboarding__title">
            {step === "welcome" && translate("onboardingWelcomeTitle")}
            {step === "connection" && translate("onboardingConnectionTitle")}
            {step === "workspace" && translate("onboardingWorkspaceTitle")}
            {step === "personality" && translate("onboardingPersonalityTitle")}
          </h2>
          <button type="button" className="onboarding__skip" onClick={() => void handleSkip()}>
            {translate("onboardingSkip")}
          </button>
        </header>

        {step === "welcome" && (
          <div className="onboarding__body">
            <p className="onboarding__lead">{translate("onboardingWelcomeLead")}</p>
            <p className="onboarding__text">{translate("onboardingWelcomeBody")}</p>
          </div>
        )}

        {step === "connection" && (
          <div className="onboarding__body">
            <p className="onboarding__text">{translate("onboardingConnectionBody")}</p>
            <label className="onboarding__field">
              <span>{translate("apiBaseUrl")}</span>
              <input
                type="url"
                value={baseUrl}
                onChange={(e) => {
                  markConnectionEdited();
                  setBaseUrl(e.target.value);
                }}
                required
                dir="ltr"
              />
            </label>
            <label className="onboarding__field">
              <span>{translate("apiKey")}</span>
              <input
                type="password"
                value={apiKey}
                onChange={(e) => {
                  markConnectionEdited();
                  setApiKey(e.target.value);
                }}
                placeholder={translate("onboardingApiKeyHint")}
                dir="ltr"
              />
            </label>
            <label className="onboarding__field">
              <span>{translate("companionModel")}</span>
              <input
                type="text"
                value={companionModel}
                onChange={(e) => {
                  markConnectionEdited();
                  setCompanionModel(e.target.value);
                }}
                required
                dir="ltr"
              />
            </label>
            <button
              type="button"
              className="onboarding__secondary"
              disabled={testing}
              onClick={() => void handleTestConnection()}
            >
              {testing ? translate("testing") : translate("testConnection")}
            </button>
            {testOk && (
              <p className="onboarding__success" role="status">
                {translate("onboardingConnectionOk")}
              </p>
            )}
          </div>
        )}

        {step === "workspace" && (
          <div className="onboarding__body">
            <p className="onboarding__text">{translate("onboardingWorkspaceBody")}</p>
            <label className="onboarding__field">
              <span>{translate("workspacePath")}</span>
              <input
                type="text"
                value={workspacePath}
                readOnly
                placeholder={translate("workspacePathPlaceholder")}
                dir="ltr"
              />
            </label>
            <button type="button" className="onboarding__secondary" onClick={() => void handleBrowseWorkspace()}>
              {translate("workspaceBrowse")}
            </button>
          </div>
        )}

        {step === "personality" && (
          <div className="onboarding__body">
            <p className="onboarding__text">{translate("onboardingPersonalityBody")}</p>
            <label className="onboarding__field">
              <span>{translate("uiLocale")}</span>
              <select
                value={uiLocale}
                onChange={(e) =>
                  setUiLocale(isUiLocale(e.target.value) ? e.target.value : "en")
                }
              >
                <option value="en">{translate("localeEn")}</option>
                <option value="fa">{translate("localeFa")}</option>
              </select>
            </label>
            <PersonalityCards value={personalityId} onChange={setPersonalityId} />
            <textarea ref={composerRef} className="sr-only" tabIndex={-1} aria-hidden="true" />
          </div>
        )}

        {error && (
          <p className="onboarding__error" role="alert">
            {error}
          </p>
        )}

        <footer className="onboarding__footer">
          {step !== "welcome" && (
            <button
              type="button"
              className="onboarding__secondary"
              onClick={() => {
                const order: Step[] = ["welcome", "connection", "workspace", "personality"];
                const idx = order.indexOf(step);
                if (idx > 0) setStep(order[idx - 1]!);
              }}
            >
              {translate("onboardingBack")}
            </button>
          )}
          <button
            type="button"
            className="onboarding__primary"
            disabled={saving || testing}
            onClick={() => {
              if (step === "welcome") setStep("connection");
              else if (step === "connection") void handleContinueFromConnection();
              else if (step === "workspace") setStep("personality");
              else void finish();
            }}
          >
            {saving
              ? translate("saving")
              : step === "personality"
                ? translate("onboardingFinish")
                : translate("onboardingNext")}
          </button>
        </footer>
      </div>
    </div>
  );
}
