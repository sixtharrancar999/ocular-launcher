import type { ReactNode } from "react";
import { useI18n } from "../lib/i18n";
import { activeVersion, useStore } from "../lib/store";
import type { Page } from "../lib/types";
import { IconLayers, IconPlay, IconSettings, OcularMark } from "./Icons";

export function Sidebar() {
  const { t } = useI18n();
  const page = useStore((state) => state.page);
  const setPage = useStore((state) => state.setPage);
  const boot = useStore((state) => state.boot);

  const version = activeVersion(boot);
  const running = Boolean(boot?.runningTag);

  const items: Array<{ id: Page; label: string; icon: ReactNode }> = [
    { id: "play", label: t("nav.play"), icon: <IconPlay size={17} /> },
    { id: "versions", label: t("nav.versions"), icon: <IconLayers size={17} /> },
    { id: "settings", label: t("nav.settings"), icon: <IconSettings size={17} /> },
  ];

  return (
    <aside className="sidebar">
      <div className="brand">
        <OcularMark size={36} className="brand-mark" />
        <div className="brand-text">
          <div className="brand-name">OCULAR</div>
          <div className="brand-sub">{t("brand.tagline")}</div>
        </div>
      </div>

      <nav className="nav">
        {items.map((item) => (
          <button
            key={item.id}
            className={`nav-item ${page === item.id ? "active" : ""}`}
            onClick={() => setPage(item.id)}
          >
            <span className="nav-icon">{item.icon}</span>
            <span>{item.label}</span>
          </button>
        ))}
      </nav>

      <div className="sidebar-spacer" />

      <button className="sidebar-card" onClick={() => setPage("settings")}>
        <span className="avatar">
          {(boot?.settings.nickname ?? "?").slice(0, 1).toUpperCase()}
        </span>
        <span className="sidebar-card-text">
          <span className="sidebar-card-name">{boot?.settings.nickname ?? "…"}</span>
          <span className="sidebar-card-sub">{t("label.offline")}</span>
        </span>
        <span className="chevron">›</span>
      </button>

      <div className="sidebar-footer">
        <span
          className={`status-dot ${running ? "running" : version?.resourcesReady ? "ready" : "idle"}`}
        />
        <span className="sidebar-version">
          {version ? `${version.tag}` : t("status.notInstalled")}
        </span>
        <span className="sidebar-app">v{boot?.appVersion ?? "0.1.0"}</span>
      </div>
    </aside>
  );
}
