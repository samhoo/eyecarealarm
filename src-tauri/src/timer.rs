//! Rest timer. One background thread ticks once per second while reminders are
//! enabled and no overlay is showing. At zero: reset to the full interval,
//! skip on full-screen apps (do-not-disturb), otherwise fire the overlay.

use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::fullscreen;
use crate::overlay;

#[derive(Clone, Serialize)]
pub struct TimerState {
    pub remaining_sec: i64,
    pub today_count: u32,
    pub enabled: bool,
}

pub fn reset_remaining(state: &crate::AppState) {
    let interval = state.settings.lock().interval_min as i64;
    state.remaining_sec.store(interval * 60, Ordering::SeqCst);
}

pub fn emit_tick(app: &AppHandle) {
    let state = app.state::<crate::AppState>();
    let enabled;
    {
        let s = state.settings.lock();
        enabled = s.enabled;
    }
    let today_count = state.stats.lock().today_count();
    let _ = app.emit(
        "timer-tick",
        TimerState {
            remaining_sec: state.remaining_sec.load(Ordering::SeqCst).max(0),
            today_count,
            enabled,
        },
    );
}

pub fn spawn(app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        let state = app.state::<crate::AppState>();
        if !state.settings.lock().enabled {
            continue; // paused; remaining freezes
        }
        if state.overlay.is_active() {
            continue; // overlay owns the clock until it closes
        }
        let remaining = state.remaining_sec.fetch_sub(1, Ordering::SeqCst) - 1;
        if remaining <= 0 {
            reset_remaining(&state);
            if !fullscreen::is_other_app_fullscreen() {
                overlay::start(&app);
            }
            // full-screen → skip this round, countdown restarted above
        }
        emit_tick(&app);
    });
}
