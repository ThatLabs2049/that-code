export interface Conversation {
  id: string;
  title: string | null;
  created_at: string;
  updated_at: string;
}

export interface StoredMessage {
  id: string;
  conversation_id: string;
  role: "user" | "companion";
  content: string;
  created_at: string;
}

export interface TaskSpec {
  objective: string;
  context: string;
  constraints: string[];
  expected_output: string;
}

export interface ActivityStep {
  step: string;
  detail: string;
}

export interface ExecutorActivity {
  taskSpec: TaskSpec;
  status: string;
  summary: string;
  activityLog: ActivityStep[];
}

export interface SendMessageResult {
  executorRunId?: string;
  hasFileChanges: boolean;
}

export type { FileChange } from "./changes";

export interface ExecutorProgressEvent {
  conversationId: string;
  phase: string;
  activity?: ExecutorActivity;
}
export interface CompanionStreamPayload {
  conversationId: string;
  streamId: string;
  phase: string;
  delta: string;
  done: boolean;
  content?: string;
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke<T>(cmd, args);
}

export function getActiveConversation(): Promise<Conversation> {
  return invoke<Conversation>("get_active_conversation");
}

export function getMessages(conversationId: string): Promise<StoredMessage[]> {
  return invoke<StoredMessage[]>("get_messages", { conversationId });
}

export function sendMessage(
  conversationId: string,
  content: string,
): Promise<SendMessageResult> {
  return invoke<SendMessageResult>("send_message", { conversationId, content });
}

export function listConversations(): Promise<Conversation[]> {
  return invoke<Conversation[]>("list_conversations");
}

export function clearHistory(conversationId: string): Promise<StoredMessage[]> {
  return invoke<StoredMessage[]>("clear_history", { conversationId });
}

export function cancelRun(conversationId: string): Promise<boolean> {
  return invoke<boolean>("cancel_run", { conversationId });
}
