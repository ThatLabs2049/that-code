import { useEffect, useState } from "react";
import type { ExecutorActivity } from "../lib/chat";
import { useLocale } from "../context/LocaleContext";
import "./AgentActivityPanel.css";

interface AgentActivityPanelProps {
  activity: ExecutorActivity;
}

const TOOL_STEP_KEYS: Record<string, "toolRunCommand" | "toolReadFile" | "toolEditFile" | "toolWriteFile" | "toolGrep" | "toolListDir"> = {
  run_command: "toolRunCommand",
  read_file: "toolReadFile",
  edit_file: "toolEditFile",
  write_file: "toolWriteFile",
  grep: "toolGrep",
  list_dir: "toolListDir",
};

function statusClass(status: string): string {
  const normalized = status.toLowerCase();
  if (normalized === "success" || normalized === "done") return "agent-panel__status--success";
  if (normalized === "running") return "agent-panel__status--running";
  if (normalized === "error") return "agent-panel__status--error";
  return "agent-panel__status--neutral";
}

export function AgentActivityPanel({ activity }: AgentActivityPanelProps) {
  const { translate } = useLocale();
  const isRunning = activity.status === "running";
  const stepCount = activity.activityLog.length;
  const [open, setOpen] = useState(true);

  useEffect(() => {
    if (isRunning) {
      setOpen(true);
    }
  }, [isRunning]);

  function formatStepLabel(step: string): string {
    if (step.startsWith("tool:")) {
      const tool = step.slice(5);
      const key = TOOL_STEP_KEYS[tool];
      return key ? translate(key) : tool;
    }
    return step;
  }

  return (
    <section className={`agent-panel${open ? " agent-panel--open" : ""}`}>
      <button
        type="button"
        className="agent-panel__header"
        aria-expanded={open}
        onClick={() => setOpen((value) => !value)}
      >
        <span className="agent-panel__summary-main">
          <span className="agent-panel__chevron" aria-hidden="true" />
          <span className="agent-panel__title">{translate("agentActivity")}</span>
          {stepCount > 0 && (
            <span className="agent-panel__count">
              {stepCount} {translate("steps").toLowerCase()}
            </span>
          )}
        </span>
        <span className={`agent-panel__status ${statusClass(activity.status)}`}>
          {activity.status}
        </span>
      </button>

      {open && (
        <div className="agent-panel__body">
          <div className="agent-panel__meta">
            <div className="agent-panel__meta-row">
              <span className="agent-panel__meta-label">{translate("objective")}</span>
              <p className="agent-panel__meta-value" dir="auto">
                {activity.taskSpec.objective}
              </p>
            </div>

            {activity.summary && (
              <div className="agent-panel__meta-row">
                <span className="agent-panel__meta-label">{translate("summary")}</span>
                <p className="agent-panel__meta-value" dir="auto">
                  {activity.summary}
                </p>
              </div>
            )}
          </div>

          {activity.activityLog.length > 0 && (
            <ol className="agent-panel__timeline">
              {activity.activityLog.map((step, index) => (
                <li key={`${step.step}-${index}`} className="agent-panel__step">
                  <span className="agent-panel__step-marker" aria-hidden="true" />
                  <div className="agent-panel__step-content">
                    <span className="agent-panel__step-name">{formatStepLabel(step.step)}</span>
                    {step.detail && (
                      <span
                        className={`agent-panel__step-detail${step.step.includes("run_command") || step.step.includes("verify") ? " agent-panel__step-detail--output" : ""}`}
                        dir="auto"
                        title={step.detail}
                      >
                        {step.detail}
                      </span>
                    )}
                  </div>
                </li>
              ))}
            </ol>
          )}
        </div>
      )}
    </section>
  );
}
