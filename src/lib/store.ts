import { create } from "zustand";
import { api } from "./api";
import { translate, resolveLang } from "./i18n";
import type {
  Bootstrap,
  LogLine,
  Operation,
  Page,
  Release,
  Settings,
  Toast,
} from "./types";

let nextLogId = 1;
let nextToastId = 1;

interface Store {
  boot: Bootstrap | null;
  page: Page;
  releases: Release[];
  releasesLoading: boolean;
  releasesError: string | null;
  op: Operation | null;
  logs: LogLine[];
  consoleOpen: boolean;
  toast: Toast | null;

  setPage: (page: Page) => void;
  setConsoleOpen: (open: boolean) => void;
  init: () => Promise<void>;
  refresh: () => Promise<void>;
  loadReleases: (force?: boolean) => Promise<void>;
  install: (tag?: string) => Promise<void>;
  prepare: (tag: string, playAfter?: boolean) => Promise<void>;
  play: () => Promise<void>;
  stop: () => Promise<void>;
  remove: (tag: string) => Promise<void>;
  select: (tag: string) => Promise<void>;
  saveSettings: (settings: Settings) => Promise<void>;
  pushLog: (stream: LogLine["stream"], text: string) => void;
  clearLogs: () => void;
  showToast: (kind: Toast["kind"], text: string) => void;
  dismissToast: () => void;
}

export const useStore = create<Store>((set, get) => ({
  boot: null,
  page: "play",
  releases: [],
  releasesLoading: false,
  releasesError: null,
  op: null,
  logs: [],
  consoleOpen: false,
  toast: null,

  setPage: (page) => set({ page }),
  setConsoleOpen: (consoleOpen) => set({ consoleOpen }),

  init: async () => {
    await get().refresh();
    void get().loadReleases(false);
  },

  refresh: async () => {
    try {
      const boot = await api.bootstrap();
      set({ boot });
    } catch (error) {
      get().pushLog("launcher", String(error));
    }
  },

  loadReleases: async (force = false) => {
    set({ releasesLoading: true, releasesError: null });
    try {
      const releases = await api.listReleases(force);
      set({ releases, releasesLoading: false });
    } catch (error) {
      set({ releasesLoading: false, releasesError: String(error) });
    }
  },

  install: async (tag) => {
    const first = get().releases[0];
    const target = tag ?? first?.tag;
    if (!target) {
      get().showToast("error", "Нет доступных релизов");
      return;
    }
    set({
      op: { kind: "install", tag: target, stage: "download", received: 0, total: 0 },
      consoleOpen: true,
    });
    try {
      await api.installRelease(target);
      await get().refresh();
      get().showToast("success", translate(currentLang(get()), "toast.installed", { tag: target }));
    } catch (error) {
      get().showToast("error", String(error));
      get().pushLog("launcher", String(error));
    } finally {
      set({ op: null });
    }
  },

  prepare: async (tag, playAfter = false) => {
    set({ op: { kind: "prepare", tag, stage: "server-jar" }, consoleOpen: true });
    try {
      await api.prepare(tag);
      await get().refresh();
      get().showToast("success", translate(currentLang(get()), "toast.prepared"));
      if (playAfter) {
        await get().play();
      }
    } catch (error) {
      get().showToast("error", String(error));
      get().pushLog("launcher", String(error));
    } finally {
      set({ op: null });
      await get().refresh();
    }
  },

  play: async () => {
    set({ op: null });
    try {
      await api.launch();
      get().showToast("success", translate(currentLang(get()), "toast.launched"));
      await get().refresh();
    } catch (error) {
      get().showToast("error", String(error));
      get().pushLog("launcher", String(error));
    }
  },

  stop: async () => {
    set({ op: null });
    try {
      await api.stop();
      get().showToast("info", translate(currentLang(get()), "toast.stopped"));
      await get().refresh();
    } catch (error) {
      get().showToast("error", String(error));
    }
  },

  remove: async (tag) => {
    try {
      await api.uninstallRelease(tag);
      await get().refresh();
      get().showToast("info", translate(currentLang(get()), "toast.removed", { tag }));
    } catch (error) {
      get().showToast("error", String(error));
    }
  },

  select: async (tag) => {
    try {
      await api.selectVersion(tag);
      await get().refresh();
      get().showToast("success", translate(currentLang(get()), "toast.selected", { tag }));
    } catch (error) {
      get().showToast("error", String(error));
    }
  },

  saveSettings: async (settings) => {
    try {
      await api.saveSettings(settings);
      await get().refresh();
      get().showToast("success", translate(currentLang(get()), "settings.saved"));
    } catch (error) {
      get().showToast("error", String(error));
    }
  },

  pushLog: (stream, text) =>
    set((state) => {
      const logs = [...state.logs, { id: nextLogId++, stream, text }];
      if (logs.length > 3000) {
        logs.splice(0, logs.length - 3000);
      }
      return { logs };
    }),

  clearLogs: () => set({ logs: [] }),

  showToast: (kind, text) =>
    set({ toast: { id: nextToastId++, kind, text } }),

  dismissToast: () => set({ toast: null }),
}));

function currentLang(state: Store) {
  return resolveLang(state.boot?.settings.language ?? "system");
}

/** The version Play would start: the selected one when installed, else the newest. */
export function activeVersion(boot: Bootstrap | null) {
  if (!boot) {
    return null;
  }
  const selected = boot.settings.selectedVersion;
  if (selected) {
    const found = boot.versions.find((version) => version.tag === selected);
    if (found) {
      return found;
    }
  }
  return boot.versions[0] ?? null;
}
