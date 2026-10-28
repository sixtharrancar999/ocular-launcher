//! Preparing a Horizon release from the player's own Minecraft: the pinned server
//! jar, a clean mirror of the player's install, then `horizon-prepare` itself.

use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::settings::Settings;
use crate::state::AppState;
use crate::util;

#[derive(Debug, Clone, Deserialize)]
struct ResearchLock {
    minecraft: LockMinecraft,
}

#[derive(Debug, Clone, Deserialize)]
struct LockMinecraft {
    version: String,
    #[serde(default)]
    manifest_sha1: String,
    #[serde(default)]
    accepted_manifest_sha1: Vec<String>,
    #[serde(default)]
    accepted_asset_index_sha1: Vec<String>,
    client_sha1: String,
    server_sha1: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PrepareProgress {
    stage: String,
    received: u64,
    total: u64,
}

#[derive(Clone, Serialize)]
struct OpLog {
    scope: String,
    line: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OperationError {
    scope: String,
    message: String,
}

#[tauri::command]
pub async fn prepare(
    app: AppHandle,
    state: State<'_, AppState>,
    tag: String,
) -> Result<(), String> {
    crate::releases::validate_tag(&tag)?;
    state.begin_prepare()?;
    let result = prepare_inner(&app, &tag).await;
    state.end_prepare();
    match &result {
        Ok(()) => {
            let mut settings = Settings::load();
            settings.selected_version = Some(tag.clone());
            let _ = settings.save();
            let _ = app.emit("state-changed", ());
        }
        Err(message) => {
            let _ = app.emit(
                "operation-error",
                OperationError {
                    scope: "prepare".to_string(),
                    message: message.clone(),
                },
            );
        }
    }
    result
}

/// Carries a prepared `.local` folder from another installed version of the same
/// Minecraft release, so updating Horizon does not prepare everything again.
#[tauri::command]
pub async fn adopt_resources(app: AppHandle, to: String) -> Result<Option<String>, String> {
    crate::releases::validate_tag(&to)?;
    if crate::paths::resources_ready(&to) {
        return Ok(None);
    }
    let to_dir = crate::paths::version_dir(&to);
    if !to_dir.is_dir() {
        return Err("Сначала установите версию.".to_string());
    }
    let to_mc = crate::paths::minecraft_version(&to_dir);
    let selected = Settings::load().selected_version;

    let mut candidates: Vec<String> = crate::paths::installed_tags()
        .into_iter()
        .filter(|tag| tag != &to)
        .filter(|tag| {
            crate::paths::minecraft_version(&crate::paths::version_dir(tag)) == to_mc
        })
        .filter(|tag| crate::paths::resources_ready(tag))
        .collect();
    // The version the player is on is the most likely source; then the others.
    candidates.sort_by_key(|tag| if Some(tag) == selected.as_ref() { 0 } else { 1 });

    let Some(source) = candidates.into_iter().next() else {
        return Ok(None);
    };
    log(
        &app,
        "prepare",
        format!("Переношу подготовленные ресурсы из {source}…"),
    );

    let from = crate::paths::version_dir(&source).join(".local");
    let destination = to_dir.join(".local");
    let destination_for_task = destination.clone();
    tauri::async_runtime::spawn_blocking(move || copy_tree(&from, &destination_for_task))
        .await
        .map_err(|e| format!("Перенос ресурсов прервался: {e}"))??;

    if !crate::paths::resources_ready(&to) {
        return Err("Ресурсы скопированы, но манифест не найден.".to_string());
    }
    log(
        &app,
        "prepare",
        format!("Ресурсы перенесены из {source}, повторная подготовка не нужна."),
    );
    let _ = app.emit("state-changed", ());
    Ok(Some(source))
}

/// Hardlinks files when the volume allows it, copies otherwise; existing files win.
fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    let entries = std::fs::read_dir(from).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let source = entry.path();
        let destination = to.join(entry.file_name());
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        if file_type.is_dir() {
            copy_tree(&source, &destination)?;
        } else if file_type.is_file() && !destination.exists() {
            if std::fs::hard_link(&source, &destination).is_err() {
                std::fs::copy(&source, &destination).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

async fn prepare_inner(app: &AppHandle, tag: &str) -> Result<(), String> {
    let version_dir = crate::paths::version_dir(tag);
    if !version_dir.is_dir() {
        return Err("Сначала установите эту версию Horizon.".to_string());
    }
    if !version_dir.join(util::prepare_exe()).is_file() {
        return Err(format!(
            "В версии {} нет {} — установка повреждена, установите заново.",
            tag,
            util::prepare_exe()
        ));
    }

    let lock = read_lock(&version_dir)?;
    let settings = Settings::load();
    let minecraft_dir = settings
        .minecraft_dir
        .as_deref()
        .map(PathBuf::from)
        .filter(|dir| dir.is_dir())
        .or_else(crate::paths::default_minecraft_dir)
        .ok_or_else(|| {
            "Не найдена папка Minecraft. Установите Minecraft: Java Edition 26.3 через официальный лаунчер и укажите папку в настройках.".to_string()
        })?;

    log(
        app,
        "prepare",
        format!(
            "Horizon {tag} · Minecraft {} · {}",
            lock.minecraft.version,
            minecraft_dir.display()
        ),
    );

    emit_stage(app, "server-jar");
    let server_jar = ensure_server_jar(app, &lock).await?;

    emit_stage(app, "mirror");
    let mirror = ensure_mirror(app, &lock.minecraft, &minecraft_dir).await?;

    emit_stage(app, "prepare");
    let include_sounds = settings.include_sounds;
    let app_for_task = app.clone();
    let tag_for_task = tag.to_string();
    let mirror_for_task = mirror.clone();
    let jar_for_task = server_jar.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_prepare(
            &app_for_task,
            &tag_for_task,
            &mirror_for_task,
            &jar_for_task,
            include_sounds,
        )
    })
    .await
    .map_err(|e| format!("Подготовка прервалась: {e}"))??;

    if !crate::paths::resources_ready(tag) {
        return Err("horizon-prepare завершился, но ресурсы не найдены.".to_string());
    }
    emit_stage(app, "done");
    log(app, "prepare", "Ресурсы готовы. Можно играть.".to_string());
    Ok(())
}

fn read_lock(version_dir: &Path) -> Result<ResearchLock, String> {
    let path = version_dir.join("research-lock.json");
    let bytes = std::fs::read(&path)
        .map_err(|e| format!("Не удалось прочитать {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| format!("research-lock.json повреждён: {e}"))
}

async fn ensure_server_jar(app: &AppHandle, lock: &ResearchLock) -> Result<PathBuf, String> {
    let path = crate::paths::server_jar_path(&lock.minecraft.version);
    if path.is_file() && util::sha1_file(&path).ok().as_deref() == Some(&lock.minecraft.server_sha1)
    {
        return Ok(path);
    }
    let url = format!(
        "https://piston-data.mojang.com/v1/objects/{}/server.jar",
        lock.minecraft.server_sha1
    );
    log(
        app,
        "prepare",
        format!("Скачиваю server.jar Minecraft {}…", lock.minecraft.version),
    );
    let emitter = app.clone();
    let mut last = Instant::now();
    util::download(&url, &path, move |received, total| {
        if last.elapsed() >= Duration::from_millis(150) || (total > 0 && received == total) {
            last = Instant::now();
            let _ = emitter.emit(
                "prepare-progress",
                PrepareProgress {
                    stage: "server-jar".to_string(),
                    received,
                    total,
                },
            );
        }
    })
    .await?;
    let sha = util::sha1_file(&path)?;
    if sha != lock.minecraft.server_sha1 {
        let _ = std::fs::remove_file(&path);
        return Err("server.jar не совпал по SHA-1 с закреплённым — попробуйте ещё раз.".to_string());
    }
    Ok(path)
}

async fn ensure_mirror(
    app: &AppHandle,
    lock: &LockMinecraft,
    minecraft_dir: &Path,
) -> Result<PathBuf, String> {
    let mirror = crate::paths::minecraft_mirror();
    let version_dir = mirror.join("versions").join(&lock.version);
    std::fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;

    let json_path = version_dir.join(format!("{}.json", lock.version));
    let mut accepted = lock.accepted_manifest_sha1.clone();
    if !lock.manifest_sha1.is_empty() {
        accepted.insert(0, lock.manifest_sha1.clone());
    }
    let json_is_known = json_path.is_file()
        && util::sha1_file(&json_path)
            .map(|sha| accepted.contains(&sha))
            .unwrap_or(false);

    let json_bytes = if json_is_known {
        std::fs::read(&json_path).map_err(|e| e.to_string())?
    } else {
        let mut found = None;
        for hash in &accepted {
            let url = format!(
                "https://piston-meta.mojang.com/v1/packages/{hash}/{}.json",
                lock.version
            );
            if let Ok(bytes) = util::get_bytes(&url).await {
                if &util::sha1_bytes(&bytes) == hash {
                    found = Some(bytes);
                    break;
                }
            }
        }
        let bytes = found.ok_or_else(|| {
            format!(
                "Не удалось получить официальный version JSON Minecraft {}. Обновите Horizon.",
                lock.version
            )
        })?;
        std::fs::write(&json_path, &bytes).map_err(|e| e.to_string())?;
        log(
            app,
            "prepare",
            format!(
                "Версия Minecraft {}: взят закреплённый официальный version JSON.",
                lock.version
            ),
        );
        bytes
    };

    let manifest: serde_json::Value = serde_json::from_slice(&json_bytes)
        .map_err(|e| format!("version JSON Minecraft не разобран: {e}"))?;

    // The client jar. It is the player's own copy, checked against the lock.
    let jar_path = version_dir.join(format!("{}.jar", lock.version));
    if !jar_path.is_file() {
        let source = minecraft_dir
            .join("versions")
            .join(&lock.version)
            .join(format!("{}.jar", lock.version));
        if !source.is_file() {
            return Err(format!(
                "В {} нет клиента Minecraft {}. Запустите официальный лаунчер и игру 26.3 один раз.",
                minecraft_dir.display(),
                lock.version
            ));
        }
        std::fs::hard_link(&source, &jar_path)
            .or_else(|_| std::fs::copy(&source, &jar_path).map(|_| ()))
            .map_err(|e| format!("Не удалось скопировать клиент Minecraft: {e}"))?;
    }
    let jar_sha = util::sha1_file(&jar_path)?;
    if jar_sha != lock.client_sha1 {
        return Err(format!(
            "Клиент Minecraft {} не совпадает с закреплённым (SHA-1 {}). Установите чистую копию через официальный лаунчер.",
            lock.version,
            lock.client_sha1
        ));
    }

    // Libraries and the asset store are the player's; linking them keeps disk use at zero.
    let libraries = mirror.join("libraries");
    if !libraries.exists() {
        let source = minecraft_dir.join("libraries");
        if !source.is_dir() {
            return Err(format!(
                "В {} нет папки libraries. Запустите официальный лаунчер и игру 26.3 один раз.",
                minecraft_dir.display()
            ));
        }
        util::link_dir(&libraries, &source)?;
    }

    let assets = mirror.join("assets");
    std::fs::create_dir_all(assets.join("indexes")).map_err(|e| e.to_string())?;
    let objects = assets.join("objects");
    if !objects.exists() {
        let source = minecraft_dir.join("assets").join("objects");
        if !source.is_dir() {
            return Err(format!(
                "В {} нет папки assets. Запустите официальный лаунчер и игру 26.3 один раз.",
                minecraft_dir.display()
            ));
        }
        util::link_dir(&objects, &source)?;
    }

    // The asset index: a pinned copy, so preparation does not depend on the player's one.
    let index_id = manifest["assetIndex"]["id"]
        .as_str()
        .unwrap_or("34")
        .to_string();
    let index_hint = manifest["assetIndex"]["sha1"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let index_path = assets.join("indexes").join(format!("{index_id}.json"));
    if !index_path.is_file() {
        let mut hashes = lock.accepted_asset_index_sha1.clone();
        if !index_hint.is_empty() {
            hashes.push(index_hint);
        }
        let mut written = false;
        for hash in &hashes {
            let url = format!(
                "https://piston-meta.mojang.com/v1/packages/{hash}/{index_id}.json"
            );
            if let Ok(bytes) = util::get_bytes(&url).await {
                if &util::sha1_bytes(&bytes) == hash {
                    std::fs::write(&index_path, bytes).map_err(|e| e.to_string())?;
                    written = true;
                    break;
                }
            }
        }
        if !written {
            let url = manifest["assetIndex"]["url"].as_str().unwrap_or("");
            if url.is_empty() {
                return Err("В version JSON Minecraft нет asset index.".to_string());
            }
            let bytes = util::get_bytes(url).await?;
            std::fs::write(&index_path, bytes).map_err(|e| e.to_string())?;
        }
    }

    log(
        app,
        "prepare",
        format!("Зеркало Minecraft готово: {}", mirror.display()),
    );
    Ok(mirror)
}

fn run_prepare(
    app: &AppHandle,
    tag: &str,
    mirror: &Path,
    server_jar: &Path,
    include_sounds: bool,
) -> Result<(), String> {
    let version_dir = crate::paths::version_dir(tag);
    let prepare = version_dir.join(util::prepare_exe());

    let mut command = Command::new(&prepare);
    command
        .current_dir(&version_dir)
        .arg("--prepare")
        .arg("--server-jar")
        .arg(server_jar)
        .arg("--minecraft-dir")
        .arg(mirror)
        .arg("--root")
        .arg(&version_dir);
    add_java(&mut command);
    run_streaming(app, command, "prepare", "horizon-prepare")?;

    if include_sounds {
        emit_stage(app, "sounds");
        let mut command = Command::new(&prepare);
        command
            .current_dir(&version_dir)
            .arg("sounds")
            .arg("--minecraft-dir")
            .arg(mirror)
            .arg("--root")
            .arg(&version_dir);
        add_java(&mut command);
        if let Err(error) = run_streaming(app, command, "prepare", "horizon-prepare sounds") {
            // Sounds are optional: the game is silent, but it still runs.
            log(
                app,
                "prepare",
                format!("Звуки не скопированы ({error}); игра будет без звука."),
            );
        }
    }

    Ok(())
}

fn add_java(command: &mut Command) {
    if let Some(home) = crate::java::detect() {
        command.env("JAVA_HOME", &home);
        if let Some(path) = std::env::var_os("PATH") {
            let mut parts = vec![home.join("bin")];
            parts.extend(std::env::split_paths(&path));
            if let Ok(joined) = std::env::join_paths(parts) {
                command.env("PATH", joined);
            }
        }
    }
}

fn run_streaming(
    app: &AppHandle,
    mut command: Command,
    scope: &str,
    label: &str,
) -> Result<(), String> {
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    util::quiet(&mut command);
    let mut child = command
        .spawn()
        .map_err(|e| format!("Не удалось запустить {label}: {e}"))?;
    let readers = spawn_readers(app, &mut child, scope);
    let status = child.wait().map_err(|e| e.to_string())?;
    join_readers(readers);
    if !status.success() {
        return Err(format!(
            "{label} завершился с кодом {}.",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

fn spawn_readers(
    app: &AppHandle,
    child: &mut Child,
    scope: &str,
) -> Vec<std::thread::JoinHandle<()>> {
    let mut readers = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        readers.push(spawn_reader(stdout, app.clone(), scope.to_string()));
    }
    if let Some(stderr) = child.stderr.take() {
        readers.push(spawn_reader(stderr, app.clone(), scope.to_string()));
    }
    readers
}

fn spawn_reader<R: Read + Send + 'static>(
    reader: R,
    app: AppHandle,
    scope: String,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            let _ = app.emit(
                "op-log",
                OpLog {
                    scope: scope.clone(),
                    line,
                },
            );
        }
    })
}

fn join_readers(readers: Vec<std::thread::JoinHandle<()>>) {
    for reader in readers {
        let _ = reader.join();
    }
}

fn log(app: &AppHandle, scope: &str, line: impl Into<String>) {
    let _ = app.emit(
        "op-log",
        OpLog {
            scope: scope.to_string(),
            line: line.into(),
        },
    );
}

fn emit_stage(app: &AppHandle, stage: &str) {
    let _ = app.emit(
        "prepare-progress",
        PrepareProgress {
            stage: stage.to_string(),
            received: 0,
            total: 0,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::copy_tree;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let unique = format!(
            "ocular-{name}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn copies_a_tree_and_keeps_existing_files() {
        let base = temp_dir("copy-resources");
        let from = base.join("from");
        let to = base.join("to");
        std::fs::create_dir_all(from.join("nested")).unwrap();
        std::fs::write(from.join("a.txt"), "one").unwrap();
        std::fs::write(from.join("nested/b.txt"), "two").unwrap();
        std::fs::create_dir_all(&to).unwrap();
        std::fs::write(to.join("a.txt"), "keep").unwrap();

        copy_tree(&from, &to).unwrap();

        assert_eq!(std::fs::read_to_string(to.join("a.txt")).unwrap(), "keep");
        assert_eq!(
            std::fs::read_to_string(to.join("nested/b.txt")).unwrap(),
            "two"
        );
        let _ = std::fs::remove_dir_all(&base);
    }
}
