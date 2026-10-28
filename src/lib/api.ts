import { invoke } from "@tauri-apps/api/core";
import type { Appearance, Bootstrap, Release, Settings } from "./types";

export const api = {
  bootstrap: () => invoke<Bootstrap>("bootstrap"),
  listReleases: (force = false) => invoke<Release[]>("list_releases", { force }),
  installRelease: (tag: string) => invoke<void>("install_release", { tag }),
  uninstallRelease: (tag: string) => invoke<void>("uninstall_release", { tag }),
  prepare: (tag: string) => invoke<void>("prepare", { tag }),
  adoptResources: (to: string) => invoke<string | null>("adopt_resources", { to }),
  launch: () => invoke<void>("launch_game"),
  stop: () => invoke<void>("stop_game"),
  saveSettings: (settings: Settings) => invoke<void>("save_settings", { settings }),
  selectVersion: (tag: string) => invoke<void>("select_version", { tag }),
  reveal: (kind: string, tag?: string) => invoke<void>("reveal", { kind, tag: tag ?? null }),
  openExternal: (url: string) => invoke<void>("open_external", { url }),

  appearanceGet: () => invoke<Appearance>("appearance_get"),
  appearanceSource: (nickname: string) => invoke<string>("appearance_source", { nickname }),
  capeSource: () => invoke<string | null>("cape_source"),
  skinSetFromFile: (path: string) => invoke<void>("skin_set_from_file", { path }),
  skinSetFromBytes: (bytes: number[]) => invoke<void>("skin_set_from_bytes", { bytes }),
  skinModelSet: (model: string) => invoke<void>("skin_model_set", { model }),
  capeSetFromFile: (path: string) => invoke<void>("cape_set_from_file", { path }),
  capeSetFromBytes: (bytes: number[]) => invoke<void>("cape_set_from_bytes", { bytes }),
  capeClear: () => invoke<void>("cape_clear"),
  appearanceReset: () => invoke<void>("appearance_reset"),
  appearanceReveal: () => invoke<void>("appearance_reveal"),
  skinsFolder: () => invoke<void>("skins_folder"),
};
