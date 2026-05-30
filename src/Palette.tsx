import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { AnimatePresence, motion } from "framer-motion";
import { Eye, FileText, Pin, Search, SearchX } from "lucide-react";
import type { ApplyOutcome } from "./lib/bindings/ApplyOutcome";
import type { FillDialogState } from "./lib/bindings/FillDialogState";
import type { Template } from "./lib/bindings/Template";
import type { TemplateSummary } from "./lib/bindings/TemplateSummary";
import { TemplateFillDialog } from "./TemplateFillDialog";
import { TemplateEditor } from "./TemplateEditor";
import { TagPill } from "./TagPill";
import { Toast, type ToastVariant } from "./Toast";
import { BodyWithVariableChips } from "./BodyWithVariableChips";
import { mergeFillValues } from "./lib/fill";
import { DURATION, EASE } from "./lib/motion";

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
  const [toast, setToast] = useState<{ msg: string; key: number; variant?: ToastVariant } | null>(null);
  const [visible, setVisible] = useState(false);
  const previewRef = useRef<HTMLDivElement | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const hideTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const selected = results[selectedIdx] ?? null;

  // Fade out then invoke the OS-level hide.
  const requestHide = useCallback(() => {
    if (hideTimeoutRef.current) clearTimeout(hideTimeoutRef.current);
    setVisible(false);
    hideTimeoutRef.current = setTimeout(() => {
      invoke("hide_palette").catch(console.error);
      hideTimeoutRef.current = null;
    }, DURATION.normal * 1000);
  }, []);

  useEffect(() => {
    const promise = listen("palette-shown", () => {
      // Cancel any pending hide (e.g. rapid hotkey press).
      if (hideTimeoutRef.current) {
        clearTimeout(hideTimeoutRef.current);
        hideTimeoutRef.current = null;
      }
      setView({ type: "search" });
      setQuery("");
      setSelectedIdx(0);
      setToast(null);
      setVisible(true);
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

  // Handle the outcome of an apply IPC. Palette closes immediately on
  // success (paste or copy-only). Only when autoPaste was attempted but
  // failed do we show a warning toast briefly before hiding.
  const finalizeApply = async (outcome: ApplyOutcome, name: string) => {
    if (outcome.pasted) {
      requestHide();
      return;
    }
    if (outcome.reason === "failed") {
      setToast({
        msg: `已复制：${name}，请手动粘贴`,
        key: Date.now(),
        variant: "warning",
      });
      setTimeout(() => {
        requestHide();
      }, TOAST_BEFORE_HIDE_MS);
    } else {
      // autoPaste disabled — clipboard write succeeded, hide immediately.
      requestHide();
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

  const handleEsc = useCallback(() => {
    requestHide();
  }, [requestHide]);

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
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: visible ? 1 : 0 }}
      transition={{ duration: DURATION.normal, ease: EASE.out }}
      className="flex h-screen flex-col bg-white font-sans dark:bg-zinc-900"
      onKeyDown={handleKeyDown}
    >
      <div className="h-1.5 shrink-0 bg-zinc-100 dark:bg-zinc-800" data-tauri-drag-region />

      <AnimatePresence mode="wait">
        {view.type === "search" && (
          <motion.div
            key="search"
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            transition={{ duration: DURATION.normal, ease: EASE.out }}
            className="flex-1 overflow-hidden"
          >
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
          </motion.div>
        )}

        {view.type === "fill" && (
          <motion.div
            key="fill"
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            transition={{ duration: DURATION.normal, ease: EASE.out }}
            className="flex-1 overflow-y-auto"
          >
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
              onCancel={() => {
                requestHide();
              }}
            />
          </motion.div>
        )}

        {view.type === "edit" && (
          <motion.div
            key="edit"
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            transition={{ duration: DURATION.normal, ease: EASE.out }}
            className="flex-1 overflow-y-auto"
          >
            <TemplateEditor
              template={view.template}
              isNew={false}
              onSave={handleEditSave}
              onCancel={handleEditCancel}
            />
          </motion.div>
        )}
      </AnimatePresence>

      {toast && (
        <Toast
          key={toast.key}
          message={toast.msg}
          variant={toast.variant}
          onDismiss={() => setToast(null)}
        />
      )}
    </motion.div>
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
      <div className="flex flex-col border-r border-zinc-200 dark:border-zinc-700">
        <div className="flex items-center gap-2 border-b border-zinc-200 px-3 py-2 dark:border-zinc-700">
          <Search size={14} className="shrink-0 text-zinc-400" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="搜索模板…"
            autoFocus
            className="flex-1 bg-transparent text-sm outline-none dark:text-zinc-100 dark:placeholder:text-zinc-500"
          />
        </div>
        <div className="flex-1 overflow-y-auto">
          {results.length === 0 ? (
            <div className="flex flex-col items-center justify-center gap-2 p-6 text-center">
              {query ? (
                <>
                  <SearchX size={24} className="text-zinc-300 dark:text-zinc-600" />
                  <div className="text-sm text-zinc-400 dark:text-zinc-500">没有匹配的模板</div>
                  <div className="text-xs text-zinc-300 dark:text-zinc-600">试试其它关键词</div>
                </>
              ) : (
                <>
                  <FileText size={24} className="text-zinc-300 dark:text-zinc-600" />
                  <div className="text-sm text-zinc-400 dark:text-zinc-500">还没有模板</div>
                  <div className="text-xs text-zinc-300 dark:text-zinc-600">从主窗口新建你的第一个模板</div>
                </>
              )}
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
                      ? "bg-zinc-900 text-white dark:bg-zinc-100 dark:text-zinc-900"
                      : "hover:bg-zinc-100 dark:text-zinc-300 dark:hover:bg-zinc-800")
                  }
                >
                  {t.isPinned ? (
                    <Pin
                      size={11}
                      className={
                        idx === selectedIdx
                          ? "shrink-0 text-amber-300 dark:text-amber-600"
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
      <div ref={previewRef} className="overflow-y-auto bg-zinc-50 dark:bg-zinc-800">
        {previewTemplate ? (
          <Preview template={previewTemplate} />
        ) : (
          <div className="flex flex-col items-center justify-center gap-2 p-6 text-center">
            <Eye size={20} className="text-zinc-300 dark:text-zinc-600" />
            <div className="text-sm text-zinc-400 dark:text-zinc-500">选中模板以预览</div>
          </div>
        )}
      </div>
    </div>
  );
}

function Preview({ template }: { template: Template }) {
  return (
    <div className="p-5">
      <h3 className="mb-2 text-base font-semibold tracking-tight text-zinc-900 dark:text-zinc-100">
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
        className="rounded border border-zinc-200 bg-white p-3 dark:border-zinc-600 dark:bg-zinc-900"
      />
    </div>
  );
}
