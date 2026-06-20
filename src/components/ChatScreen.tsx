import { useRef, useState, type RefObject } from "react";

import { Header } from "./Header";

import { MessageList } from "./MessageList";

import { Composer, focusComposer } from "./Composer";

import { SettingsPanel } from "./SettingsPanel";

import { ExecutorActivityPanel } from "./ExecutorActivityPanel";

import { ChangeReviewPanel } from "./ChangeReviewPanel";

import { StatusBar } from "./StatusBar";

import { ConfirmDialog } from "./ConfirmDialog";

import { OnboardingFlow } from "./OnboardingFlow";

import { SuggestedPrompts } from "./SuggestedPrompts";

import { WorkspaceBanner } from "./WorkspaceBanner";

import { useChat } from "../hooks/useChat";

import { useLocale } from "../context/LocaleContext";

import { formatMessage } from "../lib/i18n";

import { personalityDisplayName } from "./PersonalityCards";

import "./ChatScreen.css";

import "./SettingsPanel.css";

import "./ExecutorActivityPanel.css";

import "./ConfirmDialog.css";

import "./ChangeReviewPanel.css";

import "./StatusBar.css";

import "./OnboardingFlow.css";

import "./SuggestedPrompts.css";

import "./WorkspaceBanner.css";



export type SettingsTab = "companion" | "connection" | "workspace" | "power";



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

    executorActivity,

    delegatePhase,

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

    streamingContent,

    fileChanges,

    executorRunId,

    removeFileChanges,

    executorVisibility,

    companionDisplayName,

    personalityId,

    workspacePath,

    connectionConfigured,

  } = useChat(locale);

  const [confirmResetOpen, setConfirmResetOpen] = useState(false);

  const [resetting, setResetting] = useState(false);



  const showPlanStream =
    sending && !!streamingContent?.length && delegatePhase === "understanding";
  const showStreamingBubble =
    showPlanStream || (sending && !!streamingContent?.length && delegatePhase === "formatting");

  const showLiveStatus = sending && !streamingContent?.length;



  const displayMessages = sending

    ? [

        ...messages,

        ...(showStreamingBubble

          ? [

              {

                id: "streaming",

                role: "companion" as const,

                content: streamingContent,

                label: companionDisplayName,

                className: showPlanStream ? "streaming streaming--plan" : "streaming",

                streaming: true,

              },

            ]

          : showLiveStatus

            ? [

                {

                  id: "typing",

                  role: "companion" as const,

                  content: statusMessage,

                  label: companionDisplayName,

                  className: "typing",

                },

              ]

            : []),

      ]

    : messages;



  const showSuggestedPrompts =

    !loading && !sending && !loadError && messages.length <= 1;

  const showWorkspaceBanner = !workspacePath && !loading && !loadError;



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

        personalityId={personalityId}

        workspacePath={workspacePath}

        connectionConfigured={connectionConfigured}

        changeReviewCount={fileChanges.length}

        settingsTriggerRef={settingsTriggerRef}

        onOpenSettings={() => onOpenSettings()}

        onOpenConnectionSettings={() => onOpenSettings("connection")}

        onOpenWorkspaceSettings={() => onOpenSettings("workspace")}

        onOpenChangeReview={() => changeReviewRef.current?.scrollIntoView({ behavior: "smooth" })}

        onResetChat={() => setConfirmResetOpen(true)}

        resetDisabled={loading || sending || resetting || !!loadError}

        interactionsDisabled={onboardingOpen || settingsOpen}

      />

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

                personalityId={personalityId}

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

            <MessageList messages={displayMessages} personalityId={personalityId} />

          </>

        )}

      </main>

      <StatusBar

        visible={sending}

        phase={delegatePhase}

        message={
          showPlanStream
            ? translate("phaseUnderstanding")
            : delegatePhase === "formatting" && streamingContent
              ? translate("phaseFormatting")
              : delegatePhase
                ? statusMessage
                : translate("companionThinking")
        }

        onCancel={() => void cancel()}

      />

      {executorVisibility && executorActivity && (
        <ExecutorActivityPanel activity={executorActivity} />
      )}

      <div ref={changeReviewRef}>

        {executorRunId && fileChanges.length > 0 && (

          <ChangeReviewPanel

            runId={executorRunId}

            changes={fileChanges}

            onReverted={(paths) => removeFileChanges(paths?.length ? paths : undefined)}

          />

        )}

      </div>

      {showWorkspaceBanner && <WorkspaceBanner onOpenSettings={() => onOpenSettings("workspace")} />}

      {sendError && (
        <div className="chat-screen__send-error" role="alert">
          <p>{sendError}</p>
          <button type="button" className="chat-screen__send-error-dismiss" onClick={clearSendError}>
            {translate("sendErrorDismiss")}
          </button>
        </div>
      )}

      <Composer

        personalityId={personalityId}

        onSend={(content) => void send(content)}

        disabled={loading || sending || !!loadError || onboardingOpen}

        disabledReason={loading ? "loading" : sending ? "sending" : loadError ? "error" : undefined}

      />

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

      {confirmResetOpen && (

        <ConfirmDialog

          title={translate("confirmClearTitle")}

          body={formatMessage(locale, "confirmClearBodyDynamic", {

            name: personalityDisplayName(locale, personalityId),

          })}

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


