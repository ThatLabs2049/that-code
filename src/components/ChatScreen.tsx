import { useRef, useState, type RefObject, useEffect, useMemo } from "react";
import { Header } from "./Header";
import { MessageList } from "./MessageList";
import { Composer, focusComposer } from "./Composer";
import { SettingsPanel } from "./SettingsPanel";
import { AgentActivityPanel } from "./AgentActivityPanel";
import { ChangeReviewPanel } from "./ChangeReviewPanel";
import { StatusBar } from "./StatusBar";
import { ConfirmDialog } from "./ConfirmDialog";
import { OnboardingFlow } from "./OnboardingFlow";
import { SuggestedPrompts } from "./SuggestedPrompts";
import { WorkspaceBanner } from "./WorkspaceBanner";
import { ContextUsedPanel } from "./ContextUsedPanel";
import { PlanApprovalBanner } from "./PlanApprovalBanner";
import { TaskQueuePanel } from "./TaskQueuePanel";
import { GitStatusPanel } from "./GitStatusPanel";
import { CommandPalette, AGENT_TIER_OPTIONS } from "./CommandPalette";
import { useRagStatus } from "../hooks/useRagStatus";
import { useChat } from "../hooks/useChat";
import { getSettings } from "../lib/settings";
import type { RetrievedChunk } from "../lib/rag";
import type { AgentTier } from "../lib/settings";
import { useLocale } from "../context/LocaleContext";
import "./ChatScreen.css";
import "./SettingsPanel.css";
import "./AgentActivityPanel.css";
import "./ConfirmDialog.css";
import "./ChangeReviewPanel.css";
import "./StatusBar.css";
import "./OnboardingFlow.css";
import "./SuggestedPrompts.css";
import "./ContextUsedPanel.css";

export type SettingsTab = "agent" | "connection" | "workspace" | "power";

interface ChatScreenProps {
  settingsOpen: boolean;
  settingsInitialTab?: SettingsTab;
  settingsTriggerRef: RefObject<HTMLButtonElement | null>;
  onboardingOpen: boolean;
  onOpenSettings: (tab?: SettingsTab) => void;
  onCloseSettings: () => void;
  onSettingsChanged: () => void;
  onOnboardingComplete: () => void;
  onOnboardingSkip: () => void;
}

export function ChatScreen({
  settingsOpen,
  settingsInitialTab,
  settingsTriggerRef,
  onboardingOpen,
  onOpenSettings,
  onCloseSettings,
  onSettingsChanged,
  onOnboardingComplete,
  onOnboardingSkip,
}: ChatScreenProps) {
  const { locale, translate } = useLocale();
  const changeReviewRef = useRef<HTMLDivElement>(null);
  const {
    messages,
    agentActivity,
    agentPhase,
    loading,
    sending,
    loadError,
    sendError,
    clearSendError,
    send,
    cancel,
    reset,
    reload,
    statusMessage,
    fileChanges,
    executorRunId,
    removeFileChanges,
    agentVisibility,
    workspacePath,
    connectionConfigured,
    ragEnabled,
    retrievedContext,
    clearRetrievedContext,
    ragRefreshKey,
    agentTier,
    setAgentTier,
    pendingPlan,
    planBusy,
    respondToPlan,
    conversationId,
  } = useChat(locale);

  const { status: ragStatus, indexing: ragIndexing, indexProgress: ragIndexProgress, error: ragError, reindex: reindexRag, cancelIndex: cancelRagIndex } = useRagStatus(
    workspacePath,
    ragRefreshKey,
  );
  const [codebasePreview, setCodebasePreview] = useState<RetrievedChunk[]>([]);
  const [projectRulesFile, setProjectRulesFile] = useState<string | null>(null);
  const [projectRulesEnabled, setProjectRulesEnabled] = useState(true);

  useEffect(() => {
    void getSettings().then((settings) => {
      setProjectRulesFile(settings.projectRulesFile ?? null);
      setProjectRulesEnabled(settings.projectRulesEnabled ?? true);
    });
  }, [workspacePath, ragRefreshKey, settingsOpen]);

  const [confirmResetOpen, setConfirmResetOpen] = useState(false);
  const [resetting, setResetting] = useState(false);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [taskQueueKey, setTaskQueueKey] = useState(0);
  const [gitRefreshKey, setGitRefreshKey] = useState(0);

  useEffect(() => {
    setTaskQueueKey((key) => key + 1);
    setGitRefreshKey((key) => key + 1);
  }, [sending, agentActivity?.status, agentActivity?.summary, fileChanges.length]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (onboardingOpen) return;
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setCommandPaletteOpen((open) => !open);
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onboardingOpen]);

  const commandActions = useMemo(
    () => [
      {
        id: "settings",
        label: translate("commandOpenSettings"),
        keywords: "preferences config",
        run: () => onOpenSettings(),
      },
      {
        id: "connection",
        label: translate("commandOpenConnection"),
        keywords: "api key provider",
        run: () => onOpenSettings("connection"),
      },
      {
        id: "workspace",
        label: translate("commandOpenWorkspace"),
        keywords: "folder project",
        run: () => onOpenSettings("workspace"),
      },
      {
        id: "reindex",
        label: translate("commandReindexRag"),
        keywords: "rag index memory",
        run: () => void reindexRag(),
      },
      {
        id: "focus-composer",
        label: translate("commandFocusComposer"),
        keywords: "chat input message",
        run: () => focusComposer(),
      },
      {
        id: "reset",
        label: translate("resetChat"),
        keywords: "clear history",
        run: () => setConfirmResetOpen(true),
      },
      ...AGENT_TIER_OPTIONS.map((tier) => {
        const tierLabels: Record<string, string> = {
          auto: translate("agentTierAuto"),
          quick: translate("agentTierQuick"),
          standard: translate("agentTierStandard"),
          deep: translate("agentTierDeep"),
          explain: translate("agentTierExplain"),
        };
        return {
          id: `tier-${tier}`,
          label: `${translate("commandSetTier")}: ${tierLabels[tier] ?? tier}`,
          keywords: `agent mode tier ${tier}`,
          run: () => setAgentTier(tier as AgentTier),
        };
      }),
    ],
    [translate, onOpenSettings, reindexRag, setAgentTier],
  );

  const hasAssistantReply = messages.some(
    (message) =>
      message.role === "assistant" &&
      message.id !== "typing" &&
      Boolean(message.content.trim() || message.streaming),
  );
  const showTyping = sending && !hasAssistantReply;
  const displayMessages = sending
    ? [
        ...messages,
        ...(showTyping
          ? [
              {
                id: "typing",
                role: "assistant" as const,
                content: statusMessage,
                className: "typing",
              },
            ]
          : []),
      ]
    : messages;

  const showSuggestedPrompts = !loading && !sending && !loadError && messages.length <= 1;
  const showWorkspaceBanner = !workspacePath && !loading && !loadError;
  const showSidebar = Boolean(workspacePath) || fileChanges.length > 0 || (sending && agentVisibility);

  async function handleResetChat() {
    setResetting(true);
    setConfirmResetOpen(false);
    await reset();
    setResetting(false);
  }

  function handleOnboardingComplete() {
    onOnboardingComplete();
    void reload();
    focusComposer();
  }

  return (
    <div className={`chat-screen${onboardingOpen ? " chat-screen--onboarding" : ""}`}>
      <div className="chat-screen__ambient" aria-hidden="true" />
      <a className="skip-link" href="#chat-main">
        {translate("skipToChat")}
      </a>

      <Header
        workspacePath={workspacePath}
        connectionConfigured={connectionConfigured}
        changeReviewCount={fileChanges.length}
        ragStatus={ragStatus}
        ragIndexing={ragIndexing}
        ragIndexProgress={ragIndexProgress}
        projectRulesFile={projectRulesFile}
        projectRulesEnabled={projectRulesEnabled}
        settingsTriggerRef={settingsTriggerRef}
        onOpenSettings={() => onOpenSettings()}
        onOpenConnectionSettings={() => onOpenSettings("connection")}
        onOpenWorkspaceSettings={() => onOpenSettings("workspace")}
        onOpenRagSettings={() => onOpenSettings("power")}
        onReindexRag={() => void reindexRag()}
        onCancelRagIndex={() => void cancelRagIndex()}
        onOpenChangeReview={() => changeReviewRef.current?.scrollIntoView({ behavior: "smooth" })}
        onResetChat={() => setConfirmResetOpen(true)}
        resetDisabled={loading || sending || resetting || !!loadError}
        interactionsDisabled={onboardingOpen || settingsOpen}
      />

      <div className={`chat-shell${showSidebar ? " chat-shell--with-sidebar" : ""}`}>
        <div className="chat-shell__main">
          <main id="chat-main" className="chat-screen__messages" aria-busy={loading || sending}>
            {loading ? (
              <p className="chat-screen__status" role="status">
                {translate("loadingConversation")}
              </p>
            ) : loadError ? (
              <div className="chat-screen__status chat-screen__status--error">
                <p role="alert">{loadError}</p>
                <button type="button" className="chat-screen__retry" onClick={() => void reload()}>
                  {translate("tryAgain")}
                </button>
              </div>
            ) : (
              <>
                {showSuggestedPrompts && (
                  <SuggestedPrompts
                    disabled={sending}
                    onSelect={(prompt) => {
                      if (!connectionConfigured) {
                        onOpenSettings("connection");
                        return;
                      }
                      void send(prompt);
                    }}
                  />
                )}
                <MessageList messages={displayMessages} />
              </>
            )}
          </main>

          <StatusBar
            visible={sending}
            phase={agentPhase}
            message={statusMessage}
            onCancel={() => void cancel()}
          />

          {sendError && (
            <div className="chat-screen__send-error" role="alert">
              <p>{sendError}</p>
              <button
                type="button"
                className="chat-screen__send-error-dismiss"
                onClick={clearSendError}
              >
                {translate("sendErrorDismiss")}
              </button>
            </div>
          )}

          {ragError && (
            <div className="chat-screen__send-error" role="alert">
              <p>{ragError}</p>
            </div>
          )}

          {showWorkspaceBanner && (
            <WorkspaceBanner onOpenSettings={() => onOpenSettings("workspace")} />
          )}

          {pendingPlan && (
            <PlanApprovalBanner
              planContent={pendingPlan}
              busy={planBusy}
              onApprove={() => void respondToPlan(true)}
              onReject={() => void respondToPlan(false)}
            />
          )}

          <Composer
            agentTier={agentTier}
            onAgentTierChange={setAgentTier}
            onSend={(displayContent, attachments, exploreThenImplement, agentContent) => {
              clearRetrievedContext();
              void send(displayContent, attachments, exploreThenImplement, agentContent);
            }}
            disabled={loading || sending || !!loadError || onboardingOpen || !!pendingPlan || planBusy}
            disabledReason={
              loading
                ? "loading"
                : sending
                  ? "sending"
                  : pendingPlan
                    ? "plan"
                    : loadError
                      ? "error"
                      : undefined
            }
            ragEnabled={ragEnabled && (ragStatus?.chunkCount ?? 0) > 0}
            workspaceReady={Boolean(workspacePath)}
            codebasePreview={codebasePreview}
            onCodebasePreview={setCodebasePreview}
          />
        </div>

        {showSidebar && (
          <aside className="chat-shell__sidebar" aria-label={translate("sidebarLabel")}>
            <GitStatusPanel workspacePath={workspacePath} refreshKey={gitRefreshKey} />
            <TaskQueuePanel
              conversationId={conversationId}
              refreshKey={taskQueueKey}
              sending={sending}
            />
            {codebasePreview.length > 0 && (
              <ContextUsedPanel chunks={codebasePreview} preview />
            )}
            {retrievedContext.length > 0 && (
              <ContextUsedPanel chunks={retrievedContext} />
            )}

            {agentVisibility && agentActivity && (
              <div className="chat-shell__agent-slot">
                <AgentActivityPanel activity={agentActivity} />
              </div>
            )}

            {executorRunId && fileChanges.length > 0 && (
              <div ref={changeReviewRef} className="chat-shell__changes">
                <ChangeReviewPanel
                  runId={executorRunId}
                  changes={fileChanges}
                  onReverted={(paths) => removeFileChanges(paths?.length ? paths : undefined)}
                />
              </div>
            )}
          </aside>
        )}
      </div>

      {settingsOpen && (
        <SettingsPanel
          initialTab={settingsInitialTab}
          onClose={onCloseSettings}
          onHistoryCleared={() => void reload()}
          onSettingsChanged={() => {
            void reload();
            onSettingsChanged();
          }}
        />
      )}

      {onboardingOpen && (
        <OnboardingFlow
          onComplete={handleOnboardingComplete}
          onSkip={() => {
            onOnboardingSkip();
            void reload();
            focusComposer();
          }}
        />
      )}

      <CommandPalette
        open={commandPaletteOpen}
        onClose={() => setCommandPaletteOpen(false)}
        actions={commandActions}
      />

      {confirmResetOpen && (
        <ConfirmDialog
          title={translate("confirmClearTitle")}
          body={translate("confirmClearBody")}
          confirmLabel={translate("confirmClearConfirm")}
          cancelLabel={translate("confirmClearCancel")}
          destructive
          onConfirm={() => void handleResetChat()}
          onCancel={() => setConfirmResetOpen(false)}
        />
      )}
    </div>
  );
}
