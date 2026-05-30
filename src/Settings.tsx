import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Save, X } from "lucide-react";
import { useSettings } from "./lib/settings";
import type { Settings as SettingsType } from "./lib/bindings/Settings";

interface Props {
  onClose: () => void;
}

export function Settings({ onClose }: Props) {
  const { settings } = useSettings();
  const [staged, setStaged] = useState<SettingsType | null>(null);
  const [saving, setSaving] = useState(false);

  // Initialize staged once on first settings load. After that, user controls
  // staged via the form; remote updates (other windows) won't overwrite their
  // in-progress edits. Cancel reverts to current settings.
  useEffect(() => {
    if (settings && !staged) setStaged(settings);
  }, [settings, staged]);

  if (!staged || !settings) {
    return (
      <section className="flex-1 p-6">
        <div className="text-sm text-zinc-500">loading…</div>
      </section>
    );
  }

  const dirty = JSON.stringify(staged) !== JSON.stringify(settings);

  const onSave = async () => {
    setSaving(true);
    try {
      await invoke("save_settings", { settings: staged });
    } catch (e) {
      console.error("save_settings failed", e);
    } finally {
      setSaving(false);
    }
  };

  const onCancel = () => {
    setStaged(settings);
  };

  return (
    <section className="flex-1 p-6">
      <div className="mx-auto max-w-2xl">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-base font-semibold tracking-tight">设置</h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1.5 text-zinc-500 hover:bg-zinc-100 hover:text-zinc-900"
            title="关闭"
          >
            <X size={14} />
          </button>
        </div>

        <div className="space-y-4 rounded border border-zinc-200 bg-white p-5">
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
                className="h-4 w-4 rounded border-zinc-300 text-zinc-900 focus:ring-zinc-500"
              />
              <span className="text-sm">启用</span>
            </label>
          </SettingsRow>
        </div>

        <div className="mt-3 rounded border border-zinc-200 bg-zinc-50 p-3 text-xs leading-relaxed text-zinc-500">
          其它设置（热键、主题、数据文件夹路径）将在 Slice 7 中加入。
        </div>

        <div className="mt-4 flex justify-end gap-2">
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
    </section>
  );
}

function SettingsRow({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div className="min-w-0 flex-1">
        <div className="text-sm font-medium text-zinc-900">{title}</div>
        <div className="mt-0.5 text-xs leading-relaxed text-zinc-500">
          {description}
        </div>
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}
