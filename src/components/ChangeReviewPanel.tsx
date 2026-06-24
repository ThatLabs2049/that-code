import { useEffect, useState } from "react";
import type { FileChange } from "../lib/changes";
import { revertExecutorRun, openInEditor } from "../lib/changes";
import { useLocale } from "../context/LocaleContext";
import { formatMessage } from "../lib/i18n";
import { invokeErrorMessage } from "../lib/invokeError";
import { LazyDiffPane } from "./LazyDiffPane";
import { DiffHunkReview } from "./DiffHunkReview";
import "./ChangeReviewPanel.css";

interface ChangeReviewPanelProps {
  runId: string;
  changes: FileChange[];
  onReverted?: (revertedPaths?: string[]) => void;
}

export function ChangeReviewPanel({ runId, changes, onReverted }: ChangeReviewPanelProps) {
  const { locale, translate } = useLocale();
  const [selectedPath, setSelectedPath] = useState(changes[0]?.path ?? "");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [diffRefreshKey, setDiffRefreshKey] = useState(0);

  const selected = changes.find((change) => change.path === selectedPath) ?? changes[0];

  useEffect(() => {
    if (!changes.some((change) => change.path === selectedPath)) {
      setSelectedPath(changes[0]?.path ?? "");
    }
  }, [changes, selectedPath]);

  async function handleRevertAll() {
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      await revertExecutorRun(runId);
      setMessage(translate("changesRevertedAll"));
      onReverted?.();
    } catch (err) {
      setError(invokeErrorMessage(err, translate("changesRevertError")));
    } finally {
      setBusy(false);
    }
  }

  async function handleRevertFile() {
    if (!selected) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      await revertExecutorRun(runId, [selected.path]);
      setMessage(translate("changesRevertedFile"));
      onReverted?.([selected.path]);
    } catch (err) {
      setError(invokeErrorMessage(err, translate("changesRevertError")));
    } finally {
      setBusy(false);
    }
  }

  if (changes.length === 0) return null;

  return (
    <section className="change-review" aria-labelledby="change-review-title">
      <div className="change-review__header">
        <h3 id="change-review-title" className="change-review__title">
          {translate("changesTitle")}
        </h3>
        <span className="change-review__count">
          {formatMessage(locale, "changesCount", { count: String(changes.length) })}
        </span>
      </div>

      <div className="change-review__body">
        <ul className="change-review__files">
          {changes.map((change) => (
            <li key={change.path}>
              <button
                type="button"
                className={`change-review__file${change.path === selected?.path ? " change-review__file--active" : ""}`}
                onClick={() => setSelectedPath(change.path)}
              >
                <span className="change-review__file-path" dir="ltr">
                  {change.path}
                </span>
                <span className="change-review__file-type">{change.changeType}</span>
              </button>
              <button
                type="button"
                className="change-review__open-editor"
                title={translate("openInEditor")}
                onClick={() => void openInEditor(change.path)}
              >
                {translate("openInEditor")}
              </button>
            </li>
          ))}
        </ul>

        {selected && (
          <div className="change-review__diff-column">
            <LazyDiffPane runId={runId} path={selected.path} refreshKey={diffRefreshKey} />
            <DiffHunkReview
              runId={runId}
              path={selected.path}
              onUpdated={() => setDiffRefreshKey((key) => key + 1)}
            />
          </div>
        )}
      </div>

      <div className="change-review__actions">
        <button
          type="button"
          className="change-review__button change-review__button--secondary"
          disabled={busy}
          onClick={() => void handleRevertFile()}
        >
          {translate("changesRevertFile")}
        </button>
        <button
          type="button"
          className="change-review__button change-review__button--danger"
          disabled={busy}
          onClick={() => void handleRevertAll()}
        >
          {translate("changesRevertAll")}
        </button>
      </div>

      {message && (
        <p className="change-review__feedback change-review__feedback--success" role="status">
          {message}
        </p>
      )}
      {error && (
        <p className="change-review__feedback change-review__feedback--error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
