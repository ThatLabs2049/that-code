export type ThemePreference = "dark" | "light" | "system";

export interface AiSettingsView {
  baseUrl: string;
  apiKeyMasked: string;
  apiKeyConfigured: boolean;
  companionModel: string;
  executorModel: string;
  companionTemperature: number;
  executorTemperature: number;
  executorVisibility: boolean;
  uiLocale: string;
  workspacePath: string | null;
  allowFileOverwrites: boolean;
  ragEnabled: boolean;
  embeddingBaseUrl: string;
  embeddingModel: string;
  embeddingApiKeyMasked: string;
  embeddingApiKeyConfigured: boolean;
  ragTopK: number;
  verifyEnabled: boolean;
  verifyCommand: string | null;
  contextPackEnabled: boolean;
  personalityId: string;
  commandAllowlistExtra: string[];
  ragAutoIndex: boolean;
  taskQueueEnabled: boolean;
  mcpEnabled: boolean;
  mcpServerCommand: string | null;
  onboardingCompleted: boolean;
  themePreference: ThemePreference;
}

export interface UpdateAiSettings {
  baseUrl?: string;
  apiKey?: string;
  companionModel?: string;
  executorModel?: string;
  companionTemperature?: number;
  executorTemperature?: number;
  executorVisibility?: boolean;
  uiLocale?: string;
  workspacePath?: string | null;
  allowFileOverwrites?: boolean;
  ragEnabled?: boolean;
  embeddingBaseUrl?: string;
  embeddingModel?: string;
  embeddingApiKey?: string;
  ragTopK?: number;
  verifyEnabled?: boolean;
  verifyCommand?: string | null;
  contextPackEnabled?: boolean;
  personalityId?: string;
  commandAllowlistExtra?: string[];
  ragAutoIndex?: boolean;
  taskQueueEnabled?: boolean;
  mcpEnabled?: boolean;
  mcpServerCommand?: string | null;
  onboardingCompleted?: boolean;
  themePreference?: ThemePreference;
}

export interface AiTestResult {
  ok: boolean;
  model: string;
  message: string;
  latencyMs: number;
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke<T>(cmd, args);
}

export function getSettings(): Promise<AiSettingsView> {
  return invoke<AiSettingsView>("get_settings");
}

export function updateSettings(update: UpdateAiSettings): Promise<AiSettingsView> {
  return invoke<AiSettingsView>("update_settings", { update });
}

export function testAiConnection(probe?: UpdateAiSettings): Promise<AiTestResult> {
  return invoke<AiTestResult>("test_ai_connection", { probe: probe ?? null });
}
