import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderPlus, FolderOpen, HardDrive, Loader2 } from "lucide-react";
import type { DataFolderStatus } from "./lib/bindings/DataFolderStatus";

type Choice = "default" | "customNew" | "import";

export function Onboarding() {
  const [choice, setChoice] = useState<Choice>("default");
  const [defaultPath, setDefaultPath] = useState<string>("");

  const [customPath, setCustomPath] = useState<string | null>(null);
  const [customStatus, setCustomStatus] = useState<DataFolderStatus | null>(null);

  const [importPath, setImportPath] = useState<string | null>(null);
  const [importStatus, setImportStatus] = useState<DataFolderStatus | null>(null);

  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<string>("default_data_folder")
      .then(setDefaultPath)
      .catch((e) => {
        console.error("default_data_folder failed", e);
        setError(`无法解析默认路径: ${e}`);
      });
  }, []);

  const pickCustom = async () => {
    setError(null);
    const picked = await open({ directory: true, multiple: false });
    if (!picked || typeof picked !== "string") return;
    setCustomPath(picked);
    try {
      const status = await invoke<DataFolderStatus>("validate_path_for_new", {
        path: picked,
      });
      setCustomStatus(status);
    } catch (e) {
      setError(`路径校验失败: ${e}`);
    }
  };

  const pickImport = async () => {
    setError(null);
    const picked = await open({ directory: true, multiple: false });
    if (!picked || typeof picked !== "string") return;
    setImportPath(picked);
    try {
      const status = await invoke<DataFolderStatus>("validate_path_for_import", {
        path: picked,
      });
      setImportStatus(status);
    } catch (e) {
      setError(`路径校验失败: ${e}`);
    }
  };

  const customEligible =
    customStatus === "doesNotExist" || customStatus === "empty";
  const importEligible = importStatus === "validSnippet";

  const canSubmit =
    choice === "default" ||
    (choice === "customNew" && customPath != null && customEligible) ||
    (choice === "import" && importPath != null && importEligible);

  const onSubmit = async () => {
    setError(null);
    setSubmitting(true);
    try {
      if (choice === "default") {
        await invoke("complete_onboarding_default");
      } else if (choice === "customNew") {
        await invoke("complete_onboarding_custom_new", { path: customPath });
      } else {
        await invoke("complete_onboarding_import", { path: importPath });
      }
      // Backend hides the window on success — nothing else to do here.
    } catch (e) {
      setError(`初始化失败: ${e}`);
      setSubmitting(false);
    }
  };

  return (
    <div className="flex h-screen flex-col bg-zinc-50">
      <header className="border-b border-zinc-200 bg-white px-6 py-4">
        <h1 className="text-lg font-semibold tracking-tight text-zinc-900">
          欢迎使用 Snippet
        </h1>
        <p className="mt-1 text-xs leading-relaxed text-zinc-500">
          选择数据文件夹位置 —— 这里会保存你的模板、设置和颜色配置。
        </p>
      </header>

      <main className="flex-1 overflow-y-auto px-6 py-4">
        <div className="space-y-2.5">
          <OptionCard
            icon={<HardDrive size={18} />}
            title="使用默认路径"
            description={defaultPath || "加载中…"}
            selected={choice === "default"}
            onClick={() => setChoice("default")}
          />

          <OptionCard
            icon={<FolderPlus size={18} />}
            title="指定路径新建"
            description="选一个新的或空目录，建立全新的数据文件夹。"
            selected={choice === "customNew"}
            onClick={() => setChoice("customNew")}
          >
            <PathPicker
              path={customPath}
              status={customStatus}
              onPick={pickCustom}
              eligibleStatuses={["doesNotExist", "empty"]}
              hintIneligible="目标路径已有内容。如果是已有 Snippet 数据，请改用「从已有路径导入」。"
            />
          </OptionCard>

          <OptionCard
            icon={<FolderOpen size={18} />}
            title="从已有路径导入"
            description="挂载一个已经存在的 Snippet 数据文件夹（例如来自其它设备的备份）。"
            selected={choice === "import"}
            onClick={() => setChoice("import")}
          >
            <PathPicker
              path={importPath}
              status={importStatus}
              onPick={pickImport}
              eligibleStatuses={["validSnippet"]}
              hintIneligible="该路径不是 Snippet 数据文件夹（缺少 templates/ 或配置文件）。"
            />
          </OptionCard>
        </div>

        {error && (
          <div className="mt-4 rounded border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700">
            {error}
          </div>
        )}
      </main>

      <footer className="flex items-center justify-end border-t border-zinc-200 bg-white px-6 py-3">
        <button
          type="button"
          onClick={onSubmit}
          disabled={!canSubmit || submitting}
          className="inline-flex items-center gap-2 rounded bg-amber-500 px-4 py-1.5 text-sm font-medium text-white hover:bg-amber-600 disabled:cursor-not-allowed disabled:opacity-50"
        >
          {submitting && <Loader2 size={14} className="animate-spin" />}
          {submitting ? "初始化中…" : "开始使用"}
        </button>
      </footer>
    </div>
  );
}

interface OptionCardProps {
  icon: React.ReactNode;
  title: string;
  description: string;
  selected: boolean;
  onClick: () => void;
  children?: React.ReactNode;
}

function OptionCard({
  icon,
  title,
  description,
  selected,
  onClick,
  children,
}: OptionCardProps) {
  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onClick();
        }
      }}
      className={
        "cursor-pointer rounded-lg border bg-white p-3 transition " +
        (selected
          ? "border-amber-400 ring-2 ring-amber-200"
          : "border-zinc-200 hover:border-zinc-300")
      }
    >
      <div className="flex items-start gap-3">
        <div
          className={
            "mt-0.5 shrink-0 " + (selected ? "text-amber-600" : "text-zinc-400")
          }
        >
          {icon}
        </div>
        <div className="min-w-0 flex-1">
          <div className="text-sm font-medium text-zinc-900">{title}</div>
          <div className="mt-0.5 break-all text-xs leading-relaxed text-zinc-500">
            {description}
          </div>
        </div>
      </div>
      {selected && children && <div className="mt-3 pl-8">{children}</div>}
    </div>
  );
}

interface PathPickerProps {
  path: string | null;
  status: DataFolderStatus | null;
  onPick: () => void;
  eligibleStatuses: DataFolderStatus[];
  hintIneligible: string;
}

function PathPicker({
  path,
  status,
  onPick,
  eligibleStatuses,
  hintIneligible,
}: PathPickerProps) {
  const eligible = status != null && eligibleStatuses.includes(status);
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={onPick}
          className="rounded border border-zinc-300 bg-white px-2.5 py-1 text-xs font-medium text-zinc-700 hover:bg-zinc-50"
        >
          {path ? "重新选择…" : "选择文件夹…"}
        </button>
        {path && (
          <span
            className="truncate text-xs text-zinc-600"
            title={path}
          >
            {path}
          </span>
        )}
      </div>
      {path && status && !eligible && (
        <div className="rounded bg-red-50 px-2 py-1.5 text-xs text-red-700">
          {hintIneligible}
        </div>
      )}
      {path && status && eligible && (
        <div className="rounded bg-emerald-50 px-2 py-1.5 text-xs text-emerald-700">
          {statusFriendly(status)}
        </div>
      )}
    </div>
  );
}

function statusFriendly(s: DataFolderStatus): string {
  switch (s) {
    case "doesNotExist":
      return "路径将被创建。";
    case "empty":
      return "目录为空，可以使用。";
    case "validSnippet":
      return "检测到 Snippet 数据，将直接挂载。";
    case "occupiedByOther":
      return "目录内容不是 Snippet 数据。";
  }
}
