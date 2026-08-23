//! Multi-screen overlay orchestration.
//!
//! Windows are created ONCE at setup (hidden) and reused via show/hide. On
//! this wry/WebView2 build, creating a webview window after startup deadlocks
//! the main event loop (window stuck on about:blank), and destroying one
//! wedges later creation — so no window is ever created at reminder time or
//! destroyed at all.
//!
//!   8    input blocking on, audio starts, and the configured rest countdown begins
//!   8 + rest_sec  natural end: fade 1s, hide, count one rest, reset timer
//! Early exit (Esc / button → overlay_exit command) runs the same close path
//! without counting. Close is idempotent via `closing`.
//!
//! Known limitation: monitors hot-plugged after launch get no overlay until
//! the app restarts (windows are per-monitor at setup time).

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::settings;
use crate::timer;

pub const BLOCK_START_SEC: u64 = 8;
pub const FADE_MS: u64 = 1000;

/// 遮罩关闭后的三种去向：决定关闭后倒计时如何设定 + 是否置/清"二次提醒"标志。
enum CloseKind {
    /// 自然走完：计数 +1、重置整段、清二次提醒。
    Natural,
    /// 首次点击「稍后提醒」：设 2 分钟、置二次提醒。
    Snooze,
    /// 二次点击「退出」：重置整段、清二次提醒。
    Exit,
}

/// 依从性环形缓冲长度。
const ADHERENCE_MAX: usize = 50;

#[derive(Default)]
pub struct OverlayManager {
    active: AtomicBool,
    closing: AtomicBool,
    labels: Mutex<Vec<String>>,
}

impl OverlayManager {
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    fn labels(&self) -> Vec<String> {
        self.labels.lock().clone()
    }
}

#[derive(Clone, Serialize)]
struct ClosePayload {
    fade_ms: u64,
}

/// macOS: make the overlay a true full-screen shield. tao's always_on_top is
/// only NSFloatingWindowLevel (3), below the menu bar (24) — the menu bar
/// would stay visible above the veil. CGShieldingWindowLevel is the level
/// AppKit uses for display capture, covering the menu bar; CanJoinAllSpaces +
/// Stationary keep the veil present across spaces, FullScreenAuxiliary lets
/// it sit on fullscreen spaces.
#[cfg(target_os = "macos")]
fn make_shield(w: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
    if let Ok(ptr) = w.ns_window() {
        // SAFETY: tauri returns the window's live NSWindow on macOS.
        unsafe {
            let ns = &*(ptr as *const NSWindow);
            ns.setLevel(core_graphics::display::CGShieldingWindowLevel() as isize);
            ns.setCollectionBehavior(
                NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::Stationary
                    | NSWindowCollectionBehavior::FullScreenAuxiliary,
            );
        }
    }
}

fn monitor_rect(m: &tauri::Monitor) -> (f64, f64, f64, f64) {
    let scale = m.scale_factor();
    (
        m.position().x as f64 / scale,
        m.position().y as f64 / scale,
        m.size().width as f64 / scale,
        m.size().height as f64 / scale,
    )
}

/// Called once from setup: one hidden overlay window per current monitor.
pub fn create_windows(app: &AppHandle) {
    let state = app.state::<crate::AppState>();
    let monitors = app.available_monitors().unwrap_or_default();
    let mut labels = Vec::new();
    for (i, m) in monitors.iter().enumerate() {
        let label = format!("overlay-{i}");
        let (x, y, w, h) = monitor_rect(m);
        let win = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("overlay.html".into()))
            .title("EyeCareAlarm")
            .transparent(true)
            .decorations(false)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .focused(false)
            .visible(false)
            .position(x, y)
            .inner_size(w, h)
            .build();
        match win {
            Ok(w) => {
                #[cfg(target_os = "macos")]
                make_shield(&w);
                let _ = w.set_ignore_cursor_events(true);
                labels.push(label);
            }
            Err(e) => eprintln!("overlay window {label} failed: {e}"),
        }
    }
    *state.overlay.labels.lock() = labels;
}

pub fn start(app: &AppHandle) {
    let state = app.state::<crate::AppState>();
    if state.overlay.active.swap(true, Ordering::SeqCst) {
        return; // already showing
    }
    state.overlay.closing.store(false, Ordering::SeqCst);

    let (opacity, rest_sec, lang, volume, sound_id) = {
        let s = state.settings.lock();
        (
            s.overlay_opacity,
            s.rest_sec,
            s.lang.clone(),
            s.volume,
            s.sound.clone(),
        )
    };
    let app_show = app.clone();
    let _ = app.run_on_main_thread(move || {
        let state = app_show.state::<crate::AppState>();
        let labels = state.overlay.labels();
        let monitors = app_show.available_monitors().unwrap_or_default();
        if labels.is_empty() {
            state.overlay.active.store(false, Ordering::SeqCst);
            return;
        }
        // Re-align to current monitor geometry, then show click-through.
        for (i, label) in labels.iter().enumerate() {
            if let Some(w) = app_show.get_webview_window(label) {
                if let Some(m) = monitors.get(i) {
                    let (x, y, mw, mh) = monitor_rect(m);
                    let _ = w.set_position(LogicalPosition::new(x, y));
                    let _ = w.set_size(LogicalSize::new(mw, mh));
                }
                let _ = w.set_ignore_cursor_events(true);
                let _ = w.show();
            }
        }
        let _ = app_show.emit(
            "overlay-start",
            crate::OverlayPayload {
                overlay_opacity: opacity,
                rest_sec,
                lang,
            },
        );
    });

    let app2 = app.clone();
    let rest_duration = Duration::from_secs(rest_sec as u64);
    thread::spawn(move || {
        crate::update::maybe_check_background(&app2);
        thread::sleep(Duration::from_secs(BLOCK_START_SEC));
        {
            let app_block = app2.clone();
            let _ = app2.run_on_main_thread(move || {
                let state = app_block.state::<crate::AppState>();
                for label in state.overlay.labels() {
                    if let Some(w) = app_block.get_webview_window(&label) {
                        let _ = w.set_ignore_cursor_events(false);
                    }
                }
                if let Some(w) = state
                    .overlay
                    .labels()
                    .first()
                    .and_then(|l| app_block.get_webview_window(l))
                {
                    let _ = w.set_focus();
                }
                let _ = app_block.emit("overlay-input-ready", ());
            });
            let state = app2.state::<crate::AppState>();
            if let Some(audio) = &state.audio {
                if let Some(path) = settings::resolve_sound(&app2, &sound_id) {
                    audio.play(path, volume, rest_sec as u64, 2);
                }
            }
        }
        thread::sleep(rest_duration);
        close(&app2, CloseKind::Natural);
    });
}

/// 二次弹窗的 Esc / 退出按钮路径。
pub fn user_exit(app: &AppHandle) {
    if !app.state::<crate::AppState>().overlay.is_active() {
        return;
    }
    close(app, CloseKind::Exit);
}

/// 首次弹窗的 Esc / 稍后提醒按钮路径。
pub fn user_snooze(app: &AppHandle) {
    if !app.state::<crate::AppState>().overlay.is_active() {
        return;
    }
    close(app, CloseKind::Snooze);
}

/// 依从性环形缓冲：追加一条遮罩结果并裁剪到最近 50 条，持久化到 adherence.json。
fn record_adherence(app: &AppHandle, kind: &str) {
    let state = app.state::<crate::AppState>();
    let mut events = state.adherence.lock();
    events.push(kind.to_string());
    if events.len() > ADHERENCE_MAX {
        let excess = events.len() - ADHERENCE_MAX;
        events.drain(0..excess);
    }
    settings::save_adherence(app, &events);
}

/// Shared close path: fade out, hide, then apply the close-kind side effects.
/// Safe to call twice (timeline thread + early exit race).
fn close(app: &AppHandle, kind: CloseKind) {
    let state = app.state::<crate::AppState>();
    if state.overlay.closing.swap(true, Ordering::SeqCst) {
        return;
    }
    if let Some(audio) = &state.audio {
        audio.stop();
    }
    let _ = app.emit("overlay-close", ClosePayload { fade_ms: FADE_MS });
    thread::sleep(Duration::from_millis(FADE_MS));

    let labels = state.overlay.labels();
    let app_hide = app.clone();
    let _ = app.run_on_main_thread(move || {
        for label in labels {
            if let Some(w) = app_hide.get_webview_window(&label) {
                let _ = w.hide();
                // Back to click-through for the next round.
                let _ = w.set_ignore_cursor_events(true);
            }
        }
    });
    // 先改倒计时/标志，再置 active=false：避免计时线程在 active 已清、
    // 倒计时未改的瞬间用旧值（归零）触发新一轮遮罩。
    match kind {
        CloseKind::Natural => {
            let mut stats = state.stats.lock();
            stats.bump();
            settings::save_stats(app, &stats);
            record_adherence(app, "completed");
            timer::reset_remaining(&state);
        }
        CloseKind::Snooze => {
            record_adherence(app, "snoozed");
            state.remaining_sec.store(timer::SNOOZE_SEC, Ordering::SeqCst);
        }
        CloseKind::Exit => {
            record_adherence(app, "exited");
            timer::reset_remaining(&state);
        }
    }
    state.overlay.active.store(false, Ordering::SeqCst);
    timer::emit_tick(app);
}
