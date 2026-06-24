import { useCallback, useEffect, useState, type FormEvent } from "react";
import { clearHistory, getActiveConversation } from "../lib/chat";
import {
  getSettings,
  testAiConnection,
  updateSettings,
  type AiSettingsView,
  type UpdateAiSettings,
} from "../lib/settings";
import { createMemory, deleteMemory, listMemories, type Memory } from "../lib/memory";
import { getRagStatus, indexWorkspaceChanges, indexWorkspaceRag, testEmbeddingConnection } from "../lib/rag";
import { invokeErrorMessage, invokeSettingsErrorMessage } from "../lib/invokeError";
import { formatMessage, isUiLocale, type UiLocale, type MessageKey } from "../lib/i18n";
import { useLocale } from "../context/LocaleContext";
import { useFocusTrap } from "../hooks/useFocusTrap";
import { ConfirmDialog } from "./ConfirmDialog";
import type { SettingsTab } from "./ChatScreen";
import type { ThemePreference } from "../lib/settings";
import { MODEL_PRESETS, detectActivePreset } from "../lib/modelPresets";
import { MCP_PRESETS } from "../lib/mcpPresets";
import "./SettingsPanel.css";
import "./ConfirmDialog.css";

interface SettingsPanelProps {
  initialTab?: SettingsTab;
  onClose: () => void;
  onHistoryCleared?: () => void;
  onSettingsChanged?: () => void;
}

interface FormState {
  baseUrl: string;
  apiKey: string;
  agentModel: string;
  agentTemperature: string;
  fastModel: string;
  strongModel: string;
  autoEscalate: boolean;
  defaultAgentTier: import("../lib/settings").AgentTier;
  executorVisibility: boolean;
  uiLocale: UiLocale;
  workspacePath: string;
  allowFileOverwrites: boolean;
  ragEnabled: boolean;
  embeddingBaseUrl: string;
  embeddingModel: string;
  embeddingApiKey: string;
  ragTopK: string;
  verifyEnabled: boolean;
  verifyCommand: string;
  contextPackEnabled: boolean;
  commandAllowlistExtra: string;
  projectRulesEnabled: boolean;
  projectRulesFile: string;
  planBeforeEdit: boolean;
  editorOpenUrl: string;
  ragAutoIndex: boolean;
  taskQueueEnabled: boolean;
  mcpEnabled: boolean;
  mcpServerCommand: string;
  themePreference: ThemePreference;
}

function toFormState(settings: AiSettingsView): FormState {
  return {
    baseUrl: settings.baseUrl,
    apiKey: "",
    agentModel: settings.agentModel,
    agentTemperature: String(settings.agentTemperature),
    fastModel: settings.fastModel,
    strongModel: settings.strongModel,
    autoEscalate: settings.autoEscalate,
    defaultAgentTier: settings.defaultAgentTier ?? "auto",
    executorVisibility: settings.executorVisibility,
    uiLocale: isUiLocale(settings.uiLocale) ? settings.uiLocale : "en",
    workspacePath: settings.workspacePath ?? "",
    allowFileOverwrites: settings.allowFileOverwrites,
    ragEnabled: settings.ragEnabled,
    embeddingBaseUrl: settings.embeddingBaseUrl,
    embeddingModel: settings.embeddingModel,
    embeddingApiKey: "",
    ragTopK: String(settings.ragTopK),
    verifyEnabled: settings.verifyEnabled,
    verifyCommand: settings.verifyCommand ?? "",
    contextPackEnabled: settings.contextPackEnabled,
    commandAllowlistExtra: settings.commandAllowlistExtra.join("\n"),
    projectRulesEnabled: settings.projectRulesEnabled ?? true,
    projectRulesFile: settings.projectRulesFile ?? "",
    planBeforeEdit: settings.planBeforeEdit ?? false,
    editorOpenUrl: settings.editorOpenUrl ?? "vscode://file/{path}",
    ragAutoIndex: settings.ragAutoIndex,
    taskQueueEnabled: settings.taskQueueEnabled,
    mcpEnabled: settings.mcpEnabled,
    mcpServerCommand: settings.mcpServerCommand ?? "",
    themePreference: settings.themePreference ?? "system",
  };
}

export function SettingsPanel({
  initialTab,
  onClose,
  onHistoryCleared,
  onSettingsChanged,
}: SettingsPanelProps) {
  const { locale, setLocale, translate } = useLocale();
  const [activeTab, setActiveTab] = useState<SettingsTab>(initialTab ?? "connection");
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [form, setForm] = useState<FormState | null>(null);
  const [baselineForm, setBaselineForm] = useState<FormState | null>(null);
  const [confirmDiscardOpen, setConfirmDiscardOpen] = useState(false);
  const [confirmMcpOpen, setConfirmMcpOpen] = useState(false);
  const [apiKeyConfigured, setApiKeyConfigured] = useState(false);
  const [apiKeyMasked, setApiKeyMasked] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [confirmClearOpen, setConfirmClearOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [embeddingApiKeyConfigured, setEmbeddingApiKeyConfigured] = useState(false);
  const [embeddingApiKeyMasked, setEmbeddingApiKeyMasked] = useState("");
  const [ragChunkCount, setRagChunkCount] = useState(0);
  const [indexingRag, setIndexingRag] = useState(false);
  const [indexingChanges, setIndexingChanges] = useState(false);
  const [testingEmbedding, setTestingEmbedding] = useState(false);
  const [memories, setMemories] = useState<Memory[]>([]);
  const [newMemory, setNewMemory] = useState("");
  const [memoryBusy, setMemoryBusy] = useState(false);

  const requestClose = useCallback(() => {
    if (
      form &&
      baselineForm &&
      JSON.stringify(form) !== JSON.stringify(baselineForm)
    ) {
      setConfirmDiscardOpen(true);
      return;
    }
    onClose();
  }, [baselineForm, form, onClose]);

  const dialogRef = useFocusTrap(true, requestClose);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const settings = await getSettings();
      const nextForm = toFormState(settings);
      setForm(nextForm);
      setBaselineForm(nextForm);
      setApiKeyConfigured(settings.apiKeyConfigured);
      setApiKeyMasked(settings.apiKeyMasked);
      setEmbeddingApiKeyConfigured(settings.embeddingApiKeyConfigured);
      setEmbeddingApiKeyMasked(settings.embeddingApiKeyMasked);
      const ragStatus = await getRagStatus();
      setRagChunkCount(ragStatus.chunkCount);
      const storedMemories = await listMemories();
      setMemories(storedMemories);
    } catch {
      setError(translate("loadSettingsError"));
    } finally {
      setLoading(false);
    }
  }, [translate]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    if (initialTab) setActiveTab(initialTab);
  }, [initialTab]);

  function updateField<K extends keyof FormState>(key: K, value: FormState[K]) {
    setForm((current) => (current ? { ...current, [key]: value } : current));
    setSuccess(null);
    setTestResult(null);
  }

  async function handleAddMemory() {
    if (!newMemory.trim()) return;
    setMemoryBusy(true);
    try {
      const created = await createMemory(newMemory.trim());
      setMemories((current) => [created, ...current]);
      setNewMemory("");
    } catch (err) {
      setError(invokeErrorMessage(err, translate("memorySaveError")));
    } finally {
      setMemoryBusy(false);
    }
  }

  async function handleDeleteMemory(id: string) {
    setMemoryBusy(true);
    try {
      await deleteMemory(id);
      setMemories((current) => current.filter((memory) => memory.id !== id));
    } catch (err) {
      setError(invokeErrorMessage(err, translate("memoryDeleteError")));
    } finally {
      setMemoryBusy(false);
    }
  }

  async function handleSave(event: FormEvent) {
    event.preventDefault();
    if (!form) return;

    setSaving(true);
    setError(null);
    setSuccess(null);

    const agentTemperature = Number.parseFloat(form.agentTemperature);
    const ragTopK = Number.parseInt(form.ragTopK, 10);

    if (Number.isNaN(agentTemperature) || Number.isNaN(ragTopK)) {
      setError(translate("temperaturesInvalid"));
      setSaving(false);
      return;
    }

    if (form.baseUrl.trim() && form.agentModel.trim()) {
      const probe: UpdateAiSettings = {
        baseUrl: form.baseUrl,
        agentModel: form.agentModel,
        agentTemperature,
        fastModel: form.fastModel,
        strongModel: form.strongModel,
      };
      if (form.apiKey.trim()) {
        probe.apiKey = form.apiKey.trim();
      }
      try {
        const test = await testAiConnection(probe);
        if (!test.ok) {
          setError(test.message || translate("settingsConnectionTestRequired"));
          setSaving(false);
          return;
        }
      } catch (err) {
        setError(invokeErrorMessage(err, translate("settingsConnectionTestRequired")));
        setSaving(false);
        return;
      }
    }

    const update: UpdateAiSettings = {
      baseUrl: form.baseUrl,
      agentModel: form.agentModel,
      agentTemperature,
      fastModel: form.fastModel,
      strongModel: form.strongModel,
      autoEscalate: form.autoEscalate,
      defaultAgentTier: form.defaultAgentTier,
      executorVisibility: form.executorVisibility,
      uiLocale: form.uiLocale,
      workspacePath: form.workspacePath.trim() || null,
      allowFileOverwrites: form.allowFileOverwrites,
      ragEnabled: form.ragEnabled,
      embeddingBaseUrl: form.embeddingBaseUrl,
      embeddingModel: form.embeddingModel,
      ragTopK,
      verifyEnabled: form.verifyEnabled,
      verifyCommand: form.verifyCommand.trim() || null,
      contextPackEnabled: form.contextPackEnabled,
      commandAllowlistExtra: form.commandAllowlistExtra
        .split("\n")
        .map((line) => line.trim())
        .filter(Boolean),
      projectRulesEnabled: form.projectRulesEnabled,
      planBeforeEdit: form.planBeforeEdit,
      editorOpenUrl: form.editorOpenUrl.trim(),
      ragAutoIndex: form.ragAutoIndex,
      taskQueueEnabled: form.taskQueueEnabled,
      mcpEnabled: form.mcpEnabled,
      mcpServerCommand: form.mcpServerCommand.trim() || null,
      themePreference: form.themePreference,
    };

    if (form.embeddingApiKey.trim()) {
      update.embeddingApiKey = form.embeddingApiKey.trim();
    }

    if (form.apiKey.trim()) {
      update.apiKey = form.apiKey.trim();
    }

    try {
      const saved = await updateSettings(update);
      const nextForm = toFormState(saved);
      setForm(nextForm);
      setBaselineForm(nextForm);
      setApiKeyConfigured(saved.apiKeyConfigured);
      setApiKeyMasked(saved.apiKeyMasked);
      setEmbeddingApiKeyConfigured(saved.embeddingApiKeyConfigured);
      setEmbeddingApiKeyMasked(saved.embeddingApiKeyMasked);
      setLocale(isUiLocale(saved.uiLocale) ? saved.uiLocale : "en");
      setSuccess(translate("settingsSaved"));
      onSettingsChanged?.();
    } catch {
      setError(translate("saveSettingsError"));
    } finally {
      setSaving(false);
    }
  }

  async function handleTest() {
    if (!form) return;

    setTesting(true);
    setError(null);
    setTestResult(null);

    const agentTemperature = Number.parseFloat(form.agentTemperature);

    if (Number.isNaN(agentTemperature)) {
      setError(translate("temperaturesInvalid"));
      setTesting(false);
      return;
    }

    const probe: UpdateAiSettings = {
      baseUrl: form.baseUrl,
      agentModel: form.agentModel,
      agentTemperature,
      fastModel: form.fastModel,
      strongModel: form.strongModel,
    };

    if (form.apiKey.trim()) {
      probe.apiKey = form.apiKey.trim();
    }

    try {
      const result = await testAiConnection(probe);
      setTestResult(
        formatMessage(locale, "testConnectionSuccess", {
          model: result.model,
          message: result.message,
          latency: String(result.latencyMs),
        }),
      );
    } catch (err) {
      setError(invokeSettingsErrorMessage(err, translate("testConnectionError")));
    } finally {
      setTesting(false);
    }
  }

  async function handleBrowseWorkspace() {
    if (!form) return;

    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        multiple: false,
        title: translate("workspacePath"),
      });

      if (typeof selected === "string") {
        updateField("workspacePath", selected);
      }
    } catch {
      setError(translate("loadSettingsError"));
    }
  }

  function buildEmbeddingProbe(): UpdateAiSettings {
    if (!form) return {};

    const probe: UpdateAiSettings = {
      embeddingBaseUrl: form.embeddingBaseUrl,
      embeddingModel: form.embeddingModel,
    };

    if (form.embeddingApiKey.trim()) {
      probe.embeddingApiKey = form.embeddingApiKey.trim();
    }

    if (form.apiKey.trim()) {
      probe.apiKey = form.apiKey.trim();
    }

    return probe;
  }

  async function handleIndexRag() {
    if (!form) return;

    setIndexingRag(true);
    setError(null);
    setSuccess(null);

    try {
      const result = await indexWorkspaceRag(buildEmbeddingProbe());
      setRagChunkCount(result.chunksStored);
      setSuccess(
        formatMessage(locale, "ragIndexSuccess", {
          files: String(result.filesIndexed),
          chunks: String(result.chunksStored),
        }),
      );
    } catch (err) {
      setError(invokeErrorMessage(err, translate("ragIndexError")));
    } finally {
      setIndexingRag(false);
    }
  }

  async function handleIndexChanges() {
    if (!form) return;

    setIndexingChanges(true);
    setError(null);
    setSuccess(null);

    try {
      const result = await indexWorkspaceChanges(buildEmbeddingProbe());
      setRagChunkCount(result.chunksStored);
      setSuccess(
        formatMessage(locale, "ragIndexChangesSuccess", {
          files: String(result.filesIndexed),
          skipped: String(result.filesSkipped),
          chunks: String(result.chunksStored),
        }),
      );
    } catch (err) {
      setError(invokeErrorMessage(err, translate("ragIndexError")));
    } finally {
      setIndexingChanges(false);
    }
  }

  async function handleTestEmbedding() {
    if (!form) return;

    setTestingEmbedding(true);
    setError(null);

    const probe = buildEmbeddingProbe();

    try {
      const result = await testEmbeddingConnection(probe);
      setSuccess(
        formatMessage(locale, "embeddingTestSuccess", {
          model: result.model,
          latency: String(result.latencyMs),
        }),
      );
    } catch (err) {
      setError(invokeSettingsErrorMessage(err, translate("embeddingTestError")));
    } finally {
      setTestingEmbedding(false);
    }
  }

  async function handleClearHistory() {
    setClearing(true);
    setError(null);
    setConfirmClearOpen(false);

    try {
      const conversation = await getActiveConversation();
      await clearHistory(conversation.id);
      setSuccess(translate("historyCleared"));
      onHistoryCleared?.();
    } catch {
      setError(translate("clearHistoryError"));
    } finally {
      setClearing(false);
    }
  }

  return (
    <>
      <div className="settings-overlay" role="presentation" onClick={requestClose}>
        <div
          ref={dialogRef}
          className="settings-panel"
          role="dialog"
          aria-modal="true"
          aria-labelledby="settings-title"
          onClick={(event) => event.stopPropagation()}
        >
          <header className="settings-panel__header">
            <div>
              <h2 id="settings-title" className="settings-panel__title">
                {translate("settingsTitle")}
              </h2>
              <p className="settings-panel__subtitle">{translate("settingsSubtitle")}</p>
            </div>
            <button
              type="button"
              className="settings-panel__close"
              onClick={requestClose}
              aria-label={translate("closeSettings")}
            >
              ×
            </button>
          </header>

          {loading || !form ? (
            <p className="settings-panel__status" role="status">
              {translate("loadingSettings")}
            </p>
          ) : (
            <form className="settings-form" onSubmit={(e) => void handleSave(e)}>
              <nav className="settings-tabs" aria-label={translate("settingsTitle")}>
                {(
                  [
                    ["agent", "sectionAgent"],
                    ["connection", "sectionConnection"],
                    ["workspace", "sectionWorkspace"],
                    ["power", "sectionPower"],
                  ] as const
                ).map(([tab, labelKey]) => (
                  <button
                    key={tab}
                    type="button"
                    className={`settings-tabs__tab${activeTab === tab ? " settings-tabs__tab--active" : ""}`}
                    aria-selected={activeTab === tab}
                    onClick={() => setActiveTab(tab)}
                  >
                    {translate(labelKey)}
                  </button>
                ))}
              </nav>

              {activeTab === "agent" && (
              <>
              <section className="settings-section" aria-labelledby="settings-locale">
                <h3 id="settings-locale" className="settings-section__title">
                  {translate("uiLocale")}
                </h3>
                <label className="settings-field">
                  <span className="settings-field__label">{translate("uiLocale")}</span>
                  <select
                    className="settings-field__input"
                    value={form.uiLocale}
                    onChange={(e) =>
                      updateField("uiLocale", isUiLocale(e.target.value) ? e.target.value : "en")
                    }
                  >
                    <option value="en">{translate("localeEn")}</option>
                    <option value="fa">{translate("localeFa")}</option>
                  </select>
                </label>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("themePreference")}</span>
                  <select
                    className="settings-field__input"
                    value={form.themePreference}
                    onChange={(e) =>
                      updateField(
                        "themePreference",
                        e.target.value as ThemePreference,
                      )
                    }
                  >
                    <option value="dark">{translate("themeDark")}</option>
                    <option value="light">{translate("themeLight")}</option>
                    <option value="system">{translate("themeSystem")}</option>
                  </select>
                </label>
              </section>

              <section className="settings-section" aria-labelledby="settings-memory">
                <h3 id="settings-memory" className="settings-section__title">
                  {translate("sectionMemory")}
                </h3>
                <p className="settings-section__help">{translate("memoryHelpGeneric")}</p>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("memoryAdd")}</span>
                  <textarea
                    className="settings-field__input settings-field__textarea"
                    value={newMemory}
                    onChange={(e) => setNewMemory(e.target.value)}
                    rows={2}
                  />
                </label>
                <button
                  type="button"
                  className="settings-button settings-button--secondary"
                  disabled={memoryBusy || saving || !newMemory.trim()}
                  onClick={() => void handleAddMemory()}
                >
                  {translate("memoryAddButton")}
                </button>

                {memories.length > 0 && (
                  <ul className="settings-memory-list">
                    {memories.map((memory) => (
                      <li key={memory.id} className="settings-memory-list__item">
                        <p>{memory.content}</p>
                        <button
                          type="button"
                          className="settings-button settings-button--secondary"
                          disabled={memoryBusy || saving}
                          onClick={() => void handleDeleteMemory(memory.id)}
                        >
                          {translate("memoryDelete")}
                        </button>
                      </li>
                    ))}
                  </ul>
                )}
              </section>

              <section className="settings-section" aria-labelledby="settings-data">
                <h3 id="settings-data" className="settings-section__title">
                  {translate("sectionData")}
                </h3>
                <button
                  type="button"
                  className="settings-button settings-button--danger"
                  onClick={() => setConfirmClearOpen(true)}
                  disabled={clearing || saving}
                >
                  {clearing ? translate("clearing") : translate("clearHistory")}
                </button>
              </section>
              </>
              )}

              {activeTab === "connection" && (
              <>
              <section className="settings-section" aria-labelledby="settings-provider">
                <h3 id="settings-provider" className="settings-section__title">
                  {translate("sectionProvider")}
                </h3>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("apiBaseUrl")}</span>
                  <input
                    type="url"
                    className="settings-field__input"
                    value={form.baseUrl}
                    onChange={(e) => updateField("baseUrl", e.target.value)}
                    placeholder="https://api.openai.com/v1"
                    required
                    dir="ltr"
                  />
                </label>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("apiKey")}</span>
                  <input
                    type="password"
                    className="settings-field__input"
                    value={form.apiKey}
                    onChange={(e) => updateField("apiKey", e.target.value)}
                    placeholder={
                      apiKeyConfigured
                        ? formatMessage(locale, "apiKeyConfigured", { masked: apiKeyMasked })
                        : translate("apiKeyPlaceholder")
                    }
                    autoComplete="off"
                    dir="ltr"
                  />
                </label>

                <div className="settings-actions-inline">
                  <button
                    type="button"
                    className="settings-button settings-button--secondary"
                    onClick={() => void handleTest()}
                    disabled={testing || saving}
                  >
                    {testing ? translate("testing") : translate("testConnection")}
                  </button>
                </div>
              </section>

              <section className="settings-section" aria-labelledby="settings-models">
                <h3 id="settings-models" className="settings-section__title">
                  {translate("sectionModels")}
                </h3>

                <div className="settings-presets" role="group" aria-label={translate("modelPresetsLabel")}>
                  {MODEL_PRESETS.map((preset) => {
                    const active =
                      detectActivePreset({
                        defaultAgentTier: form.defaultAgentTier,
                        autoEscalate: form.autoEscalate,
                        planBeforeEdit: form.planBeforeEdit,
                      }) === preset.id;
                    return (
                      <button
                        key={preset.id}
                        type="button"
                        className={`settings-preset${active ? " settings-preset--active" : ""}`}
                        onClick={() => {
                          if (!form) return;
                          setForm({
                            ...form,
                            ...(preset.patch.defaultAgentTier
                              ? { defaultAgentTier: preset.patch.defaultAgentTier }
                              : {}),
                            ...(preset.patch.autoEscalate !== undefined
                              ? { autoEscalate: preset.patch.autoEscalate }
                              : {}),
                            ...(preset.patch.planBeforeEdit !== undefined
                              ? { planBeforeEdit: preset.patch.planBeforeEdit }
                              : {}),
                          });
                        }}
                      >
                        <span className="settings-preset__title">{translate(preset.labelKey as MessageKey)}</span>
                        <span className="settings-preset__desc">{translate(preset.descriptionKey as MessageKey)}</span>
                      </button>
                    );
                  })}
                </div>
                <p className="settings-section__help">{translate("modelPresetsHelp")}</p>
                <p className="settings-field__hint">{translate("modelPresetsApplyHint")}</p>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("fastModel")}</span>
                  <input
                    type="text"
                    className="settings-field__input"
                    value={form.fastModel}
                    onChange={(e) => updateField("fastModel", e.target.value)}
                    required
                    dir="ltr"
                  />
                </label>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("strongModel")}</span>
                  <input
                    type="text"
                    className="settings-field__input"
                    value={form.strongModel}
                    onChange={(e) => updateField("strongModel", e.target.value)}
                    required
                    dir="ltr"
                  />
                </label>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("agentModel")}</span>
                  <input
                    type="text"
                    className="settings-field__input"
                    value={form.agentModel}
                    onChange={(e) => updateField("agentModel", e.target.value)}
                    required
                    dir="ltr"
                  />
                </label>

                <label className="settings-field settings-field--checkbox">
                  <input
                    type="checkbox"
                    checked={form.autoEscalate}
                    onChange={(e) => updateField("autoEscalate", e.target.checked)}
                  />
                  <span>{translate("autoEscalate")}</span>
                </label>
                <p className="settings-section__help">{translate("autoEscalateHelp")}</p>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("defaultAgentTier")}</span>
                  <select
                    className="settings-field__input"
                    value={form.defaultAgentTier}
                    onChange={(e) =>
                      updateField("defaultAgentTier", e.target.value as FormState["defaultAgentTier"])
                    }
                  >
                    <option value="auto">{translate("agentTierAuto")}</option>
                    <option value="quick">{translate("agentTierQuick")}</option>
                    <option value="standard">{translate("agentTierStandard")}</option>
                    <option value="deep">{translate("agentTierDeep")}</option>
                    <option value="explain">{translate("agentTierExplain")}</option>
                  </select>
                </label>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("agentTemperature")}</span>
                  <input
                    type="number"
                    className="settings-field__input"
                    min={0}
                    max={2}
                    step={0.1}
                    value={form.agentTemperature}
                    onChange={(e) => updateField("agentTemperature", e.target.value)}
                    required
                    dir="ltr"
                  />
                </label>

                <label className="settings-checkbox">
                  <input
                    type="checkbox"
                    checked={form.planBeforeEdit}
                    onChange={(e) => updateField("planBeforeEdit", e.target.checked)}
                  />
                  <span>{translate("planBeforeEdit")}</span>
                </label>
                <p className="settings-field__hint">{translate("planBeforeEditHelp")}</p>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("editorOpenUrl")}</span>
                  <input
                    type="text"
                    className="settings-field__input"
                    value={form.editorOpenUrl}
                    onChange={(e) => updateField("editorOpenUrl", e.target.value)}
                    dir="ltr"
                  />
                  <span className="settings-field__hint">{translate("editorOpenUrlHelp")}</span>
                </label>
              </section>
              </>
              )}

              {activeTab === "workspace" && (
              <section className="settings-section" aria-labelledby="settings-workspace">
                <h3 id="settings-workspace" className="settings-section__title">
                  {translate("sectionWorkspace")}
                </h3>

                <p className="settings-section__help">{translate("workspaceHelp")}</p>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("workspacePath")}</span>
                  <input
                    type="text"
                    className="settings-field__input"
                    value={form.workspacePath}
                    onChange={(e) => updateField("workspacePath", e.target.value)}
                    placeholder={translate("workspacePathPlaceholder")}
                    readOnly
                    dir="ltr"
                  />
                </label>

                <div className="settings-actions-inline">
                  <button
                    type="button"
                    className="settings-button settings-button--secondary"
                    onClick={() => void handleBrowseWorkspace()}
                    disabled={saving}
                  >
                    {translate("workspaceBrowse")}
                  </button>
                  {form.workspacePath && (
                    <button
                      type="button"
                      className="settings-button settings-button--secondary"
                      onClick={() => updateField("workspacePath", "")}
                      disabled={saving}
                    >
                      {translate("workspaceClear")}
                    </button>
                  )}
                </div>

                <label className="settings-checkbox">
                  <input
                    type="checkbox"
                    checked={form.allowFileOverwrites}
                    onChange={(e) => updateField("allowFileOverwrites", e.target.checked)}
                  />
                  <span>{translate("allowFileOverwrites")}</span>
                </label>

                <label className="settings-checkbox">
                  <input
                    type="checkbox"
                    checked={form.verifyEnabled}
                    onChange={(e) => updateField("verifyEnabled", e.target.checked)}
                  />
                  <span>{translate("verifyEnabled")}</span>
                </label>

                {form.verifyEnabled && (
                  <label className="settings-field">
                    <span className="settings-field__label">{translate("verifyCommand")}</span>
                    <input
                      type="text"
                      className="settings-field__input"
                      value={form.verifyCommand}
                      onChange={(e) => updateField("verifyCommand", e.target.value)}
                      placeholder={translate("verifyCommandPlaceholder")}
                      dir="ltr"
                    />
                    <span className="settings-field__hint">{translate("verifyHelp")}</span>
                  </label>
                )}

                <label className="settings-checkbox">
                  <input
                    type="checkbox"
                    checked={form.contextPackEnabled}
                    onChange={(e) => updateField("contextPackEnabled", e.target.checked)}
                  />
                  <span>{translate("contextPackEnabled")}</span>
                </label>
                <p className="settings-field__hint">{translate("contextPackHelp")}</p>

                <label className="settings-checkbox">
                  <input
                    type="checkbox"
                    checked={form.projectRulesEnabled}
                    onChange={(e) => updateField("projectRulesEnabled", e.target.checked)}
                  />
                  <span>{translate("projectRulesEnabled")}</span>
                </label>
                <p className="settings-field__hint">{translate("projectRulesHelp")}</p>
                {form.projectRulesFile && (
                  <p className="settings-field__hint">
                    {formatMessage(locale, "projectRulesFile", { file: form.projectRulesFile })}
                  </p>
                )}
              </section>
              )}

              {activeTab === "power" && (
              <>
              <section className="settings-section" aria-labelledby="settings-power-basic">
                <h3 id="settings-power-basic" className="settings-section__title">
                  {translate("sectionPower")}
                </h3>

                <label className="settings-checkbox">
                  <input
                    type="checkbox"
                    checked={form.executorVisibility}
                    onChange={(e) => updateField("executorVisibility", e.target.checked)}
                  />
                  <span>{translate("executorVisibility")}</span>
                </label>

                <label className="settings-checkbox">
                  <input
                    type="checkbox"
                    checked={form.taskQueueEnabled}
                    onChange={(e) => updateField("taskQueueEnabled", e.target.checked)}
                  />
                  <span>{translate("taskQueueEnabled")}</span>
                </label>

                <button
                  type="button"
                  className="settings-button settings-button--secondary settings-advanced-toggle"
                  onClick={() => setShowAdvanced((v) => !v)}
                >
                  {showAdvanced ? translate("hideAdvanced") : translate("showAdvanced")}
                </button>
              </section>

              {showAdvanced && (
              <>
              <section className="settings-section" aria-labelledby="settings-rag">
                <h3 id="settings-rag" className="settings-section__title">
                  {translate("sectionRag")}
                </h3>

                <p className="settings-section__help">{translate("ragHelp")}</p>

                <label className="settings-checkbox">
                  <input
                    type="checkbox"
                    checked={form.ragEnabled}
                    onChange={(e) => updateField("ragEnabled", e.target.checked)}
                  />
                  <span>{translate("ragEnabled")}</span>
                </label>

                <label className="settings-checkbox">
                  <input
                    type="checkbox"
                    checked={form.ragEnabled ? form.ragAutoIndex : false}
                    disabled={!form.ragEnabled}
                    onChange={(e) => updateField("ragAutoIndex", e.target.checked)}
                  />
                  <span>{translate("ragAutoIndex")}</span>
                </label>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("embeddingBaseUrl")}</span>
                  <input
                    type="url"
                    className="settings-field__input"
                    value={form.embeddingBaseUrl}
                    onChange={(e) => updateField("embeddingBaseUrl", e.target.value)}
                    placeholder="http://localhost:11434/v1"
                    dir="ltr"
                  />
                </label>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("embeddingModel")}</span>
                  <input
                    type="text"
                    className="settings-field__input"
                    value={form.embeddingModel}
                    onChange={(e) => updateField("embeddingModel", e.target.value)}
                    placeholder="nomic-embed-text"
                    dir="ltr"
                  />
                </label>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("embeddingApiKey")}</span>
                  <input
                    type="password"
                    className="settings-field__input"
                    value={form.embeddingApiKey}
                    onChange={(e) => updateField("embeddingApiKey", e.target.value)}
                    placeholder={
                      embeddingApiKeyConfigured
                        ? formatMessage(locale, "apiKeyConfigured", {
                            masked: embeddingApiKeyMasked,
                          })
                        : translate("embeddingApiKeyPlaceholder")
                    }
                    autoComplete="off"
                    dir="ltr"
                  />
                </label>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("ragTopK")}</span>
                  <input
                    type="number"
                    className="settings-field__input"
                    min={1}
                    max={20}
                    value={form.ragTopK}
                    onChange={(e) => updateField("ragTopK", e.target.value)}
                    dir="ltr"
                  />
                </label>

                <p className="settings-field__hint">
                  {formatMessage(locale, "ragStatus", { count: String(ragChunkCount) })}
                </p>

                <div className="settings-actions-inline">
                  <button
                    type="button"
                    className="settings-button settings-button--secondary"
                    onClick={() => void handleTestEmbedding()}
                    disabled={testingEmbedding || saving || !form.ragEnabled}
                  >
                    {testingEmbedding ? translate("testing") : translate("testEmbedding")}
                  </button>
                  <button
                    type="button"
                    className="settings-button settings-button--secondary"
                    onClick={() => void handleIndexRag()}
                    disabled={indexingRag || saving || !form.ragEnabled || !form.workspacePath}
                  >
                    {indexingRag ? translate("indexingRag") : translate("indexWorkspaceRag")}
                  </button>
                  <button
                    type="button"
                    className="settings-button settings-button--secondary"
                    onClick={() => void handleIndexChanges()}
                    disabled={
                      indexingChanges || saving || !form.ragEnabled || !form.workspacePath
                    }
                  >
                    {indexingChanges
                      ? translate("indexingChanges")
                      : translate("indexWorkspaceChanges")}
                  </button>
                </div>
              </section>

              <section className="settings-section" aria-labelledby="settings-advanced">
                <h3 id="settings-advanced" className="settings-section__title">
                  {translate("sectionPreferences")}
                </h3>

                <label className="settings-field">
                  <span className="settings-field__label">{translate("commandAllowlistExtra")}</span>
                  <textarea
                    className="settings-field__input settings-field__textarea"
                    value={form.commandAllowlistExtra}
                    onChange={(e) => updateField("commandAllowlistExtra", e.target.value)}
                    placeholder={translate("commandAllowlistPlaceholder")}
                    dir="ltr"
                    rows={3}
                  />
                </label>

                <label className="settings-checkbox">
                  <input
                    type="checkbox"
                    checked={form.mcpEnabled}
                    onChange={(e) => {
                      const enabled = e.target.checked;
                      if (enabled && !form.mcpEnabled) {
                        setConfirmMcpOpen(true);
                        return;
                      }
                      updateField("mcpEnabled", enabled);
                    }}
                  />
                  <span>{translate("mcpEnabled")}</span>
                </label>

                {form.mcpEnabled && (
                  <p className="settings-field__help settings-field__help--warning" role="note">
                    {translate("mcpEnableWarning")}
                  </p>
                )}

                {form.mcpEnabled && (
                  <>
                    <div className="settings-presets settings-presets--mcp" role="group" aria-label={translate("mcpPresetsLabel")}>
                      {MCP_PRESETS.map((preset) => (
                        <button
                          key={preset.id}
                          type="button"
                          className="settings-preset"
                          onClick={() => {
                            updateField("mcpServerCommand", preset.command);
                            updateField("mcpEnabled", true);
                          }}
                        >
                          <span className="settings-preset__title">{translate(preset.labelKey as MessageKey)}</span>
                          <span className="settings-preset__desc">{translate(preset.descriptionKey as MessageKey)}</span>
                        </button>
                      ))}
                    </div>
                    <label className="settings-field">
                    <span className="settings-field__label">{translate("mcpServerCommand")}</span>
                    <input
                      type="text"
                      className="settings-field__input"
                      value={form.mcpServerCommand}
                      onChange={(e) => updateField("mcpServerCommand", e.target.value)}
                      placeholder={translate("mcpServerCommandPlaceholder")}
                      dir="ltr"
                    />
                  </label>
                  </>
                )}
              </section>
              </>
              )}
              </>
              )}

              {error && (
                <p className="settings-feedback settings-feedback--error" role="alert">
                  {error}
                </p>
              )}
              {success && (
                <p className="settings-feedback settings-feedback--success" role="status">
                  {success}
                </p>
              )}
              {testResult && (
                <p className="settings-feedback settings-feedback--success" role="status">
                  {testResult}
                </p>
              )}

              <footer className="settings-panel__footer">
                <button
                  type="button"
                  className="settings-button settings-button--secondary"
                  onClick={requestClose}
                >
                  {translate("cancel")}
                </button>
                <button
                  type="submit"
                  className="settings-button settings-button--primary"
                  disabled={saving}
                >
                  {saving ? translate("saving") : translate("saveSettings")}
                </button>
              </footer>
            </form>
          )}
        </div>
      </div>

      {confirmMcpOpen && (
        <ConfirmDialog
          title={translate("mcpEnableConfirmTitle")}
          body={translate("mcpEnableWarning")}
          confirmLabel={translate("mcpEnableConfirmAction")}
          cancelLabel={translate("confirmDiscardSettingsCancel")}
          onConfirm={() => {
            updateField("mcpEnabled", true);
            setConfirmMcpOpen(false);
          }}
          onCancel={() => setConfirmMcpOpen(false)}
        />
      )}

      {confirmDiscardOpen && (
        <ConfirmDialog
          title={translate("confirmDiscardSettingsTitle")}
          body={translate("confirmDiscardSettingsBody")}
          confirmLabel={translate("confirmDiscardSettingsConfirm")}
          cancelLabel={translate("confirmDiscardSettingsCancel")}
          destructive
          onConfirm={() => {
            setConfirmDiscardOpen(false);
            onClose();
          }}
          onCancel={() => setConfirmDiscardOpen(false)}
        />
      )}

      {confirmClearOpen && (
        <ConfirmDialog
          title={translate("confirmClearTitle")}
          body={translate("confirmClearBody")}
          confirmLabel={translate("confirmClearConfirm")}
          cancelLabel={translate("confirmClearCancel")}
          destructive
          onConfirm={() => void handleClearHistory()}
          onCancel={() => setConfirmClearOpen(false)}
        />
      )}
    </>
  );
}
