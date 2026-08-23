//! Settings + daily-stats persistence, sound library scanning.
//! JSON files in the app data dir: settings.json, stats.json.
//! Built-in sounds live in the resource dir (`sound/`), user imports in `<data>/sound/`.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::sound;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub enabled: bool,
    pub interval_min: u32,
    pub rest_sec: u32,
    pub overlay_opacity: u32,
    pub volume: u32,
    pub sound: String,
    pub lang: String,
    pub policy: String,
    pub autostart: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_min: 20,
            rest_sec: 20,
            overlay_opacity: 20,
            volume: 30,
            sound: "singing-bowl-deep-sound".into(),
            lang: crate::detect_lang(),
            policy: "standard".into(),
            autostart: true,
        }
    }
}

impl Settings {
    pub fn clamped(mut self) -> Self {
        const LANGS: [&str; 10] =
            ["zh-CN", "zh-TW", "en", "pt", "es", "ru", "fr", "ko", "de", "ja"];
        const POLICIES: [&str; 3] = ["gentle", "standard", "strict"];
        self.interval_min = self.interval_min.clamp(1, 120);
        self.rest_sec = self.rest_sec.clamp(5, 60);
        self.overlay_opacity = self.overlay_opacity.min(100);
        self.volume = self.volume.min(100);
        if !LANGS.contains(&self.lang.as_str()) {
            self.lang = crate::detect_lang();
        }
        if !POLICIES.contains(&self.policy.as_str()) {
            self.policy = "standard".into();
        }
        self
    }
}

/// Partial patch from the frontend; every field optional.
#[derive(Clone, Deserialize, Default)]
pub struct SettingsPatch {
    pub enabled: Option<bool>,
    pub interval_min: Option<u32>,
    pub rest_sec: Option<u32>,
    pub overlay_opacity: Option<u32>,
    pub volume: Option<u32>,
    pub sound: Option<String>,
    pub lang: Option<String>,
    pub policy: Option<String>,
    pub autostart: Option<bool>,
}

impl Settings {
    pub fn apply(&mut self, p: SettingsPatch) {
        if let Some(v) = p.enabled {
            self.enabled = v;
        }
        if let Some(v) = p.interval_min {
            self.interval_min = v;
        }
        if let Some(v) = p.rest_sec {
            self.rest_sec = v;
        }
        if let Some(v) = p.overlay_opacity {
            self.overlay_opacity = v;
        }
        if let Some(v) = p.volume {
            self.volume = v;
        }
        if let Some(v) = p.sound {
            self.sound = v;
        }
        if let Some(v) = p.lang {
            self.lang = v;
        }
        if let Some(v) = p.policy {
            self.policy = v;
        }
        if let Some(v) = p.autostart {
            self.autostart = v;
        }
        *self = std::mem::take(self).clamped();
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    /// Local date `YYYY-MM-DD`; count resets when the day rolls over.
    pub date: String,
    pub count: u32,
}

impl Stats {
    pub fn today_count(&self) -> u32 {
        if self.date == today() {
            self.count
        } else {
            0
        }
    }

    pub fn bump(&mut self) {
        let t = today();
        if self.date != t {
            self.date = t;
            self.count = 0;
        }
        self.count += 1;
    }
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn settings_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap().join("settings.json")
}

fn stats_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap().join("stats.json")
}

fn read_json<T: for<'de> Deserialize<'de> + Default>(path: PathBuf) -> T {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_json<T: Serialize>(path: PathBuf, value: &T) {
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(s) = serde_json::to_string_pretty(value) {
        let _ = fs::write(path, s);
    }
}

pub fn load_settings(app: &AppHandle) -> Settings {
    read_json::<Settings>(settings_path(app)).clamped()
}

pub fn save_settings(app: &AppHandle, s: &Settings) {
    write_json(settings_path(app), s);
}

pub fn load_stats(app: &AppHandle) -> Stats {
    read_json(stats_path(app))
}

pub fn save_stats(app: &AppHandle, s: &Stats) {
    write_json(stats_path(app), s);
}

fn adherence_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap().join("adherence.json")
}

/// 依从性环形缓冲：每次遮罩结果（"completed" / "exited" / "snoozed"）按时间
/// 顺序追加，最近 50 条，最新在末尾。
pub fn load_adherence(app: &AppHandle) -> Vec<String> {
    read_json::<Vec<String>>(adherence_path(app))
}

pub fn save_adherence(app: &AppHandle, events: &Vec<String>) {
    write_json(adherence_path(app), events);
}

/// Directory holding user-imported sounds. Created on demand.
pub fn user_sound_dir(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_data_dir().unwrap().join("sound");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Directory holding built-in sounds. Release: bundled resources. Dev: the
/// bundler does not copy `../` resources, so fall back to the source dirs
/// (synced copy, then the canonical public/sound).
pub fn builtin_sound_dir(app: &AppHandle) -> PathBuf {
    if let Ok(dir) = app.path().resource_dir() {
        let candidate = dir.join("sound");
        if candidate.is_dir() {
            return candidate;
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let synced = manifest.join("sound");
    if synced.is_dir() {
        return synced;
    }
    manifest.join("../public/sound")
}

/// Resolve a sound id to a playable file. `id == "none"` → None.
/// User dir wins on name collision. Unknown id falls back to the first
/// available sound (settings may reference a since-deleted file).
pub fn resolve_sound(app: &AppHandle, id: &str) -> Option<PathBuf> {
    if id == "none" {
        return None;
    }
    let all = sound::scan(app);
    let hit = all
        .iter()
        .find(|s| s.id == id)
        .or_else(|| all.first())
        .cloned()?;
    Some(hit.path)
}
