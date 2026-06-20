import { useFocusTrap } from "../hooks/useFocusTrap";
import "./ConfirmDialog.css";

interface ConfirmDialogProps {
  title: string;
  body: string;
  confirmLabel: string;
  cancelLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
  destructive?: boolean;
}

export function ConfirmDialog({
  title,
  body,
  confirmLabel,
  cancelLabel,
  onConfirm,
  onCancel,
  destructive = false,
}: ConfirmDialogProps) {
  const dialogRef = useFocusTrap(true, onCancel);

  return (
    <div className="confirm-overlay" role="presentation" onClick={onCancel}>
      <div
        ref={dialogRef}
        className="confirm-dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="confirm-title"
        aria-describedby="confirm-body"
        onClick={(event) => event.stopPropagation()}
      >
        <h3 id="confirm-title" className="confirm-dialog__title">
          {title}
        </h3>
        <p id="confirm-body" className="confirm-dialog__body">
          {body}
        </p>
        <div className="confirm-dialog__actions">
          <button type="button" className="settings-button settings-button--secondary" onClick={onCancel}>
            {cancelLabel}
          </button>
          <button
            type="button"
            className={`settings-button ${destructive ? "settings-button--danger" : "settings-button--primary"}`}
            onClick={onConfirm}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
