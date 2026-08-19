// Release: GUI subsystem, no console window (a stray console would kill the
// app when closed). Debug keeps the console for [audio]/eprintln logs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod fullscreen;
mod overlay;
mod settings;
mod sound;
mod timer;
mod tray;

use std::fs::File;
use std::sync::atomic::AtomicI64;
use std::thread;
use std::time::Duration;

use fs2::FileExt;
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_dialog::DialogExt;

use settings::{Settings, SettingsPatch, Stats};
use sound::SoundInfo;

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub stats: Mutex<Stats>,
    pub remaining_sec: AtomicI64,
    pub overlay: overlay::OverlayManager,
    /// None on machines without an audio device; playback is then a no-op.
    pub audio: Option<std::sync::Arc<audio::Audio>>,
}

impl AppState {
    fn new(app: &AppHandle) -> Self {
        let s = settings::load_settings(app);
        let remaining = s.interval_min as i64 * 60;
        let audio = audio::Audio::new().map(std::sync::Arc::new);
        if audio.is_none() {
            eprintln!("[audio] no default output device; playback disabled");
        }
        Self {
            settings: Mutex::new(s),
            stats: Mutex::new(settings::load_stats(app)),
            remaining_sec: AtomicI64::new(remaining),
            overlay: Default::default(),
            audio,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct OverlayPayload {
    pub overlay_opacity: u32,
    pub rest_sec: u32,
    pub lang: String,
}

pub fn detect_lang() -> String {
    match sys_locale::get_locale() {
        Some(l) if l.to_lowercase().starts_with("zh") => "zh-CN".into(),
        _ => "en".into(),
    }
}

// ---------- commands ----------

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Settings {
    state.settings.lock().clone()
}

#[tauri::command]
fn update_settings(app: AppHandle, patch: SettingsPatch) -> Settings {
    let state = app.state::<AppState>();
    let (interval_changed, enable_changed, snapshot) = {
        let mut s = state.settings.lock();
        let old_interval = s.interval_min;
        let old_enabled = s.enabled;
        s.apply(patch);
        settings::save_settings(&app, &s);
        apply_autostart(&app, s.autostart);
        let _ = app.emit("settings-changed", s.clone());
        (
            s.interval_min != old_interval,
            s.enabled != old_enabled,
            s.clone(),
        )
    };
    if interval_changed || enable_changed {
        timer::reset_remaining(&state);
    }
    timer::emit_tick(&app);
    snapshot
}

#[tauri::command]
fn get_timer_state(app: AppHandle) -> timer::TimerState {
    let state = app.state::<AppState>();
    let today_count = state.stats.lock().today_count();
    let enabled = state.settings.lock().enabled;
    timer::TimerState {
        remaining_sec: state.remaining_sec.load(std::sync::atomic::Ordering::SeqCst).max(0),
        today_count,
        enabled,
    }
}

#[tauri::command]
fn remind_now(app: AppHandle) {
    let state = app.state::<AppState>();
    timer::reset_remaining(&state);
    overlay::start(&app);
}

#[tauri::command]
fn list_sounds(app: AppHandle) -> Vec<SoundInfo> {
    sound::scan(&app)
}

#[tauri::command]
fn import_sound(app: AppHandle) -> Option<Vec<SoundInfo>> {
    let picked = app
        .dialog()
        .file()
        .add_filter("Audio", &["wav", "mp3", "ogg", "flac"])
        .blocking_pick_file()?;
    let src = picked.as_path()?.to_path_buf();
    let name = src.file_name()?.to_os_string();
    let dst = settings::user_sound_dir(&app).join(name);
    std::fs::copy(&src, &dst).ok()?;
    Some(sound::scan(&app))
}

#[tauri::command]
fn preview_sound(app: AppHandle, id: String) {
    let state = app.state::<AppState>();
    let Some(audio) = &state.audio else { return };
    let volume = state.settings.lock().volume;
    // PR: with no sound configured, preview falls back to the first option.
    let path = if id == "none" {
        sound::scan(&app).first().map(|s| s.path.clone())
    } else {
        settings::resolve_sound(&app, &id)
    };
    if let Some(path) = path {
        audio.play(path, volume, 5, 1);
    }
}

#[tauri::command]
fn overlay_exit(app: AppHandle) {
    overlay::user_exit(&app);
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

// ---------- helpers ----------

fn apply_autostart(app: &AppHandle, on: bool) {
    let mgr = app.autolaunch();
    let enabled = mgr.is_enabled().unwrap_or(false);
    if on && !enabled {
        let _ = mgr.enable();
    } else if !on && enabled {
        let _ = mgr.disable();
    }
}

/// Cross-platform single-instance guard: exclusive lock on a temp file.
/// First instance holds the lock for the process lifetime; a second instance
/// fails to lock and takes the toast-then-exit path.
fn acquire_single_instance() -> Option<File> {
    let path = std::env::temp_dir().join("EyeCareAlarm.lock");
    let file = File::create(path).ok()?;
    file.try_lock_exclusive().ok()?;
    Some(file)
}

fn main() {
    let lock = acquire_single_instance();
    let first_instance = lock.is_some();
    // The lock file handle must outlive main; leaking is intentional.
    std::mem::forget(lock);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            update_settings,
            get_timer_state,
            remind_now,
            list_sounds,
            import_sound,
            preview_sound,
            overlay_exit,
            quit_app,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            #[cfg(target_os = "macos")]
            let _ = handle.set_activation_policy(tauri::ActivationPolicy::Accessory);

            if !first_instance {
                // Second instance: show "护眼提醒已启动" for 3s, then exit.
                tray::show_toast(&handle);
                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(3));
                    handle.exit(0);
                });
                return Ok(());
            }

            let state = AppState::new(&handle);
            let autostart = state.settings.lock().autostart;
            handle.manage(state);

            tray::create_panel(&handle)?;
            tray::build_tray(&handle)?;
            overlay::create_windows(&handle);
            apply_autostart(&handle, autostart);
            tray::show_toast(&handle);
            timer::spawn(handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running EyeCareAlarm");
}
