import { useEffect } from "react";
import { useStore } from "../lib/store";
import { IconCheck, IconClose } from "./Icons";

export function Toasts() {
  const toast = useStore((state) => state.toast);
  const dismiss = useStore((state) => state.dismissToast);

  useEffect(() => {
    if (!toast) {
      return;
    }
    const timer = window.setTimeout(dismiss, 4500);
    return () => window.clearTimeout(timer);
  }, [toast, dismiss]);

  if (!toast) {
    return null;
  }

  return (
    <div className={`toast toast-${toast.kind}`} key={toast.id}>
      <span className="toast-icon">
        {toast.kind === "success" ? <IconCheck size={16} /> : <span className="toast-dot" />}
      </span>
      <span className="toast-text">{toast.text}</span>
      <button className="toast-close" onClick={dismiss}>
        <IconClose size={13} />
      </button>
    </div>
  );
}
