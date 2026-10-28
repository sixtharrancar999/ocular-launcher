import type { ReactNode } from "react";
import { useI18n } from "../lib/i18n";
import { api } from "../lib/api";
import { activeVersion, useStore } from "../lib/store";
import { Iris } from "../components/Iris";
import { Progress, formatBytes } from "../components/Progress";
import {
  IconDownload,
  IconFolder,
  IconLayers,
  IconPlay,
  IconSettings,
  IconStop,
  OcularMark,
} from "../components/Icons";

export function PlayPage() {
  const { t } = useI18n();
  const boot = useStore((state) => state.boot);
  const op = useStore((state) => state.op);
  const releases = useStore((state) => state.releases);
  const releasesLoading = useStore((state) => state.releasesLoading);
  const releasesError = useStore((state) => state.releasesError);
  const install = useStore((state) => state.install);
  const prepare = useStore((state) => state.prepare);
  const play = useStore((state) => state.play);
  const stop = useStore((state) => state.stop);
  const setPage = useStore((state) => state.setPage);
  const showToast = useStore((state) => state.showToast);

  const version = activeVersion(boot);
  const running = Boolean(boot?.runningTag);
  const busy = op !== null;
  const latest = releases[0];

  const primaryDisabled = running
    ? false
    : busy || (!version && (releasesLoading || !latest));

  const primaryLabel = running
    ? t("action.stop")
    : op?.kind === "install"
      ? t("action.installing")
      : op?.kind === "prepare"
        ? t("action.preparing")
        : !version
          ? t("action.installLatest")
          : !version.resourcesReady
            ? t("action.prepareAndPlay")
            : t("action.play");

  const primaryIcon = running ? (
    <IconStop size={20} />
  ) : !version || !version.resourcesReady ? (
    <IconDownload size={20} />
  ) : (
    <IconPlay size={20} />
  );

  const status = running
    ? t("status.running")
    : op
      ? op.kind === "install"
        ? t("status.installing")
        : t("status.preparing")
      : !version
        ? t("status.notInstalled")
        : !version.resourcesReady
          ? t("status.notPrepared")
          : t("status.ready");

  const hint = running
    ? t("hero.hint.running")
    : busy
      ? t("hero.hint.busy")
      : !version
        ? releasesError
          ? t("hero.hint.noReleases")
          : t("hero.hint.install")
        : !version.resourcesReady
          ? t("hero.hint.prepare")
          : t("hero.hint.ready");

  const onClick = () => {
    if (running) {
      void stop();
      return;
    }
    if (busy) {
      return;
    }
    if (!version) {
      void install(latest?.tag);
      return;
    }
    if (!version.resourcesReady) {
      void prepare(version.tag, true);
      return;
    }
    void play();
  };

  const reveal = (kind: string, tag?: string) => {
    api.reveal(kind, tag).catch((error) => showToast("error", String(error)));
  };

  const separator = boot?.platform === "windows" ? "\\" : "/";
  const gameFolder = boot ? `${boot.dataDir}${separator}game` : "";

  return (
    <div className="play-page">
      <section className="hero">
        <Iris className="hero-iris" />
        <div className="hero-content">
          <div className="eyebrow">{t("hero.eyebrow")}</div>
          <h1 className="hero-title">Horizon</h1>
          <p className="hero-sub">{t("hero.subtitle")}</p>

          <div className="hero-actions">
            <button
              className={`btn btn-primary btn-play ${running ? "btn-danger" : ""}`}
              onClick={onClick}
              disabled={primaryDisabled}
            >
              {primaryIcon}
              <span>{primaryLabel}</span>
            </button>
            <button className="btn btn-ghost" onClick={() => setPage("versions")}>
              <IconLayers size={17} />
              {t("nav.versions")}
            </button>
          </div>

          <div className={`status-line ${running ? "running" : version?.resourcesReady ? "ready" : ""}`}>
            <span className="status-dot" />
            {status}
          </div>

          {op &&
            (op.kind === "install" ? (
              <Progress
                label={op.stage === "extract" ? "Распаковка" : "Загрузка"}
                detail={
                  op.stage === "download" && op.total
                    ? `${formatBytes(op.received)} / ${formatBytes(op.total)}`
                    : undefined
                }
                value={op.total ? (op.received / op.total) * 100 : 0}
                indeterminate={op.stage !== "download"}
              />
            ) : (
              <Progress label={prepareStageLabel(op.stage)} indeterminate={op.stage !== "done"} value={0} />
            ))}

          <p className="hero-hint">{hint}</p>
        </div>

        {version && (
          <div className="hero-badge">
            <OcularMark size={64} />
            <div className="hero-badge-text">
              <span>{version.tag}</span>
              <span className="hero-badge-sub">Minecraft {version.minecraftVersion}</span>
            </div>
          </div>
        )}
      </section>

      <div className="info-grid">
        <InfoCard
          icon={<IconLayers size={18} />}
          label={t("label.version")}
          value={version?.tag ?? "—"}
          sub={
            version
              ? `Minecraft ${version.minecraftVersion} · ${version.resourcesReady ? t("label.ready") : t("label.notReady")}`
              : t("status.notInstalled")
          }
          action={t("action.open")}
          onAction={() => version && reveal("version", version.tag)}
          disabled={!version}
        />
        <InfoCard
          icon={<IconSettings size={18} />}
          label={t("label.account")}
          value={boot?.settings.nickname ?? "…"}
          sub={t("label.offline")}
          action={t("nav.settings")}
          onAction={() => setPage("settings")}
        />
        <InfoCard
          icon={<IconFolder size={18} />}
          label={t("label.gameFolder")}
          value={t("play.gameData")}
          sub={gameFolder}
          action={t("action.open")}
          onAction={() => reveal("game")}
        />
      </div>

      {latest && (
        <section className="news-card">
          <div className="news-head">
            <span className="news-title">
              {t("versions.title")} <span className="news-tag">{latest.tag}</span>
            </span>
            <button className="link-button" onClick={() => setPage("versions")}>
              {t("action.open")}
            </button>
          </div>
          <p className="news-body">{excerpt(latest.body) || "—"}</p>
        </section>
      )}
    </div>
  );
}

function InfoCard({
  icon,
  label,
  value,
  sub,
  action,
  onAction,
  disabled,
}: {
  icon: ReactNode;
  label: string;
  value: string;
  sub?: string;
  action: string;
  onAction: () => void;
  disabled?: boolean;
}) {
  return (
    <div className="info-card">
      <div className="info-card-top">
        <span className="info-icon">{icon}</span>
        <span className="info-label">{label}</span>
      </div>
      <div className="info-value" title={value}>
        {value}
      </div>
      {sub && <div className="info-sub" title={sub}>{sub}</div>}
      <button className="info-action" onClick={onAction} disabled={disabled}>
        {action}
      </button>
    </div>
  );
}

export function prepareStageLabel(stage: string): string {
  switch (stage) {
    case "server-jar":
      return "server.jar Minecraft";
    case "mirror":
      return "Зеркало Minecraft";
    case "prepare":
      return "horizon-prepare";
    case "sounds":
      return "Звуки";
    case "done":
      return "Готово";
    default:
      return stage;
  }
}

export function excerpt(body: string, limit = 240): string {
  // Release notes come from GitHub as Markdown with embedded HTML.
  const html = new DOMParser().parseFromString(body, "text/html");
  const plain = html.body.textContent ?? body;
  const text = plain
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("#") && !line.startsWith("<!--"))
    .join(" ")
    .replace(/[*_`>#]/g, "")
    .replace(/\s+/g, " ")
    .trim();
  return text.length > limit ? `${text.slice(0, limit - 1)}…` : text;
}
