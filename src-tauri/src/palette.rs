//! Palette window control + global hotkey handler.
//! See SPEC §4.9 (window mutex) and ARCHITECTURE §6 (HWND capture timing).

use crate::state::AppState;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

const MAIN_WINDOW_GLOW_EVENT: &str = "main-window-glow";
const PALETTE_SHOWN_EVENT: &str = "palette-shown";

/// Hotkey callback. ARCHITECTURE §6 timing: HWND capture must be the very
/// first synchronous step before any window check or UI dispatch — otherwise
/// `GetForegroundWindow` returns the palette itself.
pub fn on_hotkey(app: &AppHandle) {
    // 1. Capture HWND immediately.
    let hwnd = capture_foreground_hwnd();
    if let Some(state) = app.try_state::<AppState>() {
        state.cached_hwnd.store(hwnd, Ordering::Relaxed);
    }

    // 2. Window mutex (SPEC §4.9):
    //    - main visible → focus main + glow, do NOT show palette
    //    - palette already visible → just refocus, do NOT reset state
    //    - else → show palette + emit `palette-shown` (frontend resets)
    let main_visible = app
        .get_webview_window("main")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);

    if main_visible {
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.unminimize();
            let _ = main.set_focus();
            let _ = app.emit(MAIN_WINDOW_GLOW_EVENT, ());
        }
        return;
    }

    let palette_visible = app
        .get_webview_window("palette")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);

    if palette_visible {
        if let Some(palette) = app.get_webview_window("palette") {
            let _ = palette.set_focus();
        }
    } else {
        show_palette(app);
    }
}

pub fn show_palette(app: &AppHandle) {
    if let Some(palette) = app.get_webview_window("palette") {
        let _ = palette.show();
        let _ = palette.set_focus();
        // Tell the palette frontend to reset (search query empty, view = search).
        let _ = app.emit_to("palette", PALETTE_SHOWN_EVENT, ());
        info!("palette shown");
    } else {
        warn!("palette window not found");
    }
}

pub fn hide_palette(app: &AppHandle) {
    if let Some(palette) = app.get_webview_window("palette") {
        let _ = palette.hide();
    }
}

/// Show main window — and per SPEC §4.9 mutex, hide palette first.
pub fn show_main_window(app: &AppHandle) {
    hide_palette(app);
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.unminimize();
        let _ = main.set_focus();
        info!("main window shown");
    } else {
        warn!("main window not found");
    }
}

#[cfg(target_os = "windows")]
fn capture_foreground_hwnd() -> isize {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    let hwnd = unsafe { GetForegroundWindow() };
    hwnd.0 as isize
}

#[cfg(not(target_os = "windows"))]
fn capture_foreground_hwnd() -> isize {
    0
}
