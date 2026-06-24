import type { TaskSpec } from "./chat";



export interface QueuedTask {

  id: string;

  conversationId: string;

  taskSpec: TaskSpec;

  status: string;

  position: number;

  createdAt: string;

}



async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {

  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");

  return tauriInvoke<T>(cmd, args);

}



export function listQueuedTasks(conversationId: string): Promise<QueuedTask[]> {

  return invoke<QueuedTask[]>("list_queued_tasks", { conversationId });

}



export function clearCompletedTasks(conversationId: string): Promise<void> {

  return invoke<void>("clear_completed_tasks", { conversationId });

}

