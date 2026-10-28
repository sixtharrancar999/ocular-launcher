export type Page = "play" | "versions" | "settings";

export interface Settings {
  nickname: string;
  selectedVersion: string | null;
  minecraftDir: string | null;
  javaHome: string | null;
  language: string;
  includeSounds: boolean;
  extraArgs: string;
}

export interface VersionInfo {
  tag: string;
  installed: boolean;
  resourcesReady: boolean;
  minecraftVersion: string;
}

export interface Bootstrap {
  settings: Settings;
  versions: VersionInfo[];
  dataDir: string;
  minecraftDir: string | null;
  java: string | null;
  runningTag: string | null;
  busy: boolean;
  platform: string;
  appVersion: string;
}

export interface Release {
  tag: string;
  name: string;
  publishedAt: string;
  body: string;
  prerelease: boolean;
  size: number;
  downloadUrl: string;
}

export type Operation =
  | { kind: "install"; tag: string; stage: "download" | "extract" | "done"; received: number; total: number }
  | { kind: "prepare"; tag: string; stage: string };

export interface LogLine {
  id: number;
  stream: "stdout" | "stderr" | "prepare" | "launcher";
  text: string;
}

export interface Toast {
  id: number;
  kind: "error" | "success" | "info";
  text: string;
}

export interface SkinInfo {
  width: number;
  height: number;
  modifiedMs: number;
}

export interface Appearance {
  nickname: string;
  directory: string;
  skin: SkinInfo | null;
  cape: SkinInfo | null;
  model: "classic" | "slim" | string;
  needsRestart: boolean;
}
