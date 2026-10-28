//! Small helpers shared by the modules: one HTTP client, hashes, quiet child
//! processes, opening folders and links.

use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use futures_util::StreamExt;
use sha1::{Digest, Sha1};
use tokio::io::AsyncWriteExt;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(concat!(
                "OcularLauncher/",
                env!("CARGO_PKG_VERSION"),
                " (+https://github.com/horizon-rust/horizon)"
            ))
            .connect_timeout(Duration::from_secs(20))
            .build()
            .expect("HTTP client")
    })
}

pub async fn get_bytes(url: &str) -> Result<Vec<u8>, String> {
    let response = client()
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Сеть недоступна: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("HTTP {} для {url}", response.status()));
    }
    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|e| format!("Не удалось прочитать ответ: {e}"))
}

/// Downloads `url` into `dest`, calling `on_progress(received, total)` as it goes.
pub async fn download<F>(url: &str, dest: &Path, mut on_progress: F) -> Result<u64, String>
where
    F: FnMut(u64, u64),
{
    let response = client()
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Сеть недоступна: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {status} для {url}"));
    }
    let total = response.content_length().unwrap_or(0);
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let file_name = dest
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "download".to_string());
    let tmp = dest.with_file_name(format!("{file_name}.part"));
    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| e.to_string())?;

    let mut received = 0u64;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Загрузка прервалась: {e}"))?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        received += chunk.len() as u64;
        on_progress(received, total);
    }
    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);
    tokio::fs::rename(&tmp, dest)
        .await
        .map_err(|e| e.to_string())?;
    Ok(received)
}

pub fn sha1_file(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex(&hasher.finalize()))
}

pub fn sha1_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    hex(&hasher.finalize())
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// No console window for helpers on Windows.
pub fn quiet(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = command;
    }
}

pub fn client_exe() -> &'static str {
    if cfg!(windows) {
        "horizon-client.exe"
    } else {
        "horizon-client"
    }
}

pub fn prepare_exe() -> &'static str {
    if cfg!(windows) {
        "horizon-prepare.exe"
    } else {
        "horizon-prepare"
    }
}

/// The release asset for this platform.
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub fn asset_name() -> Result<&'static str, String> {
    Ok("horizon-windows-x64.zip")
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn asset_name() -> Result<&'static str, String> {
    Ok("horizon-macos-arm64.zip")
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn asset_name() -> Result<&'static str, String> {
    Ok("horizon-linux-x86_64.tar.gz")
}

#[cfg(not(any(
    all(target_os = "windows", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64")
)))]
pub fn asset_name() -> Result<&'static str, String> {
    Err("Эта платформа не поддерживается релизами Horizon.".to_string())
}

pub fn open_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

pub fn open_url(url: &str) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("Можно открывать только http(s)-ссылки.".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

/// A junction on Windows, a symlink elsewhere: cheap, no copy of a gigabyte of assets.
pub fn link_dir(link: &Path, target: &Path) -> Result<(), String> {
    if link.exists() {
        return Ok(());
    }
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Stdio;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let status = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() && link.exists() {
            Ok(())
        } else {
            Err(format!(
                "Не удалось связать {} с {} (mklink).",
                link.display(),
                target.display()
            ))
        }
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).map_err(|e| e.to_string())
    }
}
