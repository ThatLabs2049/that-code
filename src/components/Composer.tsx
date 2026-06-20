import { useState, type FormEvent, type KeyboardEvent } from "react";

import { formatMessage } from "../lib/i18n";

import { useLocale } from "../context/LocaleContext";

import { personalityDisplayName } from "./PersonalityCards";

import "./ChatScreen.css";



interface ComposerProps {

  personalityId: string;

  onSend: (content: string) => void;

  disabled?: boolean;

  disabledReason?: "loading" | "sending" | "error";

}



function placeholderKey(personalityId: string): "messagePlaceholderLuna" | "messagePlaceholderSage" | "messagePlaceholderSpark" {

  switch (personalityId) {

    case "sage":

      return "messagePlaceholderSage";

    case "spark":

      return "messagePlaceholderSpark";

    default:

      return "messagePlaceholderLuna";

  }

}



export function Composer({ personalityId, onSend, disabled = false, disabledReason }: ComposerProps) {

  const { locale, translate } = useLocale();

  const [draft, setDraft] = useState("");

  const [focused, setFocused] = useState(false);

  const companionName = personalityDisplayName(locale, personalityId);



  function submit() {

    const trimmed = draft.trim();

    if (!trimmed || disabled) return;

    onSend(trimmed);

    setDraft("");

  }



  function handleSubmit(event: FormEvent) {

    event.preventDefault();

    submit();

  }



  function handleKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {

    if (event.key === "Enter" && !event.shiftKey) {

      event.preventDefault();

      submit();

    }

  }



  const disabledHint =

    disabledReason === "sending"

      ? translate("composerDisabledSending")

      : disabledReason === "loading"

        ? translate("composerDisabledLoading")

        : disabledReason === "error"

          ? translate("composerDisabledError")

          : null;



  return (

    <>

      <form

        className="composer"

        onSubmit={handleSubmit}

        aria-label={formatMessage(locale, "messageCompanionDynamic", { name: companionName })}

      >

        <label htmlFor="chat-input" className="sr-only">

          {formatMessage(locale, "messageCompanionDynamic", { name: companionName })}

        </label>

        <textarea

          id="chat-input"

          className="composer__input"

          rows={1}

          value={draft}

          placeholder={translate(placeholderKey(personalityId))}

          disabled={disabled}

          dir="auto"

          aria-describedby={disabledHint ? "composer-disabled-hint" : focused ? "composer-hint" : undefined}

          onChange={(event) => setDraft(event.target.value)}

          onKeyDown={handleKeyDown}

          onFocus={() => setFocused(true)}

          onBlur={() => setFocused(false)}

        />

        <button

          type="submit"

          className="composer__send"

          disabled={disabled || !draft.trim()}

          aria-label={translate("sendMessage")}

        >

          {translate("send")}

        </button>

      </form>

      {disabledHint ? (

        <p id="composer-disabled-hint" className="composer__hint composer__hint--muted" role="status">

          {disabledHint}

        </p>

      ) : focused ? (

        <p id="composer-hint" className="composer__hint composer__hint--muted">

          {translate("composerNewlineHint")}

        </p>

      ) : (

        <p className="composer__hint">

          {formatMessage(locale, "composerHintDynamic", { name: companionName })}

        </p>

      )}

    </>

  );

}



export function focusComposer(): void {

  document.getElementById("chat-input")?.focus();

}


