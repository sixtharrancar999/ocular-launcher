//! Finding a Java 25 runtime for `horizon-prepare` (the game itself needs none).

use std::path::{Path, PathBuf};
use std::process::Command;

pub fn detect() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(home) = std::env::var_os("JAVA_HOME").filter(|value| !value.is_empty()) {
        candidates.push(PathBuf::from(home));
    }
    if let Some(java) = java_on_path() {
        if let Some(home) = java.parent().and_then(Path::parent) {
            candidates.push(home.to_path_buf());
        }
    }
    scan_installs(&mut candidates);

    candidates
        .into_iter()
        .find(|home| java_major(home).is_some_and(|major| major >= 25))
}

fn java_binary(home: &Path) -> PathBuf {
    if cfg!(windows) {
        home.join("bin").join("java.exe")
    } else {
        home.join("bin").join("java")
    }
}

fn java_on_path() -> Option<PathBuf> {
    let output = if cfg!(windows) {
        let mut command = Command::new("where");
        command.arg("java");
        crate::util::quiet(&mut command);
        command.output().ok()?
    } else {
        Command::new("which").arg("java").output().ok()?
    };
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_file())
}

fn java_major(home: &Path) -> Option<u32> {
    let binary = java_binary(home);
    if !binary.is_file() {
        return None;
    }
    let mut command = Command::new(binary);
    command.arg("-version");
    crate::util::quiet(&mut command);
    let output = command.output().ok()?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    parse_major(&text)
}

fn parse_major(text: &str) -> Option<u32> {
    let marker = "version \"";
    let start = text.find(marker)? + marker.len();
    let rest = &text[start..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn scan_installs(candidates: &mut Vec<PathBuf>) {
    #[cfg(windows)]
    let roots = [
        r"C:\Program Files\Microsoft",
        r"C:\Program Files\Java",
        r"C:\Program Files\Eclipse Adoptium",
        r"C:\Program Files\Amazon Corretto",
        r"C:\Program Files\Zulu",
        r"C:\Program Files\BellSoft",
        r"C:\Program Files\Semeru",
    ];
    #[cfg(all(unix, not(target_os = "macos")))]
    let roots = ["/usr/lib/jvm"];
    #[cfg(target_os = "macos")]
    let roots = ["/Library/Java/JavaVirtualMachines"];

    for root in roots {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if cfg!(target_os = "macos") {
                // /Library/Java/JavaVirtualMachines/<name>/Contents/Home
                let home = path.join("Contents").join("Home");
                if home.is_dir() {
                    candidates.push(home);
                }
            } else if java_binary(&path).is_file() {
                candidates.push(path);
            }
        }
    }
}

pub fn describe(home: &Path) -> String {
    match java_major(home) {
        Some(major) => format!("Java {major} ({})", home.display()),
        None => home.display().to_string(),
    }
}
