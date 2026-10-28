//! Local skins and capes, tied to the offline nickname.
//!
//! ```
//! <data>\skins\<nickname>\skin.png      the skin
//! <data>\skins\<nickname>\model         "classic" or "slim"
//! <data>\skins\<nickname>\cape.png      the cape, when there is one
//! ```
//!
//! The game reads them through `HORIZON_OFFLINE_SKIN`, `HORIZON_OFFLINE_SKIN_MODEL` and
//! `HORIZON_OFFLINE_CAPE` (what Horizon's own launcher passes). Nothing is uploaded: the files
//! stay on this computer, and the launcher only draws their previews.

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::settings::Settings;
use crate::util;

const MODEL_FILE: &str = "model";
const CLASSIC: &str = "classic";
const SLIM: &str = "slim";
const TEXTURE_BACKUP: &str = ".ocular-skin-backup";
const TEXTURE_MARKER: &str = ".ocular-skin.sha1";
const LEGACY_PACK: &str = "ocular-skin.zip";
const LEGACY_ENTRY: &str = "file/ocular-skin.zip";
const SKIN_NAMES: [&str; 9] = [
    "alex", "ari", "efe", "kai", "makena", "noor", "steve", "sunny", "zuri",
];

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinInfo {
    pub width: u32,
    pub height: u32,
    pub modified_ms: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Appearance {
    pub nickname: String,
    pub directory: String,
    pub skin: Option<SkinInfo>,
    pub cape: Option<SkinInfo>,
    pub model: String,
    /// The game was started since a file that the game had not yet read changed.
    pub needs_restart: bool,
}

/// What the game gets in its environment.
pub struct GameAppearance {
    pub skin: String,
    pub model: String,
    pub cape: Option<String>,
    pub fingerprint: String,
}

fn nick_dir(home: &Path, nickname: &str) -> PathBuf {
    home.join("skins").join(nickname)
}

fn skins_home() -> PathBuf {
    crate::paths::data_dir().join("skins")
}

fn skin_path(home: &Path, nickname: &str) -> PathBuf {
    nick_dir(home, nickname).join("skin.png")
}

fn cape_path(home: &Path, nickname: &str) -> PathBuf {
    nick_dir(home, nickname).join("cape.png")
}

fn read_model(home: &Path, nickname: &str) -> String {
    match std::fs::read_to_string(nick_dir(home, nickname).join(MODEL_FILE)) {
        Ok(text) if text.trim() == SLIM => SLIM.to_string(),
        _ => CLASSIC.to_string(),
    }
}

fn write_model(home: &Path, nickname: &str, model: &str) -> Result<(), String> {
    let dir = nick_dir(home, nickname);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(MODEL_FILE), model).map_err(|e| e.to_string())
}

fn png_header(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a])
}

/// Width and height from the PNG header (IHDR).
fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || !png_header(bytes) {
        return None;
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    Some((width, height))
}

fn info(path: &Path) -> Option<SkinInfo> {
    let metadata = std::fs::metadata(path).ok()?;
    let mut buffer = vec![0u8; 24];
    let mut read = 0usize;
    if let Ok(mut file) = std::fs::File::open(path) {
        use std::io::Read;
        read = file.read(&mut buffer).unwrap_or(0);
    }
    let (width, height) = png_size(&buffer[..read]).unwrap_or((0, 0));
    let modified_ms = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|since| since.as_millis() as u64)
        .unwrap_or(0);
    Some(SkinInfo {
        width,
        height,
        modified_ms,
    })
}

fn fingerprint(home: &Path, nickname: &str) -> String {
    let mut parts = Vec::new();
    for (label, path) in [
        ("skin", skin_path(home, nickname)),
        ("cape", cape_path(home, nickname)),
    ] {
        let marker = std::fs::metadata(&path)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|since| since.as_millis())
            .unwrap_or(0);
        parts.push(format!("{label}={marker}"));
    }
    parts.push(format!("model={}", read_model(home, nickname)));
    parts.join(";")
}

fn state_file(home: &Path) -> PathBuf {
    home.join("ocular-skins.json")
}

fn read_state(home: &Path) -> serde_json::Value {
    std::fs::read(state_file(home))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_else(|| serde_json::json!({}))
}

/// Records what the game was last started with, so the UI can say a restart is needed.
pub fn remember_fingerprint(home: &Path, nickname: &str, printed: &str) {
    let mut state = read_state(home);
    if let Some(object) = state.as_object_mut() {
        object.insert(
            format!("started:{nickname}"),
            serde_json::Value::String(printed.to_string()),
        );
    }
    let _ = std::fs::create_dir_all(home);
    if let Ok(text) = serde_json::to_string(&state) {
        let _ = std::fs::write(state_file(home), text);
    }
}

fn validate_nickname(nickname: &str) -> Result<(), String> {
    let valid = (3..=16).contains(&nickname.len())
        && nickname
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_');
    if valid {
        Ok(())
    } else {
        Err("Некорректный никнейм.".to_string())
    }
}

/// The paths the game should get for this nickname, when the player has a skin.
pub fn appearance_for(home: &Path, nickname: &str) -> Option<GameAppearance> {
    if validate_nickname(nickname).is_err() {
        return None;
    }
    let skin = skin_path(home, nickname);
    if !skin.is_file() {
        return None;
    }
    let cape = cape_path(home, nickname);
    Some(GameAppearance {
        skin: skin.display().to_string(),
        model: read_model(home, nickname),
        cape: cape.is_file().then(|| cape.display().to_string()),
        fingerprint: fingerprint(home, nickname),
    })
}

/// Horizon 0.2.93 does not read the offline-skin variables yet, so the local skin is
/// written into the prepared default player textures of the selected version. A resource
/// pack would work too, but the game rebuilds its whole overlay cache for one, which is a
/// very long black screen on the first launch; copying 18 small PNGs is instant.
pub fn sync_prepared_skin(
    home: &Path,
    nickname: &str,
    resources: &Path,
) -> Result<(), String> {
    let textures = resources
        .join("assets")
        .join("minecraft")
        .join("textures")
        .join("entity")
        .join("player");
    if !textures.is_dir() {
        return Ok(());
    }
    remove_legacy_pack(home);

    let backup = resources.join(TEXTURE_BACKUP);
    let marker = resources.join(TEXTURE_MARKER);
    let skin = skin_path(home, nickname);
    if !skin.is_file() {
        restore_textures(&backup, &textures)?;
        let _ = std::fs::remove_file(&marker);
        return Ok(());
    }

    let fingerprint = util::sha1_file(&skin)?;
    let current = std::fs::read_to_string(&marker).ok();
    // The marker alone is not enough: a re-prepare can put the vanilla textures back.
    let applied = std::fs::read(textures.join("wide").join("steve.png"))
        .ok()
        .map(|bytes| util::sha1_bytes(&bytes))
        .as_deref()
        == Some(fingerprint.as_str());
    if applied && backup.is_dir() && current.as_deref().map(str::trim) == Some(fingerprint.as_str())
    {
        return Ok(());
    }
    backup_textures(&textures, &backup)?;
    let bytes = std::fs::read(&skin).map_err(|e| e.to_string())?;
    for model in ["wide", "slim"] {
        for name in SKIN_NAMES {
            let path = textures.join(model).join(format!("{name}.png"));
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(path, &bytes).map_err(|e| e.to_string())?;
        }
    }
    std::fs::write(&marker, &fingerprint).map_err(|e| e.to_string())
}

fn backup_textures(textures: &Path, backup: &Path) -> Result<(), String> {
    for model in ["wide", "slim"] {
        for name in SKIN_NAMES {
            let source = textures.join(model).join(format!("{name}.png"));
            let destination = backup.join(model).join(format!("{name}.png"));
            // The first backup is the vanilla texture; later skins must not overwrite it.
            if destination.exists() || !source.is_file() {
                continue;
            }
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::copy(&source, &destination).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn restore_textures(backup: &Path, textures: &Path) -> Result<(), String> {
    if !backup.is_dir() {
        return Ok(());
    }
    for model in ["wide", "slim"] {
        for name in SKIN_NAMES {
            let source = backup.join(model).join(format!("{name}.png"));
            if !source.is_file() {
                continue;
            }
            let destination = textures.join(model).join(format!("{name}.png"));
            std::fs::copy(&source, &destination).map_err(|e| e.to_string())?;
        }
    }
    let _ = std::fs::remove_dir_all(backup);
    Ok(())
}

/// Earlier builds added a resource pack to `options.txt`; take it out again.
fn remove_legacy_pack(home: &Path) {
    let dir = home.join("resourcepacks");
    let _ = std::fs::remove_file(dir.join(LEGACY_PACK));
    let _ = std::fs::remove_file(dir.join("ocular-skin.sha1"));
    let path = home.join("options.txt");
    let Ok(original) = std::fs::read_to_string(&path) else {
        return;
    };
    let mut lines: Vec<String> = original.lines().map(str::to_string).collect();
    let mut found = false;
    for line in lines.iter_mut() {
        if let Some(rest) = line.strip_prefix("resourcePacks:") {
            let mut list: Vec<String> = serde_json::from_str(rest.trim()).unwrap_or_default();
            let length = list.len();
            list.retain(|entry| entry != LEGACY_ENTRY);
            if list.len() != length {
                *line = format!(
                    "resourcePacks:{}",
                    serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string())
                );
                found = true;
            }
        }
    }
    if found {
        let text = format!("{}\n", lines.join("\n"));
        let _ = std::fs::write(path, text);
    }
}

fn appearance(home: &Path, nickname: &str) -> Appearance {
    let skin = info(&skin_path(home, nickname));
    let cape = info(&cape_path(home, nickname));
    let current = fingerprint(home, nickname);
    let started = read_state(home)
        .get(format!("started:{nickname}"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    Appearance {
        nickname: nickname.to_string(),
        directory: nick_dir(home, nickname).display().to_string(),
        skin,
        cape,
        model: read_model(home, nickname),
        needs_restart: started.is_some_and(|started| started != current),
    }
}

fn emit_changed(app: &AppHandle) {
    let _ = app.emit("appearance-changed", ());
}

fn check_skin(bytes: &[u8]) -> Result<(u32, u32), String> {
    let (width, height) =
        png_size(bytes).ok_or_else(|| "Скин должен быть PNG-файлом.".to_string())?;
    if width != 64 || (height != 64 && height != 32) {
        return Err(format!(
            "Размер скина {width}×{height} px. Нужен 64×64 или 64×32."
        ));
    }
    Ok((width, height))
}

fn check_cape(bytes: &[u8]) -> Result<(), String> {
    let (width, height) =
        png_size(bytes).ok_or_else(|| "Плащ должен быть PNG-файлом.".to_string())?;
    if (width, height) != (64, 32) && (width, height) != (22, 17) && (width, height) != (46, 22) {
        return Err(format!(
            "Размер плаща {width}×{height} px. Нужен 64×32 (или 22×17, 46×22)."
        ));
    }
    Ok(())
}

fn save_skin(home: &Path, nickname: &str, bytes: &[u8], model: Option<&str>) -> Result<(), String> {
    validate_nickname(nickname)?;
    check_skin(bytes)?;
    let dir = nick_dir(home, nickname);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(skin_path(home, nickname), bytes).map_err(|e| e.to_string())?;
    if let Some(model) = model {
        let model = if model == SLIM { SLIM } else { CLASSIC };
        write_model(home, nickname, model)?;
    }
    Ok(())
}

// ----- commands ---------------------------------------------------------------------------

#[tauri::command]
pub fn appearance_get() -> Result<Appearance, String> {
    let settings = Settings::load();
    let home = crate::paths::game_home();
    validate_nickname(&settings.nickname)?;
    Ok(appearance(&home, &settings.nickname))
}

#[tauri::command]
pub fn appearance_source(nickname: String) -> Result<String, String> {
    validate_nickname(&nickname)?;
    let path = skin_path(&crate::paths::game_home(), &nickname);
    if !path.is_file() {
        return Err("Скин не задан.".to_string());
    }
    Ok(format!("data:image/png;base64,{}", base64_file(&path)?))
}

#[tauri::command]
pub fn cape_source() -> Result<Option<String>, String> {
    let settings = Settings::load();
    validate_nickname(&settings.nickname)?;
    let path = cape_path(&crate::paths::game_home(), &settings.nickname);
    if !path.is_file() {
        return Ok(None);
    }
    Ok(Some(format!("data:image/png;base64,{}", base64_file(&path)?)))
}

#[tauri::command]
pub fn skin_set_from_file(app: AppHandle, path: String, model: Option<String>) -> Result<(), String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("Не удалось прочитать файл: {e}"))?;
    let settings = Settings::load();
    save_skin(
        &crate::paths::game_home(),
        &settings.nickname,
        &bytes,
        model.as_deref(),
    )?;
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn skin_set_from_bytes(app: AppHandle, bytes: Vec<u8>) -> Result<(), String> {
    let settings = Settings::load();
    save_skin(
        &crate::paths::game_home(),
        &settings.nickname,
        &bytes,
        None,
    )?;
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn skin_model_set(app: AppHandle, model: String) -> Result<(), String> {
    let settings = Settings::load();
    let home = crate::paths::game_home();
    validate_nickname(&settings.nickname)?;
    if !skin_path(&home, &settings.nickname).is_file() {
        return Err("Сначала загрузите скин.".to_string());
    }
    let model = if model == SLIM { SLIM } else { CLASSIC };
    write_model(&home, &settings.nickname, model)?;
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn cape_set_from_file(app: AppHandle, path: String) -> Result<(), String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("Не удалось прочитать файл: {e}"))?;
    save_cape(&crate::paths::game_home(), &Settings::load().nickname, &bytes)?;
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn cape_set_from_bytes(app: AppHandle, bytes: Vec<u8>) -> Result<(), String> {
    save_cape(&crate::paths::game_home(), &Settings::load().nickname, &bytes)?;
    emit_changed(&app);
    Ok(())
}

fn save_cape(home: &Path, nickname: &str, bytes: &[u8]) -> Result<(), String> {
    validate_nickname(nickname)?;
    check_cape(bytes)?;
    std::fs::create_dir_all(nick_dir(home, nickname)).map_err(|e| e.to_string())?;
    std::fs::write(cape_path(home, nickname), bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cape_clear(app: AppHandle) -> Result<(), String> {
    let settings = Settings::load();
    validate_nickname(&settings.nickname)?;
    let path = cape_path(&crate::paths::game_home(), &settings.nickname);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn appearance_reset(app: AppHandle) -> Result<(), String> {
    let settings = Settings::load();
    validate_nickname(&settings.nickname)?;
    let home = crate::paths::game_home();
    let dir = nick_dir(&home, &settings.nickname);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    if let Some(tag) = Settings::load().selected_version {
        let resources = crate::paths::resources_dir(&tag);
        let _ = sync_prepared_skin(&home, &settings.nickname, &resources);
    }
    emit_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn appearance_reveal() -> Result<(), String> {
    let settings = Settings::load();
    validate_nickname(&settings.nickname)?;
    let dir = nick_dir(&crate::paths::game_home(), &settings.nickname);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    util::open_path(&dir)
}

/// The skins collection, where a renamed file can be moved to and loaded from.
#[tauri::command]
pub fn skins_folder() -> Result<(), String> {
    let home = skins_home();
    std::fs::create_dir_all(&home).map_err(|e| e.to_string())?;
    util::open_path(&home)
}

/// base64 without pulling in another crate: the skin previews are small PNGs.
fn base64_file(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    Ok(base64(bytes.as_slice()))
}

fn base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((triple >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(triple & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        bytes.extend_from_slice(&[0, 0, 0, 13]);
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes
    }

    fn home() -> PathBuf {
        let unique = format!(
            "ocular-skins-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn reads_png_dimensions() {
        assert_eq!(png_size(&png(64, 64)), Some((64, 64)));
        assert_eq!(png_size(&png(64, 32)), Some((64, 32)));
        assert_eq!(png_size(b"not a png at all........"), None);
    }

    #[test]
    fn accepts_the_vanilla_skin_sizes() {
        assert!(check_skin(&png(64, 64)).is_ok());
        assert!(check_skin(&png(64, 32)).is_ok());
        assert!(check_skin(&png(128, 128)).is_err());
        assert!(check_skin(&png(32, 32)).is_err());
        assert!(check_skin(b"jpeg").is_err());
    }

    #[test]
    fn accepts_cape_sizes() {
        assert!(check_cape(&png(64, 32)).is_ok());
        assert!(check_cape(&png(22, 17)).is_ok());
        assert!(check_cape(&png(64, 64)).is_err());
    }

    #[test]
    fn saves_a_skin_per_nickname_and_reports_it() {
        let home = home();
        assert!(appearance_for(&home, "Player").is_none());
        save_skin(&home, "Player", &png(64, 64), Some(SLIM)).unwrap();
        let saved = appearance_for(&home, "Player").unwrap();
        assert!(saved.skin.ends_with("skin.png"));
        assert!(saved.skin.contains("Player"));
        assert_eq!(saved.model, SLIM);
        assert!(saved.cape.is_none());
        // Another nickname keeps its own (empty) appearance.
        assert!(appearance_for(&home, "Other").is_none());
        let shown = appearance(&home, "Player");
        assert_eq!(shown.skin.map(|s| s.width), Some(64));
        assert_eq!(shown.model, SLIM);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn restricting_and_detecting_changes_that_need_a_restart() {
        let home = home();
        save_skin(&home, "Player", &png(64, 64), None).unwrap();
        let first = fingerprint(&home, "Player");
        remember_fingerprint(&home, "Player", &first);
        assert!(!appearance(&home, "Player").needs_restart);
        std::thread::sleep(std::time::Duration::from_millis(20));
        write_model(&home, "Player", SLIM).unwrap();
        assert!(appearance(&home, "Player").needs_restart);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn bad_nicknames_are_refused() {
        assert!(validate_nickname("Player").is_ok());
        assert!(validate_nickname("ab").is_err());
        assert!(validate_nickname("../evil").is_err());
        assert!(validate_nickname("this_name_is_way_too_long").is_err());
    }

    #[test]
    fn base64_matches_known_values() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
    }

    #[test]
    fn prepared_textures_carry_the_skin_and_restore() {
        let home = home();
        let resources = home.join("resources");
        let textures = resources.join("assets/minecraft/textures/entity/player");
        for model in ["wide", "slim"] {
            for name in SKIN_NAMES {
                let path = textures.join(model).join(format!("{name}.png"));
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(&path, format!("vanilla-{model}-{name}")).unwrap();
            }
        }

        save_skin(&home, "Player", &png(64, 64), None).unwrap();
        sync_prepared_skin(&home, "Player", &resources).unwrap();
        let steve = std::fs::read(textures.join("wide/steve.png")).unwrap();
        assert_eq!(steve, png(64, 64));
        assert!(resources.join(TEXTURE_BACKUP).is_dir());

        std::fs::remove_file(home.join("skins/Player/skin.png")).unwrap();
        sync_prepared_skin(&home, "Player", &resources).unwrap();
        assert_eq!(
            std::fs::read_to_string(textures.join("wide/steve.png")).unwrap(),
            "vanilla-wide-steve"
        );
        assert!(!resources.join(TEXTURE_BACKUP).exists());

        let _ = std::fs::remove_dir_all(&home);
    }
}
