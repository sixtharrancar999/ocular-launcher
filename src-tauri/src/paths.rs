//! Where Ocular keeps everything.
//!
//! ```
//! %LOCALAPPDATA%\Ocular Launcher\
//!   settings.json
//!   versions\<tag>\                the unpacked Horizon release
//!   versions\<tag>\.local\         resources prepared from the player's Minecraft
//!   cache\releases.json            the GitHub release list, for a few minutes
//!   cache\downloads\               release archives
//!   server-jars\26.3-server.jar    the pinned server jar, for preparation
//!   minecraft\                     our mirror of the player's install (json, jar, links)
//!   game\                          HORIZON_HOME: worlds, options, logs, screenshots
//!   logs\
//! ```

use std::path::{Path, PathBuf};

pub const DATA_DIR_NAME: &str = "Ocular Launcher";
pub const PINNED_MINECRAFT: &str = "26.3";

pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(DATA_DIR_NAME)
}

pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

pub fn versions_dir() -> PathBuf {
    data_dir().join("versions")
}

pub fn version_dir(tag: &str) -> PathBuf {
    versions_dir().join(tag)
}

pub fn cache_dir() -> PathBuf {
    data_dir().join("cache")
}

pub fn downloads_dir() -> PathBuf {
    cache_dir().join("downloads")
}

pub fn releases_cache_path() -> PathBuf {
    cache_dir().join("releases.json")
}

pub fn server_jars_dir() -> PathBuf {
    data_dir().join("server-jars")
}

pub fn server_jar_path(version: &str) -> PathBuf {
    server_jars_dir().join(format!("{version}-server.jar"))
}

pub fn minecraft_mirror() -> PathBuf {
    data_dir().join("minecraft")
}

pub fn game_home() -> PathBuf {
    data_dir().join("game")
}

pub fn logs_dir() -> PathBuf {
    data_dir().join("logs")
}

pub fn ensure_dirs() -> std::io::Result<()> {
    for dir in [
        data_dir(),
        versions_dir(),
        downloads_dir(),
        server_jars_dir(),
        minecraft_mirror(),
        game_home(),
        logs_dir(),
    ] {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// The Minecraft version a Horizon release is pinned to (`26.3`), read from its lock file.
pub fn minecraft_version(version_dir: &Path) -> String {
    std::fs::read(version_dir.join("research-lock.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| {
            value
                .get("minecraft")
                .and_then(|m| m.get("version"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| PINNED_MINECRAFT.to_string())
}

/// `<version>/.local/horizon-client/<mc>/`, what `horizon-client --resources` takes.
pub fn resources_dir(tag: &str) -> PathBuf {
    let dir = version_dir(tag);
    let mc = minecraft_version(&dir);
    dir.join(".local").join("horizon-client").join(mc)
}

pub fn resources_ready(tag: &str) -> bool {
    resources_dir(tag).join("horizon-resources.json").is_file()
}

pub fn installed_client(tag: &str) -> PathBuf {
    version_dir(tag).join(crate::util::client_exe())
}

pub fn installed(tag: &str) -> bool {
    installed_client(tag).is_file()
}

/// The official launcher's game folder, when it exists.
pub fn default_minecraft_dir() -> Option<PathBuf> {
    candidate_minecraft_dirs().into_iter().find(|dir| dir.is_dir())
}

#[cfg(target_os = "windows")]
fn candidate_minecraft_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA") {
        dirs.push(PathBuf::from(appdata).join(".minecraft"));
    }
    dirs
}

#[cfg(target_os = "macos")]
fn candidate_minecraft_dirs() -> Vec<PathBuf> {
    dirs::home_dir()
        .map(|home| vec![home.join("Library/Application Support/minecraft")])
        .unwrap_or_default()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn candidate_minecraft_dirs() -> Vec<PathBuf> {
    dirs::home_dir()
        .map(|home| vec![home.join(".minecraft")])
        .unwrap_or_default()
}
