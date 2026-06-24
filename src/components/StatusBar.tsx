import { useLocale } from "../context/LocaleContext";
import type { AgentPhase } from "../hooks/useChat";
import "./StatusBar.css";

interface StatusBarProps {
  visible: boolean;
  phase: AgentPhase;
  message: string;
  onCancel: () => void;
}

export function StatusBar({ visible, phase, message, onCancel }: StatusBarProps) {
  const { translate } = useLocale();

  if (!visible) return null;

  return (
    <div className="status-bar" role="status" aria-live="polite">
      <div className="status-bar__content">
        <span className="status-bar__phase">
          {phase === "running" ? translate("phaseRunning") : translate("agentThinking")}
        </span>
        <p className="status-bar__message" dir="auto">
          {message}
        </p>
      </div>
      <button type="button" className="status-bar__cancel" onClick={onCancel}>
        {translate("cancelRun")}
      </button>
    </div>
  );
}
