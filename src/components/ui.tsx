import { useEffect, useRef } from "react";
import type { ReactNode } from "react";
import { AlertTriangle, X } from "lucide-react";
import { useText } from "../lib/i18n";
export function Notice({
  title,
  children,
  kind = "warning",
}: {
  title?: string;
  children?: ReactNode;
  kind?: "warning" | "info" | "error";
}) {
  return (
    <div
      className={`notice ${kind}`}
      role={kind === "error" ? "alert" : undefined}
    >
      <AlertTriangle size={18} />
      <div>
        {title && <strong>{title}</strong>}
        {children && <div>{children}</div>}
      </div>
    </div>
  );
}
export function Modal({
  title,
  children,
  onClose,
  busy = false,
  error,
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
  busy?: boolean;
  error?: string | null;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  const t = useText();
  useEffect(() => {
    const d = ref.current;
    d?.showModal();
    d?.querySelector<HTMLInputElement>("input,textarea,select")?.focus();
    return () => d?.close();
  }, []);
  return (
    <dialog
      ref={ref}
      className="modal"
      aria-labelledby="dialog-title"
      onCancel={(e) => {
        e.preventDefault();
        if (!busy) onClose();
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget && !busy) {
          const rect = ref.current!.getBoundingClientRect();
          if (
            e.clientX < rect.left ||
            e.clientX > rect.right ||
            e.clientY < rect.top ||
            e.clientY > rect.bottom
          )
            onClose();
        }
      }}
    >
      <header>
        <h2 id="dialog-title">{title}</h2>
        <button
          className="icon-button"
          type="button"
          onClick={onClose}
          disabled={busy}
          aria-label={t("close")}
        >
          <X />
        </button>
      </header>
      {children}
      {error && <Notice kind="error">{error}</Notice>}
    </dialog>
  );
}
export function ModalFooter({
  onClose,
  busy,
  submit,
  disabled = false,
  danger = false,
}: {
  onClose: () => void;
  busy: boolean;
  submit: string;
  disabled?: boolean;
  danger?: boolean;
}) {
  const t = useText();
  return (
    <footer className="modal-footer">
      <button type="button" onClick={onClose} disabled={busy}>
        {t("cancel")}
      </button>
      <button
        className={danger ? "danger" : "primary"}
        type="submit"
        disabled={busy || disabled}
      >
        {busy ? t("working") : submit}
      </button>
    </footer>
  );
}
