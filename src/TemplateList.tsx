import { useState } from "react";
import { AnimatePresence } from "framer-motion";
import { Copy, Pencil, Pin, Play, Plus, Trash2 } from "lucide-react";
import type { TemplateSummary } from "./lib/bindings/TemplateSummary";
import { ConfirmDialog } from "./ConfirmDialog";
import { TagPill } from "./TagPill";

interface Props {
  templates: TemplateSummary[];
  loaded: boolean;
  tagFilter: string | null;
  onClearTagFilter: () => void;
  onTagClick: (tag: string) => void;
  onNew: () => void;
  onEdit: (id: string) => void;
  onDuplicate: (id: string) => void;
  onFill: (id: string) => void;
  onTogglePin: (id: string, pinned: boolean) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

export function TemplateList({
  templates,
  loaded,
  tagFilter,
  onTagClick,
  onNew,
  onEdit,
  onDuplicate,
  onFill,
  onTogglePin,
  onDelete,
}: Props) {
  const [confirmDelete, setConfirmDelete] = useState<TemplateSummary | null>(
    null
  );

  return (
    <>
      <section className="flex-1 p-6">
        <div className="mx-auto max-w-3xl">
          <div className="mb-4 flex items-center justify-between">
            <div className="text-xs uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
              模板（{templates.length}）
              {tagFilter && (
                <span className="ml-2 normal-case text-zinc-400 dark:text-zinc-500">
                  · 筛选 tag: {tagFilter}
                </span>
              )}
            </div>
            <button
              type="button"
              onClick={onNew}
              className="inline-flex items-center gap-1.5 rounded bg-zinc-900 px-3 py-1.5 text-sm font-medium text-white hover:bg-zinc-800 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-200"
            >
              <Plus size={14} />
              新建模板
            </button>
          </div>

          {!loaded ? (
            <div className="rounded border border-dashed border-zinc-300 p-6 text-sm text-zinc-500 dark:border-zinc-600 dark:text-zinc-400">
              loading…
            </div>
          ) : templates.length === 0 ? (
            <EmptyState filtered={!!tagFilter} />
          ) : (
            <ul className="divide-y divide-zinc-200 overflow-hidden rounded border border-zinc-200 bg-white dark:divide-zinc-700 dark:border-zinc-700 dark:bg-zinc-800">
              {templates.map((t) => (
                <li
                  key={t.id}
                  className="group flex items-center gap-2 px-4 py-2.5 text-sm"
                >
                  <button
                    type="button"
                    title={t.isPinned ? "取消置顶" : "置顶"}
                    onClick={() => onTogglePin(t.id, !t.isPinned)}
                    className={
                      "shrink-0 cursor-pointer rounded p-1.5 transition-colors hover:bg-zinc-100 dark:hover:bg-zinc-700 " +
                      (t.isPinned
                        ? "text-amber-500"
                        : "text-zinc-300 hover:text-zinc-500 dark:text-zinc-600 dark:hover:text-zinc-400")
                    }
                  >
                    <Pin
                      size={14}
                      fill={t.isPinned ? "currentColor" : "none"}
                    />
                  </button>
                  <button
                    type="button"
                    onClick={() => onEdit(t.id)}
                    className="min-w-0 flex-1 truncate text-left font-medium text-zinc-900 hover:underline dark:text-zinc-100"
                  >
                    {t.displayName || (
                      <span className="italic text-zinc-400 dark:text-zinc-500">（未命名）</span>
                    )}
                  </button>
                  {t.tags.length > 0 && (
                    <div className="flex shrink-0 items-center gap-1">
                      {t.tags.slice(0, 3).map((tag) => (
                        <TagPill
                          key={tag}
                          tag={tag}
                          onClick={() => onTagClick(tag)}
                        />
                      ))}
                      {t.tags.length > 3 && (
                        <span className="text-xs text-zinc-400">
                          +{t.tags.length - 3}
                        </span>
                      )}
                    </div>
                  )}
                  <code className="shrink-0 font-mono text-xs text-zinc-400 dark:text-zinc-500">
                    {t.id.slice(0, 8)}
                  </code>
                  <div className="flex shrink-0 items-center gap-0.5 opacity-40 transition-opacity group-hover:opacity-100">
                    <IconButton title="试用" onClick={() => onFill(t.id)}>
                      <Play size={14} />
                    </IconButton>
                    <IconButton title="编辑" onClick={() => onEdit(t.id)}>
                      <Pencil size={14} />
                    </IconButton>
                    <IconButton title="复制" onClick={() => onDuplicate(t.id)}>
                      <Copy size={14} />
                    </IconButton>
                    <IconButton
                      title="删除"
                      onClick={() => setConfirmDelete(t)}
                      destructive
                    >
                      <Trash2 size={14} />
                    </IconButton>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      </section>

      <AnimatePresence>
        {confirmDelete && (
          <ConfirmDialog
            title="删除模板"
            message={`确定要删除"${confirmDelete.displayName || "（未命名）"}"？此操作不可撤销。`}
            confirmText="删除"
            destructive
            onConfirm={async () => {
              await onDelete(confirmDelete.id);
              setConfirmDelete(null);
            }}
            onCancel={() => setConfirmDelete(null)}
          />
        )}
      </AnimatePresence>
    </>
  );
}

function IconButton({
  children,
  title,
  onClick,
  destructive,
}: {
  children: React.ReactNode;
  title: string;
  onClick: () => void;
  destructive?: boolean;
}) {
  return (
    <button
      type="button"
      title={title}
      onClick={onClick}
      className={
        "rounded p-1.5 transition-colors " +
        (destructive
          ? "text-zinc-500 hover:bg-red-50 hover:text-red-600 dark:text-zinc-400 dark:hover:bg-red-950/40 dark:hover:text-red-400"
          : "text-zinc-500 hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-700 dark:hover:text-zinc-200")
      }
    >
      {children}
    </button>
  );
}

function EmptyState({ filtered }: { filtered: boolean }) {
  if (filtered) {
    return (
      <div className="rounded border border-dashed border-zinc-300 p-6 text-sm text-zinc-500 dark:border-zinc-600 dark:text-zinc-400">
        当前筛选下没有模板。点击左侧"全部模板"清除筛选。
      </div>
    );
  }
  return (
    <div className="rounded border border-dashed border-zinc-300 p-6 text-sm leading-relaxed text-zinc-600 dark:border-zinc-600 dark:text-zinc-400">
      <div className="mb-2 font-medium text-zinc-700 dark:text-zinc-300">还没有模板。</div>
      <div>
        点击右上角"新建模板"创建一个，或者按 <kbd className="rounded border border-zinc-300 bg-white px-1.5 py-0.5 text-xs dark:border-zinc-600 dark:bg-zinc-800 dark:text-zinc-300">Ctrl+Alt+Space</kbd> 唤起 palette。
      </div>
    </div>
  );
}
