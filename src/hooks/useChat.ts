import { useCallback, useEffect, useRef, useState } from "react";

import type { Message } from "../components/MessageList";

import type { FileChange } from "../lib/changes";

import { getExecutorRunChanges } from "../lib/changes";

import {

  cancelRun,

  clearHistory,

  getActiveConversation,

  getMessages,

  sendMessage as sendMessageCommand,

  type CompanionStreamPayload,

  type ExecutorActivity,

  type ExecutorProgressEvent,

  type SendMessageResult,

  type StoredMessage,

} from "../lib/chat";

import { isConnectionConfigured } from "../lib/connection";

import { friendlyError } from "../lib/errors";

import { getSettings } from "../lib/settings";

import { t, type MessageKey, type UiLocale } from "../lib/i18n";



function companionLabel(locale: UiLocale, personalityId: string): string {

  switch (personalityId) {

    case "sage":

      return t(locale, "companionNameSage");

    case "spark":

      return t(locale, "companionNameSpark");

    default:

      return t(locale, "companionNameLuna");

  }

}



function companionMessagesAfterUser(

  stored: StoredMessage[],

  userContent: string,

): StoredMessage[] {

  const needle = userContent.trim();

  let userIndex = -1;

  for (let i = stored.length - 1; i >= 0; i -= 1) {

    if (stored[i].role === "user" && stored[i].content.trim() === needle) {

      userIndex = i;

      break;

    }

  }

  if (userIndex === -1) return [];

  return stored.slice(userIndex + 1).filter((message) => message.role === "companion");

}



function storedIncludesStreamedCompanion(

  stored: StoredMessage[],

  userContent: string,

  streamed: string,

): boolean {

  if (!streamed) return false;

  return companionMessagesAfterUser(stored, userContent).some(

    (message) => message.content.trim() === streamed,

  );

}



function sleep(ms: number): Promise<void> {

  return new Promise((resolve) => setTimeout(resolve, ms));

}



function toUiMessage(

  message: StoredMessage,

  locale: UiLocale,

  personalityId: string,

): Message {

  return {

    id: message.id,

    role: message.role,

    content: message.content,

    label:

      message.role === "companion" ? companionLabel(locale, personalityId) : undefined,

  };

}



function phaseMessageKey(phase: DelegatePhase): MessageKey {

  switch (phase) {

    case "understanding":

      return "phaseUnderstanding";

    case "executing":

      return "phaseExecuting";

    case "formatting":

      return "phaseFormatting";

    default:

      return "companionThinking";

  }

}



export type DelegatePhase = "understanding" | "executing" | "formatting" | null;



export function useChat(locale: UiLocale = "en") {

  const [personalityId, setPersonalityId] = useState("luna");

  const [workspacePath, setWorkspacePath] = useState<string | null>(null);

  const [connectionConfigured, setConnectionConfigured] = useState(false);

  const [conversationId, setConversationId] = useState<string | null>(null);

  const [messages, setMessages] = useState<Message[]>([]);

  const [executorActivity, setExecutorActivity] = useState<ExecutorActivity | null>(null);

  const [delegatePhase, setDelegatePhase] = useState<DelegatePhase>(null);

  const [interimHolding, setInterimHolding] = useState<string | null>(null);

  const [streamingContent, setStreamingContent] = useState<string | null>(null);

  const [fileChanges, setFileChanges] = useState<FileChange[]>([]);

  const [executorRunId, setExecutorRunId] = useState<string | null>(null);

  const [loading, setLoading] = useState(true);

  const [sending, setSending] = useState(false);

  const [executorVisibility, setExecutorVisibility] = useState(true);

  const [loadError, setLoadError] = useState<string | null>(null);

  const [sendError, setSendError] = useState<string | null>(null);

  const streamBufferRef = useRef("");

  const loadGenerationRef = useRef(0);

  const sendingRef = useRef(false);

  const cancelRequestedRef = useRef(false);

  const delegationSucceededRef = useRef(false);



  const load = useCallback(async () => {

    const generation = ++loadGenerationRef.current;

    setLoading(true);

    setLoadError(null);

    if (!sendingRef.current) {

      setSendError(null);

      setExecutorActivity(null);

      setDelegatePhase(null);

      setInterimHolding(null);

      setStreamingContent(null);

      setFileChanges([]);

      setExecutorRunId(null);

    }



    try {

      const conversation = await getActiveConversation();

      const stored = await getMessages(conversation.id);

      const settings = await getSettings();

      if (generation !== loadGenerationRef.current) return;

      const pid = settings.personalityId ?? "luna";

      setPersonalityId(pid);

      setWorkspacePath(settings.workspacePath ?? null);

      setExecutorVisibility(settings.executorVisibility ?? false);

      setConnectionConfigured(

        isConnectionConfigured(settings.baseUrl, settings.apiKeyConfigured),

      );

      setConversationId(conversation.id);

      if (!sendingRef.current) {

        setMessages(stored.map((m) => toUiMessage(m, locale, pid)));

      }

    } catch (err) {

      if (generation !== loadGenerationRef.current) return;

      setLoadError(

        friendlyError(

          err instanceof Error ? err.message : "",

          t(locale, "loadConversationError"),

        ),

      );

    } finally {

      if (generation === loadGenerationRef.current) {

        setLoading(false);

      }

    }

  }, [locale]);



  useEffect(() => {

    void load();

  }, [load]);



  const cancel = useCallback(async () => {

    if (!conversationId || !sending) return;

    cancelRequestedRef.current = true;

    setSending(false);

    sendingRef.current = false;

    setDelegatePhase(null);

    setInterimHolding(null);

    setStreamingContent(null);

    setExecutorActivity(null);



    try {

      await cancelRun(conversationId);

    } catch {

      // Best-effort cancel; backend may already be finished.

    }

  }, [conversationId, sending]);



  const send = useCallback(

    async (content: string) => {

      if (!conversationId || sending) return;



      if (!connectionConfigured) {

        setSendError(t(locale, "connectionError"));

        return;

      }



      setSending(true);

      sendingRef.current = true;

      // Invalidate any in-flight conversation reload that could overwrite live send state.

      loadGenerationRef.current += 1;

      cancelRequestedRef.current = false;

      delegationSucceededRef.current = false;

      setSendError(null);

      setExecutorActivity(null);

      setDelegatePhase(null);

      setInterimHolding(null);

      setStreamingContent(null);

      setFileChanges([]);

      setExecutorRunId(null);

      streamBufferRef.current = "";



      const pendingUserId = `pending-user-${Date.now()}`;

      setMessages((current) => [

        ...current,

        {

          id: pendingUserId,

          role: "user",

          content,

          label: undefined,

        },

      ]);



      let unlistenProgress: (() => void) | undefined;

      let unlistenStream: (() => void) | undefined;

      let invokeFailed = false;

      let invokeErrorMessage = "";



      const reloadFromDb = async (

        sentContent: string,

        delaysMs: number[] = [0, 250, 750, 1500, 2500],

      ): Promise<boolean> => {

        const streamedFinal = streamBufferRef.current.trim();

        for (const delay of delaysMs) {

          if (delay > 0) {

            await sleep(delay);

          }

          try {

            const stored = await getMessages(conversationId);

            const companionReplies = companionMessagesAfterUser(stored, sentContent);

            const lastCompanion = companionReplies[companionReplies.length - 1];

            if (!lastCompanion) {

              continue;

            }



            if (delegationSucceededRef.current && streamedFinal.length > 0) {

              const hasStreamedFinal = storedIncludesStreamedCompanion(

                stored,

                sentContent,

                streamedFinal,

              );

              // Delegated runs insert a holding message before the final; wait for both.

              if (!hasStreamedFinal && companionReplies.length < 2) {

                continue;

              }

            }



            loadGenerationRef.current += 1;

            setMessages(stored.map((m) => toUiMessage(m, locale, personalityId)));

            return true;

          } catch {

            // Retry while the backend finishes writing.

          }

        }

        return false;

      };



      const commitStreamFallback = async (sentContent: string): Promise<boolean> => {

        const streamed = streamBufferRef.current.trim();

        if (!streamed) return false;



        try {

          const stored = await getMessages(conversationId);

          if (storedIncludesStreamedCompanion(stored, sentContent, streamed)) {

            loadGenerationRef.current += 1;

            setMessages(stored.map((m) => toUiMessage(m, locale, personalityId)));

            return true;

          }



          loadGenerationRef.current += 1;

          setMessages([

            ...stored.map((m) => toUiMessage(m, locale, personalityId)),

            {

              id: `companion-stream-${Date.now()}`,

              role: "companion",

              content: streamed,

              label: companionLabel(locale, personalityId),

            },

          ]);

          return true;

        } catch {

          setMessages((current) => {

            if (

              current.some(

                (message) =>

                  message.role === "companion" && message.content.trim() === streamed,

              )

            ) {

              return current.filter((message) => message.id !== pendingUserId);

            }

            return [

              ...current.filter((message) => message.id !== pendingUserId),

              {

                id: `companion-stream-${Date.now()}`,

                role: "companion",

                content: streamed,

                label: companionLabel(locale, personalityId),

              },

            ];

          });

          return true;

        }

      };



      try {

        const { listen } = await import("@tauri-apps/api/event");



        unlistenProgress = await listen<ExecutorProgressEvent>("executor-progress", (event) => {

          if (event.payload.conversationId !== conversationId) return;



          if (event.payload.phase === "holding") {

            setDelegatePhase("understanding");

            if (event.payload.activity?.summary) {

              setInterimHolding(event.payload.activity.summary);

            }

          } else if (event.payload.phase === "executing") {

            setDelegatePhase("executing");

            if (event.payload.activity) {

              setExecutorActivity(event.payload.activity);

              const status = event.payload.activity.status.toLowerCase();

              if (status === "success" || status === "done") {

                delegationSucceededRef.current = true;

              }

            }

          } else if (event.payload.phase === "formatting") {

            setDelegatePhase("formatting");

            delegationSucceededRef.current = true;

          }

        });



        unlistenStream = await listen<CompanionStreamPayload>("companion-stream", (event) => {

          const payload = event.payload;

          if (payload.conversationId !== conversationId) return;



          if (payload.phase === "plan") {

            setDelegatePhase("understanding");

            if (payload.done && payload.content) {

              setInterimHolding(payload.content);

              streamBufferRef.current = payload.content;

              setStreamingContent(payload.content);

              return;

            }



            streamBufferRef.current += payload.delta;

            setInterimHolding(streamBufferRef.current);

            setStreamingContent(streamBufferRef.current);

            return;

          }



          if (payload.phase === "final") {

            setDelegatePhase("formatting");

            if (payload.done && payload.content) {

              streamBufferRef.current = payload.content;

              setStreamingContent(payload.content);

              delegationSucceededRef.current = true;

              return;

            }



            streamBufferRef.current += payload.delta;

            setStreamingContent(streamBufferRef.current);

          }

        });



        let result: SendMessageResult | null = null;

        try {

          result = await sendMessageCommand(conversationId, content);

        } catch (err) {

          invokeFailed = true;

          invokeErrorMessage = friendlyError(

            err instanceof Error ? err.message : "",

            t(locale, "sendMessageError"),

          );

          result = null;

        }



        const synced = await reloadFromDb(content);

        if (synced || delegationSucceededRef.current) {

          setSendError(null);

        } else if (invokeFailed) {

          setMessages((current) => current.filter((message) => message.id !== pendingUserId));

          setSendError(invokeErrorMessage);

        }



        if (result?.hasFileChanges && result.executorRunId) {

          try {

            const changes = await getExecutorRunChanges(result.executorRunId);

            if (changes.length > 0) {

              setFileChanges(changes);

              setExecutorRunId(result.executorRunId);

            }

          } catch {

            // File changes can be reviewed later from the executor run record.

          }

        }

      } catch (err) {

        const synced = await reloadFromDb(content, [0, 250]);

        if (synced || delegationSucceededRef.current) {

          setSendError(null);

        } else {

          setSendError(

            friendlyError(

              err instanceof Error ? err.message : "",

              t(locale, "genericError"),

            ),

          );

        }

      } finally {

        unlistenProgress?.();

        unlistenStream?.();



        if (conversationId) {

          let synced = await reloadFromDb(content, [0, 400, 1200]);

          if (!synced && streamBufferRef.current.trim()) {

            synced = await commitStreamFallback(content);

          }

          if (synced || delegationSucceededRef.current) {

            setSendError(null);

          }

        }



        if (cancelRequestedRef.current && conversationId) {

          try {

            const stored = await getMessages(conversationId);

            loadGenerationRef.current += 1;

            setMessages(stored.map((m) => toUiMessage(m, locale, personalityId)));

          } catch {

            // Keep optimistic UI if reload fails.

          }

        }



        cancelRequestedRef.current = false;

        setSending(false);

        sendingRef.current = false;

        setDelegatePhase(null);

        setInterimHolding(null);

        setStreamingContent(null);

        streamBufferRef.current = "";

      }

    },

    [conversationId, sending, locale, personalityId, connectionConfigured],

  );



  const statusMessage =

    (streamingContent != null && streamingContent.length > 0

      ? streamingContent

      : null) ??

    interimHolding ??

    t(locale, phaseMessageKey(delegatePhase));



  const reset = useCallback(async () => {

    if (!conversationId) return;



    setSendError(null);

    setExecutorActivity(null);



    try {

      await clearHistory(conversationId);

      const stored = await getMessages(conversationId);

      setMessages(stored.map((m) => toUiMessage(m, locale, personalityId)));

    } catch (err) {

      setSendError(

        friendlyError(

          err instanceof Error ? err.message : "",

          t(locale, "clearHistoryError"),

        ),

      );

    }

  }, [conversationId, locale, personalityId]);



  return {

    messages,

    executorActivity,

    delegatePhase,

    statusMessage,

    streamingContent,

    fileChanges,

    executorRunId,

    removeFileChanges: (paths?: string[]) => {

      setFileChanges((current) => {

        const next =

          paths && paths.length > 0

            ? current.filter((change) => !paths.includes(change.path))

            : [];

        if (next.length === 0) {

          setExecutorRunId(null);

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

    companionDisplayName: companionLabel(locale, personalityId),

    personalityId,

    workspacePath,

    connectionConfigured,

    executorVisibility,

  };

}


