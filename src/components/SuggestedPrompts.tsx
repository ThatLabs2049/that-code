import { useLocale } from "../context/LocaleContext";
import "./SuggestedPrompts.css";

interface SuggestedPromptsProps {
  personalityId: string;
  onSelect: (prompt: string) => void;
  disabled?: boolean;
}

type PromptKey =
  | "suggestedLuna1"
  | "suggestedLuna2"
  | "suggestedLuna3"
  | "suggestedLuna4"
  | "suggestedSage1"
  | "suggestedSage2"
  | "suggestedSage3"
  | "suggestedSage4"
  | "suggestedSpark1"
  | "suggestedSpark2"
  | "suggestedSpark3"
  | "suggestedSpark4";

function promptsForPersonality(id: string): PromptKey[] {
  switch (id) {
    case "sage":
      return ["suggestedSage1", "suggestedSage2", "suggestedSage3", "suggestedSage4"];
    case "spark":
      return ["suggestedSpark1", "suggestedSpark2", "suggestedSpark3", "suggestedSpark4"];
    default:
      return ["suggestedLuna1", "suggestedLuna2", "suggestedLuna3", "suggestedLuna4"];
  }
}

export function SuggestedPrompts({ personalityId, onSelect, disabled }: SuggestedPromptsProps) {
  const { translate } = useLocale();
  const keys = promptsForPersonality(personalityId);

  return (
    <div className="suggested-prompts" role="group" aria-label={translate("suggestedPromptsLabel")}>
      <p className="suggested-prompts__title">{translate("suggestedPromptsTitle")}</p>
      <div className="suggested-prompts__list">
        {keys.map((key) => (
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
