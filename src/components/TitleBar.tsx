import { getCurrentWindow } from "@tauri-apps/api/window";
import { IconClose, IconMaximize, IconMinimize } from "./Icons";

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

function currentWindow() {
  return inTauri ? getCurrentWindow() : null;
}

export function TitleBar({ title }: { title: string }) {
  return (
    <header
      className="topbar"
      data-tauri-drag-region
      onMouseDown={(event) => {
        const target = event.target as HTMLElement;
        if (event.button !== 0 || target.closest("button")) {
          return;
        }
        void currentWindow()?.startDragging();
      }}
      onDoubleClick={() => void currentWindow()?.toggleMaximize()}
    >
      <div className="topbar-title" data-tauri-drag-region>
        {title}
      </div>
      <div className="window-controls">
        <button
          className="window-button"
          title="Свернуть"
          onClick={() => void currentWindow()?.minimize()}
        >
          <IconMinimize />
        </button>
        <button
          className="window-button"
          title="Развернуть"
          onClick={() => void currentWindow()?.toggleMaximize()}
        >
          <IconMaximize />
        </button>
        <button
          className="window-button window-close"
          title="Закрыть"
          onClick={() => void currentWindow()?.close()}
        >
          <IconClose />
        </button>
      </div>
    </header>
  );
}
