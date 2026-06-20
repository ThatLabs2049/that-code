export function isLocalProvider(baseUrl: string): boolean {
  const url = baseUrl.trim().toLowerCase();
  return (
    url.includes("localhost") ||
    url.includes("127.0.0.1") ||
    url.includes("[::1]") ||
    url.includes("0.0.0.0")
  );
}

export function isConnectionConfigured(baseUrl: string, apiKeyConfigured: boolean): boolean {
  return apiKeyConfigured || isLocalProvider(baseUrl);
}
