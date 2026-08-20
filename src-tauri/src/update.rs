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

const RELEASES_API: &str = "https://eca-proxy.samhoo.workers.dev/";
pub const RELEASES_PAGE: &str = "https://github.com/samhoo/eyecarealarm/releases/latest";
const THROTTLE: chrono::Duration = chrono::Duration::hours(24);

/// System HTTP proxy for the update endpoint (workers.dev is not directly
/// reachable from some networks). Order: env vars (ureq convention), then
/// OS settings (Windows registry / macOS scutil). None → direct connection.
fn system_proxy() -> Option<ureq::Proxy> {
    if let Some(p) = ureq::Proxy::try_from_env() {
        return Some(p);
    }
    platform_proxy()
}

#[cfg(windows)]
fn platform_proxy() -> Option<ureq::Proxy> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Internet Settings")
        .ok()?;
    let enabled: u32 = key.get_value("ProxyEnable").ok()?;
    if enabled == 0 {
        return None;
    }
    let server: String = key.get_value("ProxyServer").ok()?;
    let server = server.trim();
    if server.is_empty() {
        return None;
    }
    // Two shapes: "host:port" (all protocols) or per-protocol
    // "http=h:p;https=h:p;socks=h:p".
    let (scheme, addr) = if server.contains('=') {
        let mut chosen: (&str, &str) = ("http", "");
        for part in server.split(';') {
            if let Some((k, v)) = part.split_once('=') {
                match k.trim().to_ascii_lowercase().as_str() {
                    "https" | "http" => {
                        chosen = ("http", v.trim());
                        break;
                    }
                    "socks" if chosen.1.is_empty() => chosen = ("socks5", v.trim()),
                    _ => {}
                }
            }
        }
        if chosen.1.is_empty() {
            return None;
        }
        chosen
    } else {
        ("http", server)
    };
    ureq::Proxy::new(&format!("{scheme}://{addr}")).ok()
}

#[cfg(target_os = "macos")]
fn platform_proxy() -> Option<ureq::Proxy> {
    let out = std::process::Command::new("scutil").arg("--proxies").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let get = |key: &str| -> Option<String> {
        text.lines()
            .map(str::trim)
            .find(|l| l.starts_with(key))
            .and_then(|l| l.split(':').nth(1))
            .map(|v| v.trim().to_string())
    };
    let enabled = get("HTTPSEnable").or_else(|| get("HTTPEnable"))? == "1";
    if !enabled {
        return None;
    }
    let host = get("HTTPSProxy").or_else(|| get("HTTPProxy"))?;
    let port = get("HTTPSPort").or_else(|| get("HTTPPort")).unwrap_or_else(|| "8080".into());
    ureq::Proxy::new(&format!("http://{host}:{port}")).ok()
}

#[cfg(not(any(windows, target_os = "macos")))]
fn platform_proxy() -> Option<ureq::Proxy> {
    None
}

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
/// The endpoint is our own proxy service: 404 (or any HTTP error) means the
/// service is misbehaving → report Failed so the user sees "检查失败".
pub fn run_check(app: &AppHandle) -> CheckResult {
    let mut builder = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(6)))
        .user_agent("EyeCareAlarm");
    if let Some(proxy) = system_proxy() {
        builder = builder.proxy(Some(proxy));
    }
    let agent: ureq::Agent = builder.build().into();
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
        Err(_) => return CheckResult::Failed,
    };
    let Some(tag) = tag else {
        return CheckResult::Failed; // 200 but unexpected payload
    };

    let info = UpdateInfo {
        latest: tag.trim_start_matches('v').to_string(),
        checked_at: chrono::Utc::now().to_rfc3339(),
        update_available: newer_than_current(&tag).is_some(),
    };
    save(app, &info);
    {
        let state = app.state::<crate::AppState>();
        *state.update.lock() = Some(info.clone());
    }
    if let Some(latest) = newer_than_current(&tag) {
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
