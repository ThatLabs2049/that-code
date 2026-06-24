import { useLocale } from "../context/LocaleContext";
import "./SuggestedPrompts.css";

const PROMPT_KEYS = [
  "suggested1",
  "suggested2",
  "suggested3",
  "suggested4",
] as const;

interface SuggestedPromptsProps {
  onSelect: (prompt: string) => void;
  disabled?: boolean;
}

export function SuggestedPrompts({ onSelect, disabled }: SuggestedPromptsProps) {
  const { translate } = useLocale();

  return (
    <div className="suggested-prompts" role="group" aria-label={translate("suggestedPromptsLabel")}>
      <p className="suggested-prompts__title">{translate("suggestedPromptsTitle")}</p>
      <div className="suggested-prompts__list">
        {PROMPT_KEYS.map((key) => (
          <button
            key={key}
            type="button"
            className="suggested-prompts__chip"
            disabled={disabled}
            onClick={() => onSelect(translate(key))}
          >
            {translate(key)}
          </button>
        ))}
      </div>
    </div>
  );
}
