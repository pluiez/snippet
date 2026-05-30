import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Palette as PaletteIcon, Settings as SettingsIcon, X } from "lucide-react";
import type { AppInfo } from "./lib/bindings/AppInfo";
import type { ApplyOutcome } from "./lib/bindings/ApplyOutcome";
import type { FillDialogState } from "./lib/bindings/FillDialogState";
import type { Template } from "./lib/bindings/Template";
import type { TemplateSummary } from "./lib/bindings/TemplateSummary";
import { TemplateList } from "./TemplateList";
import { TemplateEditor } from "./TemplateEditor";
import { TemplateFillDialog } from "./TemplateFillDialog";
import { Toast } from "./Toast";
import { ColorManagement } from "./ColorManagement";
import { Settings } from "./Settings";
import { mergeFillValues } from "./lib/fill";

type ReturnTo =
  | { type: "list" }
  | { type: "fill"; state: FillDialogState; values: Record<string, string> };

type View =
  | { type: "list" }
  | { type: "colors" }
  | { type: "settings" }
  | {
      type: "edit";
      template: Template;
      isNew: boolean;
      returnTo: ReturnTo;
    }
  | { type: "fill"; state: FillDialogState };

export default function App() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [templates, setTemplates] = useState<TemplateSummary[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [view, setView] = useState<View>({ type: "list" });
  const [toast, setToast] = useState<{ msg: string; key: number } | null>(null);
  const [glowing, setGlowing] = useState(false);
  const [tagFilter, setTagFilter] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const list = await invoke<TemplateSummary[]>("list_templates");
      setTemplates(list);
    } catch (e) {
      console.error("list_templates failed", e);
    }
  }, []);

  useEffect(() => {
    invoke<AppInfo>("app_info").then(setInfo).catch(console.error);
    refresh().finally(() => setLoaded(true));
    const changed = listen("templates-changed", () => {
      refresh();
    });
    const glow = listen("main-window-glow", () => {
      setGlowing(true);
      setTimeout(() => setGlowing(false), 900);
    });
    return () => {
      changed.then((fn) => fn());
      glow.then((fn) => fn());
    };
  }, [refresh]);

  const showToast = (msg: string) => setToast({ msg, key: Date.now() });

  const toastForOutcome = (outcome: ApplyOutcome, name: string) => {
    if (outcome.pasted) {
      // Focus is gone to the previous app; toast would be unseen. Skip.
      return;
    }
    if (outcome.reason === "failed") {
      showToast(`已复制：${name}，请手动粘贴`);
    } else {
      showToast(`已复制：${name}`);
    }
  };

  const filteredTemplates = useMemo(() => {
    if (!tagFilter) return templates;
    return templates.filter((t) =>
      t.tags.some((x) => x.toLowerCase() === tagFilter.toLowerCase())
    );
  }, [templates, tagFilter]);

  const handleEditSave = async (t: Template) => {
    await invoke("save_template", { template: t });
    if (view.type === "edit" && view.returnTo.type === "fill") {
      const newState = await invoke<FillDialogState>("prepare_fill_dialog", {
        id: t.id,
      });
      const merged = mergeFillValues(newState, view.returnTo.values);
      setView({
        type: "fill",
        state: { ...newState, initialValues: merged },
      });
    } else {
      setView({ type: "list" });
    }
  };

  const handleEditCancel = () => {
    if (view.type === "edit" && view.returnTo.type === "fill") {
      const { state, values } = view.returnTo;
      setView({
        type: "fill",
        state: { ...state, initialValues: values },
      });
    } else {
      setView({ type: "list" });
    }
  };

  const inSideBarView =
    view.type === "list" || view.type === "colors" || view.type === "settings";

  return (
    <main
      className={
        "min-h-screen bg-zinc-50 font-sans transition-shadow duration-300 " +
        (glowing ? "shadow-[inset_0_0_0_4px_rgb(251_191_36)]" : "")
      }
    >
      <header className="flex items-baseline gap-3 border-b border-zinc-200 bg-white px-6 py-3">
        <h1 className="text-base font-semibold tracking-tight">Snippet</h1>
        <span className="text-xs text-zinc-500">
          {info ? `v${info.version}` : ""}
        </span>
      </header>

      {inSideBarView && (
        <div className="flex">
          <MainNav
            active={view.type as "list" | "colors" | "settings"}
            tagFilter={tagFilter}
            onSelectAll={() => {
              setTagFilter(null);
              setView({ type: "list" });
            }}
            onSelectColors={() => setView({ type: "colors" })}
            onSelectSettings={() => setView({ type: "settings" })}
            onClearTag={() => setTagFilter(null)}
          />

          {view.type === "list" && (
            <TemplateList
              templates={filteredTemplates}
              loaded={loaded}
              tagFilter={tagFilter}
              onClearTagFilter={() => setTagFilter(null)}
              onTagClick={(t) => setTagFilter(t)}
              onNew={() =>
                setView({
                  type: "edit",
                  template: makeNewTemplate(),
                  isNew: true,
                  returnTo: { type: "list" },
                })
              }
              onEdit={async (id) => {
                const t = await invoke<Template | null>("get_template", { id });
                if (t)
                  setView({
                    type: "edit",
                    template: t,
                    isNew: false,
                    returnTo: { type: "list" },
                  });
              }}
              onDuplicate={async (id) => {
                const t = await invoke<Template>("duplicate_template", {
                  sourceId: id,
                });
                setView({
                  type: "edit",
                  template: t,
                  isNew: false,
                  returnTo: { type: "list" },
                });
              }}
              onFill={async (id) => {
                const state = await invoke<FillDialogState>(
                  "prepare_fill_dialog",
                  { id }
                );
                if (state.orderedVariables.length === 0) {
                  const outcome = await invoke<ApplyOutcome>("apply_template", {
                    id,
                    values: {},
                  });
                  toastForOutcome(outcome, state.template.displayName);
                } else {
                  setView({ type: "fill", state });
                }
              }}
              onTogglePin={async (id, pinned) => {
                try {
                  await invoke("set_pinned", { id, pinned });
                } catch (e) {
                  console.error("set_pinned failed", e);
                }
              }}
              onDelete={async (id) => {
                await invoke("delete_template", { id });
              }}
            />
          )}

          {view.type === "colors" && (
            <ColorManagement onClose={() => setView({ type: "list" })} />
          )}

          {view.type === "settings" && (
            <Settings onClose={() => setView({ type: "list" })} />
          )}
        </div>
      )}

      {view.type === "edit" && (
        <TemplateEditor
          template={view.template}
          isNew={view.isNew}
          onSave={handleEditSave}
          onCancel={handleEditCancel}
          onMutexTransfer={(name) => showToast(`已从 ${name} 转移`)}
        />
      )}

      {view.type === "fill" && (
        <TemplateFillDialog
          state={view.state}
          onApply={async (values) => {
            const outcome = await invoke<ApplyOutcome>("apply_template", {
              id: view.state.template.id,
              values,
            });
            toastForOutcome(outcome, view.state.template.displayName);
            setView({ type: "list" });
          }}
          onUnlock={(values) => {
            setView({
              type: "edit",
              template: view.state.template,
              isNew: false,
              returnTo: {
                type: "fill",
                state: view.state,
                values,
              },
            });
          }}
          onCancel={() => setView({ type: "list" })}
        />
      )}

      {toast && (
        <Toast
          key={toast.key}
          message={toast.msg}
          onDismiss={() => setToast(null)}
        />
      )}
    </main>
  );
}

function MainNav({
  active,
  tagFilter,
  onSelectAll,
  onSelectColors,
  onSelectSettings,
  onClearTag,
}: {
  active: "list" | "colors" | "settings";
  tagFilter: string | null;
  onSelectAll: () => void;
  onSelectColors: () => void;
  onSelectSettings: () => void;
  onClearTag: () => void;
}) {
  return (
    <nav className="w-44 shrink-0 border-r border-zinc-200 bg-white p-3">
      <NavItem active={active === "list" && !tagFilter} onClick={onSelectAll}>
        全部模板
      </NavItem>
      {tagFilter && (
        <div className="mt-1 flex items-center gap-1.5 rounded border border-amber-200 bg-amber-50 px-3 py-1.5 text-xs text-amber-800">
          <span className="truncate">tag: {tagFilter}</span>
          <button
            type="button"
            onClick={onClearTag}
            className="ml-auto shrink-0 rounded p-0.5 hover:bg-amber-100"
            title="清除筛选"
          >
            <X size={12} />
          </button>
        </div>
      )}
      <div className="my-2 border-t border-zinc-200" />
      <NavItem active={active === "colors"} onClick={onSelectColors}>
        <PaletteIcon size={14} className="shrink-0" />
        颜色管理
      </NavItem>
      <NavItem active={active === "settings"} onClick={onSelectSettings}>
        <SettingsIcon size={14} className="shrink-0" />
        设置
      </NavItem>
    </nav>
  );
}

function NavItem({
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
        "flex w-full items-center gap-2 rounded px-3 py-1.5 text-left text-sm font-medium transition-colors " +
        (active
          ? "bg-zinc-100 text-zinc-900"
          : "text-zinc-600 hover:bg-zinc-50")
      }
    >
      {children}
    </button>
  );
}

function makeNewTemplate(): Template {
  const now = new Date().toISOString();
  return {
    schemaVersion: 1,
    id: crypto.randomUUID(),
    displayName: "",
    body: "",
    variables: [],
    tags: [],
    isPinned: false,
    createdAt: now,
    updatedAt: now,
    lastUsedAt: null,
    useCount: 0,
  };
}
