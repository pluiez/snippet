import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Pin, Search } from "lucide-react";
import type { ApplyOutcome } from "./lib/bindings/ApplyOutcome";
import type { FillDialogState } from "./lib/bindings/FillDialogState";
import type { Template } from "./lib/bindings/Template";
import type { TemplateSummary } from "./lib/bindings/TemplateSummary";
import { TemplateFillDialog } from "./TemplateFillDialog";
import { TemplateEditor } from "./TemplateEditor";
import { TagPill } from "./TagPill";
import { Toast } from "./Toast";
import { BodyWithVariableChips } from "./BodyWithVariableChips";
import { mergeFillValues } from "./lib/fill";

type ReturnTo = {
  type: "fill";
  state: FillDialogState;
  values: Record<string, string>;
};

type View =
  | { type: "search" }
  | { type: "fill"; state: FillDialogState }
  | { type: "edit"; template: Template; returnTo: ReturnTo };

// If autoPaste was attempted but failed, show toast this long before hiding
// the palette so the user has time to read it.
const TOAST_BEFORE_HIDE_MS = 1500;

export function Palette() {
  const [view, setView] = useState<View>({ type: "search" });
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<TemplateSummary[]>([]);
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [previewTemplate, setPreviewTemplate] = useState<Template | null>(null);
  const [toast, setToast] = useState<{ msg: string; key: number } | null>(null);
  const previewRef = useRef<HTMLDivElement | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);

  const selected = results[selectedIdx] ?? null;

  useEffect(() => {
    const promise = listen("palette-shown", () => {
      setView({ type: "search" });
      setQuery("");
      setSelectedIdx(0);
      setToast(null);
      setTimeout(() => inputRef.current?.focus(), 0);
    });
    return () => {
      promise.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (view.type !== "search") return;
    invoke<TemplateSummary[]>("search_templates", { query })
      .then((r) => {
        setResults(r);
        setSelectedIdx(0);
      })
      .catch(console.error);
  }, [query, view.type]);

  useEffect(() => {
    const promise = listen("templates-changed", () => {
      invoke<TemplateSummary[]>("search_templates", { query })
        .then(setResults)
        .catch(console.error);
    });
    return () => {
      promise.then((fn) => fn());
    };
  }, [query]);

  useEffect(() => {
    if (selected) {
      invoke<Template | null>("get_template", { id: selected.id })
        .then(setPreviewTemplate)
        .catch(console.error);
    } else {
      setPreviewTemplate(null);
    }
  }, [selected]);

  // Handle the outcome of an apply IPC. If autoPaste was attempted but failed,
  // show a toast for ~1.5s before hiding the palette. Otherwise hide immediately.
  const finalizeApply = async (outcome: ApplyOutcome, name: string) => {
    if (outcome.pasted) {
      await invoke("hide_palette");
      return;
    }
    if (outcome.reason === "failed") {
      setToast({
        msg: `已复制：${name}，请手动粘贴`,
        key: Date.now(),
      });
      setTimeout(() => {
        invoke("hide_palette").catch(console.error);
      }, TOAST_BEFORE_HIDE_MS);
    } else {
      // autoPaste disabled — just hide; user will switch app and paste manually.
      await invoke("hide_palette");
    }
  };

  const handleEnter = useCallback(async () => {
    if (!selected) return;
    const fillState = await invoke<FillDialogState>("prepare_fill_dialog", {
      id: selected.id,
    });
    if (fillState.orderedVariables.length === 0) {
      const outcome = await invoke<ApplyOutcome>("apply_template", {
        id: selected.id,
        values: {},
      });
      await finalizeApply(outcome, fillState.template.displayName);
    } else {
      setView({ type: "fill", state: fillState });
    }
  }, [selected]);

  const handleEsc = useCallback(async () => {
    await invoke("hide_palette");
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (view.type !== "search") return;
    if (e.nativeEvent.isComposing) return;
    if ((e.metaKey || e.ctrlKey) && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
      e.preventDefault();
      if (previewRef.current) {
        const dir = e.key === "ArrowDown" ? 1 : -1;
        previewRef.current.scrollBy({ top: dir * 120, behavior: "smooth" });
      }
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIdx((i) => Math.min(i + 1, Math.max(0, results.length - 1)));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIdx((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      handleEnter();
    } else if (e.key === "Escape") {
      e.preventDefault();
      handleEsc();
    }
  };

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
    }
  };

  const handleEditCancel = () => {
    if (view.type === "edit" && view.returnTo.type === "fill") {
      const { state, values } = view.returnTo;
      setView({
        type: "fill",
        state: { ...state, initialValues: values },
      });
    }
  };

  return (
    <div
      className="flex h-screen flex-col bg-white font-sans"
      onKeyDown={handleKeyDown}
    >
      <div className="h-1.5 shrink-0 bg-zinc-100" data-tauri-drag-region />

      {view.type === "search" && (
        <SearchView
          query={query}
          setQuery={setQuery}
          results={results}
          selectedIdx={selectedIdx}
          previewTemplate={previewTemplate}
          previewRef={previewRef}
          inputRef={inputRef}
          onItemClick={(idx) => setSelectedIdx(idx)}
          onItemDoubleClick={(idx) => {
            setSelectedIdx(idx);
            setTimeout(handleEnter, 0);
          }}
        />
      )}

      {view.type === "fill" && (
        <div className="flex-1 overflow-y-auto">
          <TemplateFillDialog
            state={view.state}
            onApply={async (values) => {
              const outcome = await invoke<ApplyOutcome>("apply_template", {
                id: view.state.template.id,
                values,
              });
              await finalizeApply(outcome, view.state.template.displayName);
            }}
            onUnlock={(values) => {
              setView({
                type: "edit",
                template: view.state.template,
                returnTo: { type: "fill", state: view.state, values },
              });
            }}
            onCancel={async () => {
              await invoke("hide_palette");
            }}
          />
        </div>
      )}

      {view.type === "edit" && (
        <div className="flex-1 overflow-y-auto">
          <TemplateEditor
            template={view.template}
            isNew={false}
            onSave={handleEditSave}
            onCancel={handleEditCancel}
          />
        </div>
      )}

      {toast && (
        <Toast
          key={toast.key}
          message={toast.msg}
          onDismiss={() => setToast(null)}
        />
      )}
    </div>
  );
}

function SearchView({
  query,
  setQuery,
  results,
  selectedIdx,
  previewTemplate,
  previewRef,
  inputRef,
  onItemClick,
  onItemDoubleClick,
}: {
  query: string;
  setQuery: (s: string) => void;
  results: TemplateSummary[];
  selectedIdx: number;
  previewTemplate: Template | null;
  previewRef: React.MutableRefObject<HTMLDivElement | null>;
  inputRef: React.MutableRefObject<HTMLInputElement | null>;
  onItemClick: (idx: number) => void;
  onItemDoubleClick: (idx: number) => void;
}) {
  return (
    <div className="grid flex-1 grid-cols-[40%_60%] overflow-hidden">
      <div className="flex flex-col border-r border-zinc-200">
        <div className="flex items-center gap-2 border-b border-zinc-200 px-3 py-2">
          <Search size={14} className="shrink-0 text-zinc-400" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="搜索模板…"
            autoFocus
            className="flex-1 bg-transparent text-sm outline-none"
          />
        </div>
        <div className="flex-1 overflow-y-auto">
          {results.length === 0 ? (
            <div className="p-4 text-center text-sm text-zinc-400">
              {query ? "无匹配" : "无模板"}
            </div>
          ) : (
            <ul>
              {results.map((t, idx) => (
                <li
                  key={t.id}
                  onClick={() => onItemClick(idx)}
                  onDoubleClick={() => onItemDoubleClick(idx)}
                  className={
                    "flex cursor-pointer items-center gap-2 px-3 py-2 text-sm " +
                    (idx === selectedIdx
                      ? "bg-zinc-900 text-white"
                      : "hover:bg-zinc-100")
                  }
                >
                  {t.isPinned ? (
                    <Pin
                      size={11}
                      className={
                        idx === selectedIdx
                          ? "shrink-0 text-amber-300"
                          : "shrink-0 text-amber-500"
                      }
                      fill="currentColor"
                    />
                  ) : (
                    <span className="w-3 shrink-0" />
                  )}
                  <span className="truncate font-medium">
                    {t.displayName || (
                      <span className="italic opacity-60">（未命名）</span>
                    )}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
      <div ref={previewRef} className="overflow-y-auto bg-zinc-50">
        {previewTemplate ? (
          <Preview template={previewTemplate} />
        ) : (
          <div className="p-6 text-sm text-zinc-400">（无选中）</div>
        )}
      </div>
    </div>
  );
}

function Preview({ template }: { template: Template }) {
  return (
    <div className="p-5">
      <h3 className="mb-2 text-base font-semibold tracking-tight text-zinc-900">
        {template.displayName}
      </h3>
      {template.tags.length > 0 && (
        <div className="mb-3 flex flex-wrap gap-1">
          {template.tags.map((t) => (
            <TagPill key={t} tag={t} />
          ))}
        </div>
      )}
      <BodyWithVariableChips
        body={template.body}
        variables={template.variables}
        className="rounded border border-zinc-200 bg-white p-3"
      />
    </div>
  );
}
