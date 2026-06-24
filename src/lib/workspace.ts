export interface WorkspacePathHit {

  path: string;

  kind: "file" | "folder";

}



export interface WorkspaceSymbolHit {

  name: string;

  path: string;

  line: number;

  kind: string;

}



export interface MessageAttachment {

  path: string;

  kind: "file" | "folder" | "symbol";

  line?: number;

  symbol?: string;

}



export interface WorkspaceGitStatus {

  isRepo: boolean;

  branch?: string | null;

  filesChanged: number;

  insertions: number;

  deletions: number;

}



async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {

  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");

  return tauriInvoke<T>(cmd, args);

}



export function searchWorkspacePaths(query: string): Promise<WorkspacePathHit[]> {

  return invoke<WorkspacePathHit[]>("search_workspace_paths", { query });

}



export function searchWorkspaceSymbols(query: string): Promise<WorkspaceSymbolHit[]> {

  return invoke<WorkspaceSymbolHit[]>("search_workspace_symbols", { query });

}



export function getWorkspaceGitStatus(): Promise<WorkspaceGitStatus> {

  return invoke<WorkspaceGitStatus>("get_workspace_git_status");

}

