import { useEffect, useState } from "react";
import { getExecutorRunFileDiff } from "../lib/changes";
import { useLocale } from "../context/LocaleContext";
import { invokeErrorMessage } from "../lib/invokeError";
import "./ChangeReviewPanel.css";

const DIFF_PREVIEW_BYTES = 4_096;

interface LazyDiffPaneProps {
  runId: string;
  path: string;
  refreshKey?: number;
}

export function LazyDiffPane({ runId, path, refreshKey = 0 }: LazyDiffPaneProps) {
  const { translate } = useLocale();
  const [diff, setDiff] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    setDiff(null);
    setExpanded(false);

    void getExecutorRunFileDiff(runId, path)
      .then((content) => {
        if (!cancelled) {
          setDiff(content);
          setLoading(false);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(invokeErrorMessage(err, translate("changesDiffError")));
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [runId, path, refreshKey, translate]);

  if (loading) {
    return (
      <p className="change-review__diff-status" role="status">
        {translate("changesLoadingDiff")}
      </p>
    );
  }

  if (error) {
    return (
      <p className="change-review__diff-status change-review__diff-status--error" role="alert">
        {error}
      </p>
    );
  }

  if (!diff) {
    return (
      <p className="change-review__diff-status" role="status">
        {translate("changesNoDiff")}
      </p>
    );
  }

  const isLarge = diff.length > DIFF_PREVIEW_BYTES;
  const displayDiff =
    isLarge && !expanded ? `${diff.slice(0, DIFF_PREVIEW_BYTES)}…` : diff;

  return (
    <div className="change-review__diff-wrap">
      <pre className="change-review__diff" dir="ltr">
        {displayDiff}
      </pre>
      {isLarge && !expanded && (
        <button
          type="button"
          className="change-review__load-diff"
          onClick={() => setExpanded(true)}
        >
          {translate("changesLoadFullDiff")}
        </button>
      )}
    </div>
  );
}
