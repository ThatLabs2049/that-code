import { useEffect, useState } from "react";
import { getWorkspaceGitStatus, type WorkspaceGitStatus } from "../lib/workspace";
import { useLocale } from "../context/LocaleContext";
import { formatMessage } from "../lib/i18n";
import "./GitStatusPanel.css";

interface GitStatusPanelProps {
  workspacePath?: string | null;
  refreshKey?: number;
}

export function GitStatusPanel({ workspacePath, refreshKey = 0 }: GitStatusPanelProps) {
  const { locale, translate } = useLocale();
  const [status, setStatus] = useState<WorkspaceGitStatus | null>(null);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    if (!workspacePath) {
      setStatus(null);
      setLoaded(false);
      return;
    }

    let cancelled = false;
    setLoaded(false);
    void getWorkspaceGitStatus()
      .then((git) => {
        if (!cancelled) {
          setStatus(git);
          setLoaded(true);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setStatus(null);
          setLoaded(true);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [workspacePath, refreshKey]);

  if (!workspacePath || !loaded) return null;

  if (!status?.isRepo) {
    return (
      <section className="git-status git-status--not-repo" aria-label={translate("gitStatusLabel")}>
        <p className="git-status__stat">{translate("gitNotARepo")}</p>
      </section>
    );
  }

  const statLine =
    status.filesChanged > 0
      ? formatMessage(locale, "gitDiffStat", {
          files: String(status.filesChanged),
          insertions: String(status.insertions),
          deletions: String(status.deletions),
        })
      : translate("gitClean");

  return (
    <section className="git-status" aria-label={translate("gitStatusLabel")}>
      <div className="git-status__branch" title={status.branch ?? undefined}>
        <span className="git-status__icon" aria-hidden="true">
          branch
        </span>
        <span className="git-status__branch-name">{status.branch ?? "—"}</span>
      </div>
      <p className="git-status__stat">{statLine}</p>
    </section>
  );
}
