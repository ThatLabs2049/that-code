export interface FileChange {
  path: string;
  changeType: string;
  diff: string;
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
