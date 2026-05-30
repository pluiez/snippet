//! Hotkey string ↔ `Shortcut` parsing and runtime re-registration.
//!
//! Frontend stores the user's chosen hotkey as a `+`-joined string in
//! settings.json (e.g. `"Ctrl+Alt+Space"`). Slice 7b lets the user change
//! that string from the Settings page and re-register without restart.

use anyhow::{anyhow, bail, Result};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tracing::{info, warn};

/// Parse `"Ctrl+Alt+Space"` / `"Cmd+Shift+P"` / `"F1"` (case-insensitive,
/// `+`-separated) into a `Shortcut`. Empty pieces and trailing/leading `+`
/// are rejected. Requires at least one non-modifier key.
///
/// Modifier aliases: `Ctrl`/`Control`, `Alt`/`Option`, `Shift`,
/// `Cmd`/`Meta`/`Super`/`Win`. Per Tauri/keyboard-types convention `SUPER`
/// maps to the Windows key on Windows and Cmd on macOS.
pub fn parse_hotkey(s: &str) -> Result<Shortcut> {
    let parts: Vec<&str> = s
        .split('+')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        bail!("empty hotkey string");
    }
    // No early "len < 2" reject here — F1-F24 alone is legal. The loop
    // below catches lone-modifier inputs like "Ctrl" because the final-
    // piece check bails when a modifier is in the last slot.

    let mut modifiers = Modifiers::empty();
    let mut code: Option<Code> = None;

    for (i, raw) in parts.iter().enumerate() {
        let last = i == parts.len() - 1;
        let lower = raw.to_lowercase();
        // Accept aliases. Modifiers can appear in any position; the LAST piece
        // must be the non-modifier key.
        match lower.as_str() {
            "ctrl" | "control" => {
                if last {
                    bail!("'{}' must not be the final key — add a key after it", raw);
                }
                modifiers |= Modifiers::CONTROL;
            }
            "alt" | "option" => {
                if last {
                    bail!("'{}' must not be the final key — add a key after it", raw);
                }
                modifiers |= Modifiers::ALT;
            }
            "shift" => {
                if last {
                    bail!("'{}' must not be the final key — add a key after it", raw);
                }
                modifiers |= Modifiers::SHIFT;
            }
            "cmd" | "command" | "meta" | "super" | "win" | "windows" => {
                if last {
                    bail!("'{}' must not be the final key — add a key after it", raw);
                }
                modifiers |= Modifiers::SUPER;
            }
            _ => {
                if !last {
                    bail!("non-modifier key '{}' must be the last piece", raw);
                }
                code = Some(parse_code(raw)?);
            }
        }
    }

    let code = code.ok_or_else(|| anyhow!("no key specified"))?;
    if modifiers.is_empty() {
        // Single function keys are OK without modifiers; everything else
        // would conflict with normal typing.
        let is_function_key = matches!(
            code,
            Code::F1
                | Code::F2
                | Code::F3
                | Code::F4
                | Code::F5
                | Code::F6
                | Code::F7
                | Code::F8
                | Code::F9
                | Code::F10
                | Code::F11
                | Code::F12
                | Code::F13
                | Code::F14
                | Code::F15
                | Code::F16
                | Code::F17
                | Code::F18
                | Code::F19
                | Code::F20
                | Code::F21
                | Code::F22
                | Code::F23
                | Code::F24
        );
        if !is_function_key {
            bail!("at least one modifier is required (or use a function key)");
        }
    }

    Ok(Shortcut::new(Some(modifiers), code))
}

fn parse_code(raw: &str) -> Result<Code> {
    let trimmed = raw.trim();
    let upper = trimmed.to_uppercase();

    // Single letters A-Z
    if upper.len() == 1 {
        let ch = upper.chars().next().unwrap();
        if ch.is_ascii_alphabetic() {
            return Ok(letter_to_code(ch));
        }
        if ch.is_ascii_digit() {
            return Ok(digit_to_code(ch));
        }
    }

    // Function keys F1..F24
    if let Some(rest) = upper.strip_prefix('F') {
        if let Ok(n) = rest.parse::<u8>() {
            if (1..=24).contains(&n) {
                return Ok(function_to_code(n));
            }
        }
    }

    // Named keys (case-insensitive)
    let lower = trimmed.to_lowercase();
    let code = match lower.as_str() {
        "space" => Code::Space,
        "enter" | "return" => Code::Enter,
        "tab" => Code::Tab,
        "escape" | "esc" => Code::Escape,
        "backspace" => Code::Backspace,
        "delete" | "del" => Code::Delete,
        "insert" | "ins" => Code::Insert,
        "up" | "arrowup" => Code::ArrowUp,
        "down" | "arrowdown" => Code::ArrowDown,
        "left" | "arrowleft" => Code::ArrowLeft,
        "right" | "arrowright" => Code::ArrowRight,
        "home" => Code::Home,
        "end" => Code::End,
        "pageup" | "pgup" => Code::PageUp,
        "pagedown" | "pgdn" => Code::PageDown,
        _ => bail!("unknown key '{}'", raw),
    };
    Ok(code)
}

fn letter_to_code(ch: char) -> Code {
    match ch {
        'A' => Code::KeyA, 'B' => Code::KeyB, 'C' => Code::KeyC, 'D' => Code::KeyD,
        'E' => Code::KeyE, 'F' => Code::KeyF, 'G' => Code::KeyG, 'H' => Code::KeyH,
        'I' => Code::KeyI, 'J' => Code::KeyJ, 'K' => Code::KeyK, 'L' => Code::KeyL,
        'M' => Code::KeyM, 'N' => Code::KeyN, 'O' => Code::KeyO, 'P' => Code::KeyP,
        'Q' => Code::KeyQ, 'R' => Code::KeyR, 'S' => Code::KeyS, 'T' => Code::KeyT,
        'U' => Code::KeyU, 'V' => Code::KeyV, 'W' => Code::KeyW, 'X' => Code::KeyX,
        'Y' => Code::KeyY, 'Z' => Code::KeyZ,
        _ => unreachable!("letter_to_code requires ascii alphabetic"),
    }
}

fn digit_to_code(ch: char) -> Code {
    match ch {
        '0' => Code::Digit0, '1' => Code::Digit1, '2' => Code::Digit2,
        '3' => Code::Digit3, '4' => Code::Digit4, '5' => Code::Digit5,
        '6' => Code::Digit6, '7' => Code::Digit7, '8' => Code::Digit8, '9' => Code::Digit9,
        _ => unreachable!("digit_to_code requires ascii digit"),
    }
}

fn function_to_code(n: u8) -> Code {
    match n {
        1 => Code::F1, 2 => Code::F2, 3 => Code::F3, 4 => Code::F4,
        5 => Code::F5, 6 => Code::F6, 7 => Code::F7, 8 => Code::F8,
        9 => Code::F9, 10 => Code::F10, 11 => Code::F11, 12 => Code::F12,
        13 => Code::F13, 14 => Code::F14, 15 => Code::F15, 16 => Code::F16,
        17 => Code::F17, 18 => Code::F18, 19 => Code::F19, 20 => Code::F20,
        21 => Code::F21, 22 => Code::F22, 23 => Code::F23, 24 => Code::F24,
        _ => unreachable!("function_to_code requires 1..=24"),
    }
}

/// Register `new_hotkey` as the global shortcut, removing `old_hotkey` if
/// supplied. On registration failure, attempts to restore `old_hotkey` and
/// returns an error — caller should NOT persist the new value.
pub fn re_register_hotkey(
    app: &AppHandle,
    old_hotkey: Option<&str>,
    new_hotkey: &str,
) -> Result<()> {
    let new_shortcut =
        parse_hotkey(new_hotkey).map_err(|e| anyhow!("invalid hotkey '{}': {}", new_hotkey, e))?;
    let gs = app.global_shortcut();

    if let Some(old) = old_hotkey {
        // Best effort: if old can't be parsed (corrupt settings) we skip the
        // unregister. Worst case is a stale binding alongside the new one
        // until next restart.
        match parse_hotkey(old) {
            Ok(old_shortcut) => {
                if let Err(e) = gs.unregister(old_shortcut) {
                    warn!(error = ?e, hotkey = old, "failed to unregister old hotkey; continuing");
                }
            }
            Err(e) => warn!(
                error = %e,
                hotkey = old,
                "old hotkey unparseable; skipping unregister"
            ),
        }
    }

    if let Err(e) = gs.register(new_shortcut) {
        // Roll back: try to put the old binding back so the user isn't left
        // with NO hotkey.
        if let Some(old) = old_hotkey {
            if let Ok(old_shortcut) = parse_hotkey(old) {
                if let Err(e2) = gs.register(old_shortcut) {
                    warn!(error = ?e2, hotkey = old, "rollback re-register also failed");
                }
            }
        }
        bail!("registering '{}' failed: {}", new_hotkey, e);
    }

    info!(hotkey = new_hotkey, "global hotkey registered");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_combo() {
        let s = parse_hotkey("Ctrl+Alt+Space").unwrap();
        assert!(s.mods.contains(Modifiers::CONTROL));
        assert!(s.mods.contains(Modifiers::ALT));
        assert_eq!(s.key, Code::Space);
    }

    #[test]
    fn parse_case_insensitive() {
        let a = parse_hotkey("CTRL+ALT+SPACE").unwrap();
        let b = parse_hotkey("ctrl+alt+space").unwrap();
        let c = parse_hotkey("cTrL+aLt+SpAcE").unwrap();
        assert_eq!(a.mods, b.mods);
        assert_eq!(b.mods, c.mods);
        assert_eq!(a.key, c.key);
    }

    #[test]
    fn parse_letter_keys() {
        assert_eq!(parse_hotkey("Ctrl+K").unwrap().key, Code::KeyK);
        assert_eq!(parse_hotkey("Ctrl+Shift+P").unwrap().key, Code::KeyP);
    }

    #[test]
    fn parse_digit_keys() {
        assert_eq!(parse_hotkey("Ctrl+1").unwrap().key, Code::Digit1);
    }

    #[test]
    fn parse_named_keys() {
        assert_eq!(parse_hotkey("Ctrl+Up").unwrap().key, Code::ArrowUp);
        assert_eq!(parse_hotkey("Ctrl+Esc").unwrap().key, Code::Escape);
        assert_eq!(parse_hotkey("Ctrl+Enter").unwrap().key, Code::Enter);
        assert_eq!(parse_hotkey("Ctrl+PgUp").unwrap().key, Code::PageUp);
    }

    #[test]
    fn parse_function_keys_no_modifier() {
        let s = parse_hotkey("F1").unwrap();
        assert!(s.mods.is_empty());
        assert_eq!(s.key, Code::F1);
    }

    #[test]
    fn parse_modifier_aliases() {
        let cmd = parse_hotkey("Cmd+K").unwrap();
        let meta = parse_hotkey("Meta+K").unwrap();
        let win = parse_hotkey("Win+K").unwrap();
        assert!(cmd.mods.contains(Modifiers::SUPER));
        assert!(meta.mods.contains(Modifiers::SUPER));
        assert!(win.mods.contains(Modifiers::SUPER));
    }

    #[test]
    fn reject_empty() {
        assert!(parse_hotkey("").is_err());
        assert!(parse_hotkey("+").is_err());
    }

    #[test]
    fn reject_modifier_only() {
        assert!(parse_hotkey("Ctrl").is_err());
        assert!(parse_hotkey("Ctrl+Shift").is_err());
    }

    #[test]
    fn reject_no_modifier_letter() {
        assert!(parse_hotkey("K").is_err());
        assert!(parse_hotkey("Space").is_err());
    }

    #[test]
    fn reject_unknown_key() {
        assert!(parse_hotkey("Ctrl+Foo").is_err());
        assert!(parse_hotkey("Ctrl+F25").is_err());
    }

    #[test]
    fn reject_modifier_after_key() {
        assert!(parse_hotkey("K+Ctrl").is_err());
    }
}
