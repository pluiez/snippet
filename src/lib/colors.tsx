// Color maps context: shared by every component in a window. SPEC §6 says
// the central maps are the source of truth for variable / tag colors; we
// fetch them once per window via `ColorMapsProvider` and refresh whenever
// the backend emits `colors-changed`.

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
import type { VariableColorMap } from "./bindings/VariableColorMap";
import type { TagColorMap } from "./bindings/TagColorMap";

export interface ColorMaps {
  variables: Record<string, string>;
  tags: Record<string, string>;
  refresh: () => Promise<void>;
}

const Ctx = createContext<ColorMaps>({
  variables: {},
  tags: {},
  refresh: async () => {},
});

export function ColorMapsProvider({ children }: { children: ReactNode }) {
  const [variables, setVariables] = useState<Record<string, string>>({});
  const [tags, setTags] = useState<Record<string, string>>({});

  const refresh = useCallback(async () => {
    try {
      const [v, t] = await Promise.all([
        invoke<VariableColorMap>("get_variable_colors"),
        invoke<TagColorMap>("get_tag_colors"),
      ]);
      // ts-rs emits HashMap<String, String> values as `string | undefined`,
      // but the backend never inserts null entries — assert away the
      // optionality so consumers can use Record<string, string> cleanly.
      setVariables(v.map as Record<string, string>);
      setTags(t.map as Record<string, string>);
    } catch (e) {
      console.error("loadColorMaps failed", e);
    }
  }, []);

  useEffect(() => {
    refresh();
    const promise = listen("colors-changed", () => refresh());
    return () => {
      promise.then((fn) => fn());
    };
  }, [refresh]);

  return (
    <Ctx.Provider value={{ variables, tags, refresh }}>{children}</Ctx.Provider>
  );
}

export function useColorMaps(): ColorMaps {
  return useContext(Ctx);
}

const FALLBACK_COLOR = "#a1a1aa"; // zinc-400, neutral when name not yet in map.

export function variableColor(name: string, maps: ColorMaps): string {
  return maps.variables[name.toLowerCase()] ?? FALLBACK_COLOR;
}

export function tagColor(name: string, maps: ColorMaps): string {
  return maps.tags[name.toLowerCase()] ?? FALLBACK_COLOR;
}
