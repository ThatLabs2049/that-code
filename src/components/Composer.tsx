import { useEffect, useRef, useState, type FormEvent, type KeyboardEvent } from "react";

import type { RetrievedChunk } from "../lib/rag";

import type { AgentTier } from "../lib/settings";

import { searchCodebase } from "../lib/rag";

import { searchWorkspacePaths, searchWorkspaceSymbols } from "../lib/workspace";

import type { MessageAttachment, WorkspacePathHit, WorkspaceSymbolHit } from "../lib/workspace";

import { useLocale } from "../context/LocaleContext";

import { invokeErrorMessage } from "../lib/invokeError";

import { ContextPicker } from "./ContextPicker";

import "./ChatScreen.css";

import "./ContextPicker.css";



interface ComposerProps {

  onSend: (
    displayContent: string,
    attachments: MessageAttachment[],
    exploreThenImplement?: boolean,
    agentContent?: string,
  ) => void;

  agentTier: AgentTier;

  onAgentTierChange: (tier: AgentTier) => void;

  disabled?: boolean;

  disabledReason?: "loading" | "sending" | "error" | "plan";

  ragEnabled?: boolean;

  workspaceReady?: boolean;

  codebasePreview?: RetrievedChunk[];

  onCodebasePreview?: (chunks: RetrievedChunk[]) => void;

}



function parseAtQuery(text: string): string | null {

  const match = text.match(/@([\w./\-]*)$/);

  return match ? match[1] : null;

}



function stripAtQuery(text: string): string {

  return text.replace(/@[\w./\-]*$/, "").trimEnd();

}



export function Composer({

  onSend,

  agentTier,

  onAgentTierChange,

  disabled = false,

  disabledReason,

  ragEnabled = false,

  workspaceReady = false,

  codebasePreview = [],

  onCodebasePreview,

}: ComposerProps) {

  const { translate } = useLocale();

  const [draft, setDraft] = useState("");

  const [focused, setFocused] = useState(false);

  const [searchingCodebase, setSearchingCodebase] = useState(false);

  const [codebaseError, setCodebaseError] = useState<string | null>(null);

  const [pathAttachments, setPathAttachments] = useState<MessageAttachment[]>([]);

  const [pickerQuery, setPickerQuery] = useState<string | null>(null);

  const [pickerHits, setPickerHits] = useState<WorkspacePathHit[]>([]);

  const [symbolHits, setSymbolHits] = useState<WorkspaceSymbolHit[]>([]);

  const [pickerLoading, setPickerLoading] = useState(false);

  const composerRef = useRef<HTMLDivElement>(null);
  const pickerQueryRef = useRef(pickerQuery);
  pickerQueryRef.current = pickerQuery;

  function attachmentKey(attachment: MessageAttachment): string {
    return attachment.kind === "symbol"
      ? `symbol:${attachment.path}:${attachment.line}:${attachment.symbol}`
      : `${attachment.kind}:${attachment.path}`;
  }



  const pickerOpen = workspaceReady && pickerQuery !== null;



  useEffect(() => {

    if (!pickerOpen) {

      setPickerHits([]);

      setSymbolHits([]);

      return;

    }



    const query = pickerQuery;

    const timer = window.setTimeout(() => {

      setPickerLoading(true);

      const pathPromise = searchWorkspacePaths(query).catch(() => [] as WorkspacePathHit[]);

      const symbolPromise =

        query.trim().length >= 2

          ? searchWorkspaceSymbols(query).catch(() => [] as WorkspaceSymbolHit[])

          : Promise.resolve([] as WorkspaceSymbolHit[]);

      void Promise.all([pathPromise, symbolPromise])
        .then(([paths, symbols]) => {
          if (query !== pickerQueryRef.current) return;
          setPickerHits(paths);
          setSymbolHits(symbols);
        })

        .finally(() => setPickerLoading(false));

    }, 180);



    return () => window.clearTimeout(timer);

  }, [pickerOpen, pickerQuery]);



  function submit(exploreThenImplement = false) {

    const trimmed = draft.trim();

    if ((!trimmed && pathAttachments.length === 0 && codebasePreview.length === 0) || disabled) {

      return;

    }



    let content = trimmed;

    if (codebasePreview.length > 0) {

      const block = codebasePreview

        .map(

          (chunk) =>

            `[${chunk.sourcePath}] (score ${chunk.score.toFixed(2)})\n${chunk.snippet}`,

        )

        .join("\n\n");

      content = `${trimmed}\n\n--- @codebase context ---\n${block}\n---`;

    }



    onSend(
      trimmed,
      pathAttachments,
      exploreThenImplement,
      content !== trimmed ? content : undefined,
    );

    setDraft("");

    setPathAttachments([]);

    setPickerQuery(null);

    onCodebasePreview?.([]);

    setCodebaseError(null);

  }



  function handleSubmit(event: FormEvent) {

    event.preventDefault();

    submit();

  }



  function handleKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {

    if (event.key === "Escape" && pickerOpen) {

      event.preventDefault();

      setPickerQuery(null);

      return;

    }

    if (event.key === "Enter" && !event.shiftKey && !pickerOpen) {

      event.preventDefault();

      submit();

    }

  }



  function handleDraftChange(value: string) {

    setDraft(value);

    const atQuery = parseAtQuery(value);

    setPickerQuery(atQuery);

  }



  function handleSelectSymbol(hit: WorkspaceSymbolHit) {

    setPathAttachments((current) => {

      const key = `${hit.path}:${hit.line}:${hit.name}`;

      if (current.some((item) => item.kind === "symbol" && `${item.path}:${item.line}:${item.symbol}` === key)) {

        return current;

      }

      return [

        ...current,

        { path: hit.path, kind: "symbol", line: hit.line, symbol: hit.name },

      ];

    });

    setDraft((current) => stripAtQuery(current));

    setPickerQuery(null);

  }



  function handleSelectPath(hit: WorkspacePathHit) {

    setPathAttachments((current) => {

      if (current.some((item) => item.path === hit.path && item.kind === hit.kind)) {

        return current;

      }

      return [...current, { path: hit.path, kind: hit.kind }];

    });

    setDraft((current) => stripAtQuery(current));

    setPickerQuery(null);

  }



  function removeAttachment(attachment: MessageAttachment) {

    const key = attachmentKey(attachment);
    setPathAttachments((current) =>
      current.filter((item) => attachmentKey(item) !== key),
    );

  }



  function openContextPicker() {

    if (!workspaceReady) return;

    setDraft((current) => (current.endsWith("@") ? current : `${current}@`));

    setPickerQuery("");

  }



  async function handleCodebaseSearch() {

    const query = draft.trim() || translate("codebaseDefaultQuery");

    setSearchingCodebase(true);

    setCodebaseError(null);

    try {

      const chunks = await searchCodebase(query);

      onCodebasePreview?.(chunks);

      if (chunks.length === 0) {

        setCodebaseError(translate("codebaseNoResults"));

      }

    } catch (err) {

      setCodebaseError(invokeErrorMessage(err, translate("codebaseSearchError")));

      onCodebasePreview?.([]);

    } finally {

      setSearchingCodebase(false);

    }

  }



  const disabledHint =

    disabledReason === "sending"

      ? translate("composerDisabledSending")

        : disabledReason === "loading"

        ? translate("composerDisabledLoading")

        : disabledReason === "plan"

          ? translate("composerDisabledPlan")

        : disabledReason === "error"

          ? translate("composerDisabledError")

          : null;



  const hasAttachments = pathAttachments.length > 0 || codebasePreview.length > 0;



  return (

    <div className="composer-wrap" ref={composerRef}>

      {hasAttachments && (

        <div className="composer__attachments" role="status">

          {pathAttachments.map((attachment) => (

            <span key={attachmentKey(attachment)} className="composer__chip">

              <span className="composer__chip-label">

                {attachment.kind === "symbol"

                  ? `@symbol:${attachment.symbol} (${attachment.path}:${attachment.line})`

                  : `@${attachment.kind === "folder" ? "folder" : "file"}:${attachment.path}`}

              </span>

              <button

                type="button"

                className="composer__chip-remove"

                onClick={() => removeAttachment(attachment)}

                aria-label={translate("contextChipRemove")}

              >

                ×

              </button>

            </span>

          ))}

          {codebasePreview.length > 0 && (

            <span className="composer__chip composer__chip--codebase">

              <span className="composer__chip-label">{translate("codebaseAttached")}</span>

              <button

                type="button"

                className="composer__chip-remove"

                onClick={() => onCodebasePreview?.([])}

                aria-label={translate("codebaseClear")}

              >

                ×

              </button>

            </span>

          )}

        </div>

      )}

      <div className="composer__field">

        {pickerOpen && (

          <ContextPicker

            pathHits={pickerHits}

            symbolHits={symbolHits}

            loading={pickerLoading}

            query={pickerQuery ?? ""}

            onSelectPath={handleSelectPath}

            onSelectSymbol={handleSelectSymbol}

          />

        )}

        <form className="composer" onSubmit={handleSubmit} aria-label={translate("messageInputLabel")}>

          <label htmlFor="agent-tier" className="sr-only">

            {translate("agentTierLabel")}

          </label>

          <select

            id="agent-tier"

            className="composer__tier"

            value={agentTier}

            disabled={disabled}

            onChange={(event) => onAgentTierChange(event.target.value as AgentTier)}

            title={translate("agentTierHint")}

            aria-label={translate("agentTierLabel")}

          >

            <option value="auto">{translate("agentTierAuto")}</option>

            <option value="quick">{translate("agentTierQuick")}</option>

            <option value="standard">{translate("agentTierStandard")}</option>

            <option value="deep">{translate("agentTierDeep")}</option>

            <option value="explain">{translate("agentTierExplain")}</option>

          </select>

          <label htmlFor="chat-input" className="sr-only">

            {translate("messageInputLabel")}

          </label>

          {workspaceReady && (

            <button

              type="button"

              className="composer__context"

              disabled={disabled}

              onClick={openContextPicker}

              title={translate("contextActionHint")}

              aria-label={translate("contextAction")}

            >

              {translate("contextAction")}

            </button>

          )}

          {workspaceReady && (

            <button

              type="button"

              className="composer__explore"

              disabled={disabled || !draft.trim()}

              onClick={() => submit(true)}

              title={translate("exploreSubagentHint")}

              aria-label={translate("exploreSubagent")}

            >

              {translate("exploreSubagent")}

            </button>

          )}

          {ragEnabled && (

            <button

              type="button"

              className="composer__codebase"

              disabled={disabled || searchingCodebase}

              onClick={() => void handleCodebaseSearch()}

              title={translate("codebaseActionHint")}

              aria-label={translate("codebaseAction")}

            >

              {searchingCodebase ? "…" : translate("codebaseAction")}

            </button>

          )}

          <textarea

            id="chat-input"

            className="composer__input"

            rows={1}

            value={draft}

            placeholder={translate("messagePlaceholder")}

            disabled={disabled}

            dir="auto"

            aria-describedby={disabledHint ? "composer-disabled-hint" : focused ? "composer-hint" : undefined}

            onChange={(event) => handleDraftChange(event.target.value)}

            onKeyDown={handleKeyDown}

            onFocus={() => setFocused(true)}

            onBlur={() => setFocused(false)}

          />

          <button

            type="submit"

            className="composer__send"

            disabled={disabled || (!draft.trim() && pathAttachments.length === 0 && codebasePreview.length === 0)}

            aria-label={translate("sendMessage")}

          >

            {translate("send")}

          </button>

        </form>

      </div>

      {codebaseError && (

        <p className="composer__hint composer__hint--error" role="alert">

          {codebaseError}

        </p>

      )}

      {disabledHint ? (

        <p id="composer-disabled-hint" className="composer__hint composer__hint--muted" role="status">

          {disabledHint}

        </p>

      ) : focused ? (

        <p id="composer-hint" className="composer__hint composer__hint--muted">

          {translate("composerNewlineHint")}

        </p>

      ) : (

        <p className="composer__hint">{translate("composerHint")}</p>

      )}

    </div>

  );

}



export function focusComposer(): void {

  document.getElementById("chat-input")?.focus();

}


