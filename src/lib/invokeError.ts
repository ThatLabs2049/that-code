import { friendlyError } from "./errors";

function extractInvokeMessage(err: unknown): string {
  if (typeof err === "string") {
    return err;
  }
  if (err instanceof Error) {
    return err.message;
  }
  return "";
}

/** Hide raw IPC/DB errors; keep provider messages (auth, model, timeout, etc.). */
export function invokeSettingsErrorMessage(err: unknown, fallback: string): string {
  const trimmed = extractInvokeMessage(err).trim();
  if (!trimmed) return fallback;

  if (
    trimmed.includes("rusqlite") ||
    trimmed.includes("serde") ||
    trimmed.includes("invalid type") ||
    /^[a-z_]+:\s/.test(trimmed)
  ) {
    return fallback;
  }

  if (trimmed.length > 280) {
    return `${trimmed.slice(0, 277)}…`;
  }

  return trimmed;
}

export function invokeErrorMessage(err: unknown, fallback: string): string {
  return friendlyError(extractInvokeMessage(err), fallback);
}
