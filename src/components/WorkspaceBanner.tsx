import { useLocale } from "../context/LocaleContext";
import "./WorkspaceBanner.css";

interface WorkspaceBannerProps {
  onOpenSettings: () => void;
}

export function WorkspaceBanner({ onOpenSettings }: WorkspaceBannerProps) {
  const { translate } = useLocale();

  return (
    <div className="workspace-banner" role="status">
      <p className="workspace-banner__text">{translate("workspaceBannerText")}</p>
      <button type="button" className="workspace-banner__action" onClick={onOpenSettings}>
        {translate("workspaceBannerAction")}
      </button>
    </div>
  );
}
