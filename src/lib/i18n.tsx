import { createContext, useContext, useMemo, type ReactNode } from "react";

const ru = {
  "nav.play": "Играть",
  "nav.versions": "Версии",
  "nav.settings": "Настройки",
  "brand.tagline": "лаунчер Horizon",

  "status.ready": "Готово к запуску",
  "status.running": "Игра запущена",
  "status.notInstalled": "Horizon не установлен",
  "status.notPrepared": "Нужна подготовка",
  "status.installing": "Установка…",
  "status.preparing": "Подготовка ресурсов…",

  "action.play": "Играть",
  "action.stop": "Остановить",
  "action.install": "Установить",
  "action.installLatest": "Установить Horizon",
  "action.prepare": "Подготовить",
  "action.prepareAndPlay": "Подготовить и играть",
  "action.installing": "Установка…",
  "action.preparing": "Подготовка…",
  "action.remove": "Удалить",
  "action.select": "Выбрать",
  "action.selected": "Выбрана",
  "action.refresh": "Обновить",
  "action.open": "Открыть",
  "action.copy": "Копировать",
  "action.clear": "Очистить",

  "label.version": "Версия",
  "label.minecraft": "Minecraft",
  "label.account": "Аккаунт",
  "label.offline": "оффлайн",
  "label.gameFolder": "Папка игры",
  "label.size": "Размер",
  "label.published": "Опубликована",
  "label.latest": "последняя",
  "label.installed": "установлена",
  "label.ready": "ресурсы готовы",
  "label.notReady": "без ресурсов",
  "play.gameData": "Миры и настройки",

  "hero.eyebrow": "MINECRAFT: JAVA EDITION 26.3 · НАТИВНЫЙ RUST",
  "hero.subtitle":
    "Тот самый мир, собранный заново на Rust: ванильная генерация, шейдеры и дальность прорисовки до 1024 чанков.",
  "hero.hint.install": "Скачаем последний релиз с GitHub — около 32 МБ.",
  "hero.hint.prepare":
    "Первый запуск: ресурсы собираются из вашей копии Minecraft 26.3. Нужны официальный лаунчер и Java 25.",
  "hero.hint.ready": "Всё готово. Приятной игры!",
  "hero.hint.running": "Лаунчер можно свернуть — игра продолжится.",
  "hero.hint.noReleases": "Не удалось получить список релизов. Проверьте интернет и обновите.",
  "hero.hint.busy": "Дождитесь завершения операции.",
  "hero.hint.needVersion": "Установите версию Horizon, чтобы начать.",
  "update.title": "Доступно обновление",
  "update.action": "Обновить",

  "settings.title": "Настройки",
  "settings.subtitle": "Лаунчер, аккаунт и папки.",
  "settings.account": "Аккаунт",
  "settings.nickname": "Никнейм",
  "settings.nicknameHint":
    "Оффлайн-имя для игры: 3–16 латинских букв, цифр или _. Вход Microsoft добавим позже.",
  "settings.game": "Игра",
  "settings.minecraftDir": "Папка Minecraft",
  "settings.minecraftDirHint":
    "Откуда берутся клиент, библиотеки и ассеты. По умолчанию — официальный лаунчер.",
  "settings.java": "Java для подготовки",
  "settings.javaHint": "Нужна Java 25; лаунчер находит её сам.",
  "settings.sounds": "Копировать звуки при подготовке",
  "settings.soundsHint":
    "Около 380 МБ из asset store Minecraft. Без них игра будет беззвучной.",
  "settings.extraArgs": "Дополнительные аргументы игры",
  "settings.extraArgsHint": "Например: --render-distance 16",
  "settings.launcher": "Лаунчер",
  "settings.language": "Язык",
  "settings.folders": "Папки",
  "settings.folderData": "Данные лаунчера",
  "settings.folderVersions": "Версии",
  "settings.folderGame": "Мир и настройки игры",
  "settings.folderLogs": "Логи",
  "settings.folderMinecraft": "Зеркало Minecraft",
  "settings.save": "Сохранить",
  "settings.saved": "Настройки сохранены",
  "settings.about": "О лаунчере",
  "settings.version": "Версия лаунчера",
  "settings.notAffiliated":
    "Неофициальный проект. Не связан с Mojang или Microsoft. Для игры нужна своя копия Minecraft: Java Edition.",

  "versions.title": "Версии",
  "versions.subtitle":
    "Релизы Horizon с GitHub. Установленная версия запускается кнопкой «Играть».",
  "versions.empty": "Релизов пока нет.",
  "versions.error": "Не удалось загрузить релизы.",
  "versions.retry": "Повторить",
  "versions.removeConfirm": "Удалить {tag}? Мир и настройки игры останутся.",

  "console.title": "Консоль",
  "console.empty": "Здесь появится вывод подготовки и игры.",
  "console.hint": "horizon-prepare и игра",

  "toast.installed": "{tag} установлена",
  "toast.prepared": "Ресурсы готовы",
  "toast.launched": "Игра запускается",
  "toast.stopped": "Игра остановлена",
  "toast.exited": "Игра завершилась (код {code})",
  "toast.exitedOk": "Игра закрыта",
  "toast.removed": "{tag} удалена",
  "toast.selected": "Выбрана {tag}",
  "toast.updated": "Обновлено до {tag}",
  "toast.copied": "Скопировано",
} as const;

type Key = keyof typeof ru;

const en: Record<Key, string> = {
  "nav.play": "Play",
  "nav.versions": "Versions",
  "nav.settings": "Settings",
  "brand.tagline": "Horizon launcher",

  "status.ready": "Ready to play",
  "status.running": "Game running",
  "status.notInstalled": "Horizon is not installed",
  "status.notPrepared": "Resources need preparing",
  "status.installing": "Installing…",
  "status.preparing": "Preparing resources…",

  "action.play": "Play",
  "action.stop": "Stop",
  "action.install": "Install",
  "action.installLatest": "Install Horizon",
  "action.prepare": "Prepare",
  "action.prepareAndPlay": "Prepare and play",
  "action.installing": "Installing…",
  "action.preparing": "Preparing…",
  "action.remove": "Remove",
  "action.select": "Select",
  "action.selected": "Selected",
  "action.refresh": "Refresh",
  "action.open": "Open",
  "action.copy": "Copy",
  "action.clear": "Clear",

  "label.version": "Version",
  "label.minecraft": "Minecraft",
  "label.account": "Account",
  "label.offline": "offline",
  "label.gameFolder": "Game folder",
  "label.size": "Size",
  "label.published": "Published",
  "label.latest": "latest",
  "label.installed": "installed",
  "label.ready": "resources ready",
  "label.notReady": "no resources",
  "play.gameData": "Worlds & settings",

  "hero.eyebrow": "MINECRAFT: JAVA EDITION 26.3 · NATIVE RUST",
  "hero.subtitle":
    "The same world, rebuilt in Rust: vanilla generation, shaders, and a view distance of up to 1024 chunks.",
  "hero.hint.install": "We will download the latest GitHub release — about 32 MB.",
  "hero.hint.prepare":
    "First run: resources are built from your own Minecraft 26.3 copy. Needs the official launcher and Java 25.",
  "hero.hint.ready": "Everything is ready. Have fun!",
  "hero.hint.running": "You can minimise the launcher — the game keeps running.",
  "hero.hint.noReleases": "Could not load releases. Check your connection and refresh.",
  "hero.hint.busy": "Wait for the current operation to finish.",
  "hero.hint.needVersion": "Install a Horizon version to begin.",
  "update.title": "Update available",
  "update.action": "Update",

  "settings.title": "Settings",
  "settings.subtitle": "Launcher, account and folders.",
  "settings.account": "Account",
  "settings.nickname": "Nickname",
  "settings.nicknameHint":
    "The offline name the game plays as: 3–16 letters, digits or _. Microsoft sign-in comes later.",
  "settings.game": "Game",
  "settings.minecraftDir": "Minecraft folder",
  "settings.minecraftDirHint":
    "Where the client, libraries and assets come from. Defaults to the official launcher.",
  "settings.java": "Java for preparing",
  "settings.javaHint": "Java 25 is required; the launcher finds it automatically.",
  "settings.sounds": "Copy sounds while preparing",
  "settings.soundsHint":
    "About 380 MB from Minecraft's asset store. Without it the game is silent.",
  "settings.extraArgs": "Extra game arguments",
  "settings.extraArgsHint": "For example: --render-distance 16",
  "settings.launcher": "Launcher",
  "settings.language": "Language",
  "settings.folders": "Folders",
  "settings.folderData": "Launcher data",
  "settings.folderVersions": "Versions",
  "settings.folderGame": "Game saves and options",
  "settings.folderLogs": "Logs",
  "settings.folderMinecraft": "Minecraft mirror",
  "settings.save": "Save",
  "settings.saved": "Settings saved",
  "settings.about": "About",
  "settings.version": "Launcher version",
  "settings.notAffiliated":
    "An unofficial project. Not affiliated with Mojang or Microsoft. Playing needs your own copy of Minecraft: Java Edition.",

  "versions.title": "Versions",
  "versions.subtitle":
    "Horizon releases from GitHub. The installed version starts with Play.",
  "versions.empty": "No releases yet.",
  "versions.error": "Could not load releases.",
  "versions.retry": "Retry",
  "versions.removeConfirm": "Remove {tag}? Your worlds and options stay.",

  "console.title": "Console",
  "console.empty": "Preparation and game output appears here.",
  "console.hint": "horizon-prepare and the game",

  "toast.installed": "{tag} installed",
  "toast.prepared": "Resources ready",
  "toast.launched": "Starting the game",
  "toast.stopped": "Game stopped",
  "toast.exited": "Game exited (code {code})",
  "toast.exitedOk": "Game closed",
  "toast.removed": "{tag} removed",
  "toast.selected": "{tag} selected",
  "toast.updated": "Updated to {tag}",
  "toast.copied": "Copied",
};

export type Lang = "ru" | "en";

const tables: Record<Lang, Record<Key, string>> = { ru, en };

export function resolveLang(setting: string): Lang {
  if (setting === "ru" || setting === "en") {
    return setting;
  }
  const locale = typeof navigator !== "undefined" ? navigator.language : "en";
  return locale.toLowerCase().startsWith("ru") ? "ru" : "en";
}

export function translate(
  lang: Lang,
  key: Key,
  params?: Record<string, string | number>,
): string {
  let text = tables[lang][key] ?? tables.en[key] ?? key;
  if (params) {
    for (const [name, value] of Object.entries(params)) {
      text = text.replace(`{${name}}`, String(value));
    }
  }
  return text;
}

const I18nContext = createContext<{ lang: Lang; t: (key: Key) => string }>({
  lang: "ru",
  t: (key) => translate("ru", key),
});

export function I18nProvider({ lang, children }: { lang: Lang; children: ReactNode }) {
  const value = useMemo(
    () => ({ lang, t: (key: Key) => translate(lang, key) }),
    [lang],
  );
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  return useContext(I18nContext);
}

export type { Key as TranslationKey };
