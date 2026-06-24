import { useEffect, useState } from "react";
import {
  getExecutorRunDiffHunks,
  rejectExecutorHunks,
  type DiffHunk,
} from "../lib/changes";
import { useLocale } from "../context/LocaleContext";
import { invokeErrorMessage } from "../lib/invokeError";
import "./ChangeReviewPanel.css";

interface DiffHunkReviewProps {
  runId: string;
  path: string;
  onUpdated?: () => void;
}

export function DiffHunkReview({ runId, path, onUpdated }: DiffHunkReviewProps) {
  const { translate } = useLocale();
  const [hunks, setHunks] = useState<DiffHunk[]>([]);
  const [rejected, setRejected] = useState<Set<number>>(new Set());
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void getExecutorRunDiffHunks(runId, path)
      .then(setHunks)
      .catch(() => setHunks([]));
    setRejected(new Set());
  }, [runId, path]);

  if (hunks.length === 0) return null;

  async function applyRejections() {
    if (rejected.size === 0) return;
    setBusy(true);
    setError(null);
    try {
      await rejectExecutorHunks(runId, path, Array.from(rejected));
      setRejected(new Set());
      onUpdated?.();
    } catch (err) {
      setError(invokeErrorMessage(err, translate("hunkRejectError")));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="change-review__hunks">
      <p className="change-review__hunks-title">{translate("hunkReviewTitle")}</p>
      <ul className="change-review__hunk-list">
        {hunks.map((hunk) => {
          const isRejected = rejected.has(hunk.index);
          return (
            <li key={hunk.index} className="change-review__hunk">
              <button
                type="button"
                className={`change-review__hunk-toggle${isRejected ? " change-review__hunk-toggle--rejected" : ""}`}
                disabled={busy}
                onClick={() =>
                  setRejected((current) => {
                    const next = new Set(current);
                    if (next.has(hunk.index)) next.delete(hunk.index);
                    else next.add(hunk.index);
                    return next;
                  })
                }
              >
                {isRejected ? translate("hunkRejected") : translate("hunkAccept")}
              </button>
              <pre className="change-review__hunk-diff" dir="ltr">
                {hunk.oldLines.map((line) => `-${line}\n`).join("")}
                {hunk.newLines.map((line) => `+${line}\n`).join("")}
              </pre>
            </li>
          );
        })}
      </ul>
      {rejected.size > 0 && (
        <button
          type="button"
          className="change-review__button change-review__button--secondary"
          disabled={busy}
          onClick={() => void applyRejections()}
        >
          {translate("hunkApplyRejections")}
        </button>
      )}
      {error && (
        <p className="change-review__feedback change-review__feedback--error" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
