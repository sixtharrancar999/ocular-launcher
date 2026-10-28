import { invoke } from "@tauri-apps/api/core";
import type { Bootstrap, Release, Settings } from "./types";

export const api = {
  bootstrap: () => invoke<Bootstrap>("bootstrap"),
  listReleases: (force = false) => invoke<Release[]>("list_releases", { force }),
  installRelease: (tag: string) => invoke<void>("install_release", { tag }),
  uninstallRelease: (tag: string) => invoke<void>("uninstall_release", { tag }),
  prepare: (tag: string) => invoke<void>("prepare", { tag }),
  launch: () => invoke<void>("launch_game"),
  stop: () => invoke<void>("stop_game"),
  saveSettings: (settings: Settings) => invoke<void>("save_settings", { settings }),
  selectVersion: (tag: string) => invoke<void>("select_version", { tag }),
  reveal: (kind: string, tag?: string) => invoke<void>("reveal", { kind, tag: tag ?? null }),
  openExternal: (url: string) => invoke<void>("open_external", { url }),
};
