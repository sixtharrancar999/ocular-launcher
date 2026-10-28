import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { I18nProvider, resolveLang, translate } from "./lib/i18n";
import { useStore } from "./lib/store";
import { Sidebar } from "./components/Sidebar";
import { TitleBar } from "./components/TitleBar";
import { ConsoleDock } from "./components/ConsoleDock";
import { Toasts } from "./components/Toasts";
import { PlayPage } from "./pages/PlayPage";
import { VersionsPage } from "./pages/VersionsPage";
import { SettingsPage } from "./pages/SettingsPage";

interface InstallProgressPayload {
  tag: string;
  stage: "download" | "extract" | "done";
  received: number;
  total: number;
}

interface PrepareProgressPayload {
  stage: string;
  received: number;
  total: number;
}

interface OpLogPayload {
  scope: string;
  line: string;
}

interface GameLogPayload {
  stream: "stdout" | "stderr";
  line: string;
}

interface GameStartedPayload {
  tag: string;
  pid: number;
}

interface GameExitPayload {
  tag: string;
  code: number | null;
}

interface OperationErrorPayload {
  scope: string;
  message: string;
}

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export default function App() {
  const boot = useStore((state) => state.boot);
  const page = useStore((state) => state.page);
  const lang = resolveLang(boot?.settings.language ?? "system");

  useEffect(() => {
    void useStore.getState().init();
    const timer = window.setTimeout(() => {
      void useStore.getState().checkLauncherUpdate(false);
    }, 4000);
    return () => window.clearTimeout(timer);
  }, []);

  useEffect(() => {
    if (!inTauri) {
      return;
    }
    let disposed = false;
    let refreshTimer: number | undefined;
    const unlisteners: UnlistenFn[] = [];

    const scheduleRefresh = () => {
      if (refreshTimer !== undefined) {
        window.clearTimeout(refreshTimer);
      }
      refreshTimer = window.setTimeout(() => {
        void useStore.getState().refresh();
      }, 250);
    };

    const register = async () => {
      const registerOne = async <T,>(event: string, handler: (payload: T) => void) => {
        const unlisten = await listen<T>(event, (message) => handler(message.payload));
        if (disposed) {
          unlisten();
        } else {
          unlisteners.push(unlisten);
        }
      };

      await registerOne<InstallProgressPayload>("install-progress", (payload) => {
        const current = useStore.getState().op;
        // Late events after the command resolved must not resurrect the operation.
        if (current?.kind !== "install" || current.tag !== payload.tag) {
          return;
        }
        useStore.setState({
          op: {
            kind: "install",
            tag: payload.tag,
            stage: payload.stage,
            received: payload.received,
            total: payload.total,
          },
        });
      });

      await registerOne<PrepareProgressPayload>("prepare-progress", (payload) => {
        const current = useStore.getState().op;
        if (current?.kind !== "prepare") {
          return;
        }
        useStore.setState({ op: { ...current, stage: payload.stage } });
      });

      await registerOne<OpLogPayload>("op-log", (payload) => {
        useStore
          .getState()
          .pushLog(payload.scope === "prepare" ? "prepare" : "launcher", payload.line);
      });

      await registerOne<GameLogPayload>("game-log", (payload) => {
        useStore.getState().pushLog(payload.stream, payload.line);
      });

      await registerOne<GameStartedPayload>("game-started", () => {
        useStore.getState().setConsoleOpen(true);
        scheduleRefresh();
      });

      await registerOne<GameExitPayload>("game-exit", (payload) => {
        scheduleRefresh();
        if (payload.code === null) {
          return;
        }
        const store = useStore.getState();
        const language = resolveLang(store.boot?.settings.language ?? "system");
        if (payload.code === 0) {
          store.showToast("info", translate(language, "toast.exitedOk"));
        } else {
          store.showToast(
            "error",
            translate(language, "toast.exited", { code: payload.code }),
          );
        }
      });

      await registerOne<OperationErrorPayload>("operation-error", (payload) => {
        useStore.getState().showToast("error", payload.message);
      });

      await registerOne<null>("appearance-changed", () => {
        void useStore.getState().loadAppearance();
      });

      await registerOne<null>("state-changed", () => {
        scheduleRefresh();
      });
    };

    void register();

    return () => {
      disposed = true;
      if (refreshTimer !== undefined) {
        window.clearTimeout(refreshTimer);
      }
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  }, []);

  const title =
    page === "play"
      ? "Ocular Launcher"
      : page === "versions"
        ? translate(lang, "versions.title")
        : translate(lang, "settings.title");

  return (
    <I18nProvider lang={lang}>
      <div className="app">
        <Sidebar />
        <div className="main">
          <TitleBar title={title} />
          <div className="page-scroll">
            <div className="page" key={page}>
              {page === "play" && <PlayPage />}
              {page === "versions" && <VersionsPage />}
              {page === "settings" && <SettingsPage />}
            </div>
          </div>
          <ConsoleDock />
        </div>
        <Toasts />
      </div>
    </I18nProvider>
  );
}
