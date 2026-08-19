//! Tray icon, panel window, launch toast.
//!
//! Both left and right tray clicks toggle the panel (PR requirement). The
//! panel is a borderless always-on-top window positioned next to the tray
//! icon; it hides on focus loss like a native tray flyout. The toast is a
//! small click-through strip destroyed 3s after creation.

use std::thread;
use std::time::Duration;

use tauri::image::Image;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

const PANEL_W: f64 = 360.0;

/// Hide the traffic-light buttons (close/min/zoom) of a decorated macOS
/// window — the panel wants native rounded corners + shadow but no titlebar
/// controls (design: 原生托盘弹窗，无窗口按钮).
#[cfg(target_os = "macos")]
fn hide_traffic_lights(w: &WebviewWindow) {
    use objc2_app_kit::{NSColor, NSWindow, NSWindowButton};
    if let Ok(ptr) = w.ns_window() {
        // SAFETY: tauri returns the window's live NSWindow on macOS.
        let ns = unsafe { &*(ptr as *const NSWindow) };
        for b in [
            NSWindowButton::CloseButton,
            NSWindowButton::MiniaturizeButton,
            NSWindowButton::ZoomButton,
        ] {
            if let Some(btn) = ns.standardWindowButton(b) {
                btn.setHidden(true);
            }
        }
        // 隐藏标题栏区域仍会按窗口背景色画出一条稍亮的横带；把窗口背景
        // 设为面板同色（macOS --surface #E3E2EA），整条标题带与面板融为一体。
        let surface = NSColor::colorWithSRGBRed_green_blue_alpha(
            227.0 / 255.0,
            226.0 / 255.0,
            234.0 / 255.0,
            1.0,
        );
        ns.setBackgroundColor(Some(&surface));
    }
}

/// Panel window, created hidden at setup; shown by tray clicks.
pub fn create_panel(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let builder = WebviewWindowBuilder::new(app, "panel", WebviewUrl::App("panel.html".into()))
        .title("EyeCareAlarm")
        .inner_size(PANEL_W, 560.0)
        .resizable(false)
        .shadow(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false);
    // macOS: decorated window with hidden titlebar. Native rounded corners +
    // shadow come from AppKit — a transparent borderless window would need
    // CSS-drawn corners, but this machine's WKWebView drops fully-opaque
    // background layers on transparent windows (only text/borders paint),
    // so the CSS-corner approach is not viable here.
    #[cfg(target_os = "macos")]
    let builder = builder
        .decorations(true)
        .hidden_title(true)
        // Overlay = titlebarAppearsTransparent + fullSizeContentView：去掉标题栏
        // 背景带和分隔线，内容延伸到窗口顶部（hidden_title 只藏标题文字）。
        .title_bar_style(tauri::TitleBarStyle::Overlay);
    // Windows: borderless + opaque; Win11 DWM rounds the corners and the CSS
    // 1px border reads like the design's edge stroke.
    #[cfg(not(target_os = "macos"))]
    let builder = builder.decorations(false);
    let panel = builder.build()?;
    #[cfg(target_os = "macos")]
    hide_traffic_lights(&panel);

    let p = panel.clone();
    panel.on_window_event(move |e| {
        if let WindowEvent::Focused(false) = e {
            let _ = p.hide();
        }
    });
    Ok(panel)
}

pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;
    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("EyeCareAlarm")
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            let TrayIconEvent::Click {
                button,
                button_state,
                position,
                rect,
                ..
            } = event
            else {
                return;
            };
            if button_state != MouseButtonState::Up {
                return;
            }
            if !matches!(button, MouseButton::Left | MouseButton::Right) {
                return;
            }
            toggle_panel(tray.app_handle(), position.x, position.y, rect);
        })
        .build(app)?;
    Ok(())
}

fn toggle_panel(app: &AppHandle, tray_x: f64, tray_y: f64, icon_rect: tauri::Rect) {
    let _ = &icon_rect; // only macOS uses the icon rect for alignment
    let Some(panel) = app.get_webview_window("panel") else {
        return;
    };
    if panel.is_visible().unwrap_or(false) {
        let _ = panel.hide();
        return;
    }
    let size = panel.outer_size().unwrap_or(tauri::PhysicalSize::new(320, 560));
    let (mut x, mut y) = (tray_x - size.width as f64 / 2.0, 0.0);

    if let Ok(Some(mon)) = app.monitor_from_point(tray_x, tray_y) {
        let mp = mon.position();
        let ms = mon.size();
        #[cfg(target_os = "macos")]
        {
            // macOS 菜单栏恒在屏幕顶部（24pt，与点击位置无关）：面板紧贴菜单栏
            // 下沿弹出，像原生菜单一样，而不是按点击 y 偏移（会留出飘变的缝隙）。
            let scale = mon.scale_factor();
            y = mp.y as f64 + 24.0 * scale;
            // 水平方向与原生菜单栏面板一致——优先左对齐（面板左缘 = 图标左缘）；
            // 右侧空间不足则与图标居中对齐；仍不足则右对齐（面板右缘 = 图标右缘）；
            // 最后兜底为屏幕内夹取。
            let panel_w = size.width as f64;
            let icon_pos = icon_rect.position.to_physical::<f64>(scale);
            let icon_size = icon_rect.size.to_physical::<f64>(scale);
            let icon_left = icon_pos.x;
            let icon_right = icon_left + icon_size.width;
            let screen_left = mp.x as f64;
            let screen_right = screen_left + ms.width as f64;
            let fits = |left: f64| left >= screen_left + 4.0 && left + panel_w <= screen_right - 4.0;
            let x_left = icon_left;
            let x_center = icon_left + (icon_size.width - panel_w) / 2.0;
            let x_right = icon_right - panel_w;
            x = if fits(x_left) {
                x_left
            } else if fits(x_center) {
                x_center
            } else if fits(x_right) {
                x_right
            } else {
                (screen_right - panel_w - 4.0).max(screen_left + 4.0)
            };
        }
        #[cfg(not(target_os = "macos"))]
        {
            let upper_half = tray_y < mp.y as f64 + ms.height as f64 / 2.0;
            y = if upper_half {
                tray_y + 24.0 // top taskbar: drop below the icon
            } else {
                tray_y - size.height as f64 - 8.0 // bottom taskbar: pop above the icon
            };
            y = y.clamp(mp.y as f64 + 4.0, mp.y as f64 + ms.height as f64 - size.height as f64 - 4.0);
        }
        // clamp into the monitor
        x = x.clamp(mp.x as f64 + 4.0, mp.x as f64 + ms.width as f64 - size.width as f64 - 4.0);
    }
    let _ = panel.set_position(PhysicalPosition::new(x as i32, y as i32));
    let _ = panel.show();
    let _ = panel.set_focus();
}

/// "护眼提醒已启动" strip; hidden 3s later. Used at first launch and by
/// second instances (which exit right after).
///
/// The window is created once and reused: on this wry/WebView2 build,
/// destroying a webview window wedges later window creation (deadlocks the
/// main event loop), so windows are never destroyed, only hidden.
pub fn show_toast(app: &AppHandle) {
    let (w, h) = (280.0, 56.0);
    let (mut x, mut y) = (100.0, 64.0);
    if let Ok(Some(mon)) = app.primary_monitor() {
        let scale = mon.scale_factor();
        let mp = mon.position();
        let ms = mon.size();
        x = mp.x as f64 / scale + (ms.width as f64 / scale - w) / 2.0;
        y = mp.y as f64 / scale + 64.0;
    }
    if let Some(existing) = app.get_webview_window("toast") {
        let _ = existing.set_position(tauri::LogicalPosition::new(x, y));
        let _ = existing.show();
    } else {
        let built = WebviewWindowBuilder::new(app, "toast", WebviewUrl::App("toast.html".into()))
            .title("")
            .inner_size(w, h)
            .position(x, y)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .focused(false)
            .build();
        if built.is_ok() {
            if let Some(tw) = app.get_webview_window("toast") {
                let _ = tw.set_ignore_cursor_events(true);
            }
        }
    }
    let handle = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));
        let handle2 = handle.clone();
        let _ = handle.run_on_main_thread(move || {
            if let Some(w) = handle2.get_webview_window("toast") {
                let _ = w.hide();
            }
        });
    });
}
