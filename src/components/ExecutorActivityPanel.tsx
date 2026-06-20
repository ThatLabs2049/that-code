import type { ExecutorActivity } from "../lib/chat";
import { useLocale } from "../context/LocaleContext";
import "./ExecutorActivityPanel.css";

interface ExecutorActivityPanelProps {
  activity: ExecutorActivity;
}

function statusClass(status: string): string {
  const normalized = status.toLowerCase();
  if (normalized === "success" || normalized === "done") return "executor-panel__status--success";
  if (normalized === "running") return "executor-panel__status--running";
  if (normalized === "error") return "executor-panel__status--error";
  return "executor-panel__status--neutral";
}

function formatStepLabel(step: string): string {
  if (step.startsWith("tool:")) {
    return step.slice(5);
  }
  return step;
}

export function ExecutorActivityPanel({ activity }: ExecutorActivityPanelProps) {
  const { translate } = useLocale();
  const isRunning = activity.status === "running";
  const stepCount = activity.activityLog.length;

  return (
    <details className="executor-panel" open={isRunning}>
      <summary className="executor-panel__summary">
        <span className="executor-panel__summary-main">
          <span className="executor-panel__chevron" aria-hidden="true" />
          <span className="executor-panel__title">{translate("executorActivity")}</span>
          {stepCount > 0 && (
            <span className="executor-panel__count">
              {stepCount} {translate("steps").toLowerCase()}
            </span>
          )}
        </span>
        <span className={`executor-panel__status ${statusClass(activity.status)}`}>
          {activity.status}
        </span>
      </summary>

      <div className="executor-panel__body">
        <div className="executor-panel__meta">
          <div className="executor-panel__meta-row">
            <span className="executor-panel__meta-label">{translate("objective")}</span>
            <p className="executor-panel__meta-value" dir="auto">
              {activity.taskSpec.objective}
            </p>
          </div>

          {activity.summary && (
            <div className="executor-panel__meta-row">
              <span className="executor-panel__meta-label">{translate("summary")}</span>
              <p className="executor-panel__meta-value" dir="auto">
                {activity.summary}
              </p>
            </div>
          )}
        </div>

        {activity.activityLog.length > 0 && (
          <ol className="executor-panel__timeline">
            {activity.activityLog.map((step, index) => (
              <li key={`${step.step}-${index}`} className="executor-panel__step">
                <span className="executor-panel__step-marker" aria-hidden="true" />
                <div className="executor-panel__step-content">
                  <span className="executor-panel__step-name">{formatStepLabel(step.step)}</span>
                  {step.detail && (
                    <span className="executor-panel__step-detail" dir="auto" title={step.detail}>
                      {step.detail}
                    </span>
                  )}
                </div>
              </li>
            ))}
          </ol>
        )}
      </div>
    </details>
  );
}
