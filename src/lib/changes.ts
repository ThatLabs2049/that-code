export interface FileChange {
  path: string;
  changeType: string;
  diff?: string;
  beforeContent?: string;
  afterContent?: string;
}

export async function revertExecutorRun(
  runId: string,
  paths?: string[],
): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke<void>("revert_executor_run", {
    runId,
    paths: paths ?? null,
  });
}

export async function getExecutorRunChanges(runId: string): Promise<FileChange[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<FileChange[]>("get_executor_run_changes", { runId });
}

export interface DiffHunk {
  index: number;
  oldLines: string[];
  newLines: string[];
}

export async function getExecutorRunDiffHunks(
  runId: string,
  path: string,
): Promise<DiffHunk[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<DiffHunk[]>("get_executor_run_diff_hunks", { runId, path });
}

export async function rejectExecutorHunks(
  runId: string,
  path: string,
  hunkIndices: number[],
): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke<void>("reject_executor_hunks", { runId, path, hunkIndices });
}

export async function openInEditor(path: string, line?: number): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke<void>("open_in_editor", { path, line: line ?? null });
}

export async function getExecutorRunFileDiff(runId: string, path: string): Promise<string> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<string>("get_executor_run_file_diff", { runId, path });
}
