import { useLocale } from "../context/LocaleContext";

import { formatMessage } from "../lib/i18n";

import type { IndexProgress, RagStatus } from "../lib/rag";

import "./ChatScreen.css";



interface RagStatusBadgeProps {

  status: RagStatus | null;

  indexing: boolean;

  indexProgress?: IndexProgress | null;

  disabled?: boolean;

  onReindex: () => void;

  onCancelIndex?: () => void;

  onOpenSettings?: () => void;

}



function formatIndexedAgo(iso: string | null | undefined, locale: string): string {

  if (!iso) return "";

  const date = new Date(iso);

  if (Number.isNaN(date.getTime())) return "";

  const diffMs = Date.now() - date.getTime();

  const minutes = Math.floor(diffMs / 60_000);

  if (minutes < 1) return locale === "fa" ? "همین الان" : "just now";

  if (minutes < 60) return locale === "fa" ? `${minutes} دقیقه پیش` : `${minutes}m ago`;

  const hours = Math.floor(minutes / 60);

  if (hours < 48) return locale === "fa" ? `${hours} ساعت پیش` : `${hours}h ago`;

  return date.toLocaleDateString();

}



export function RagStatusBadge({

  status,

  indexing,

  indexProgress,

  disabled,

  onReindex,

  onCancelIndex,

  onOpenSettings,

}: RagStatusBadgeProps) {

  const { locale, translate } = useLocale();



  if (!status?.enabled) {

    return (

      <button

        type="button"

        className="chat-header__rag chat-header__rag--off"

        disabled={disabled}

        onClick={onOpenSettings}

        title={translate("ragEnableHint")}

      >

        {translate("ragStatusOff")}

      </button>

    );

  }



  if (indexing) {

    const progressLabel = indexProgress

      ? formatMessage(locale, "ragIndexProgress", {

          done: String(indexProgress.filesDone),

          total: String(indexProgress.filesTotal),

          chunks: String(indexProgress.chunksStored),

        })

      : translate("ragIndexing");



    return (

      <span className="chat-header__rag chat-header__rag--indexing" role="status">

        <span title={indexProgress?.currentFile}>{progressLabel}</span>

        {onCancelIndex && (

          <button

            type="button"

            className="chat-header__rag-cancel"

            disabled={disabled}

            onClick={onCancelIndex}

            aria-label={translate("ragIndexCancel")}

          >

            {translate("cancel")}

          </button>

        )}

      </span>

    );

  }



  const label = formatMessage(locale, "ragStatusBadge", {

    count: String(status.chunkCount),

    ago: formatIndexedAgo(status.lastIndexedAt, locale) || translate("ragStatusNever"),

  });



  return (

    <button

      type="button"

      className="chat-header__rag"

      disabled={disabled}

      onClick={onReindex}

      title={translate("ragReindexHint")}

    >

      {label}

    </button>

  );

}


