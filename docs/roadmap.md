# Ocular Launcher — roadmap

Three items, no scope creep. Microsoft OAuth is not planned: the launcher always plays under an
offline nickname. That is the point of the project, not something to be fixed later.

## [x] 1. Game and launcher updates

**Game — done.**

- At start (and whenever the list refreshes) the launcher compares the installed version with the
  newest release of `horizon-rust/horizon`; the list is cached for 10 minutes.
- When a newer version is out, Play shows an "Update available" banner with a button.
- Updating downloads the new release if needed, carries the already prepared resources over from
  any matching version (hard links, `adopt_resources`) and selects the new version.
- No prepared version anywhere means a normal preparation.
- Rollback: the old version stays installed and can be picked on the Versions page.

**Launcher — done.**

- Checks `https://github.com/sixtharrancar999/ocular-launcher/releases/latest/download/latest.json`
  at start and from the Settings button.
- A banner on Play and an "Updates" panel in Settings; download with progress, minisign signature
  check, restart.
- Releasing: tag `vX.Y.Z` → GitHub Actions builds the NSIS installer, signs it and publishes the
  release with `latest.json`.
- The private signing key lives outside the repository and in GitHub Secrets
  (`TAURI_SIGNING_PRIVATE_KEY`).

## [x] 2. Local skins

- The skin and cape are PNG files on this PC under
  `%LOCALAPPDATA%\Ocular Launcher\game\skins\<nickname>\` (`skin.png`, `cape.png`, `model`);
  nothing is downloaded from the network.
- Settings → Appearance: a figure built from the texture, uploading a skin (file dialog or
  drag-and-drop), the classic/slim choice, uploading and removing a cape, "restart the game to
  apply".
- Format checks: PNG 64×64 or 64×32 (skin), 64×32 / 22×17 / 46×22 (cape).
- On launch the launcher passes `HORIZON_OFFLINE_SKIN`, `HORIZON_OFFLINE_SKIN_MODEL` and
  `HORIZON_OFFLINE_CAPE`.
- **While the 0.2.93 client does not read those variables**, the launcher writes the skin into the
  prepared default player textures (`assets/minecraft/textures/entity/player/{wide,slim}`), keeping
  the originals in `.ocular-skin-backup` for rollback. No resource pack, so the game never rebuilds
  its 28 000-file overlay cache and does not show a long black screen on the first launch.
  Side effect: other players with default skins look the same.
- In 0.2.93 the arm model (classic/slim) is chosen by the client itself; the switch changes the
  preview and will work in game once Horizon reads `HORIZON_OFFLINE_SKIN_MODEL`.
- Appearance belongs to the nickname: change the name and you get its own look; change back and the
  skin is still there.

## [ ] 3. Instances

- An instance is its own `HORIZON_HOME` folder with its own worlds, `options.txt`, screenshots and
  launch settings.
- Creating an empty instance, copying an existing one (without logs or caches), renaming, deleting
  to the trash.
- Choosing the instance on Play; the last choice is remembered.
- Every instance has its own nickname and skin (through the skin system).
- Instances leave the main `game/` alone: it stays the first instance.

## Out of scope

Microsoft OAuth and online accounts, mods and modpacks, server hosting, macOS/Linux builds,
telemetry.
