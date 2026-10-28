import { useEffect, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from "../lib/i18n";
import { api } from "../lib/api";
import { useStore } from "../lib/store";
import { CapePreview, FigurePreview } from "../components/SkinPreview";
import { IconFolder, IconTrash } from "../components/Icons";

const inTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

function useObjectUrl(source: string | null) {
  const [url, setUrl] = useState<string | null>(null);
  useEffect(() => {
    if (!source) {
      setUrl(null);
      return;
    }
    let objectUrl: string | null = null;
    const [, base64 = ""] = source.split(",", 2);
    if (source.startsWith("data:image/png;base64,") && base64) {
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let index = 0; index < binary.length; index += 1) {
        bytes[index] = binary.charCodeAt(index);
      }
      objectUrl = URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
      setUrl(objectUrl);
    } else {
      setUrl(source);
    }
    return () => {
      if (objectUrl) {
        URL.revokeObjectURL(objectUrl);
      }
    };
  }, [source]);
  return url;
}

export function SkinSection() {
  const { t } = useI18n();
  const appearance = useStore((state) => state.appearance);
  const skinSource = useStore((state) => state.appearanceSource);
  const capeSource = useStore((state) => state.appearanceCapeSource);
  const busy = useStore((state) => state.appearanceBusy);
  const updateAppearance = useStore((state) => state.updateAppearance);
  const [dragging, setDragging] = useState<"skin" | "cape" | null>(null);
  const skinInput = useRef<HTMLInputElement>(null);
  const capeInput = useRef<HTMLInputElement>(null);

  const skinUrl = useObjectUrl(skinSource);
  const capeUrl = useObjectUrl(capeSource);

  const pickSkin = async () => {
    if (!inTauri) {
      skinInput.current?.click();
      return;
    }
    const picked = await open({
      multiple: false,
      filters: [{ name: "PNG", extensions: ["png"] }],
    });
    if (typeof picked === "string") {
      await updateAppearance(
        (bridge) => bridge.skinSetFromFile(picked),
        t("skin.saved"),
      );
    }
  };

  const pickCape = async () => {
    if (!inTauri) {
      capeInput.current?.click();
      return;
    }
    const picked = await open({
      multiple: false,
      filters: [{ name: "PNG", extensions: ["png"] }],
    });
    if (typeof picked === "string") {
      await updateAppearance(
        (bridge) => bridge.capeSetFromFile(picked),
        t("skin.capeSaved"),
      );
    }
  };

  const readDropped = async (file: File) =>
    Array.from(new Uint8Array(await file.arrayBuffer()));

  return (
    <section className="panel skin-panel">
      <h3 className="panel-title">{t("skin.title")}</h3>
      <div className="skin-layout">
        <div
          className={`skin-stage ${dragging === "skin" ? "dragging" : ""}`}
          onDragOver={(event) => {
            event.preventDefault();
            setDragging("skin");
          }}
          onDragLeave={() => setDragging(null)}
          onDrop={(event) => {
            event.preventDefault();
            setDragging(null);
            const file = event.dataTransfer.files[0];
            if (!file) {
              return;
            }
            void (async () =>
              updateAppearance(
                async (bridge) =>
                  bridge.skinSetFromBytes(await readDropped(file)),
                t("skin.saved"),
              ))();
          }}
        >
          {skinUrl ? (
            <FigurePreview src={skinUrl} slim={appearance?.model === "slim"} />
          ) : (
            <div className="skin-empty">
              <span className="skin-empty-mark">?</span>
              <span>{t("skin.none")}</span>
            </div>
          )}
          {appearance?.needsRestart && (
            <span className="skin-restart">{t("skin.restartHint")}</span>
          )}
        </div>

        <div className="skin-controls">
          <div className="field">
            <span className="field-label">{t("skin.nick")}</span>
            <span className="readonly">{appearance?.nickname ?? "—"}</span>
            <p className="field-hint">{t("skin.nickHint")}</p>
          </div>

          <div className="field">
            <span className="field-label">{t("skin.model")}</span>
            <div className="segmented">
              {(["classic", "slim"] as const).map((model) => (
                <button
                  key={model}
                  className={`segment ${appearance?.model === model ? "active" : ""}`}
                  disabled={busy || !appearance?.skin}
                  onClick={() =>
                    void updateAppearance(
                      (bridge) => bridge.skinModelSet(model),
                      t("skin.modelSaved"),
                    )
                  }
                >
                  {model === "classic" ? t("skin.classic") : t("skin.slim")}
                </button>
              ))}
            </div>
          </div>

          <div className="field">
            <div className="skin-buttons">
              <button
                className="btn btn-primary btn-small"
                onClick={() => void pickSkin()}
                disabled={busy}
              >
                {appearance?.skin ? t("skin.replace") : t("skin.upload")}
              </button>
              {appearance?.skin && (
                <button
                  className="btn btn-ghost btn-small"
                  onClick={() =>
                    void updateAppearance(
                      (bridge) => bridge.appearanceReset(),
                      t("skin.removed"),
                    )
                  }
                  disabled={busy}
                >
                  <IconTrash size={15} />
                  {t("skin.remove")}
                </button>
              )}
            </div>
            <p className="field-hint">{t("skin.uploadHint")}</p>
          </div>

          <div
            className={`field cape-field ${dragging === "cape" ? "dragging" : ""}`}
            onDragOver={(event) => {
              event.preventDefault();
              setDragging("cape");
            }}
            onDragLeave={() => setDragging(null)}
            onDrop={(event) => {
              event.preventDefault();
              setDragging(null);
              const file = event.dataTransfer.files[0];
              if (!file) {
                return;
              }
              void (async () =>
                updateAppearance(
                  async (bridge) =>
                    bridge.capeSetFromBytes(await readDropped(file)),
                  t("skin.capeSaved"),
                ))();
            }}
          >
            <span className="field-label">{t("skin.cape")}</span>
            <div className="cape-row">
              {capeUrl ? (
                <CapePreview src={capeUrl} />
              ) : (
                <span className="cape-empty">{t("skin.capeNone")}</span>
              )}
              <div className="cape-buttons">
                <button
                  className="btn btn-ghost btn-small"
                  onClick={() => void pickCape()}
                  disabled={busy}
                >
                  {capeUrl ? t("skin.capeReplace") : t("skin.capeUpload")}
                </button>
                {capeUrl && (
                  <button
                    className="icon-button danger"
                    title={t("skin.capeRemove")}
                    onClick={() =>
                      void updateAppearance(
                        (bridge) => bridge.capeClear(),
                        t("skin.capeRemoved"),
                      )
                    }
                    disabled={busy}
                  >
                    <IconTrash size={15} />
                  </button>
                )}
              </div>
            </div>
            <p className="field-hint">{t("skin.capeHint")}</p>
          </div>

          <div className="skin-footer">
            <button className="link-button" onClick={() => void api.skinsFolder()}>
              <IconFolder size={14} /> {t("skin.openFolder")}
            </button>
            <button
              className="link-button"
              onClick={() => void api.appearanceReveal()}
            >
              {t("skin.openNickFolder")}
            </button>
          </div>
        </div>
      </div>

      <input
        ref={skinInput}
        type="file"
        accept="image/png"
        hidden
        onChange={(event) => {
          const file = event.target.files?.[0];
          event.target.value = "";
          if (!file) {
            return;
          }
          void (async () =>
            updateAppearance(
              async (bridge) => bridge.skinSetFromBytes(await readDropped(file)),
              t("skin.saved"),
            ))();
        }}
      />
      <input
        ref={capeInput}
        type="file"
        accept="image/png"
        hidden
        onChange={(event) => {
          const file = event.target.files?.[0];
          event.target.value = "";
          if (!file) {
            return;
          }
          void (async () =>
            updateAppearance(
              async (bridge) => bridge.capeSetFromBytes(await readDropped(file)),
              t("skin.capeSaved"),
            ))();
        }}
      />
    </section>
  );
}
