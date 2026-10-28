//! Installing a Horizon release: download the platform archive, unpack it into
//! `versions/<tag>`, remove it again.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::state::AppState;
use crate::util;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallProgress {
    tag: String,
    stage: String,
    received: u64,
    total: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OperationError {
    scope: String,
    message: String,
}

#[tauri::command]
pub async fn install_release(
    app: AppHandle,
    state: State<'_, AppState>,
    tag: String,
) -> Result<(), String> {
    crate::releases::validate_tag(&tag)?;
    state.begin_install()?;
    let result = install_inner(&app, &tag).await;
    state.end_install();
    if result.is_ok() {
        let _ = app.emit("state-changed", ());
    } else if let Err(message) = &result {
        let _ = app.emit(
            "operation-error",
            OperationError {
                scope: "install".to_string(),
                message: message.clone(),
            },
        );
    }
    result
}

async fn install_inner(app: &AppHandle, tag: &str) -> Result<(), String> {
    let asset = util::asset_name()?;
    let url = crate::releases::download_url(tag)?;
    let archive = crate::paths::downloads_dir().join(format!("{tag}-{asset}"));

    emit(app, tag, "download", 0, 0);
    let emitter = app.clone();
    let progress_tag = tag.to_string();
    let mut last = Instant::now();
    util::download(&url, &archive, move |received, total| {
        // A few events per second are enough for a progress bar.
        if last.elapsed() >= Duration::from_millis(120) || (total > 0 && received == total) {
            last = Instant::now();
            emit(&emitter, &progress_tag, "download", received, total);
        }
    })
    .await?;

    emit(app, tag, "extract", 0, 0);
    let staging = crate::paths::versions_dir().join(format!(".staging-{tag}"));
    let destination = crate::paths::version_dir(tag);
    let archive_for_task = archive.clone();
    let staging_for_task = staging.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        if staging_for_task.exists() {
            std::fs::remove_dir_all(&staging_for_task).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir_all(&staging_for_task).map_err(|e| e.to_string())?;
        extract(&archive_for_task, &staging_for_task)
    })
    .await
    .map_err(|e| format!("Распаковка прервалась: {e}"))??;

    if destination.exists() {
        std::fs::remove_dir_all(&destination).map_err(|e| e.to_string())?;
    }
    std::fs::rename(&staging, &destination).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&archive);

    if !crate::paths::installed(tag) {
        return Err(format!(
            "После распаковки не найден {}.",
            util::client_exe()
        ));
    }

    let mut settings = crate::settings::Settings::load();
    if settings.selected_version.is_none() {
        settings.selected_version = Some(tag.to_string());
        let _ = settings.save();
    }

    emit(app, tag, "done", 0, 0);
    Ok(())
}

fn emit(app: &AppHandle, tag: &str, stage: &str, received: u64, total: u64) {
    let _ = app.emit(
        "install-progress",
        InstallProgress {
            tag: tag.to_string(),
            stage: stage.to_string(),
            received,
            total,
        },
    );
}

#[tauri::command]
pub fn uninstall_release(app: AppHandle, state: State<'_, AppState>, tag: String) -> Result<(), String> {
    crate::releases::validate_tag(&tag)?;
    if state.running_tag().as_deref() == Some(tag.as_str()) {
        return Err("Сначала остановите игру.".to_string());
    }
    let dir = crate::paths::version_dir(&tag);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    let mut settings = crate::settings::Settings::load();
    if settings.selected_version.as_deref() == Some(tag.as_str()) {
        settings.selected_version = None;
    }
    let _ = settings.save();
    let _ = app.emit("state-changed", ());
    Ok(())
}

fn extract(archive: &Path, destination: &Path) -> Result<(), String> {
    let is_tar = archive
        .file_name()
        .map(|name| name.to_string_lossy().ends_with(".tar.gz"))
        .unwrap_or(false);
    if is_tar {
        extract_tar_gz(archive, destination)
    } else {
        extract_zip(archive, destination)
    }
}

fn extract_zip(archive: &Path, destination: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("Архив повреждён: {e}"))?;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(|e| e.to_string())?;
        let name = entry.name().replace('\\', "/");
        let Some(relative) = strip_root(&name)? else {
            continue;
        };
        let out = destination.join(&relative);
        if entry.is_dir() {
            std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut target = std::fs::File::create(&out).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut target).map_err(|e| e.to_string())?;
        }
    }
    mark_executables(destination);
    Ok(())
}

fn extract_tar_gz(archive: &Path, destination: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);
    let entries = tar.entries().map_err(|e| e.to_string())?;
    for entry in entries {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry
            .path()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let Some(relative) = strip_root(&path)? else {
            continue;
        };
        let out = destination.join(&relative);
        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut target = std::fs::File::create(&out).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut target).map_err(|e| e.to_string())?;
        }
    }
    mark_executables(destination);
    Ok(())
}

/// The archives hold one top folder (`horizon-windows-x64/`); drop it.
fn strip_root(name: &str) -> Result<Option<PathBuf>, String> {
    let mut parts = name.split('/').filter(|part| !part.is_empty() && *part != ".");
    if parts.next().is_none() {
        return Ok(None);
    }
    let mut relative = PathBuf::new();
    for part in parts {
        if part == ".." {
            return Err("Архив содержит недопустимый путь.".to_string());
        }
        relative.push(part);
    }
    if relative.as_os_str().is_empty() {
        Ok(None)
    } else {
        Ok(Some(relative))
    }
}

#[cfg(unix)]
fn mark_executables(directory: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        if !entry.path().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("horizon-") {
            if let Ok(metadata) = entry.metadata() {
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o755);
                let _ = std::fs::set_permissions(entry.path(), permissions);
            }
        }
    }
}

#[cfg(not(unix))]
fn mark_executables(_directory: &Path) {}

#[cfg(test)]
mod tests {
    use super::strip_root;

    #[test]
    fn strips_the_release_folder() {
        assert_eq!(
            strip_root("horizon-windows-x64/horizon-client.exe").unwrap(),
            Some(std::path::PathBuf::from("horizon-client.exe"))
        );
        assert_eq!(
            strip_root("horizon-windows-x64/tools/lab.py").unwrap(),
            Some(std::path::PathBuf::from("tools/lab.py"))
        );
    }

    #[test]
    fn skips_the_folder_entry_itself() {
        assert_eq!(strip_root("horizon-windows-x64/").unwrap(), None);
        assert_eq!(strip_root("").unwrap(), None);
    }

    #[test]
    fn refuses_paths_that_escape_the_folder() {
        assert!(strip_root("root/../../evil.exe").is_err());
        assert!(strip_root("root/..").is_err());
    }
}
