export interface HealthStatus {
  status: string;
  app: string;
  version: string;
}

export async function healthCheck(): Promise<HealthStatus> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<HealthStatus>("health_check");
}
