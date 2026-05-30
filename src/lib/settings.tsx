// Settings context: shared per-window, refreshed on `settings-changed` event.

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Settings } from "./bindings/Settings";

interface SettingsCtx {
  settings: Settings | null;
  refresh: () => Promise<void>;
}

const Ctx = createContext<SettingsCtx>({
  settings: null,
  refresh: async () => {},
});

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings | null>(null);

  const refresh = useCallback(async () => {
    try {
      const s = await invoke<Settings>("get_settings");
      setSettings(s);
    } catch (e) {
      console.error("get_settings failed", e);
    }
  }, []);

  useEffect(() => {
    refresh();
    const promise = listen("settings-changed", () => refresh());
    return () => {
      promise.then((fn) => fn());
    };
  }, [refresh]);

  return <Ctx.Provider value={{ settings, refresh }}>{children}</Ctx.Provider>;
}

export function useSettings(): SettingsCtx {
  return useContext(Ctx);
}
