import { useLocale } from "../context/LocaleContext";
import type { DelegatePhase } from "../hooks/useChat";
import "./StatusBar.css";

interface StatusBarProps {
  visible: boolean;
  phase: DelegatePhase;
  message: string;
  onCancel: () => void;
}

function phaseLabel(
  translate: (key: "phaseUnderstanding" | "phaseExecuting" | "phaseFormatting" | "companionThinking") => string,
  phase: DelegatePhase,
): string {
  switch (phase) {
    case "understanding":
      return translate("phaseUnderstanding");
    case "executing":
      return translate("phaseExecuting");
    case "formatting":
      return translate("phaseFormatting");
    default:
      return translate("companionThinking");
  }
}

export function StatusBar({ visible, phase, message, onCancel }: StatusBarProps) {
  const { translate } = useLocale();

  if (!visible) return null;

  return (
    <div className="status-bar" role="status" aria-live="polite">
      <div className="status-bar__content">
        <span className="status-bar__phase">{phaseLabel(translate, phase)}</span>
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
