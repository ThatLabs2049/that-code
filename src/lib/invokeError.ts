import { friendlyError } from "./errors";

export function invokeErrorMessage(err: unknown, fallback: string): string {
  let message = "";
  if (typeof err === "string") {
    message = err;
  } else if (err instanceof Error) {
    message = err.message;
  }

  return friendlyError(message, fallback);
}
