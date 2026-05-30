import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RotateCw, Save, X } from "lucide-react";
import { useColorMaps } from "./lib/colors";

interface Props {
  onClose: () => void;
}

type Tab = "vars" | "tags";

export function ColorManagement({ onClose }: Props) {
  const maps = useColorMaps();
  const [tab, setTab] = useState<Tab>("vars");
  const [stagedVars, setStagedVars] = useState<Record<string, string>>(maps.variables);
  const [stagedTags, setStagedTags] = useState<Record<string, string>>(maps.tags);
  const [varsDirty, setVarsDirty] = useState(false);
  const [tagsDirty, setTagsDirty] = useState(false);
  const [saving, setSaving] = useState(false);

  // When backend pushes new maps and we haven't staged changes, sync them in.
  useEffect(() => {
    if (!varsDirty) setStagedVars(maps.variables);
  }, [maps.variables, varsDirty]);
  useEffect(() => {
    if (!tagsDirty) setStagedTags(maps.tags);
  }, [maps.tags, tagsDirty]);

  const dirty = varsDirty || tagsDirty;

  const current = tab === "vars" ? stagedVars : stagedTags;
  const setCurrent = (next: Record<string, string>) => {
    if (tab === "vars") {
      setStagedVars(next);
      setVarsDirty(true);
    } else {
      setStagedTags(next);
      setTagsDirty(true);
    }
  };

  const refreshOne = async (key: string) => {
    try {
      const newColor = await invoke<string>("random_color");
      setCurrent({ ...current, [key]: newColor });
    } catch (e) {
      console.error("random_color failed", e);
    }
  };

  const customizeOne = (key: string, hexColor: string) => {
    setCurrent({ ...current, [key]: hexColor });
  };

  const resetAll = async () => {
    try {
      const next: Record<string, string> = {};
      for (const key of Object.keys(current)) {
        next[key] = await invoke<string>("random_color");
      }
      setCurrent(next);
    } catch (e) {
      console.error("resetAll failed", e);
    }
  };

  const onSave = async () => {
    setSaving(true);
    try {
      if (varsDirty) {
        await invoke("save_variable_colors", { map: stagedVars });
        setVarsDirty(false);
      }
      if (tagsDirty) {
        await invoke("save_tag_colors", { map: stagedTags });
        setTagsDirty(false);
      }
    } catch (e) {
      console.error("save colors failed", e);
    } finally {
      setSaving(false);
    }
  };

  const onCancel = () => {
    setStagedVars(maps.variables);
    setStagedTags(maps.tags);
    setVarsDirty(false);
    setTagsDirty(false);
  };

  return (
    <section className="flex-1 p-6">
      <div className="mx-auto max-w-3xl">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-base font-semibold tracking-tight">颜色管理</h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1.5 text-zinc-500 hover:bg-zinc-100 hover:text-zinc-900"
            title="关闭"
          >
            <X size={14} />
          </button>
        </div>

        <div className="mb-3 inline-flex rounded border border-zinc-200 bg-white p-0.5 text-sm">
          <TabButton active={tab === "vars"} onClick={() => setTab("vars")}>
            变量颜色（{Object.keys(stagedVars).length}）
          </TabButton>
          <TabButton active={tab === "tags"} onClick={() => setTab("tags")}>
            tag 颜色（{Object.keys(stagedTags).length}）
          </TabButton>
        </div>

        <div className="rounded border border-zinc-200 bg-white">
          {Object.keys(current).length === 0 ? (
            <div className="p-6 text-center text-sm text-zinc-400">（空）</div>
          ) : (
            <div className="divide-y divide-zinc-100">
              {Object.entries(current)
                .sort(([a], [b]) => a.localeCompare(b))
                .map(([key, color]) => (
                  <ColorRow
                    key={key}
                    name={key}
                    color={color}
                    onRefresh={() => refreshOne(key)}
                    onCustomize={(hex) => customizeOne(key, hex)}
                  />
                ))}
            </div>
          )}
        </div>

        <div className="mt-4 flex items-center justify-between">
          <button
            type="button"
            onClick={resetAll}
            disabled={Object.keys(current).length === 0 || saving}
            className="inline-flex items-center gap-1.5 rounded border border-zinc-300 bg-white px-3 py-1.5 text-sm font-medium text-zinc-700 hover:bg-zinc-50 disabled:opacity-50"
          >
            <RotateCw size={14} />
            重置全部
          </button>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={onCancel}
              disabled={!dirty || saving}
              className="inline-flex items-center gap-1.5 rounded border border-zinc-300 bg-white px-3 py-1.5 text-sm font-medium text-zinc-700 hover:bg-zinc-50 disabled:opacity-50"
            >
              <X size={14} />
              取消
            </button>
            <button
              type="button"
              onClick={onSave}
              disabled={!dirty || saving}
              className="inline-flex items-center gap-1.5 rounded bg-zinc-900 px-3 py-1.5 text-sm font-medium text-white hover:bg-zinc-800 disabled:opacity-50"
            >
              <Save size={14} />
              {saving ? "保存中…" : "保存"}
            </button>
          </div>
        </div>
      </div>
    </section>
  );
}

function TabButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={
        "rounded px-3 py-1 transition-colors " +
        (active
          ? "bg-zinc-900 text-white"
          : "text-zinc-700 hover:bg-zinc-100")
      }
    >
      {children}
    </button>
  );
}

function ColorRow({
  name,
  color,
  onRefresh,
  onCustomize,
}: {
  name: string;
  color: string;
  onRefresh: () => void;
  onCustomize: (hex: string) => void;
}) {
  const inputRef = useRef<HTMLInputElement>(null);
  return (
    <div className="flex items-center gap-3 px-3 py-2">
      <button
        type="button"
        onClick={() => inputRef.current?.click()}
        className="h-7 w-12 shrink-0 rounded border border-zinc-200"
        style={{ backgroundColor: color }}
        title="自定义颜色"
      />
      <input
        ref={inputRef}
        type="color"
        className="hidden"
        onChange={(e) => onCustomize(e.target.value)}
      />
      <span className="flex-1 truncate font-mono text-sm">{name}</span>
      <code className="hidden font-mono text-xs text-zinc-400 sm:inline">
        {color}
      </code>
      <button
        type="button"
        onClick={onRefresh}
        title="随机刷新"
        className="rounded p-1.5 text-zinc-500 hover:bg-zinc-100 hover:text-zinc-900"
      >
        <RotateCw size={14} />
      </button>
    </div>
  );
}
