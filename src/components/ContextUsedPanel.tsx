import type { RetrievedChunk } from "../lib/rag";
import { useLocale } from "../context/LocaleContext";
import "./ContextUsedPanel.css";

interface ContextUsedPanelProps {
  chunks: RetrievedChunk[];
  preview?: boolean;
}

export function ContextUsedPanel({ chunks, preview = false }: ContextUsedPanelProps) {
  const { translate } = useLocale();

  if (chunks.length === 0) return null;

  return (
    <section
      className={`context-used${preview ? " context-used--preview" : ""}`}
      aria-labelledby="context-used-title"
    >
      <h3 id="context-used-title" className="context-used__title">
        {preview ? translate("contextPreviewTitle") : translate("contextUsedTitle")}
      </h3>
      <ul className="context-used__list">
        {chunks.map((chunk) => (
          <li key={`${chunk.sourcePath}-${chunk.score}`} className="context-used__item">
            <div className="context-used__meta">
              <span className="context-used__path" dir="ltr">
                {chunk.sourcePath}
              </span>
              <span className="context-used__score">{chunk.score.toFixed(2)}</span>
            </div>
            <p className="context-used__snippet" dir="ltr">
              {chunk.snippet}
            </p>
          </li>
        ))}
      </ul>
    </section>
  );
}
