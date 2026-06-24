import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";
import type { AgentTier } from "../lib/settings";
import { useLocale } from "../context/LocaleContext";
import "./CommandPalette.css";

export interface CommandPaletteAction {
  id: string;
  label: string;
  keywords?: string;
  run: () => void;
}

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
  actions: CommandPaletteAction[];
}

export function CommandPalette({ open, onClose, actions }: CommandPaletteProps) {
  const { translate } = useLocale();
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return actions;
    return actions.filter(
      (action) =>
        action.label.toLowerCase().includes(q) ||
        action.keywords?.toLowerCase().includes(q),
    );
  }, [actions, query]);

  useEffect(() => {
    if (!open) {
      setQuery("");
      setActiveIndex(0);
      return;
    }
    const timer = window.setTimeout(() => inputRef.current?.focus(), 0);
    return () => window.clearTimeout(timer);
  }, [open]);

  useEffect(() => {
    setActiveIndex(0);
  }, [query]);

  useEffect(() => {
    if (!open) return;
    const active = listRef.current?.querySelector(".command-palette__item--active");
    active?.scrollIntoView({ block: "nearest" });
  }, [activeIndex, open, filtered.length]);

  if (!open) return null;

  function runAction(action: CommandPaletteAction) {
    onClose();
    action.run();
  }

  function handleKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setActiveIndex((i) => Math.min(i + 1, Math.max(filtered.length - 1, 0)));
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setActiveIndex((i) => Math.max(i - 1, 0));
      return;
    }
    if (event.key === "Enter" && filtered[activeIndex]) {
      event.preventDefault();
      runAction(filtered[activeIndex]);
    }
  }

  return (
    <div className="command-palette-backdrop" role="presentation" onClick={onClose}>
      <div
        className="command-palette"
        role="dialog"
        aria-modal="true"
        aria-label={translate("commandPaletteTitle")}
        onClick={(e) => e.stopPropagation()}
      >
        <input
          ref={inputRef}
          type="search"
          className="command-palette__input"
          role="combobox"
          aria-expanded="true"
          aria-autocomplete="list"
          placeholder={translate("commandPalettePlaceholder")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          aria-controls="command-palette-list"
          autoComplete="off"
        />
        <ul id="command-palette-list" ref={listRef} className="command-palette__list" role="listbox">
          {filtered.length === 0 ? (
            <li className="command-palette__empty">{translate("commandPaletteEmpty")}</li>
          ) : (
            filtered.map((action, index) => (
              <li key={action.id}>
                <button
                  type="button"
                  className={`command-palette__item${index === activeIndex ? " command-palette__item--active" : ""}`}
                  role="option"
                  aria-selected={index === activeIndex}
                  onMouseEnter={() => setActiveIndex(index)}
                  onClick={() => runAction(action)}
                >
                  {action.label}
                </button>
              </li>
            ))
          )}
        </ul>
        <p className="command-palette__hint">{translate("commandPaletteHint")}</p>
      </div>
    </div>
  );
}

export const AGENT_TIER_OPTIONS: AgentTier[] = ["auto", "quick", "standard", "deep", "explain"];
