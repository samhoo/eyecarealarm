//! Rest timer. One background thread ticks once per second while reminders are
//! enabled, the session is unlocked and no overlay is showing. At zero the
//! `due_action` decision runs: fire, defer (2-minute recheck) or reset the full
//! interval — per the configured policy (gentle / standard / strict).

use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::fullscreen;
use crate::overlay;

/// 坏时机推迟步进（无硬截止，每次归零再查）。
pub const SNOOZE_SEC: i64 = 2 * 60;
/// "正在打字"判定阈值：距最后输入 < 3 秒。
pub const TYPING_THRESHOLD_SEC: u64 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DueAction {
    /// 立即触发遮罩。
    Fire,
    /// 推迟重查：倒计时设 2 分钟，下次归零再查。
    Defer,
    /// 跳过本次：倒计时重置为完整间隔。
    Reset,
}

/// 到点决策纯函数（锁屏不进入此处）。档位非法按温和处理。
pub fn due_action(policy: &str, fullscreen: bool, dialog: bool, typing: bool) -> DueAction {
    match policy {
        "standard" => {
            if dialog {
                DueAction::Defer
            } else if fullscreen {
                DueAction::Reset
            } else {
                DueAction::Fire
            }
        }
        "strict" => DueAction::Fire,
        _ => {
            // gentle（含非法档位）：全屏/勿扰 → 跳过重置；打字/对话框 → 推迟 2 分钟。
            if fullscreen {
                DueAction::Reset
            } else if dialog || typing {
                DueAction::Defer
            } else {
                DueAction::Fire
            }
        }
    }
}

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
    thread::spawn(move || {
        // 记录是否处于「锁屏/全屏/勿扰」暂停中：解除的下一 tick 重置整段重新计时。
        let mut was_suspended = false;
        loop {
            thread::sleep(Duration::from_secs(1));
            let state = app.state::<crate::AppState>();
            if !state.settings.lock().enabled {
                continue; // paused; remaining freezes
            }
            // 锁屏（三档统一）与全屏/勿扰（仅温和/标准）：暂停倒计时，
            // 不递减、不触发、不广播；解除后重置整段重新计时。
            let policy_is_strict = state.settings.lock().policy == "strict";
            if fullscreen::is_session_locked()
                || (!policy_is_strict && fullscreen::is_other_app_fullscreen())
            {
                was_suspended = true;
                continue;
            }
            if was_suspended {
                was_suspended = false;
                reset_remaining(&state);
                emit_tick(&app);
                continue;
            }
            if state.overlay.is_active() {
                continue; // overlay owns the clock until it closes
            }
            let remaining = state.remaining_sec.fetch_sub(1, Ordering::SeqCst) - 1;
            if remaining <= 0 {
                let policy = state.settings.lock().policy.clone();
                let dialog = state.dialog_open.load(Ordering::SeqCst);
                let fullscreen_now = fullscreen::is_other_app_fullscreen();
                let typing = fullscreen::idle_seconds() < TYPING_THRESHOLD_SEC;
                match due_action(&policy, fullscreen_now, dialog, typing) {
                    DueAction::Fire => overlay::start(&app),
                    DueAction::Defer => {
                        state.remaining_sec.store(SNOOZE_SEC, Ordering::SeqCst);
                    }
                    DueAction::Reset => reset_remaining(&state),
                }
            }
            emit_tick(&app);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gentle_resets_fullscreen_defers_typing_and_dialog() {
        assert_eq!(due_action("gentle", true, false, false), DueAction::Reset);
        assert_eq!(due_action("gentle", false, true, false), DueAction::Defer);
        assert_eq!(due_action("gentle", false, false, true), DueAction::Defer);
        assert_eq!(due_action("gentle", false, false, false), DueAction::Fire);
    }

    #[test]
    fn standard_resets_fullscreen_defers_dialog_ignores_typing() {
        assert_eq!(due_action("standard", true, false, false), DueAction::Reset);
        assert_eq!(due_action("standard", false, true, false), DueAction::Defer);
        assert_eq!(due_action("standard", false, false, true), DueAction::Fire);
        assert_eq!(due_action("standard", false, false, false), DueAction::Fire);
    }

    #[test]
    fn strict_always_fires() {
        assert_eq!(due_action("strict", true, false, false), DueAction::Fire);
        assert_eq!(due_action("strict", false, true, false), DueAction::Fire);
        assert_eq!(due_action("strict", false, false, true), DueAction::Fire);
        assert_eq!(due_action("strict", false, false, false), DueAction::Fire);
    }

    #[test]
    fn unknown_policy_falls_back_to_gentle() {
        assert_eq!(due_action("bogus", true, false, false), DueAction::Reset);
        assert_eq!(due_action("bogus", false, false, false), DueAction::Fire);
    }
}
