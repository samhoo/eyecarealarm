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
use std::sync::atomic::{AtomicBool, AtomicI64};
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
    /// 文件对话框（「+ 我的音效」）打开期间为 true：面板让出 key window
    /// 是预期行为，此时不应触发失焦自动隐藏。
    pub dialog_open: AtomicBool,
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
            dialog_open: AtomicBool::new(false),
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
    let l = sys_locale::get_locale().unwrap_or_default().to_lowercase();
    if l.starts_with("zh") {
        return if l.contains("tw") || l.contains("hk") || l.contains("hant") {
            "zh-TW".into()
        } else {
            "zh-CN".into()
        };
    }
    for lang in ["pt", "es", "ru", "fr", "ko", "de", "ja"] {
        if l.starts_with(lang) {
            return lang.into();
        }
    }
    "en".into()
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
async fn import_sound(app: AppHandle) -> Option<Vec<SoundInfo>> {
    // 不能用 blocking_pick_file：tauri 命令在非主线程执行，而 rfd 的同步
    // 阻塞对话框在 macOS 上只能在主线程运行——对话框弹出后 AppKit 事件循环
    // 被卡死，整个 app 无响应。改用回调式异步对话框 + channel 等待结果。
    let (tx, mut rx) = tauri::async_runtime::channel(1);
    let mut dlg = app
        .dialog()
        .file()
        .add_filter("Audio", &["wav", "mp3", "ogg", "flac"]);
    // 挂到面板窗口上：macOS 以 sheet 形式附着在面板（独立窗口会跑到主桌面
    // Space——用户在全屏 Space 里看不到也点不到）；挂接后面板让出 key window
    // 属预期，打开期间暂停面板的失焦自动隐藏。
    let panel = app.get_webview_window("panel");
    if let Some(p) = &panel {
        dlg = dlg.set_parent(p);
    }
    app.state::<AppState>()
        .dialog_open
        .store(true, std::sync::atomic::Ordering::SeqCst);
    dlg.pick_file(move |picked| {
        let _ = tx.try_send(picked);
    });
    let picked = rx.recv().await.flatten();
    app.state::<AppState>()
        .dialog_open
        .store(false, std::sync::atomic::Ordering::SeqCst);
    if let Some(p) = &panel {
        // 恢复面板 key 状态，之后点击他处才能正常触发失焦隐藏。
        let _ = p.set_focus();
    }
    let picked = picked?;
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

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
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
        .build(tauri::generate_context!())
        .expect("error while building EyeCareAlarm");
    app.run(|handle, event| {
        // macOS 重复启动一个运行中的 app 不会产生第二个进程（LaunchServices
        // 只向已运行实例发 reopen 事件），单实例锁的第二进程路径走不到。
        // 在已运行实例收到 reopen 时补弹「护眼提醒已启动」，与 Windows 对齐。
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = event {
            tray::show_toast(handle);
        }
        #[cfg(not(target_os = "macos"))]
        let _ = (handle, event);
    });
}
