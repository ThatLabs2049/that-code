import { useLocale } from "../context/LocaleContext";
import "./PersonalityCards.css";

const PERSONALITIES = ["luna", "sage", "spark"] as const;

type PersonalityId = (typeof PERSONALITIES)[number];

interface PersonalityCardsProps {
  value: string;
  onChange: (id: string) => void;
  disabled?: boolean;
}

function personalityLabelKey(id: PersonalityId): "personalityLuna" | "personalitySage" | "personalitySpark" {
  switch (id) {
    case "sage":
      return "personalitySage";
    case "spark":
      return "personalitySpark";
    default:
      return "personalityLuna";
  }
}

function personalityDescKey(id: PersonalityId): "personalityDescLuna" | "personalityDescSage" | "personalityDescSpark" {
  switch (id) {
    case "sage":
      return "personalityDescSage";
    case "spark":
      return "personalityDescSpark";
    default:
      return "personalityDescLuna";
  }
}

function personalitySampleKey(
  id: PersonalityId,
): "personalitySampleLuna" | "personalitySampleSage" | "personalitySampleSpark" {
  switch (id) {
    case "sage":
      return "personalitySampleSage";
    case "spark":
      return "personalitySampleSpark";
    default:
      return "personalitySampleLuna";
  }
}

export function PersonalityCards({ value, onChange, disabled }: PersonalityCardsProps) {
  const { translate } = useLocale();

  return (
    <div className="personality-cards" role="radiogroup" aria-label={translate("personalityId")}>
      {PERSONALITIES.map((id) => {
        const selected = value === id;
        return (
          <button
            key={id}
            type="button"
            role="radio"
            aria-checked={selected}
            className={`personality-card${selected ? " personality-card--selected" : ""}`}
            data-personality={id}
            disabled={disabled}
            onClick={() => onChange(id)}
          >
            <span className="personality-card__marker" aria-hidden="true" />
            <span className="personality-card__name">{translate(personalityLabelKey(id))}</span>
            <span className="personality-card__desc">{translate(personalityDescKey(id))}</span>
            <span className="personality-card__sample">{translate(personalitySampleKey(id))}</span>
          </button>
        );
      })}
      <p className="personality-cards__hint">{translate("personalityGreetingHint")}</p>
    </div>
  );
}

export function personalityDisplayName(locale: "en" | "fa", id: string): string {
  switch (id) {
    case "sage":
      return locale === "fa" ? "سیج" : "Sage";
    case "spark":
      return locale === "fa" ? "اسپارک" : "Spark";
    default:
      return locale === "fa" ? "لونا" : "Luna";
  }
}
