//! Update checking against GitHub Releases.
//!
//! Manual path: `check_update` command (panel "检查更新…" row) — always runs.
//! Background path: `maybe_check_background` from overlay::start — throttled
//! to once per 24h, result persisted to update.json so the red dot survives
//! restarts. On a newer version, emits "update-available" to open panels.

use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

const RELEASES_API: &str =
    "https://api.github.com/repos/samhoo/eyecarealarm/releases/latest";
pub const RELEASES_PAGE: &str = "https://github.com/samhoo/eyecarealarm/releases/latest";
const THROTTLE: chrono::Duration = chrono::Duration::hours(24);

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Latest version seen on GitHub (e.g. "0.1.24").
    pub latest: String,
    /// RFC3339 timestamp of the last successful check.
    pub checked_at: String,
    pub update_available: bool,
}

/// Outcome reported to the manual "检查更新…" row.
#[derive(Clone, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum CheckResult {
    /// A newer release exists.
    Available { latest: String },
    /// Already on the latest version.
    Current,
    /// Network/parse failure.
    Failed,
}

fn update_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap().join("update.json")
}

pub fn load(app: &AppHandle) -> Option<UpdateInfo> {
    fs::read_to_string(update_path(app))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn save(app: &AppHandle, info: &UpdateInfo) {
    let path = update_path(app);
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(s) = serde_json::to_string_pretty(info) {
        let _ = fs::write(path, s);
    }
}

fn newer_than_current(tag: &str) -> Option<String> {
    let latest = semver::Version::parse(tag.trim_start_matches('v')).ok()?;
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).ok()?;
    (latest > current).then(|| latest.to_string())
}

/// One check round-trip. Updates state + persistence on success.
///
/// 404 semantics: the repo has no Release yet, which means nothing newer
/// exists — report Current, not Failed. Only network/parse errors are Failed.
pub fn run_check(app: &AppHandle) -> CheckResult {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .user_agent("EyeCareAlarm")
        .build()
        .into();
    let resp = agent
        .get(RELEASES_API)
        .header("Accept", "application/vnd.github+json")
        .call();

    let tag = match resp {
        Ok(mut r) => r
            .body_mut()
            .read_json::<serde_json::Value>()
            .ok()
            .and_then(|v| v.get("tag_name")?.as_str().map(str::to_string)),
        Err(ureq::Error::StatusCode(404)) => None, // no releases published yet
        Err(_) => return CheckResult::Failed,
    };

    let info = UpdateInfo {
        latest: tag
            .as_deref()
            .map(|t| t.trim_start_matches('v').to_string())
            .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()),
        checked_at: chrono::Utc::now().to_rfc3339(),
        update_available: tag.as_deref().and_then(newer_than_current).is_some(),
    };
    save(app, &info);
    {
        let state = app.state::<crate::AppState>();
        *state.update.lock() = Some(info.clone());
    }
    if let Some(latest) = tag.as_deref().and_then(newer_than_current) {
        let _ = app.emit("update-available", &info);
        CheckResult::Available { latest }
    } else {
        CheckResult::Current
    }
}

/// Background check with 24h throttle; no-op if a check ran recently.
/// Spawns a thread; never blocks the caller (overlay start path).
pub fn maybe_check_background(app: &AppHandle) {
    let fresh = app
        .state::<crate::AppState>()
        .update
        .lock()
        .as_ref()
        .and_then(|i| chrono::DateTime::parse_from_rfc3339(&i.checked_at).ok())
        .map(|t| chrono::Utc::now() - t.with_timezone(&chrono::Utc) < THROTTLE)
        .unwrap_or(false);
    if fresh {
        return;
    }
    let app2 = app.clone();
    thread::spawn(move || {
        let _ = run_check(&app2);
    });
}
