import { useCallback, useEffect, useState } from "react";

import { getSettings, testAiConnection, updateSettings, type UpdateAiSettings } from "../lib/settings";

import { isLocalProvider } from "../lib/connection";

import { invokeErrorMessage, invokeSettingsErrorMessage } from "../lib/invokeError";

import { isUiLocale, type UiLocale } from "../lib/i18n";

import { useLocale } from "../context/LocaleContext";

import { useFocusTrap } from "../hooks/useFocusTrap";

import { indexWorkspaceRag, testEmbeddingConnection } from "../lib/rag";

import "./OnboardingFlow.css";



interface OnboardingFlowProps {

  onComplete: () => void;

  onSkip: () => void;

}



type Step = "welcome" | "connection" | "workspace" | "index";



const DEFAULT_EMBEDDING_URL = "http://localhost:11434/v1";

const DEFAULT_EMBEDDING_MODEL = "nomic-embed-text";



export function OnboardingFlow({ onComplete, onSkip }: OnboardingFlowProps) {

  const { locale, setLocale, translate } = useLocale();

  const dialogRef = useFocusTrap(true);

  const [step, setStep] = useState<Step>("welcome");

  const [baseUrl, setBaseUrl] = useState("https://api.openai.com/v1");

  const [apiKey, setApiKey] = useState("");

  const [agentModel, setAgentModel] = useState("gpt-4o-mini");

  const [workspacePath, setWorkspacePath] = useState("");

  const [uiLocale, setUiLocale] = useState<UiLocale>(locale);

  const [enableIndex, setEnableIndex] = useState(true);

  const [embeddingBaseUrl, setEmbeddingBaseUrl] = useState(DEFAULT_EMBEDDING_URL);

  const [embeddingModel, setEmbeddingModel] = useState(DEFAULT_EMBEDDING_MODEL);

  const [testing, setTesting] = useState(false);

  const [indexing, setIndexing] = useState(false);

  const [saving, setSaving] = useState(false);

  const [error, setError] = useState<string | null>(null);

  const [testOk, setTestOk] = useState(false);

  const [indexOk, setIndexOk] = useState(false);



  useEffect(() => {

    void getSettings().then((settings) => {

      setBaseUrl(settings.baseUrl);

      setAgentModel(settings.agentModel);

      setWorkspacePath(settings.workspacePath ?? "");

      setEmbeddingBaseUrl(settings.embeddingBaseUrl || DEFAULT_EMBEDDING_URL);

      setEmbeddingModel(settings.embeddingModel || DEFAULT_EMBEDDING_MODEL);

      setUiLocale(isUiLocale(settings.uiLocale) ? settings.uiLocale : "en");

      if (settings.apiKeyConfigured || isLocalProvider(settings.baseUrl)) {

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

      agentModel,

      uiLocale,

      workspacePath: workspacePath.trim() || null,

      onboardingCompleted: true,

    };

    if (apiKey.trim()) {

      update.apiKey = apiKey.trim();

    }

    if (enableIndex && workspacePath.trim() && !indexOk) {
      setError(translate("onboardingRagIndexRequired"));
      setSaving(false);
      return;
    }

    if (enableIndex && workspacePath.trim() && indexOk) {

      update.ragEnabled = true;

      update.ragAutoIndex = true;

      update.embeddingBaseUrl = embeddingBaseUrl;

      update.embeddingModel = embeddingModel;

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

    agentModel,

    embeddingBaseUrl,

    embeddingModel,

    enableIndex,

    indexOk,

    onComplete,

    setLocale,

    translate,

    uiLocale,

    workspacePath,

  ]);



  async function handleTestConnection() {

    setTesting(true);

    setError(null);

    const probe: UpdateAiSettings = { baseUrl, agentModel };

    if (apiKey.trim()) probe.apiKey = apiKey.trim();

    try {

      await testAiConnection(probe);

      setTestOk(true);

    } catch (err) {

      setTestOk(false);

      setError(invokeSettingsErrorMessage(err, translate("testConnectionError")));

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

        setIndexOk(false);

      }

    } catch {

      setError(translate("loadSettingsError"));

    }

  }



  async function handleIndexWorkspace() {

    if (!workspacePath.trim()) return;

    setIndexing(true);

    setError(null);

    const probe: UpdateAiSettings = {

      ragEnabled: true,

      embeddingBaseUrl,

      embeddingModel,

    };

    try {

      await testEmbeddingConnection(probe);

      await indexWorkspaceRag(probe);

      setIndexOk(true);

    } catch (err) {

      setIndexOk(false);

      setError(invokeErrorMessage(err, translate("ragIndexError")));

    } finally {

      setIndexing(false);

    }

  }



  async function handleSkip() {

    const update: UpdateAiSettings = {

      onboardingCompleted: true,

      uiLocale,

    };

    if (baseUrl.trim()) update.baseUrl = baseUrl.trim();

    if (agentModel.trim()) update.agentModel = agentModel.trim();

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

    const probe: UpdateAiSettings = { baseUrl, agentModel };

    if (apiKey.trim()) probe.apiKey = apiKey.trim();



    try {

      await testAiConnection(probe);

      setTestOk(true);

      setStep("workspace");

    } catch (err) {

      setTestOk(false);

      setError(invokeSettingsErrorMessage(err, translate("testConnectionError")));

    } finally {

      setTesting(false);

    }

  }



  function handlePrimaryAction() {

    if (step === "welcome") {

      setStep("connection");

      return;

    }

    if (step === "connection") {

      void handleContinueFromConnection();

      return;

    }

    if (step === "workspace") {

      if (workspacePath.trim()) {

        setStep("index");

      } else {

        void finish();

      }

      return;

    }

    void finish();

  }



  const stepOrder: Step[] = ["welcome", "connection", "workspace", "index"];



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

            {step === "index" && translate("onboardingIndexTitle")}

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

              <span>{translate("agentModel")}</span>

              <input

                type="text"

                value={agentModel}

                onChange={(e) => {

                  markConnectionEdited();

                  setAgentModel(e.target.value);

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

          </div>

        )}



        {step === "index" && (

          <div className="onboarding__body">

            <p className="onboarding__text">{translate("onboardingIndexBody")}</p>

            <label className="onboarding__checkbox">

              <input

                type="checkbox"

                checked={enableIndex}

                onChange={(e) => setEnableIndex(e.target.checked)}

              />

              <span>{translate("onboardingIndexEnable")}</span>

            </label>

            {enableIndex && (

              <>

                <label className="onboarding__field">

                  <span>{translate("embeddingBaseUrl")}</span>

                  <input

                    type="url"

                    value={embeddingBaseUrl}

                    onChange={(e) => {

                      setEmbeddingBaseUrl(e.target.value);

                      setIndexOk(false);

                    }}

                    dir="ltr"

                  />

                </label>

                <label className="onboarding__field">

                  <span>{translate("embeddingModel")}</span>

                  <input

                    type="text"

                    value={embeddingModel}

                    onChange={(e) => {

                      setEmbeddingModel(e.target.value);

                      setIndexOk(false);

                    }}

                    dir="ltr"

                  />

                </label>

                <button

                  type="button"

                  className="onboarding__secondary"

                  disabled={indexing || !workspacePath.trim()}

                  onClick={() => void handleIndexWorkspace()}

                >

                  {indexing ? translate("indexingWorkspace") : translate("onboardingIndexNow")}

                </button>

                {indexOk && (

                  <p className="onboarding__success" role="status">

                    {translate("onboardingIndexOk")}

                  </p>

                )}

              </>

            )}

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

                const idx = stepOrder.indexOf(step);

                if (idx > 0) setStep(stepOrder[idx - 1]!);

              }}

            >

              {translate("onboardingBack")}

            </button>

          )}

          <button

            type="button"

            className="onboarding__primary"

            disabled={saving || testing || indexing}

            onClick={handlePrimaryAction}

          >

            {saving

              ? translate("saving")

              : step === "index" || (step === "workspace" && !workspacePath.trim())

                ? translate("onboardingFinish")

                : step === "workspace"

                  ? translate("onboardingNext")

                  : translate("onboardingNext")}

          </button>

        </footer>

      </div>

    </div>

  );

}


