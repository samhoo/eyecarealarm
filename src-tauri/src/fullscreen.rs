//! Full-screen do-not-disturb detection.
//!
//! Windows: SHQueryUserNotificationState covers games (D3D full-screen),
//! presentation mode and "busy" (full-screen apps); additionally a foreground
//! window exactly covering its monitor counts (video players etc.), excluding
//! our own process and the shell.
//!
//! macOS: a frontmost-layer window from another process whose bounds cover a
//! full screen counts. [INFERENCE] Needs on-device verification; the CGWindow
//! heuristic is best-effort since macOS has no public "is fullscreen" API.

#[cfg(windows)]
pub fn is_other_app_fullscreen() -> bool {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
    use windows::Win32::UI::Shell::SHQueryUserNotificationState;
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId};

    unsafe {
        // QUNS_BUSY=2, QUNS_RUNNING_D3D_FULL_SCREEN=3,
        // QUNS_PRESENTATION_MODE=4, QUNS_QUIET_TIME=6 (Focus Assist /
        // do-not-disturb). QUNS_ACCEPTS_NOTIFICATIONS=5 is the normal
        // desktop state and must not suppress reminders.
        if let Ok(state) = SHQueryUserNotificationState() {
            if matches!(state.0, 2 | 3 | 4 | 6) {
                return true;
            }
        }

        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return false;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == std::process::id() {
            return false; // our own overlay/panel
        }
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }
        let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(hmon, &mut mi).as_bool() {
            return false;
        }
        let m = mi.rcMonitor;
        rect.left == m.left && rect.top == m.top && rect.right == m.right && rect.bottom == m.bottom
    }
}

#[cfg(target_os = "macos")]
pub fn is_other_app_fullscreen() -> bool {
    use core_graphics::display::{CGMainDisplayID, CGWindowListCopyWindowInfo, kCGNullWindowID, kCGWindowListOptionOnScreenOnly};
    use core_foundation::array::CFArrayRef;
    use core_foundation::dictionary::CFDictionaryRef;
    use std::ffi::c_void;

    unsafe {
        let list = CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, kCGNullWindowID);
        if list.is_null() {
            return false;
        }
        let array = list as CFArrayRef;
        let count = core_foundation::array::CFArrayGetCount(array);
        let screen = core_graphics::display::CGDisplay::new(CGMainDisplayID()).bounds();
        let own_pid = std::process::id() as i32;
        for i in 0..count {
            let dict = core_foundation::array::CFArrayGetValueAtIndex(array, i) as CFDictionaryRef;
            // layer 0 = normal app windows
            let layer = dict_get_i32(dict, "kCGWindowLayer").unwrap_or(-1);
            if layer != 0 {
                continue;
            }
            let pid = dict_get_i32(dict, "kCGWindowOwnerPID").unwrap_or(-1);
            if pid == own_pid || pid <= 0 {
                continue;
            }
            let Some((w, h)) = dict_get_size(dict, "kCGWindowBounds") else {
                continue;
            };
            if w >= screen.size.width - 1.0 && h >= screen.size.height - 1.0 {
                core_foundation::base::CFRelease(list as *const c_void);
                return true;
            }
        }
        core_foundation::base::CFRelease(list as *const c_void);
        false
    }
}

#[cfg(target_os = "macos")]
unsafe fn dict_get_i32(dict: core_foundation::dictionary::CFDictionaryRef, key: &str) -> Option<i32> {
    use core_foundation::base::TCFType;
    use core_foundation::number::{CFNumber, CFNumberRef};
    use core_foundation::string::CFString;
    let key = CFString::new(key);
    let v = unsafe { core_foundation::dictionary::CFDictionaryGetValue(dict, key.as_concrete_TypeRef() as *const _) };
    if v.is_null() {
        return None;
    }
    let n = unsafe { CFNumber::wrap_under_get_rule(v as CFNumberRef) };
    n.to_i32()
}

#[cfg(target_os = "macos")]
unsafe fn dict_get_size(dict: core_foundation::dictionary::CFDictionaryRef, key: &str) -> Option<(f64, f64)> {
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::string::CFString;
    let key = CFString::new(key);
    let v = unsafe { core_foundation::dictionary::CFDictionaryGetValue(dict, key.as_concrete_TypeRef() as *const _) };
    if v.is_null() {
        return None;
    }
    let d = unsafe { CFDictionary::wrap_under_get_rule(v as CFDictionaryRef) };
    let rect: core_graphics::geometry::CGRect = core_graphics::geometry::CGRect::from_dict_representation(&d)?;
    Some((rect.size.width, rect.size.height))
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn is_other_app_fullscreen() -> bool {
    false
}

/// 当前会话是否处于锁屏状态（场景 ④：冻结倒计时，不触发、不重置）。
///
/// Windows：锁屏后输入桌面（Winlogon 安全桌面）对外不可访问，
/// `OpenInputDesktop` 会因访问被拒而失败。Best-effort，探测失败按"未锁屏"。
#[cfg(windows)]
pub fn is_session_locked() -> bool {
    use windows::Win32::System::StationsAndDesktops::{
        CloseDesktop, OpenInputDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_SWITCHDESKTOP,
    };
    unsafe {
        match OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_SWITCHDESKTOP) {
            Ok(h) => {
                let _ = CloseDesktop(h);
                false
            }
            Err(_) => true,
        }
    }
}

/// macOS：会话字典的"是否在控制台会话"标志。锁屏 / 快速用户切换后本会话
/// 让出控制台，`kCGSSessionOnConsoleKey` 变 false。API 不可用时按"未锁屏"。
#[cfg(target_os = "macos")]
pub fn is_session_locked() -> bool {
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::CFDictionaryGetValue;
    use core_foundation::string::CFString;
    use std::ffi::c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGSessionCopyCurrentDictionary() -> core_foundation::dictionary::CFDictionaryRef;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFBooleanGetValue(value: *const c_void) -> u8;
    }

    unsafe {
        let dict = CGSessionCopyCurrentDictionary();
        if dict.is_null() {
            return false;
        }
        let key = CFString::new("kCGSSessionOnConsoleKey");
        let v = CFDictionaryGetValue(dict, key.as_concrete_TypeRef() as *const _);
        let locked = if v.is_null() {
            false
        } else {
            CFBooleanGetValue(v) == 0
        };
        core_foundation::base::CFRelease(dict as *const c_void);
        locked
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn is_session_locked() -> bool {
    false
}

/// 距用户最后一次键鼠输入的秒数（场景 ③：<3 秒视为"正在打字"）。
///
/// Windows：系统最后输入 tick 与当前 tick 之差。tick 为 u32，49.7 天回绕，
/// `wrapping_sub` 在回绕时仍给出正确差值。
#[cfg(windows)]
pub fn idle_seconds() -> u64 {
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    unsafe {
        let mut lii = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut lii).as_bool() {
            (GetTickCount().wrapping_sub(lii.dwTime) / 1000) as u64
        } else {
            u64::MAX
        }
    }
}

/// macOS：事件源最后输入时间（`CGEventSourceSecondsSinceLastEventType`）。
/// 该函数未在 core-graphics crate 暴露，直接链接 CoreGraphics。探测失败返回
/// 极大值（视为非打字中）。
#[cfg(target_os = "macos")]
pub fn idle_seconds() -> u64 {
    const kCGEventSourceStateCombinedSessionState: i32 = 0;
    const kCGAnyInputEventType: u32 = u32::MAX;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
    }

    unsafe {
        let secs = CGEventSourceSecondsSinceLastEventType(
            kCGEventSourceStateCombinedSessionState,
            kCGAnyInputEventType,
        );
        if secs < 0.0 {
            u64::MAX
        } else {
            secs as u64
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn idle_seconds() -> u64 {
    u64::MAX
}
