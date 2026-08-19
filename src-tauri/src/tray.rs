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

/// Panel window, created hidden at setup; shown by tray clicks.
pub fn create_panel(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let panel = WebviewWindowBuilder::new(app, "panel", WebviewUrl::App("panel.html".into()))
        .title("EyeCareAlarm")
        .inner_size(PANEL_W, 560.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()?;

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
            toggle_panel(tray.app_handle(), position.x, position.y);
        })
        .build(app)?;
    Ok(())
}

fn toggle_panel(app: &AppHandle, tray_x: f64, tray_y: f64) {
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
        let upper_half = tray_y < mp.y as f64 + ms.height as f64 / 2.0;
        y = if upper_half {
            tray_y + 24.0 // macOS menu bar: drop below the icon
        } else {
            tray_y - size.height as f64 - 8.0 // taskbar: pop above the icon
        };
        // clamp into the monitor
        x = x.clamp(mp.x as f64 + 4.0, mp.x as f64 + ms.width as f64 - size.width as f64 - 4.0);
        y = y.clamp(mp.y as f64 + 4.0, mp.y as f64 + ms.height as f64 - size.height as f64 - 4.0);
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
