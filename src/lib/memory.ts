export interface Memory {
  id: string;
  content: string;
  createdAt: string;
  updatedAt: string;
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke<T>(cmd, args);
}

export function listMemories(): Promise<Memory[]> {
  return invoke<Memory[]>("list_memories");
}

export function createMemory(content: string): Promise<Memory> {
  return invoke<Memory>("create_memory", { content });
}

export function updateMemory(id: string, content: string): Promise<Memory> {
  return invoke<Memory>("update_memory", { id, content });
}

export function deleteMemory(id: string): Promise<void> {
  return invoke<void>("delete_memory", { id });
}
