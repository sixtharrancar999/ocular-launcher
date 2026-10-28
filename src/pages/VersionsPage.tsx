import { useI18n, translate } from "../lib/i18n";
import { useStore } from "../lib/store";
import { IconDownload, IconRefresh, IconTrash } from "../components/Icons";
import { Progress, formatBytes } from "../components/Progress";
import { excerpt } from "./PlayPage";
import type { Release } from "../lib/types";

export function VersionsPage() {
  const { t, lang } = useI18n();
  const boot = useStore((state) => state.boot);
  const releases = useStore((state) => state.releases);
  const loading = useStore((state) => state.releasesLoading);
  const error = useStore((state) => state.releasesError);
  const op = useStore((state) => state.op);
  const loadReleases = useStore((state) => state.loadReleases);
  const install = useStore((state) => state.install);
  const remove = useStore((state) => state.remove);
  const select = useStore((state) => state.select);
  const prepare = useStore((state) => state.prepare);
  const knownGameTags = useStore((state) => state.knownGameTags);

  // Ocular tags its releases `vX.Y.Z` like the game; a tag never seen from Horizon before and
  // missing its asset is the launcher's own release, not a game version.
  const isLauncherRelease = (release: Release) =>
    knownGameTags.length > 0 && !knownGameTags.includes(release.tag) && release.size === 0;
  const shown = releases.filter((release) => !isLauncherRelease(release));

  const versions = boot?.versions ?? [];
  const selected = boot?.settings.selectedVersion;
  const busy = op !== null;

  const isSelected = (release: Release) =>
    selected === release.tag || (!selected && versions[0]?.tag === release.tag);

  const onRemove = (release: Release) => {
    const text = translate(lang, "versions.removeConfirm", { tag: release.tag });
    if (window.confirm(text)) {
      void remove(release.tag);
    }
  };

  return (
    <div className="versions-page">
      <div className="page-head">
        <div>
          <h2 className="page-title">{t("versions.title")}</h2>
          <p className="page-sub">{t("versions.subtitle")}</p>
        </div>
        <button
          className="btn btn-ghost btn-small"
          onClick={() => void loadReleases(true)}
          disabled={loading}
        >
          <IconRefresh size={16} className={loading ? "spin" : ""} />
          {t("action.refresh")}
        </button>
      </div>

      {error && (
        <div className="notice notice-error">
          <span>{t("versions.error")}</span>
          <code>{error}</code>
          <button className="link-button" onClick={() => void loadReleases(true)}>
            {t("versions.retry")}
          </button>
        </div>
      )}

      {shown.length === 0 && !error && (
        <div className="empty">{loading ? t("action.refresh") + "…" : t("versions.empty")}</div>
      )}

      <div className="release-list">
        {shown.map((release) => {
          const installed = versions.find((version) => version.tag === release.tag);
          const installing = op?.kind === "install" && op.tag === release.tag;
          const preparing = op?.kind === "prepare" && op.tag === release.tag;
          const latest = shown[0]?.tag === release.tag;

          return (
            <article
              key={release.tag}
              className={`release ${installed ? "installed" : ""} ${isSelected(release) ? "selected" : ""}`}
            >
              <div className="release-main">
                <div className="release-head">
                  <span className="release-tag">{release.tag}</span>
                  {latest && <span className="badge badge-accent">{t("label.latest")}</span>}
                  {installed && <span className="badge">{t("label.installed")}</span>}
                  {installed?.resourcesReady && (
                    <span className="badge badge-ok">{t("label.ready")}</span>
                  )}
                  {isSelected(release) && installed && (
                    <span className="badge badge-selected">{t("action.selected")}</span>
                  )}
                </div>
                <div className="release-meta">
                  <span>{formatDate(release.publishedAt, lang)}</span>
                  {release.size > 0 && (
                    <span>
                      {t("label.size")}: {formatBytes(release.size)}
                    </span>
                  )}
                </div>
                <p className="release-body">{excerpt(release.body) || "—"}</p>

                {installing && (
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
                )}
                {preparing && <Progress label="horizon-prepare" indeterminate value={0} />}
              </div>

              <div className="release-actions">
                {!installed && (
                  <button
                    className="btn btn-primary btn-small"
                    onClick={() => void install(release.tag)}
                    disabled={busy}
                  >
                    <IconDownload size={16} />
                    {t("action.install")}
                  </button>
                )}
                {installed && !isSelected(release) && (
                  <button
                    className="btn btn-ghost btn-small"
                    onClick={() => void select(release.tag)}
                    disabled={busy}
                  >
                    {t("action.select")}
                  </button>
                )}
                {installed && !installed.resourcesReady && (
                  <button
                    className="btn btn-ghost btn-small"
                    onClick={() => void prepare(release.tag, false)}
                    disabled={busy}
                  >
                    {t("action.prepare")}
                  </button>
                )}
                {installed && (
                  <button
                    className="icon-button danger"
                    title={t("action.remove")}
                    onClick={() => onRemove(release)}
                    disabled={busy}
                  >
                    <IconTrash size={16} />
                  </button>
                )}
              </div>
            </article>
          );
        })}
      </div>
    </div>
  );
}

function formatDate(iso: string, lang: string): string {
  if (!iso) {
    return "—";
  }
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return "—";
  }
  return date.toLocaleDateString(lang === "ru" ? "ru-RU" : "en-US", {
    day: "numeric",
    month: "short",
    year: "numeric",
  });
}
