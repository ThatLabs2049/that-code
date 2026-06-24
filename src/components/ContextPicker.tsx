import type { WorkspacePathHit, WorkspaceSymbolHit } from "../lib/workspace";
import { useLocale } from "../context/LocaleContext";
import "./ContextPicker.css";

interface ContextPickerProps {
  pathHits: WorkspacePathHit[];
  symbolHits: WorkspaceSymbolHit[];
  loading: boolean;
  query: string;
  onSelectPath: (hit: WorkspacePathHit) => void;
  onSelectSymbol: (hit: WorkspaceSymbolHit) => void;
}

export function ContextPicker({
  pathHits,
  symbolHits,
  loading,
  query,
  onSelectPath,
  onSelectSymbol,
}: ContextPickerProps) {
  const { translate } = useLocale();
  const empty = !loading && pathHits.length === 0 && symbolHits.length === 0;

  if (!query && empty) {
    return (
      <div className="context-picker" role="listbox" aria-label={translate("contextPickerLabel")}>
        <p className="context-picker__hint">{translate("contextPickerHint")}</p>
      </div>
    );
  }

  return (
    <div className="context-picker" role="listbox" aria-label={translate("contextPickerLabel")}>
      {loading && (
        <p className="context-picker__status" role="status">
          {translate("contextPickerSearching")}
        </p>
      )}
      {!loading && empty && query && (
        <p className="context-picker__status">{translate("contextPickerEmpty")}</p>
      )}
      {symbolHits.length > 0 && (
        <ul className="context-picker__list">
          {symbolHits.map((hit) => (
            <li key={`sym:${hit.path}:${hit.line}:${hit.name}`}>
              <button
                type="button"
                className="context-picker__item context-picker__item--symbol"
                role="option"
                onMouseDown={(event) => {
                  event.preventDefault();
                  onSelectSymbol(hit);
                }}
              >
                <span className="context-picker__kind">⌁</span>
                <span className="context-picker__path">
                  {hit.name}
                  <span className="context-picker__meta">
                    {hit.path}:{hit.line}
                  </span>
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
      {pathHits.length > 0 && (
        <ul className="context-picker__list">
          {pathHits.map((hit) => (
            <li key={`${hit.kind}:${hit.path}`}>
              <button
                type="button"
                className="context-picker__item"
                role="option"
                onMouseDown={(event) => {
                  event.preventDefault();
                  onSelectPath(hit);
                }}
              >
                <span className="context-picker__kind">{hit.kind === "folder" ? "📁" : "📄"}</span>
                <span className="context-picker__path">{hit.path}</span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
