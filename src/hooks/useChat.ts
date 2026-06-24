import { useCallback, useEffect, useRef, useState } from "react";
import type { Message } from "../components/MessageList";
import type { FileChange } from "../lib/changes";
import { getExecutorRunChanges } from "../lib/changes";
import {
  cancelRun,
  clearHistory,
  getActiveConversation,
  getMessages,
  getPendingPlan,
  sendMessage as sendMessageCommand,
  respondToAgentPlan,
  type AssistantStreamPayload,
  type ExecutorActivity,
  type ExecutorProgressEvent,
  type SendMessageResult,
  type StoredMessage,
} from "../lib/chat";
import { isConnectionConfigured } from "../lib/connection";
import { friendlyError } from "../lib/errors";
import { getSettings, updateSettings, type AgentTier } from "../lib/settings";
import { t, type UiLocale } from "../lib/i18n";

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function toUiMessage(message: StoredMessage): Message {
  return {
    id: message.id,
    role: message.role === "companion" ? "assistant" : "user",
    content: message.content,
  };
}

function assistantRepliesAfterLatestUser(stored: StoredMessage[]): StoredMessage[] {
  let userIndex = -1;
  for (let i = stored.length - 1; i >= 0; i -= 1) {
    if (stored[i].role === "user") {
      userIndex = i;
      break;
    }
  }
  if (userIndex === -1) return [];
  return stored
    .slice(userIndex + 1)
    .filter((message) => message.role === "companion" && message.content.trim().length > 0);
}

function storedToUiMessages(stored: StoredMessage[]): Message[] {
  return stored
    .filter((message) => message.role !== "companion" || message.content.trim().length > 0)
    .map(toUiMessage);
}

function applyAssistantReply(
  current: Message[],
  content: string,
  streamingAssistantId: string,
): Message[] {
  const trimmed = content.trim();
  if (!trimmed) return current;

  const streamingIndex = current.findIndex((message) => message.id === streamingAssistantId);
  if (streamingIndex >= 0) {
    return current.map((message) =>
      message.id === streamingAssistantId
        ? { ...message, content: trimmed, streaming: false }
        : message,
    );
  }

  const withoutTyping = current.filter((message) => message.id !== "typing");
  return [
    ...withoutTyping,
    {
      id: `assistant-${Date.now()}`,
      role: "assistant",
      content: trimmed,
    },
  ];
}

const AGENT_TIERS: readonly AgentTier[] = ["auto", "quick", "standard", "deep", "explain"];

export type AgentPhase = "running" | null;

export function useChat(locale: UiLocale = "en") {
  const [workspacePath, setWorkspacePath] = useState<string | null>(null);
  const [connectionConfigured, setConnectionConfigured] = useState(false);
  const [conversationId, setConversationId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [agentActivity, setAgentActivity] = useState<ExecutorActivity | null>(null);
  const [agentPhase, setAgentPhase] = useState<AgentPhase>(null);
  const [fileChanges, setFileChanges] = useState<FileChange[]>([]);
  const [executorRunId, setExecutorRunId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [agentVisibility, setAgentVisibility] = useState(true);
  const [ragEnabled, setRagEnabled] = useState(false);
  const [retrievedContext, setRetrievedContext] = useState<import("../lib/rag").RetrievedChunk[]>([]);
  const [ragRefreshKey, setRagRefreshKey] = useState(0);
  const [agentTier, setAgentTier] = useState<AgentTier>("auto");
  const [pendingPlan, setPendingPlan] = useState<string | null>(null);
  const [planBusy, setPlanBusy] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [sendError, setSendError] = useState<string | null>(null);
  const loadGenerationRef = useRef(0);
  const sendGenerationRef = useRef(0);
  const sendingRef = useRef(false);
  const cancelRequestedRef = useRef(false);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  const load = useCallback(async () => {
    const generation = ++loadGenerationRef.current;
    setLoading(true);
    setLoadError(null);

    if (!sendingRef.current) {
      setSendError(null);
      setAgentActivity(null);
      setAgentPhase(null);
      setFileChanges([]);
      setExecutorRunId(null);
      setRetrievedContext([]);
    }

    try {
      const conversation = await getActiveConversation();
      const [stored, settings, pending] = await Promise.all([
        getMessages(conversation.id),
        getSettings(),
        getPendingPlan(conversation.id),
      ]);
      if (generation !== loadGenerationRef.current || !mountedRef.current) return;

      setWorkspacePath(settings.workspacePath ?? null);
      setRagEnabled(settings.ragEnabled ?? false);
      setAgentVisibility(settings.executorVisibility ?? true);
      setAgentTier(
        AGENT_TIERS.includes(settings.defaultAgentTier as AgentTier)
          ? settings.defaultAgentTier
          : "auto",
      );
      setConnectionConfigured(
        isConnectionConfigured(settings.baseUrl, settings.apiKeyConfigured),
      );
      setConversationId(conversation.id);

      if (!sendingRef.current) {
        setMessages(storedToUiMessages(stored));
        setPendingPlan(pending?.briefing ?? null);
      }
    } catch (err) {
      if (generation !== loadGenerationRef.current || !mountedRef.current) return;
      setLoadError(
        friendlyError(
          err instanceof Error ? err.message : "",
          t(locale, "loadConversationError"),
        ),
      );
    } finally {
      if (generation === loadGenerationRef.current && mountedRef.current) {
        setLoading(false);
      }
    }
  }, [locale]);

  useEffect(() => {
    void load();
  }, [load]);

  const cancel = useCallback(async () => {
    if (!conversationId || !sendingRef.current) return;
    cancelRequestedRef.current = true;
    setAgentPhase(null);
    setAgentActivity(null);

    try {
      await cancelRun(conversationId);
    } catch {
      // Best-effort cancel
    }
  }, [conversationId]);

  const changeAgentTier = useCallback(async (tier: AgentTier) => {
    setAgentTier(tier);
    try {
      await updateSettings({ defaultAgentTier: tier });
    } catch {
      // Composer tier still applies to the next send
    }
  }, []);

  const send = useCallback(
    async (
      content: string,
      attachments: import("../lib/workspace").MessageAttachment[] = [],
      exploreThenImplement = false,
      agentContent?: string,
    ) => {
      if (!conversationId || sending || pendingPlan) return;

      const messageForUi = content;
      const messageForApi = agentContent ?? content;

      if (!connectionConfigured) {
        setSendError(t(locale, "connectionError"));
        return;
      }

      const generation = ++sendGenerationRef.current;
      setSending(true);
      sendingRef.current = true;
      loadGenerationRef.current += 1;
      cancelRequestedRef.current = false;
      setSendError(null);
      setAgentActivity(null);
      setAgentPhase("running");
      setFileChanges([]);
      setExecutorRunId(null);
      setRetrievedContext([]);

      let pendingUserId = `pending-user-${Date.now()}`;
      const streamingAssistantId = `streaming-assistant-${Date.now()}`;
      let streamActive = false;
      let invokeFailed = false;
      let invokeErrorMessage = "";
      let invokeSucceeded = false;
      let result: SendMessageResult | null = null;

      const reloadFromDb = async (
        delaysMs: number[] = [0, 250, 750, 1500, 2500, 4000],
        force = false,
      ): Promise<boolean> => {
        if (!force && streamActive) return false;
        for (const delay of delaysMs) {
          if (generation !== sendGenerationRef.current) return false;
          if (delay > 0) await sleep(delay);
          if ((!force && streamActive) || generation !== sendGenerationRef.current) return false;
          try {
            const stored = await getMessages(conversationId);
            const replies = assistantRepliesAfterLatestUser(stored);
            if (!replies.length) continue;
            loadGenerationRef.current += 1;
            setMessages(storedToUiMessages(stored));
            return true;
          } catch {
            // Retry while backend finishes writing
          }
        }
        return false;
      };

      const reconcileSendOutcome = async () => {
        const syncDelays = invokeFailed
          ? [0, 500, 1500, 3000, 5000, 8000, 12000, 15000]
          : [0, 400, 1200, 2500, 4000];

        // Always sync from DB at end — streamed ack is ephemeral; final message is persisted separately.
        const synced = await reloadFromDb(syncDelays, true);
        if (synced) {
          setSendError(null);
          return;
        }

        const fallbackMessage = result?.assistantMessage?.trim();
        if (fallbackMessage) {
          setMessages((current) =>
            applyAssistantReply(current, fallbackMessage, streamingAssistantId),
          );
          setSendError(null);
          return;
        }

        if (streamActive) {
          setSendError(null);
          return;
        }

        if (invokeSucceeded) {
          setSendError(t(locale, "sendMessageError"));
          return;
        }

        let stored: StoredMessage[] = [];
        try {
          stored = await getMessages(conversationId);
        } catch {
          setMessages((current) => current.filter((message) => message.id !== pendingUserId));
          setSendError(invokeErrorMessage);
          return;
        }

        const userInDb = stored.some(
          (message) =>
            message.role === "user" && message.content.trim() === messageForApi.trim(),
        );
        const hasReply = assistantRepliesAfterLatestUser(stored).length > 0;

        if (hasReply) {
          loadGenerationRef.current += 1;
          setMessages(storedToUiMessages(stored));
          setSendError(null);
          return;
        }

        if (userInDb) {
          setSendError(null);
          void reloadFromDb([2000, 5000, 10000, 15000], true);
          return;
        }

        setMessages((current) => current.filter((message) => message.id !== pendingUserId));
        setSendError(invokeErrorMessage);
      };

      setMessages((current) => [
        ...current,
        { id: pendingUserId, role: "user", content: messageForUi },
      ]);

      let unlistenProgress: (() => void) | undefined;
      let unlistenStream: (() => void) | undefined;
      let unlistenMessages: (() => void) | undefined;

      try {
        const { listen } = await import("@tauri-apps/api/event");

        unlistenProgress = await listen<ExecutorProgressEvent>("executor-progress", (event) => {
          if (event.payload.conversationId !== conversationId || !mountedRef.current) return;
          const phase = event.payload.phase;
          if (phase === "complete" || phase === "error" || phase === "plan_review") {
            if (phase !== "plan_review") {
              setAgentPhase(null);
            }
          } else if (phase === "running" || phase === "executing" || phase === "holding") {
            setAgentPhase("running");
          }
          if (event.payload.activity) {
            setAgentActivity(event.payload.activity);
          }
        });

        unlistenStream = await listen<AssistantStreamPayload>("assistant-stream", (event) => {
          if (event.payload.conversationId !== conversationId || !mountedRef.current) return;
          const { delta, done, content } = event.payload;

          if (done) {
            const hadStream = streamActive;
            streamActive = false;
            if (!hadStream) {
              if (content?.trim()) {
                setMessages((current) => [
                  ...current,
                  {
                    id: streamingAssistantId,
                    role: "assistant",
                    content,
                    streaming: false,
                  },
                ]);
              }
              return;
            }

            setMessages((current) =>
              current.map((message) =>
                message.id === streamingAssistantId
                  ? {
                      ...message,
                      content: content ?? message.content,
                      streaming: false,
                    }
                  : message,
              ),
            );
            return;
          }

          if (!streamActive) {
            streamActive = true;
            setMessages((current) => [
              ...current,
              {
                id: streamingAssistantId,
                role: "assistant",
                content: delta,
                streaming: true,
              },
            ]);
            return;
          }

          if (delta) {
            setMessages((current) =>
              current.map((message) =>
                message.id === streamingAssistantId
                  ? { ...message, content: message.content + delta, streaming: true }
                  : message,
              ),
            );
          }
        });

        unlistenMessages = await listen<string>("messages-updated", (event) => {
          if (event.payload !== conversationId || streamActive) return;
          void reloadFromDb([0, 50, 200, 500]);
        });

        try {
          result = await sendMessageCommand(
            conversationId,
            messageForApi,
            agentTier,
            attachments,
            exploreThenImplement,
          );
          invokeSucceeded = true;

          if (result?.assistantMessage?.trim() && mountedRef.current) {
            setMessages((current) =>
              applyAssistantReply(current, result!.assistantMessage!, streamingAssistantId),
            );
          }
        } catch (err) {
          invokeFailed = true;
          invokeErrorMessage = friendlyError(
            err instanceof Error ? err.message : "",
            t(locale, "sendMessageError"),
          );
        }

        if (result?.hasFileChanges && result.executorRunId) {
          try {
            const changes = await getExecutorRunChanges(result.executorRunId);
            if (changes.length > 0) {
              setFileChanges(changes);
              setExecutorRunId(result.executorRunId);
            }
          } catch {
            // Changes can be reviewed later from run record
          }
        }

        if (result?.awaitingPlanApproval && result.planContent) {
          const planText = result.planContent.trim();
          setPendingPlan(result.planContent);
          setMessages((current) =>
            current.filter(
              (message) =>
                message.role !== "assistant" || message.content.trim() !== planText,
            ),
          );
        }

        if (result?.retrievedContext?.length && mountedRef.current) {
          setRetrievedContext(result.retrievedContext);
          setRagRefreshKey((key) => key + 1);
        }
      } catch (err) {
        if (!invokeSucceeded) {
          invokeFailed = true;
          invokeErrorMessage = friendlyError(
            err instanceof Error ? err.message : "",
            t(locale, "sendMessageError"),
          );
        }
      } finally {
        unlistenProgress?.();
        unlistenStream?.();
        unlistenMessages?.();

        if (mountedRef.current) {
          setSending(false);
          sendingRef.current = false;
          setAgentPhase(null);
        }

        await reconcileSendOutcome();

        // Drop ephemeral stream bubble — DB holds the canonical assistant reply.
        if (mountedRef.current) {
          try {
            const stored = await getMessages(conversationId);
            if (assistantRepliesAfterLatestUser(stored).length > 0) {
              loadGenerationRef.current += 1;
              setMessages(storedToUiMessages(stored));
            }
          } catch {
            // reconcileSendOutcome already retried
          }
        }

        if (
          result?.awaitingPlanApproval &&
          result.planContent &&
          mountedRef.current
        ) {
          const planText = result.planContent.trim();
          setMessages((current) =>
            current.filter(
              (message) =>
                message.role !== "assistant" || message.content.trim() !== planText,
            ),
          );
        }

        if (cancelRequestedRef.current && conversationId && mountedRef.current) {
          try {
            const stored = await getMessages(conversationId);
            loadGenerationRef.current += 1;
            setMessages(storedToUiMessages(stored));
          } catch {
            // Keep optimistic UI
          }
        }

        cancelRequestedRef.current = false;
      }
    },
    [conversationId, sending, pendingPlan, locale, connectionConfigured, agentTier],
  );

  const reset = useCallback(async () => {
    if (!conversationId) return;
    setSendError(null);
    setAgentActivity(null);
    setPendingPlan(null);

    try {
      await clearHistory(conversationId);
      const stored = await getMessages(conversationId);
      setMessages(storedToUiMessages(stored));
    } catch (err) {
      setSendError(
        friendlyError(
          err instanceof Error ? err.message : "",
          t(locale, "clearHistoryError"),
        ),
      );
    }
  }, [conversationId, locale]);

  const respondToPlan = useCallback(
    async (approved: boolean) => {
      if (!conversationId || planBusy || sending) return;
      setPlanBusy(true);
      setSending(true);
      sendingRef.current = true;
      const generation = ++sendGenerationRef.current;
      setAgentPhase("running");
      setSendError(null);
      setAgentActivity(null);

      let unlistenProgress: (() => void) | undefined;
      let unlistenStream: (() => void) | undefined;

      try {
        const { listen } = await import("@tauri-apps/api/event");

        unlistenProgress = await listen<ExecutorProgressEvent>("executor-progress", (event) => {
          if (event.payload.conversationId !== conversationId || !mountedRef.current) return;
          const phase = event.payload.phase;
          if (phase === "complete" || phase === "error" || phase === "plan_review") {
            if (phase !== "plan_review") {
              setAgentPhase(null);
            }
          } else if (phase === "running" || phase === "executing" || phase === "holding") {
            setAgentPhase("running");
          }
          if (event.payload.activity) {
            setAgentActivity(event.payload.activity);
          }
        });

        unlistenStream = await listen<AssistantStreamPayload>("assistant-stream", (event) => {
          if (event.payload.conversationId !== conversationId || !mountedRef.current) return;
          if (!event.payload.done && event.payload.delta) {
            setAgentPhase("running");
          }
        });

        const result = await respondToAgentPlan(conversationId, approved);
        if (generation !== sendGenerationRef.current || !mountedRef.current) return;
        setPendingPlan(null);

        if (result.assistantMessage?.trim()) {
          setMessages((current) => [
            ...current.filter((message) => message.role !== "assistant" || message.id === "typing"),
            {
              id: `assistant-${Date.now()}`,
              role: "assistant",
              content: result.assistantMessage!,
            },
          ]);
        }

        if (result.hasFileChanges && result.executorRunId) {
          try {
            const changes = await getExecutorRunChanges(result.executorRunId);
            if (changes.length > 0) {
              setFileChanges(changes);
              setExecutorRunId(result.executorRunId);
            }
          } catch {
            // Changes can be reviewed later from run record
          }
        }

        const stored = await getMessages(conversationId);
        if (mountedRef.current) {
          setMessages(storedToUiMessages(stored));
          setSendError(null);
        }
      } catch (err) {
        if (mountedRef.current) {
          try {
            const stored = await getMessages(conversationId);
            const hasReply = assistantRepliesAfterLatestUser(stored).length > 0;
            if (hasReply) {
              setMessages(storedToUiMessages(stored));
              setPendingPlan(null);
              setSendError(null);
              return;
            }
          } catch {
            // Fall through to error display
          }
          setSendError(
            friendlyError(err instanceof Error ? err.message : "", t(locale, "genericError")),
          );
        }
      } finally {
        unlistenProgress?.();
        unlistenStream?.();
        if (mountedRef.current) {
          setSending(false);
          sendingRef.current = false;
          setAgentPhase(null);
          setPlanBusy(false);
        }
      }
    },
    [conversationId, planBusy, sending, locale],
  );

  const statusMessage =
    agentActivity?.summary?.trim() ||
    agentActivity?.taskSpec.objective ||
    t(locale, agentPhase === "running" ? "phaseRunning" : "agentThinking");

  return {
    messages,
    agentActivity,
    agentPhase,
    statusMessage,
    fileChanges,
    executorRunId,
    removeFileChanges: (paths?: string[]) => {
      setFileChanges((current) => {
        const next =
          paths && paths.length > 0
            ? current.filter((change) => !paths.includes(change.path))
            : [];
        if (next.length === 0) {
          queueMicrotask(() => setExecutorRunId(null));
        }
        return next;
      });
    },
    loading,
    sending,
    loadError,
    sendError,
    clearSendError: () => setSendError(null),
    send,
    cancel,
    reset,
    reload: load,
    workspacePath,
    connectionConfigured,
    agentVisibility,
    ragEnabled,
    retrievedContext,
    clearRetrievedContext: () => setRetrievedContext([]),
    ragRefreshKey,
    agentTier,
    setAgentTier: changeAgentTier,
    pendingPlan,
    planBusy,
    respondToPlan,
    conversationId,
  };
}
