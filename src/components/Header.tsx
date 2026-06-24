import { type RefObject } from "react";
import { formatMessage } from "../lib/i18n";
import { useLocale } from "../context/LocaleContext";
import { RagStatusBadge } from "./RagStatusBadge";
import type { RagStatus, IndexProgress } from "../lib/rag";
import "./ChatScreen.css";

interface HeaderProps {
  workspacePath?: string | null;
  connectionConfigured?: boolean;
  changeReviewCount?: number;
  ragStatus?: RagStatus | null;
  ragIndexing?: boolean;
  ragIndexProgress?: IndexProgress | null;
  projectRulesFile?: string | null;
  projectRulesEnabled?: boolean;
  settingsTriggerRef: RefObject<HTMLButtonElement | null>;
  onOpenSettings: () => void;
  onOpenConnectionSettings?: () => void;
  onOpenWorkspaceSettings?: () => void;
  onOpenRagSettings?: () => void;
  onOpenChangeReview?: () => void;
  onReindexRag?: () => void;
  onCancelRagIndex?: () => void;
  onResetChat: () => void;
  resetDisabled?: boolean;
  interactionsDisabled?: boolean;
}

function truncatePath(path: string, maxLen = 24): string {
  if (path.length <= maxLen) return path;
  const parts = path.replace(/\\/g, "/").split("/");
  const name = parts[parts.length - 1] || path;
  return name.length <= maxLen ? name : `${name.slice(0, maxLen - 1)}…`;
}

export function Header({
  workspacePath,
  connectionConfigured,
  changeReviewCount = 0,
  ragStatus,
  ragIndexing = false,
  ragIndexProgress,
  projectRulesFile,
  projectRulesEnabled = true,
  settingsTriggerRef,
  onOpenSettings,
  onOpenConnectionSettings,
  onOpenWorkspaceSettings,
  onOpenRagSettings,
  onOpenChangeReview,
  onReindexRag,
  onCancelRagIndex,
  onResetChat,
  resetDisabled,
  interactionsDisabled,
}: HeaderProps) {
  const { locale, translate } = useLocale();

  return (
    <header className={`chat-header${interactionsDisabled ? " chat-header--disabled" : ""}`}>
      <div className="chat-header__brand">
        <div className="chat-header__logo-wrap">
          <img className="chat-header__logo" src="/thatcode-icon.svg" alt="" width={32} height={32} />
        </div>
        <div className="chat-header__title-wrap">
          <h1 className="chat-header__title">{translate("appName")}</h1>
          <p className="chat-header__subtitle">{translate("appSubtitle")}</p>
        </div>
      </div>
      <div className="chat-header__meta">
        <span
          className={`chat-header__connection${connectionConfigured ? " chat-header__connection--ok" : " chat-header__connection--missing"}`}
          role="status"
        >
          {connectionConfigured ? (
            translate("connectionStatusOk")
          ) : (
            <button
              type="button"
              className="chat-header__connection-action"
              disabled={interactionsDisabled}
              onClick={onOpenConnectionSettings ?? onOpenSettings}
            >
              {translate("connectionStatusMissing")}
            </button>
          )}
        </span>
        {workspacePath ? (
          <button
            type="button"
            className="chat-header__workspace"
            onClick={onOpenWorkspaceSettings ?? onOpenSettings}
            title={workspacePath}
            aria-label={translate("openWorkspaceSettings")}
          >
            {truncatePath(workspacePath)}
          </button>
        ) : (
          <button
            type="button"
            className="chat-header__workspace chat-header__workspace--unset"
            onClick={onOpenWorkspaceSettings ?? onOpenSettings}
            disabled={interactionsDisabled}
          >
            {translate("workspaceUnset")}
          </button>
        )}
        {projectRulesEnabled && projectRulesFile && (
          <span
            className="chat-header__rules"
            title={formatMessage(locale, "projectRulesActive", { file: projectRulesFile })}
          >
            {formatMessage(locale, "projectRulesBadge", { file: projectRulesFile })}
          </span>
        )}
        {workspacePath && onReindexRag && (
          <RagStatusBadge
            status={ragStatus ?? null}
            indexing={ragIndexing}
            indexProgress={ragIndexProgress}
            disabled={interactionsDisabled}
            onReindex={onReindexRag}
            onCancelIndex={onCancelRagIndex}
            onOpenSettings={onOpenRagSettings ?? onOpenSettings}
          />
        )}
        {changeReviewCount > 0 && onOpenChangeReview && (
          <button
            type="button"
            className="chat-header__changes-badge"
            onClick={onOpenChangeReview}
            aria-label={formatMessage(locale, "changesCount", {
              count: String(changeReviewCount),
            })}
          >
            {formatMessage(locale, "changesBadge", { count: String(changeReviewCount) })}
          </button>
        )}
        <button
          type="button"
          className="chat-header__settings"
          onClick={onResetChat}
          disabled={resetDisabled || interactionsDisabled}
          aria-label={translate("resetChat")}
        >
          {translate("resetChat")}
        </button>
        <button
          ref={settingsTriggerRef}
          type="button"
          className="chat-header__settings"
          onClick={onOpenSettings}
          disabled={interactionsDisabled}
          aria-label={translate("openSettings")}
          aria-haspopup="dialog"
        >
          {translate("settings")}
        </button>
      </div>
    </header>
  );
}
