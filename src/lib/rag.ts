import type { UpdateAiSettings } from "./settings";

export interface RagStatus {
  enabled: boolean;
  chunkCount: number;
  lastIndexedAt: string | null;
}

export interface RagIndexResult {
  filesIndexed: number;
  filesSkipped: number;
  chunksStored: number;
}

export interface EmbeddingTestResult {
  ok: boolean;
  model: string;
  latencyMs: number;
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke<T>(cmd, args);
}

export function getRagStatus(): Promise<RagStatus> {
  return invoke<RagStatus>("get_rag_status");
}

function buildEmbeddingProbe(probe?: UpdateAiSettings): UpdateAiSettings | null {
  return probe ?? null;
}

export function indexWorkspaceRag(probe?: UpdateAiSettings): Promise<RagIndexResult> {
  return invoke<RagIndexResult>("index_workspace_rag", {
    probe: buildEmbeddingProbe(probe),
  });
}

export function indexWorkspaceChanges(probe?: UpdateAiSettings): Promise<RagIndexResult> {
  return invoke<RagIndexResult>("index_workspace_changes", {
    probe: buildEmbeddingProbe(probe),
  });
}

export function testEmbeddingConnection(
  probe?: UpdateAiSettings,
): Promise<EmbeddingTestResult> {
  return invoke<EmbeddingTestResult>("test_embedding_connection", { probe: probe ?? null });
}
