import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { AlertTriangle, FolderOpen, Save, X } from "lucide-react";
import { useSettings } from "./lib/settings";
import { ConfirmDialog } from "./ConfirmDialog";
import { HotkeyInput } from "./HotkeyInput";
import type { Settings as SettingsType } from "./lib/bindings/Settings";
import type { ThemePreference } from "./lib/bindings/ThemePreference";
import type { DataFolderStatus } from "./lib/bindings/DataFolderStatus";

interface Props {
  onClose: () => void;
}

export function Settings({ onClose }: Props) {
  const { settings } = useSettings();
  const [staged, setStaged] = useState<SettingsType | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  const [dataFolder, setDataFolder] = useState<string>("");
  const [pendingFolderChange, setPendingFolderChange] = useState<string | null>(null);

  useEffect(() => {
    if (settings && !staged) setStaged(settings);
  }, [settings, staged]);

  const refreshDataFolder = useCallback(async () => {
    try {
      const p = await invoke<string>("current_data_folder");
      setDataFolder(p);
    } catch (e) {
      console.error("current_data_folder failed", e);
    }
  }, []);

  useEffect(() => {
    refreshDataFolder();
  }, [refreshDataFolder]);

  if (!staged || !settings) {
    return (
      <section className="flex-1 p-6">
        <div className="text-sm text-zinc-500 dark:text-zinc-400">loading…</div>
      </section>
    );
  }

  const dirty = JSON.stringify(staged) !== JSON.stringify(settings);

  const onSave = async () => {
    setSaveError(null);
    setSaving(true);
    try {
      await invoke("save_settings", { settings: staged });
    } catch (e) {
      // Backend returns string errors. Hotkey conflicts surface here:
      // "registering 'Ctrl+Alt+K' failed: ..." or "invalid hotkey '...': ...".
      setSaveError(String(e));
    } finally {
      setSaving(false);
    }
  };

  const onCancel = () => {
    setStaged(settings);
    setSaveError(null);
  };

  const onPickNewDataFolder = async () => {
    const picked = await open({ directory: true, multiple: false });
    if (!picked || typeof picked !== "string") return;
    try {
      const status = await invoke<DataFolderStatus>("validate_path_for_import", {
        path: picked,
      });
      if (status !== "validSnippet") {
        alert(
          "所选路径不是 Snippet 数据文件夹（缺少 templates/ 或配置文件）。\n" +
            "如果要从空目录建立新的数据集，请重置 bootstrap.json 后重启应用。"
        );
        return;
      }
      setPendingFolderChange(picked);
    } catch (e) {
      console.error("validate_path_for_import failed", e);
      alert(`路径校验失败: ${e}`);
    }
  };

  const onConfirmFolderChange = async () => {
    if (!pendingFolderChange) return;
    try {
      await invoke("set_data_folder_path", { path: pendingFolderChange });
      setPendingFolderChange(null);
      const willExit = window.confirm(
        "数据文件夹路径已保存。\n\n需要重启应用才能加载新路径下的数据。是否立即退出？"
      );
      if (willExit) {
        await invoke("exit_app");
      }
    } catch (e) {
      console.error("set_data_folder_path failed", e);
      alert(`保存失败: ${e}`);
      setPendingFolderChange(null);
    }
  };

  return (
    <section className="flex-1 overflow-y-auto p-6">
      <div className="mx-auto max-w-2xl">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-base font-semibold tracking-tight dark:text-zinc-100">设置</h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1.5 text-zinc-500 hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-700 dark:hover:text-zinc-200"
            title="关闭"
          >
            <X size={14} />
          </button>
        </div>

        <div className="space-y-4 rounded border border-zinc-200 bg-white p-5 dark:border-zinc-700 dark:bg-zinc-800">
          <SettingsRow
            title="全局热键"
            description={
              <>
                按下组合键来录制；Esc 取消。需要至少一个修饰键（Ctrl / Alt / Shift / Cmd）+
                普通键，或单独的 F1–F24 功能键。
              </>
            }
          >
            <HotkeyInput
              value={staged.hotkey}
              onChange={(hotkey) => {
                setStaged({ ...staged, hotkey });
                setSaveError(null);
              }}
              disabled={saving}
            />
          </SettingsRow>

          <hr className="border-zinc-100 dark:border-zinc-700" />

          <SettingsRow
            title="自动粘贴"
            description="复制完成后，自动切回原焦点窗口并模拟 Ctrl+V。失败时会降级为仅剪贴板 + 提示手动粘贴。"
          >
            <label className="inline-flex cursor-pointer items-center gap-2">
              <input
                type="checkbox"
                checked={staged.autoPaste}
                onChange={(e) =>
                  setStaged({ ...staged, autoPaste: e.target.checked })
                }
                className="h-4 w-4 rounded border-zinc-300 text-zinc-900 focus:ring-zinc-500 dark:border-zinc-600"
              />
              <span className="text-sm dark:text-zinc-300">启用</span>
            </label>
          </SettingsRow>

          <hr className="border-zinc-100 dark:border-zinc-700" />

          <SettingsRow
            title="数据文件夹"
            description={
              <>
                <div className="mt-0.5 break-all font-mono text-xs text-zinc-700 dark:text-zinc-300">
                  {dataFolder || "（加载中…）"}
                </div>
                <div className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
                  改路径后需要重启应用。仅支持指向已存在的 Snippet
                  数据文件夹（新建空库请走 onboarding 流程）。
                </div>
              </>
            }
          >
            <button
              type="button"
              onClick={onPickNewDataFolder}
              className="inline-flex items-center gap-1.5 rounded border border-zinc-300 bg-white px-2.5 py-1 text-xs font-medium text-zinc-700 hover:bg-zinc-50 dark:border-zinc-600 dark:bg-zinc-800 dark:text-zinc-300 dark:hover:bg-zinc-700"
            >
              <FolderOpen size={12} />
              更改…
            </button>
          </SettingsRow>
        </div>

        <div className="mt-4 space-y-4 rounded border border-zinc-200 bg-white p-5 dark:border-zinc-700 dark:bg-zinc-800">
          <SettingsRow
            title="主题"
            description="切换浅色 / 深色外观，或跟随系统设置。"
          >
            <ThemeRadio
              value={staged.theme}
              onChange={(theme) => setStaged({ ...staged, theme })}
            />
          </SettingsRow>
        </div>

        {saveError && (
          <div className="mt-3 flex items-start gap-2 rounded border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700 dark:border-red-800 dark:bg-red-950/40 dark:text-red-300">
            <AlertTriangle size={14} className="mt-0.5 shrink-0" />
            <div className="leading-relaxed">
              <div className="font-medium">保存失败</div>
              <div className="mt-0.5 break-all">{saveError}</div>
              <div className="mt-1 text-red-600/80 dark:text-red-400/80">
                设置未持久化；旧值仍生效。请修改后重试。
              </div>
            </div>
          </div>
        )}

        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            disabled={!dirty || saving}
            className="inline-flex items-center gap-1.5 rounded border border-zinc-300 bg-white px-3 py-1.5 text-sm font-medium text-zinc-700 hover:bg-zinc-50 disabled:opacity-50 dark:border-zinc-600 dark:bg-zinc-800 dark:text-zinc-300 dark:hover:bg-zinc-700"
          >
            <X size={14} />
            取消
          </button>
          <button
            type="button"
            onClick={onSave}
            disabled={!dirty || saving}
            className="inline-flex items-center gap-1.5 rounded bg-zinc-900 px-3 py-1.5 text-sm font-medium text-white hover:bg-zinc-800 disabled:opacity-50 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-200"
          >
            <Save size={14} />
            {saving ? "保存中…" : "保存"}
          </button>
        </div>
      </div>

      {pendingFolderChange && (
        <ConfirmDialog
          title="更改数据文件夹"
          message={`将把路径切换到:\n${pendingFolderChange}\n\n保存后需要重启应用。`}
          confirmText="保存"
          onConfirm={onConfirmFolderChange}
          onCancel={() => setPendingFolderChange(null)}
        />
      )}
    </section>
  );
}

function SettingsRow({
  title,
  description,
  children,
}: {
  title: string;
  description: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div className="min-w-0 flex-1">
        <div className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
          {title}
        </div>
        <div className="mt-0.5 text-xs leading-relaxed text-zinc-500 dark:text-zinc-400">
          {description}
        </div>
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

const THEME_OPTIONS: { value: ThemePreference; label: string }[] = [
  { value: "light", label: "浅色" },
  { value: "dark", label: "深色" },
  { value: "system", label: "跟随系统" },
];

function ThemeRadio({
  value,
  onChange,
}: {
  value: ThemePreference;
  onChange: (v: ThemePreference) => void;
}) {
  return (
    <div className="flex items-center gap-3">
      {THEME_OPTIONS.map((opt) => (
        <label
          key={opt.value}
          className="inline-flex cursor-pointer items-center gap-1.5 text-sm"
        >
          <input
            type="radio"
            name="theme"
            value={opt.value}
            checked={value === opt.value}
            onChange={() => onChange(opt.value)}
            className="text-zinc-900 focus:ring-zinc-500 dark:text-zinc-100"
          />
          <span className="text-zinc-700 dark:text-zinc-300">{opt.label}</span>
        </label>
      ))}
    </div>
  );
}
