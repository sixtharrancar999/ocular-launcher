//! Ocular Launcher: installs Horizon releases, prepares them from the player's own
//! Minecraft, and starts the game.

mod game;
mod install;
mod java;
mod paths;
mod prepare;
mod releases;
mod settings;
mod state;
mod util;

use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use settings::Settings;
use state::AppState;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub tag: String,
    pub installed: bool,
    pub resources_ready: bool,
    pub minecraft_version: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub settings: Settings,
    pub versions: Vec<VersionInfo>,
    pub data_dir: String,
    pub minecraft_dir: Option<String>,
    pub java: Option<String>,
    pub running_tag: Option<String>,
    pub busy: bool,
    pub platform: String,
    pub app_version: String,
}

#[tauri::command]
fn bootstrap(state: State<'_, AppState>) -> Bootstrap {
    let settings = Settings::load();
    let versions = installed_versions();
    let minecraft_dir = settings
        .minecraft_dir
        .clone()
        .map(std::path::PathBuf::from)
        .or_else(paths::default_minecraft_dir);
    let busy = state.install_busy.load(Ordering::SeqCst) || state.prepare_busy.load(Ordering::SeqCst);
    Bootstrap {
        running_tag: state.running_tag(),
        settings,
        versions,
        data_dir: paths::data_dir().display().to_string(),
        minecraft_dir: minecraft_dir.map(|dir| dir.display().to_string()),
        java: java::detect().map(|home| java::describe(&home)),
        busy,
        platform: std::env::consts::OS.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
async fn list_releases(force: bool) -> Result<Vec<releases::Release>, String> {
    releases::fetch(force).await
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    settings.normalized().save()?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
fn select_version(app: AppHandle, tag: String) -> Result<(), String> {
    releases::validate_tag(&tag)?;
    let mut settings = Settings::load();
    settings.selected_version = Some(tag);
    settings.save()?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
fn launch_game(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let settings = Settings::load();
    let tag = resolve_version(&settings)
        .ok_or_else(|| "Сначала установите версию Horizon.".to_string())?;
    game::launch(&app, &state, &tag, &settings)
}

#[tauri::command]
fn stop_game(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    game::stop(&app, &state)
}

#[tauri::command]
fn reveal(kind: String, tag: Option<String>) -> Result<(), String> {
    let path = match kind.as_str() {
        "data" => paths::data_dir(),
        "versions" => paths::versions_dir(),
        "game" => paths::game_home(),
        "logs" => paths::logs_dir(),
        "minecraft" => paths::minecraft_mirror(),
        "version" => {
            let tag = tag.ok_or_else(|| "Не указана версия.".to_string())?;
            releases::validate_tag(&tag)?;
            paths::version_dir(&tag)
        }
        other => return Err(format!("Неизвестная папка: {other}")),
    };
    util::open_path(&path)
}

#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    util::open_url(&url)
}

fn resolve_version(settings: &Settings) -> Option<String> {
    if let Some(tag) = settings.selected_version.as_deref() {
        if paths::installed(tag) {
            return Some(tag.to_string());
        }
    }
    installed_versions().into_iter().next().map(|v| v.tag)
}

fn installed_versions() -> Vec<VersionInfo> {
    let mut versions = Vec::new();
    let Ok(entries) = std::fs::read_dir(paths::versions_dir()) else {
        return versions;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let tag = entry.file_name().to_string_lossy().into_owned();
        if tag.starts_with('.') || !paths::installed(&tag) {
            continue;
        }
        versions.push(VersionInfo {
            minecraft_version: paths::minecraft_version(&path),
            resources_ready: paths::resources_ready(&tag),
            installed: true,
            tag,
        });
    }
    versions.sort_by_key(|version| std::cmp::Reverse(releases::version_key(&version.tag)));
    versions
}

pub fn run() {
    let _ = paths::ensure_dirs();
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            list_releases,
            install::install_release,
            install::uninstall_release,
            prepare::prepare,
            prepare::adopt_resources,
            launch_game,
            stop_game,
            save_settings,
            select_version,
            reveal,
            open_external,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ocular Launcher");
}
