**Русский** · **[English](README.md)**

# Ocular Launcher

Лаунчер для [Horizon](https://github.com/horizon-rust/horizon) — Minecraft: Java Edition 26.3, переписанной на Rust.

Ocular ставит релизы Horizon с GitHub, готовит их ресурсы из вашей собственной копии Minecraft и запускает игру. **Никакого входа через Microsoft: играете оффлайн, под своим ником.** Неофициальный проект, не связанный с Mojang, Microsoft или командой Horizon.

![Play](docs/screenshots/play.png)

## Оффлайн в первую очередь: никакого Microsoft-аккаунта

Это главная идея Ocular, а не его пробел.

- **Аккаунт Microsoft не нужен.** Ни окна входа, ни токенов, ни системной ключницы, ни требования «войдите, чтобы играть». Ocular никогда не попросит ваш пароль от Microsoft.
- **Ничего не уходит с вашего ПК.** Лаунчер обращается к GitHub (за релизами Horizon) и к Mojang (за двумя файлами, которые нужны `horizon-prepare`); ник, скины, миры и настройки остаются на этом компьютере. Ни телеметрии, ни аналитики, ни сервера аккаунтов.
- **Имя — ваше.** Любой оффлайн-ник из 3–16 латинских букв, цифр или `_`, и игра запускается сразу. Имена переключаются в настройках в любой момент.
- **Скин без интернета.** Скин и плащ — обычные PNG-файлы рядом с вашим ником. Загружаете их с диска, и игра получает их при запуске: без загрузки, без модерации, без ожидания.
- **Работает без интернета**, когда Horizon уже установлен и подготовлен: одиночная игра, свои миры, LAN. Связь нужна только для установки, обновления и первой подготовки.
- **Online-mode серверы недоступны** — Mojang требует для входа Microsoft-сессию. Это свойство оффлайн-режима, которое Ocular не может обойти. Серверы в offline-mode (в том числе серверы Horizon) принимают ваш ник.

## Что уже работает

- **Версии** — игровые релизы Horizon с GitHub (`vX.Y.Z`), скачивание, распаковка в `versions/<tag>/`, удаление.
- **Подготовка игры** — `horizon-prepare` из установленного релиза; официальный `version JSON` Minecraft по закреплённому хешу из `research-lock.json`; автоматическая загрузка `server.jar` с Mojang; `libraries`/`assets` подключаются ссылками, а не копируются.
- **Play** — выбранная версия, ваш оффлайн-ник (`--name`), своя папка `HORIZON_HOME`, вывод игры в консоль лаунчера, кнопка «Остановить».
- **Локальные скины** — скин и плащ для каждого ника на диске, превью фигуры из текстуры, classic/slim, выбор файла или перетаскивание. Клиент 0.2.93 ещё не читает `HORIZON_OFFLINE_SKIN`, поэтому лаунчер записывает скин в подготовленные дефолтные текстуры игрока (оригиналы сохраняются для отката) — без ресурспака и без пересборки кэша.
- **Обновление игры** — баннер «Доступно обновление» на Play; подготовленные ресурсы переносятся, поэтому обновление никогда не готовит их заново.
- **Обновление лаунчера** — подписанное, из `latest.json` этого репозитория, с проверкой minisign-подписи перед установкой.
- **Консоль** — вывод подготовки и игры, очистка и копирование.
- **Настройки** — ник, папка Minecraft, Java для подготовки, копирование звуков, дополнительные аргументы игры, язык (ru / en / system), быстрый доступ ко всем папкам.
- **Внешний вид** — тёмная тема с янтарным «ирисом» и собственным заголовком окна; страницы Play, «Версии» и «Настройки».

| Игра | Внешность | Версии |
| --- | --- | --- |
| ![Game](docs/screenshots/game.png) | ![Skin](docs/screenshots/skin.png) | ![Versions](docs/screenshots/versions.png) |

## Требования

- Windows 10/11 x64 и WebView2 (в Windows 11 уже есть).
- **Своя копия Minecraft: Java Edition 26.3** — клиент и ассеты на диске. Ocular не скачивает и не распространяет файлы игры; те немногие файлы Mojang, которые он берёт сам (официальный `version JSON`, asset index, `server.jar`), сверяются с хешами из `research-lock.json`.
- **Java 25** — только для подготовки ресурсов (один раз на версию Minecraft). Лаунчер находит её сам (`JAVA_HOME`, PATH, `Program Files`). Самой игре Java не нужна.

### Откуда берётся установка Minecraft

Официальный лаунчер не обязателен: Ocular читает его папку по умолчанию (`%APPDATA%\.minecraft`) или ту, что вы выбрали в настройках. Подходят установки Prism Launcher, MultiMC, ATLauncher, CurseForge и Modrinth App — главное, чтобы в них был **оригинальный клиент 26.3** (`versions\26.3\26.3.jar`, не пропатченный) и папка `assets`. Запускать игру официальным лаунчером заранее не нужно.

### Онлайн-режим в самой игре

В настройках Horizon есть переключатель «Online mode»: включённый — игра обращается к сервисам Mojang, выключенный — ни к чему. Ocular оставляет этот выбор вам.

## Разработка

```powershell
npm install
npm run tauri dev      # окно лаунчера + Vite (http://127.0.0.1:1420)
npm run build          # только фронтенд (tsc + vite)
npm run tauri build    # установщик NSIS в src-tauri/target/release/bundle/
```

Требуются Rust (stable, MSVC) и Visual Studio Build Tools с нагрузкой «Разработка классических приложений на C++» (MSVC + Windows SDK).

## Как это устроено

```
Ocular Launcher/
  src/                     React + TypeScript интерфейс
    pages/                 Play, Версии, Настройки
    components/            Sidebar, TitleBar, ConsoleDock, SkinPreview, Iris, Icons
    lib/                   api (invoke), store (zustand), i18n, types
  src-tauri/               Rust-бэкенд
    src/lib.rs             команды Tauri и состояние
    src/install.rs         скачивание и распаковка релизов
    src/prepare.rs         server.jar, зеркало Minecraft, horizon-prepare
    src/game.rs            запуск/остановка игры, стриминг лога
    src/skins.rs           скин и плащ для каждого ника, подмена текстур
    src/releases.rs        GitHub API и кеш
    src/java.rs            поиск Java 25
    src/paths.rs           раскладка данных
  tools/make_icon.py       иконка приложения (Pillow)
```

### Данные лаунчера

`%LOCALAPPDATA%\Ocular Launcher\`:

| Путь | Что там |
| --- | --- |
| `settings.json` | настройки |
| `versions/<tag>/` | распакованный релиз Horizon и его подготовленный `.local/` |
| `cache/releases.json` | кеш списка релизов (10 минут) |
| `cache/downloads/` | скачанные архивы |
| `server-jars/26.3-server.jar` | закреплённый server.jar |
| `minecraft/` | зеркало Minecraft: version JSON Mojang, hardlink на клиент, junction на `libraries`/`assets` |
| `game/` | `HORIZON_HOME`: миры, options.txt, логи, скриншоты, скины |

### Зачем своё зеркало Minecraft

`horizon-prepare` сверяет клиент и version JSON с `research-lock.json`. Если Mojang переопубликовал version JSON (а это случается) или установку «пропатчил» другой лаунчер, подготовка падает. Ocular вместо этого:

1. берёт `accepted_manifest_sha1` из `research-lock.json` установленного релиза и скачивает **точный** version JSON с `piston-meta.mojang.com` по content-addressed URL;
2. проверяет клиентский jar игрока по `client_sha1` и делает hardlink вместо копирования;
3. подключает `libraries` и `assets` junction-ссылками;
4. скачивает закреплённый asset index и `server.jar` с Mojang.

Поэтому подготовка не зависит от того, что лежит в `versions/26.3/26.3.json` у игрока.

## Отличия от оригинального лаунчера

| | Horizon launcher (GPUI/Rust) | Ocular (Tauri/React) |
| --- | --- | --- |
| Стек | Rust + GPUI | Rust + Tauri 2, React 19, TypeScript |
| Аккаунты | Microsoft OAuth, системная ключница | только оффлайн-ник — осознанно, без Microsoft |
| Инстансы, миры, моды, серверы | есть | в планах |
| Подготовка | своя логика + импорт из версий | зеркало + закреплённый manifest |
| Размер | ~19 МБ | сборка Tauri, WebView2 из системы |

## Обновление лаунчера

Ocular проверяет `latest.json` в [релизах этого репозитория](https://github.com/sixtharrancar999/ocular-launcher/releases) при старте и по кнопке в настройках. Скачанный установщик проверяется по minisign-подписи, затем лаунчер перезапускается.

- Публичный ключ лежит в `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`).
- Приватный ключ хранится вне репозитория (`%USERPROFILE%\.tauri\ocular-launcher.key`) и в GitHub → **Settings → Secrets and variables → Actions** как `TAURI_SIGNING_PRIVATE_KEY`.
- Выпуск: поднять версию в `package.json`, `src-tauri/Cargo.toml` и `src-tauri/tauri.conf.json`, закоммитить, поставить тег `vX.Y.Z` и запушить его. GitHub Actions соберёт NSIS-установщик, подпишет его и опубликует релиз с `latest.json`.
- Приватный ключ нужно сохранить: без него нельзя подписать следующие обновления. Уже установленные копии продолжат работать.
- Пошагово, включая ручной выпуск: [docs/release.md](docs/release.md).

## Планы

- [x] **Обновление игры и лаунчера** — игра обновляется с переносом подготовленных ресурсов; лаунчер — из своего подписанного `latest.json`.
- [x] **Локальные скины** — скин и плащ на этом ПК, превью в лаунчере, classic/slim.
- [ ] **Инстансы** — отдельные папки `HORIZON_HOME` со своими мирами, настройками и копируемым профилем.

Это всё. Microsoft OAuth, моды, хостинг серверов, сборки для macOS/Linux и телеметрия не планируются. Подробности: [docs/roadmap.ru.md](docs/roadmap.ru.md) · [in English](docs/roadmap.md).

## Лицензия

Код самого Ocular — MIT. Horizon — MIT (его собственный код). Minecraft — торговая марка Mojang; файлы игры не входят в этот репозиторий и никогда не распространяются.
