//! Auto-paste: focus the cached foreground HWND and simulate Ctrl+V.
//! Windows-only; other platforms return an error so the caller falls back
//! to clipboard-only (SPEC §4.6 / §8.2).

#[cfg(target_os = "windows")]
pub fn paste_into(hwnd_raw: isize) -> Result<(), String> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings as EnigoSettings};
    use std::thread;
    use std::time::Duration;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;

    if hwnd_raw == 0 {
        return Err("no cached HWND (hotkey hasn't been pressed in this session)".to_string());
    }
    let hwnd = HWND(hwnd_raw as *mut std::ffi::c_void);

    // Windows restricts foreground changes; SetForegroundWindow returns BOOL.
    let ok = unsafe { SetForegroundWindow(hwnd) }.as_bool();
    if !ok {
        return Err(
            "SetForegroundWindow returned false (OS refused the focus change)".to_string(),
        );
    }

    // Let the OS settle focus before sending input. 50ms is conservative;
    // shorter values race the focus change on slower machines.
    thread::sleep(Duration::from_millis(50));

    let mut enigo = Enigo::new(&EnigoSettings::default())
        .map_err(|e| format!("enigo init failed: {e}"))?;
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| format!("ctrl press failed: {e}"))?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| format!("v key failed: {e}"))?;
    enigo
        .key(Key::Control, Direction::Release)
        .map_err(|e| format!("ctrl release failed: {e}"))?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn paste_into(_hwnd_raw: isize) -> Result<(), String> {
    Err("auto-paste not implemented on this platform".to_string())
}
