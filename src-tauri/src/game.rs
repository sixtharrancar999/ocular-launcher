//! Starting and stopping the game, and the console it writes to.

use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::settings::Settings;
use crate::state::{AppState, RunningGame};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GameStarted {
    tag: String,
    pid: u32,
}

#[derive(Clone, Serialize)]
struct GameLog {
    stream: &'static str,
    line: String,
}

#[derive(Clone, Serialize)]
pub struct GameExit {
    pub tag: String,
    pub code: Option<i32>,
}

pub fn launch(app: &AppHandle, state: &AppState, tag: &str, settings: &Settings) -> Result<(), String> {
    if state.is_running() {
        return Err("Игра уже запущена.".to_string());
    }
    let program = crate::paths::installed_client(tag);
    if !program.is_file() {
        return Err("Файлы этой версии не найдены — установите её заново.".to_string());
    }
    if !crate::paths::resources_ready(tag) {
        return Err("Ресурсы ещё не подготовлены.".to_string());
    }

    let home = crate::paths::game_home();
    std::fs::create_dir_all(&home).map_err(|e| e.to_string())?;
    let resources = crate::paths::resources_dir(tag);

    let mut command = Command::new(&program);
    command
        .current_dir(&home)
        .arg("--resources")
        .arg(&resources);
    if !settings.nickname.is_empty() {
        command.arg("--name").arg(&settings.nickname);
    }
    if let Some(skin) = crate::skins::appearance_for(&home, &settings.nickname) {
        command.env("HORIZON_OFFLINE_SKIN", &skin.skin);
        command.env("HORIZON_OFFLINE_SKIN_MODEL", &skin.model);
        if let Some(cape) = &skin.cape {
            command.env("HORIZON_OFFLINE_CAPE", cape);
        }
        crate::skins::remember_fingerprint(&home, &settings.nickname, &skin.fingerprint);
    }
    for word in settings.extra_args.split_whitespace() {
        command.arg(word);
    }
    command.env("HORIZON_HOME", &home);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    crate::util::quiet(&mut command);

    let mut child = command
        .spawn()
        .map_err(|e| format!("Не удалось запустить игру: {e}"))?;
    let pid = child.id();
    if let Some(stdout) = child.stdout.take() {
        spawn_log_reader(stdout, app.clone(), "stdout");
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_log_reader(stderr, app.clone(), "stderr");
    }

    {
        let mut guard = state
            .game
            .lock()
            .map_err(|_| "Состояние лаунчера занято.".to_string())?;
        *guard = Some(RunningGame {
            tag: tag.to_string(),
            child,
        });
    }

    let _ = app.emit(
        "game-started",
        GameStarted {
            tag: tag.to_string(),
            pid,
        },
    );
    let _ = app.emit("state-changed", ());
    spawn_waiter(app.clone());
    Ok(())
}

pub fn stop(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let mut guard = state
        .game
        .lock()
        .map_err(|_| "Состояние лаунчера занято.".to_string())?;
    let Some(mut game) = guard.take() else {
        return Ok(());
    };
    let tag = game.tag.clone();
    let _ = game.child.kill();
    let _ = game.child.wait();
    drop(guard);
    let _ = app.emit("game-exit", GameExit { tag, code: None });
    let _ = app.emit("state-changed", ());
    Ok(())
}

fn spawn_log_reader<R: Read + Send + 'static>(reader: R, app: AppHandle, stream: &'static str) {
    std::thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            let _ = app.emit(
                "game-log",
                GameLog {
                    stream,
                    line,
                },
            );
        }
    });
}

/// Watches the process so the window learns when the player quits (or it crashes).
fn spawn_waiter(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(350));
        let state = app.state::<AppState>();
        let outcome = {
            let Ok(mut guard) = state.game.lock() else {
                return;
            };
            let Some(game) = guard.as_mut() else {
                return;
            };
            match game.child.try_wait() {
                Ok(Some(status)) => {
                    let tag = game.tag.clone();
                    *guard = None;
                    Some((tag, status.code()))
                }
                Ok(None) => None,
                Err(_) => {
                    let tag = game.tag.clone();
                    *guard = None;
                    Some((tag, None))
                }
            }
        };
        if let Some((tag, code)) = outcome {
            let _ = app.emit("game-exit", GameExit { tag, code });
            let _ = app.emit("state-changed", ());
            return;
        }
    });
}
