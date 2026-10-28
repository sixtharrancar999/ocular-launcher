# Выпуск обновления лаунчера

## Один раз: залить репозиторий и добавить ключ

1. Запушить код (репозиторий уже создан и настроен как `origin`):

   ```powershell
   cd "C:\Vibe\Project Ocular Launcher\Ocular Launcher"
   git push -u origin main
   ```

   Git Credential Manager откроет окно входа GitHub — войди под `sixtharrancar999`.

2. Добавить приватный ключ подписи в секреты репозитория:

   - Скопируй содержимое файла целиком:

     ```powershell
     Get-Content "$env:USERPROFILE\.tauri\ocular-launcher.key" -Raw | Set-Clipboard
     ```

   - GitHub → репозиторий → **Settings → Secrets and variables → Actions → New repository secret**.
   - Имя: `TAURI_SIGNING_PRIVATE_KEY`.
   - Значение: вставь из буфера (одна строка base64).

   Ключ с паролем не создавался, поэтому второй секрет не нужен.

3. Сохрани копию ключа в надёжном месте (менеджер паролей, зашифрованный диск).
   Файл `%USERPROFILE%\.tauri\ocular-launcher.key` — единственный экземпляр. Если он потеряется,
   подписать следующие обновления будет нельзя; уже выпущенные продолжат работать.

## Каждый релиз

1. Поднять версию в трёх файлах:
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`

   и в `package-lock.json` (или `npm version X.Y.Z --no-git-tag-version`).

2. Закоммитить и запушить:

   ```powershell
   git add -A
   git commit -m "Release vX.Y.Z"
   git push
   ```

3. Поставить тег и запушить его:

   ```powershell
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

4. GitHub Actions (`Release`) соберёт Windows-установщик, подпишет его ключом
   и создаст релиз с `latest.json`. Через несколько минут он появится на странице
   **Releases**; установленные лаунчеры увидят его при следующей проверке.

## Ручной способ (без Actions)

Если Actions недоступен, соберите и загрузите файлы сами:

```powershell
cd "C:\Vibe\Project Ocular Launcher\Ocular Launcher"
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$env:TAURI_SIGNING_PRIVATE_KEY = "$env:USERPROFILE\.tauri\ocular-launcher.key"
npm run tauri build
```

Получите `src-tauri\target\release\bundle\nsis\Ocular Launcher_X.Y.Z_x64-setup.exe` и `.sig`,
затем на GitHub: **Releases → Draft a new release → Choose a tag → создать `vX.Y.Z`**,
приложить `.exe` и `.sig`, опубликовать. Файл `latest.json` при ручной загрузке нужно
положить в релиз вручную (Actions делает это сам).

## Обновление самой игры

Версии Horizon берутся из `horizon-rust/horizon`, это отдельный канал: после выхода
новой версии игры лаунчер предложит обновление сам.

## Как это работает

- Лаунчер запрашивает
  `https://github.com/sixtharrancar999/ocular-launcher/releases/latest/download/latest.json`.
- Если версия новее текущей, показывает баннер на Play и раздел в настройках.
- Скачивается установщик, проверяется minisign-подпись публичным ключом из
  `src-tauri/tauri.conf.json`, запускается установка, лаунчер перезапускается.
- Проверка идёт при старте (тихо) и по кнопке «Проверить обновления» (с ошибками в UI).

## Если что-то не так

- **Actions падает на подписи** — не задан или неверен секрет `TAURI_SIGNING_PRIVATE_KEY`.
- **Лаунчер не видит обновление** — проверь, что релиз не черновик и не prerelease,
  и что в нём есть `latest.json` и `*.exe.sig`.
- **«Could not fetch a valid release JSON»** — релиза ещё нет или GitHub недоступен;
  после первого релиза ошибка исчезнет.
- **Сменил ключ** — старые версии не примут новые обновления (публичный ключ вшит).
  Менять ключ можно только вместе с ручной переустановкой у всех пользователей.
