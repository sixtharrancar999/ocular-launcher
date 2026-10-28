import { useEffect, useRef } from "react";
import { useI18n } from "../lib/i18n";
import { useStore } from "../lib/store";
import { IconChevron, IconCopy, IconTerminal, IconTrash } from "./Icons";

export function ConsoleDock() {
  const { t } = useI18n();
  const open = useStore((state) => state.consoleOpen);
  const setOpen = useStore((state) => state.setConsoleOpen);
  const logs = useStore((state) => state.logs);
  const clearLogs = useStore((state) => state.clearLogs);
  const pushLog = useStore((state) => state.pushLog);
  const showToast = useStore((state) => state.showToast);
  const bodyRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) {
      return;
    }
    const body = bodyRef.current;
    if (body) {
      body.scrollTop = body.scrollHeight;
    }
  }, [logs, open]);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(logs.map((line) => line.text).join("\n"));
      showToast("info", t("toast.copied"));
    } catch (error) {
      pushLog("launcher", String(error));
    }
  };

  return (
    <section className={`console ${open ? "open" : ""}`}>
      <button className="console-bar" onClick={() => setOpen(!open)}>
        <span className="console-title">
          <IconTerminal size={16} />
          {t("console.title")}
          {logs.length > 0 && <span className="console-count">{logs.length}</span>}
        </span>
        <span className="console-hint">{t("console.hint")}</span>
        <span className="console-actions">
          <span
            className="console-action"
            role="button"
            tabIndex={0}
            title={t("action.clear")}
            onClick={(event) => {
              event.stopPropagation();
              clearLogs();
            }}
          >
            <IconTrash size={15} />
          </span>
          <span
            className="console-action"
            role="button"
            tabIndex={0}
            title={t("action.copy")}
            onClick={(event) => {
              event.stopPropagation();
              void copy();
            }}
          >
            <IconCopy size={15} />
          </span>
          <IconChevron size={16} className={`console-chevron ${open ? "open" : ""}`} />
        </span>
      </button>

      {open && (
        <div className="console-body" ref={bodyRef}>
          {logs.length === 0 ? (
            <div className="console-empty">{t("console.empty")}</div>
          ) : (
            logs.map((line) => (
              <div key={line.id} className={`log-line log-${line.stream}`}>
                <span className="log-mark" />
                <span className="log-text">{line.text}</span>
              </div>
            ))
          )}
        </div>
      )}
    </section>
  );
}
