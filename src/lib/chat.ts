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

import type { RetrievedChunk } from "./rag";
import type { AgentTier } from "./settings";
import type { MessageAttachment } from "./workspace";

export type { AgentTier };

export interface SendMessageResult {
  executorRunId?: string;
  hasFileChanges: boolean;
  retrievedContext?: RetrievedChunk[];
  awaitingPlanApproval?: boolean;
  planContent?: string;
  assistantMessage?: string;
}

export type { RetrievedChunk };

export type { FileChange } from "./changes";

export interface ExecutorProgressEvent {
  conversationId: string;
  phase: string;
  activity?: ExecutorActivity;
}
export interface AssistantStreamPayload {
  conversationId: string;
  streamId: string;
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
  agentTier?: AgentTier,
  attachments?: MessageAttachment[],
  exploreThenImplement?: boolean,
): Promise<SendMessageResult> {
  return invoke<SendMessageResult>("send_message", {
    conversationId,
    content,
    agentTier: agentTier ?? null,
    attachments: attachments?.length ? attachments : null,
    exploreThenImplement: exploreThenImplement ?? null,
  });
}

export function clearHistory(conversationId: string): Promise<StoredMessage[]> {
  return invoke<StoredMessage[]>("clear_history", { conversationId });
}

export function respondToAgentPlan(
  conversationId: string,
  approved: boolean,
): Promise<SendMessageResult> {
  return invoke<SendMessageResult>("respond_to_agent_plan", { conversationId, approved });
}

export interface PendingPlanView {
  briefing: string;
}

export function getPendingPlan(conversationId: string): Promise<PendingPlanView | null> {
  return invoke<PendingPlanView | null>("get_pending_plan", { conversationId });
}

export function cancelRun(conversationId: string): Promise<boolean> {
  return invoke<boolean>("cancel_run", { conversationId });
}
