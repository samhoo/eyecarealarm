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
        // QUNS_BUSY=2, QUNS_RUNNING_D3D_FULL_SCREEN=3, QUNS_PRESENTATION_MODE=5
        if let Ok(state) = SHQueryUserNotificationState() {
            if matches!(state.0, 2 | 3 | 5) {
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
    use core_graphics::display::{CGWindowListCopyWindowInfo, kCGWindowListOptionOnScreenOnly, kCGNullWindowID};
    use core_foundation::array::CFArrayRef;
    use core_foundation::dictionary::CFDictionaryRef;
    use core_foundation::number::CFNumberRef;
    use core_foundation::string::CFStringRef;
    use std::ffi::c_void;

    unsafe {
        let list = CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, kCGNullWindowID);
        if list.is_null() {
            return false;
        }
        let array = list as CFArrayRef;
        let count = core_foundation::array::CFArrayGetCount(array);
        let screen = core_graphics::display::CGMainDisplayID().bounds();
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
    let rect: core_graphics::geometry::CGRect = core_graphics::geometry::CGRect::from_dictionary_representation(&d)?;
    Some((rect.size.width, rect.size.height))
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn is_other_app_fullscreen() -> bool {
    false
}
