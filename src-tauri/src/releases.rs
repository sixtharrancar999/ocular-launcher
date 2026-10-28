//! Horizon's published releases, read from GitHub (and cached for a few minutes).

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const REPO: &str = "horizon-rust/horizon";
const CACHE_TTL_SECS: u64 = 600;

/// Ocular's own releases (used only to hide the game's page instead of installing it).
const LAUNCHER_REPO: &str = "sixtharrancar999/ocular-launcher";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub tag: String,
    pub name: String,
    pub published_at: String,
    pub body: String,
    pub prerelease: bool,
    pub size: u64,
    pub download_url: String,
}

#[derive(Serialize, Deserialize)]
struct Cached {
    at: u64,
    releases: Vec<Release>,
}

/// Game tags are `vX.Y.Z`; `launcher@…` and `nightly…` are not installed here.
pub fn is_game_tag(tag: &str) -> bool {
    tag.strip_prefix('v')
        .and_then(|rest| rest.chars().next())
        .is_some_and(|c| c.is_ascii_digit())
}

/// `cache/horizon-tags.json`: every game tag seen, so a tag that also exists in
/// Ocular's own repository is never mistaken for a Horizon release.
#[tauri::command]
pub fn known_game_tags() -> Vec<String> {
    std::fs::read_to_string(knowledge_path())
        .ok()
        .and_then(|text| serde_json::from_str::<Vec<String>>(&text).ok())
        .unwrap_or_default()
}

fn knowledge_path() -> std::path::PathBuf {
    crate::paths::cache_dir().join("horizon-tags.json")
}

fn remember_game_tags(tags: &[String]) {
    let mut known = known_game_tags();
    let mut changed = false;
    for tag in tags {
        if !known.contains(tag) {
            known.push(tag.clone());
            changed = true;
        }
    }
    if changed {
        if let Ok(text) = serde_json::to_string(&known) {
            let _ = std::fs::write(knowledge_path(), text);
        }
    }
}

/// The tags of this repository's own releases (the launcher's, not Horizon's).
async fn launcher_tags() -> Vec<String> {
    let url = format!("https://api.github.com/repos/{LAUNCHER_REPO}/releases?per_page=100");
    let Ok(response) = crate::util::client()
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
    else {
        return Vec::new();
    };
    let Ok(value) = response.json::<serde_json::Value>().await else {
        return Vec::new();
    };
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item["tag_name"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub fn validate_tag(tag: &str) -> Result<(), String> {
    let valid_chars = tag
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if tag.is_empty() || tag.len() > 64 || !valid_chars || !is_game_tag(tag) {
        return Err("Некорректный тег релиза.".to_string());
    }
    Ok(())
}

pub fn version_key(tag: &str) -> (u64, u64, u64, u64) {
    let mut parts = tag
        .trim_start_matches('v')
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

pub fn download_url(tag: &str) -> Result<String, String> {
    validate_tag(tag)?;
    Ok(format!(
        "https://github.com/{REPO}/releases/download/{tag}/{}",
        crate::util::asset_name()?
    ))
}

pub async fn fetch(force: bool) -> Result<Vec<Release>, String> {
    if !force {
        if let Some(cached) = read_cache() {
            if now() - cached.at < CACHE_TTL_SECS && !cached.releases.is_empty() {
                return Ok(cached.releases);
            }
        }
    }

    let url = format!("https://api.github.com/repos/{REPO}/releases?per_page=100");
    let response = crate::util::client()
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("GitHub недоступен: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("GitHub ответил HTTP {}: {url}", response.status()));
    }
    let value: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Неожиданный ответ GitHub: {e}"))?;
    let Some(items) = value.as_array() else {
        return Err("Неожиданный ответ GitHub: это не список релизов.".to_string());
    };
    let asset_name = crate::util::asset_name()?;
    let own_tags: Vec<String> = launcher_tags().await;
    let known: Vec<String> = known_game_tags();

    let mut releases = Vec::new();
    let mut game_tags = Vec::new();
    for item in items {
        if item["draft"].as_bool().unwrap_or(false) {
            continue;
        }
        let tag = item["tag_name"].as_str().unwrap_or_default();
        if !is_game_tag(tag) {
            continue;
        }
        game_tags.push(tag.to_string());
        let Some(asset) = item["assets"]
            .as_array()
            .and_then(|assets| assets.iter().find(|asset| asset["name"].as_str() == Some(asset_name)))
        else {
            continue;
        };
        // Ocular tags its own releases the same way; one that exists in both places is only
        // the game's release when the game was seen publishing it.
        let own = own_tags.iter().any(|own| own == tag)
            && !known.iter().any(|seen| seen == tag);
        if own {
            continue;
        }
        releases.push(Release {
            tag: tag.to_string(),
            name: item["name"].as_str().unwrap_or(tag).to_string(),
            published_at: item["published_at"].as_str().unwrap_or_default().to_string(),
            body: item["body"].as_str().unwrap_or_default().to_string(),
            prerelease: item["prerelease"].as_bool().unwrap_or(false),
            size: asset["size"].as_u64().unwrap_or(0),
            download_url: asset["browser_download_url"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        });
    }
    releases.sort_by_key(|release| std::cmp::Reverse(version_key(&release.tag)));

    remember_game_tags(&game_tags);
    write_cache(&releases);
    Ok(releases)
}

fn read_cache() -> Option<Cached> {
    let bytes = std::fs::read(crate::paths::releases_cache_path()).ok()?;
    serde_json::from_slice::<Cached>(&bytes).ok()
}

fn write_cache(releases: &[Release]) {
    let cached = Cached {
        at: now(),
        releases: releases.to_vec(),
    };
    if let Ok(text) = serde_json::to_string(&cached) {
        let _ = std::fs::write(crate::paths::releases_cache_path(), text);
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::{is_game_tag, validate_tag, version_key};

    #[test]
    fn only_game_releases_count() {
        assert!(is_game_tag("v0.2.93"));
        assert!(!is_game_tag("launcher@0.2.20"));
        assert!(!is_game_tag("nightly-2026-10-28"));
        assert!(!is_game_tag("vfoo"));
    }

    #[test]
    fn versions_sort_numerically() {
        assert!(version_key("v0.2.10") > version_key("v0.2.9"));
        assert!(version_key("v0.3.0") > version_key("v0.2.99"));
        assert!(version_key("v1.0.0") > version_key("v0.99.99"));
    }

    #[test]
    fn tags_are_sanitised_before_they_reach_a_path() {
        assert!(validate_tag("v0.2.93").is_ok());
        assert!(validate_tag("../../etc").is_err());
        assert!(validate_tag("v1/2").is_err());
        assert!(validate_tag("launcher@1.0.0").is_err());
        assert!(validate_tag("").is_err());
    }
}
