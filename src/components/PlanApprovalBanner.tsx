import { useLocale } from "../context/LocaleContext";
import "./PlanApprovalBanner.css";

interface PlanApprovalBannerProps {
  planContent: string;
  busy?: boolean;
  onApprove: () => void;
  onReject: () => void;
}

export function PlanApprovalBanner({
  planContent,
  busy = false,
  onApprove,
  onReject,
}: PlanApprovalBannerProps) {
  const { translate } = useLocale();

  return (
    <section className="plan-banner" aria-labelledby="plan-banner-title">
      <h3 id="plan-banner-title" className="plan-banner__title">
        {translate("planApprovalTitle")}
      </h3>
      <p className="plan-banner__help">{translate("planApprovalHelp")}</p>
      <pre className="plan-banner__content" dir="auto">
        {planContent}
      </pre>
      <div className="plan-banner__actions">
        <button
          type="button"
          className="plan-banner__approve"
          disabled={busy}
          onClick={onApprove}
        >
          {translate("planApprovalApprove")}
        </button>
        <button
          type="button"
          className="plan-banner__reject"
          disabled={busy}
          onClick={onReject}
        >
          {translate("planApprovalReject")}
        </button>
      </div>
    </section>
  );
}
