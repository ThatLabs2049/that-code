import { type RefObject } from "react";
import { formatMessage } from "../lib/i18n";
import { useLocale } from "../context/LocaleContext";
import { personalityDisplayName } from "./PersonalityCards";
import "./ChatScreen.css";

interface HeaderProps {
  personalityId: string;
  workspacePath?: string | null;
  connectionConfigured?: boolean;
  changeReviewCount?: number;
  settingsTriggerRef: RefObject<HTMLButtonElement | null>;
  onOpenSettings: () => void;
  onOpenConnectionSettings?: () => void;
  onOpenWorkspaceSettings?: () => void;
  onOpenChangeReview?: () => void;
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
  personalityId,
  workspacePath,
  connectionConfigured,
  changeReviewCount = 0,
  settingsTriggerRef,
  onOpenSettings,
  onOpenConnectionSettings,
  onOpenWorkspaceSettings,
  onOpenChangeReview,
  onResetChat,
  resetDisabled,
  interactionsDisabled,
}: HeaderProps) {
  const { locale, translate } = useLocale();
  const companionName = personalityDisplayName(locale, personalityId);

  return (
    <header className={`chat-header${interactionsDisabled ? " chat-header--disabled" : ""}`}>
      <div className="chat-header__brand">
        <div className="chat-header__logo-wrap">
          <img className="chat-header__logo" src="/muse-logo.svg" alt="" width={24} height={24} />
        </div>
        <div className="chat-header__title-wrap">
          <div className="chat-header__title-row">
            <h1 className="chat-header__title">{translate("appName")}</h1>
            <span className="chat-header__personality" data-personality={personalityId}>
              <span className="chat-header__personality-marker" aria-hidden="true" />
              {companionName}
            </span>
          </div>
          <p className="chat-header__subtitle">
            {formatMessage(locale, "appSubtitleDynamic", { name: companionName })}
          </p>
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
        ) : null}
        {changeReviewCount > 0 && (
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
