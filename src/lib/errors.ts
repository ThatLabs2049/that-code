const CONNECTION_PATTERNS = [
  /network/i,
  /fetch/i,
  /connection/i,
  /timeout/i,
  /unreachable/i,
  /api error/i,
  /401/i,
  /403/i,
  /invalid.*key/i,
];

export function friendlyError(message: string, fallback: string): string {
  const trimmed = message.trim();
  if (!trimmed) return fallback;

  if (CONNECTION_PATTERNS.some((pattern) => pattern.test(trimmed))) {
    return fallback;
  }

  // Never surface raw IPC / Rust error strings to users.
  if (
    trimmed.includes("rusqlite") ||
    trimmed.includes("serde") ||
    trimmed.includes("invalid type") ||
    /^[a-z_]+:\s/.test(trimmed)
  ) {
    return fallback;
  }

  if (trimmed.length > 120) {
    return fallback;
  }

  return trimmed;
}
