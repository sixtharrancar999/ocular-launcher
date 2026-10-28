//! What is running right now: the game process, and the one long operation.

use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

pub struct RunningGame {
    pub tag: String,
    pub child: Child,
}

#[derive(Default)]
pub struct AppState {
    pub install_busy: AtomicBool,
    pub prepare_busy: AtomicBool,
    pub game: Mutex<Option<RunningGame>>,
}

impl AppState {
    pub fn running_tag(&self) -> Option<String> {
        self.game
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|game| game.tag.clone()))
    }

    pub fn is_running(&self) -> bool {
        self.game.lock().map(|guard| guard.is_some()).unwrap_or(false)
    }

    pub fn begin_install(&self) -> Result<(), String> {
        if self
            .install_busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            Ok(())
        } else {
            Err("Установка уже идёт.".to_string())
        }
    }

    pub fn end_install(&self) {
        self.install_busy.store(false, Ordering::SeqCst);
    }

    pub fn begin_prepare(&self) -> Result<(), String> {
        if self
            .prepare_busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            Ok(())
        } else {
            Err("Подготовка уже идёт.".to_string())
        }
    }

    pub fn end_prepare(&self) {
        self.prepare_busy.store(false, Ordering::SeqCst);
    }
}
