import { useRef, useState, type KeyboardEvent } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  value: string;
  onChange: (s: string) => void;
  disabled?: boolean;
}

const MODIFIER_KEYS = new Set(["Control", "Alt", "Shift", "Meta"]);

export function HotkeyInput({ value, onChange, disabled }: Props) {
  const [capturing, setCapturing] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  const handleFocus = async () => {
    if (disabled) return;
    setCapturing(true);
    // Unregister the live global hotkey so the OS doesn't route the same
    // combo to the palette handler before our keydown listener sees it.
    // Without this, you can never "type" the currently-bound hotkey to
    // pick a different one.
    try {
      await invoke("pause_hotkey");
    } catch (e) {
      console.warn("pause_hotkey failed", e);
    }
  };

  const handleBlur = async () => {
    setCapturing(false);
    // Re-register the active hotkey. If the user picked a NEW value, the
    // subsequent `save_settings` will re-register again (old → new), which
    // is fine — just slightly wasteful.
    try {
      await invoke("resume_hotkey");
    } catch (e) {
      console.warn("resume_hotkey failed", e);
    }
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (!capturing) return;
    // Swallow ALL keys while capturing — including Tab so focus doesn't
    // escape, and Enter so it doesn't submit a parent form.
    e.preventDefault();
    e.stopPropagation();

    // Plain Escape cancels capture (returns focus to background)
    if (
      e.key === "Escape" &&
      !e.ctrlKey &&
      !e.altKey &&
      !e.shiftKey &&
      !e.metaKey
    ) {
      setCapturing(false);
      ref.current?.blur();
      return;
    }

    // Enter is reserved (form submit) — never bind it
    if (e.key === "Enter") return;
    // Plain Tab is reserved (focus navigation when not capturing)
    if (e.key === "Tab" && !e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey)
      return;

    // Lone modifier key — wait for a real key
    if (MODIFIER_KEYS.has(e.key)) return;

    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    if (e.metaKey) parts.push("Cmd");

    const keyName = codeToHotkeyKey(e.code);
    if (!keyName) return; // unknown physical key; ignore so user can retry

    parts.push(keyName);
    onChange(parts.join("+"));
    setCapturing(false);
    ref.current?.blur();
  };

  const display = capturing
    ? "请按下键组合…（Esc 取消）"
    : value
      ? formatForDisplay(value)
      : "（未设置）";

  return (
    <div
      ref={ref}
      role="button"
      tabIndex={disabled ? -1 : 0}
      onFocus={handleFocus}
      onBlur={handleBlur}
      onKeyDown={handleKeyDown}
      onClick={() => !disabled && ref.current?.focus()}
      className={
        "min-w-[16ch] cursor-pointer select-none rounded border px-2.5 py-1 font-mono text-xs transition " +
        (disabled
          ? "cursor-not-allowed border-zinc-200 bg-zinc-50 text-zinc-400"
          : capturing
            ? "border-amber-400 bg-amber-50 text-amber-900 ring-2 ring-amber-200"
            : "border-zinc-300 bg-white text-zinc-900 hover:border-zinc-400")
      }
      title={
        capturing
          ? "按下任意键组合；Esc 取消"
          : "点击后按下新键组合（含修饰键 Ctrl / Alt / Shift / Cmd）"
      }
    >
      {display}
    </div>
  );
}

/**
 * Map `KeyboardEvent.code` (layout-independent physical key) to the names
 * understood by the Rust `parse_hotkey` parser in `src-tauri/src/hotkey.rs`.
 */
function codeToHotkeyKey(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3); // KeyA → A
  if (/^Digit[0-9]$/.test(code)) return code.slice(5); // Digit1 → 1
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code; // F1..F24
  const named: Record<string, string> = {
    Space: "Space",
    Tab: "Tab",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
    Delete: "Delete",
    Insert: "Insert",
    Backspace: "Backspace",
  };
  return named[code] ?? null;
}

function formatForDisplay(value: string): string {
  return value
    .split("+")
    .map((p) => p.trim())
    .filter(Boolean)
    .join(" + ");
}
