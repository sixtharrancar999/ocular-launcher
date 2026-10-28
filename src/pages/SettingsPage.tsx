import { useEffect, useState } from "react";
import { useI18n } from "../lib/i18n";
import { api } from "../lib/api";
import { useStore } from "../lib/store";
import { IconFolder } from "../components/Icons";
import type { Settings } from "../lib/types";

export function SettingsPage() {
  const { t } = useI18n();
  const boot = useStore((state) => state.boot);
  const saveSettings = useStore((state) => state.saveSettings);
  const showToast = useStore((state) => state.showToast);
  const [form, setForm] = useState<Settings | null>(null);

  useEffect(() => {
    if (boot && !form) {
      setForm(boot.settings);
    }
  }, [boot, form]);

  if (!form) {
    return <div className="empty">…</div>;
  }

  const patch = (changes: Partial<Settings>) =>
    setForm((current) => (current ? { ...current, ...changes } : current));

  const reveal = (kind: string) => {
    api.reveal(kind).catch((error) => showToast("error", String(error)));
  };

  return (
    <div className="settings-page">
      <div className="page-head">
        <div>
          <h2 className="page-title">{t("settings.title")}</h2>
          <p className="page-sub">{t("settings.subtitle")}</p>
        </div>
      </div>

      <section className="panel">
        <h3 className="panel-title">{t("settings.account")}</h3>
        <div className="field">
          <label className="field-label" htmlFor="nickname">
            {t("settings.nickname")}
          </label>
          <input
            id="nickname"
            className="input"
            value={form.nickname}
            maxLength={16}
            spellCheck={false}
            onChange={(event) => patch({ nickname: event.target.value })}
          />
          <p className="field-hint">{t("settings.nicknameHint")}</p>
        </div>
      </section>

      <section className="panel">
        <h3 className="panel-title">{t("settings.game")}</h3>
        <div className="field">
          <label className="field-label" htmlFor="minecraftDir">
            {t("settings.minecraftDir")}
          </label>
          <div className="input-row">
            <input
              id="minecraftDir"
              className="input"
              value={form.minecraftDir ?? ""}
              placeholder={boot?.minecraftDir ?? ""}
              spellCheck={false}
              onChange={(event) =>
                patch({ minecraftDir: event.target.value.trim() || null })
              }
            />
            <button
              className="btn btn-ghost btn-small"
              onClick={() => patch({ minecraftDir: null })}
            >
              Auto
            </button>
          </div>
          <p className="field-hint">
            {t("settings.minecraftDirHint")} {boot?.minecraftDir ? `Сейчас: ${boot.minecraftDir}` : ""}
          </p>
        </div>

        <div className="field">
          <span className="field-label">{t("settings.java")}</span>
          <div className="readonly">{boot?.java ?? "Java 25 не найдена"}</div>
          <p className="field-hint">{t("settings.javaHint")}</p>
        </div>

        <div className="field field-row">
          <div>
            <span className="field-label">{t("settings.sounds")}</span>
            <p className="field-hint">{t("settings.soundsHint")}</p>
          </div>
          <label className="switch">
            <input
              type="checkbox"
              checked={form.includeSounds}
              onChange={(event) => patch({ includeSounds: event.target.checked })}
            />
            <span className="switch-track" />
          </label>
        </div>

        <div className="field">
          <label className="field-label" htmlFor="extraArgs">
            {t("settings.extraArgs")}
          </label>
          <input
            id="extraArgs"
            className="input"
            value={form.extraArgs}
            placeholder="--render-distance 16"
            spellCheck={false}
            onChange={(event) => patch({ extraArgs: event.target.value })}
          />
          <p className="field-hint">{t("settings.extraArgsHint")}</p>
        </div>
      </section>

      <section className="panel">
        <h3 className="panel-title">{t("settings.launcher")}</h3>
        <div className="field">
          <label className="field-label" htmlFor="language">
            {t("settings.language")}
          </label>
          <select
            id="language"
            className="input select"
            value={form.language}
            onChange={(event) => patch({ language: event.target.value })}
          >
            <option value="system">Системный / System</option>
            <option value="ru">Русский</option>
            <option value="en">English</option>
          </select>
        </div>
      </section>

      <section className="panel">
        <h3 className="panel-title">{t("settings.folders")}</h3>
        <div className="folder-list">
          <FolderRow label={t("settings.folderData")} path={boot?.dataDir} onClick={() => reveal("data")} />
          <FolderRow label={t("settings.folderVersions")} onClick={() => reveal("versions")} />
          <FolderRow label={t("settings.folderGame")} onClick={() => reveal("game")} />
          <FolderRow label={t("settings.folderLogs")} onClick={() => reveal("logs")} />
          <FolderRow label={t("settings.folderMinecraft")} onClick={() => reveal("minecraft")} />
        </div>
      </section>

      <section className="panel">
        <h3 className="panel-title">{t("settings.about")}</h3>
        <div className="about-row">
          <span>{t("settings.version")}</span>
          <span className="about-value">v{boot?.appVersion ?? "0.1.0"}</span>
        </div>
        <p className="field-hint about-note">{t("settings.notAffiliated")}</p>
        <div className="about-links">
          <button
            className="link-button"
            onClick={() => void api.openExternal("https://github.com/horizon-rust/horizon")}
          >
            GitHub · horizon-rust/horizon
          </button>
          <button
            className="link-button"
            onClick={() => void api.openExternal("https://playhorizon.dev")}
          >
            playhorizon.dev
          </button>
        </div>
      </section>

      <div className="save-bar">
        <button className="btn btn-primary" onClick={() => void saveSettings(form)}>
          {t("settings.save")}
        </button>
      </div>
    </div>
  );
}

function FolderRow({
  label,
  path,
  onClick,
}: {
  label: string;
  path?: string;
  onClick: () => void;
}) {
  return (
    <div className="folder-row">
      <div className="folder-text">
        <span className="field-label">{label}</span>
        {path && <span className="folder-path">{path}</span>}
      </div>
      <button className="icon-button" title="Открыть" onClick={onClick}>
        <IconFolder size={16} />
      </button>
    </div>
  );
}
