import { useMemo, useState } from "react";
import { Save, X } from "lucide-react";
import type { Template } from "./lib/bindings/Template";
import type { Variable } from "./lib/bindings/Variable";
import { bodyToDisplay, bodyToStorage } from "./lib/body";
import { VariableList } from "./VariableList";
import { TagInput } from "./TagInput";

interface Props {
  template: Template;
  isNew: boolean;
  onSave: (t: Template) => Promise<void>;
  onCancel: () => void;
  onMutexTransfer?: (fromName: string) => void;
}

export function TemplateEditor({
  template,
  isNew,
  onSave,
  onCancel,
  onMutexTransfer,
}: Props) {
  const [displayName, setDisplayName] = useState(template.displayName);
  const [variables, setVariables] = useState<Variable[]>(template.variables);
  const [tags, setTags] = useState<string[]>(template.tags);

  // Body is held in storage form (`{<guid>}` placeholders) so SPEC §13
  // invariant 1 holds without any body rewriting on rename. The textarea
  // value is derived from this on every render via bodyToDisplay; user
  // input round-trips through bodyToStorage on every keystroke.
  const [bodyStorage, setBodyStorage] = useState(template.body);

  const bodyDisplay = useMemo(
    () => bodyToDisplay(bodyStorage, variables),
    [bodyStorage, variables]
  );

  // Track which variable's displayName was edited last. Used to scope the
  // duplicate-name error to the variable the user is actively renaming
  // (rather than flagging both / all variants of the duplicate group).
  const [lastEditedGuid, setLastEditedGuid] = useState<string | null>(null);

  const [saving, setSaving] = useState(false);

  const handleBodyChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setBodyStorage(bodyToStorage(e.target.value, variables));
  };

  const handleVariablesChange = (newVars: Variable[]) => {
    let editedGuid: string | null = null;
    for (const newVar of newVars) {
      const old = variables.find((v) => v.guid === newVar.guid);
      if (old && old.displayName !== newVar.displayName) {
        editedGuid = newVar.guid;
        break;
      }
    }
    if (editedGuid !== null) {
      setLastEditedGuid(editedGuid);
    }
    setVariables(newVars);
  };

  const handleVariableDelete = (deletedGuid: string) => {
    // SPEC §13 invariant 2: clear the deleted variable's placeholders
    // from body. Working in storage form means the match is exact and
    // never confused by name collisions.
    setBodyStorage((b) =>
      b.replace(new RegExp(`\\{${escapeRegex(deletedGuid)}\\}`, "g"), "")
    );
    if (lastEditedGuid === deletedGuid) {
      setLastEditedGuid(null);
    }
  };

  // Per-variable error map for inline display. Duplicate-name errors are
  // shown only on the most recently edited variable (the one the user is
  // actively renaming); save blocking uses hasBlockingErrors below.
  const variableErrors = useMemo(() => {
    const errors = new Map<string, string>();
    const nameCount = new Map<string, number>();
    for (const v of variables) {
      const trimmed = v.displayName.trim();
      if (trimmed) {
        nameCount.set(trimmed, (nameCount.get(trimmed) ?? 0) + 1);
      }
    }
    for (const v of variables) {
      const trimmed = v.displayName.trim();
      if (!trimmed) {
        errors.set(v.guid, "显示名不能为空");
      } else if (
        (nameCount.get(trimmed) ?? 0) > 1 &&
        v.guid === lastEditedGuid
      ) {
        errors.set(v.guid, "显示名与其它变量重复");
      }
      if (
        v.type === "enum" &&
        (!v.options || v.options.length === 0) &&
        !errors.has(v.guid)
      ) {
        errors.set(v.guid, "枚举至少需要一个选项");
      }
    }
    return errors;
  }, [variables, lastEditedGuid]);

  // Save-blocking validation, independent of the per-variable error display.
  // We always block save on any duplicate, even if no per-variable error is
  // shown (e.g. when the editor opens with pre-existing duplicates).
  const hasBlockingErrors = useMemo(() => {
    if (variables.some((v) => !v.displayName.trim())) return true;
    if (
      variables.some(
        (v) => v.type === "enum" && (!v.options || v.options.length === 0)
      )
    ) {
      return true;
    }
    const names = variables
      .map((v) => v.displayName.trim())
      .filter((n) => n.length > 0);
    if (new Set(names).size !== names.length) return true;
    return false;
  }, [variables]);

  const canSave =
    displayName.trim().length > 0 && !hasBlockingErrors && !saving;

  const handleSave = async () => {
    if (!canSave) return;
    setSaving(true);
    try {
      await onSave({
        ...template,
        displayName: displayName.trim(),
        body: bodyStorage,
        variables: variables.map((v) => ({
          ...v,
          displayName: v.displayName.trim(),
        })),
        tags,
      });
    } catch (e) {
      console.error("save failed", e);
      setSaving(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.nativeEvent.isComposing) return;
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      handleSave();
    } else if (e.key === "Escape") {
      e.preventDefault();
      onCancel();
    }
  };

  return (
    <div onKeyDown={handleKeyDown}>
      <div className="flex items-center justify-between border-b border-amber-200 bg-amber-50 px-6 py-1.5 text-xs uppercase tracking-wide text-amber-800">
        <span>编辑模式</span>
      </div>

      <div className="p-6">
        <div className="mx-auto max-w-3xl">
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-base font-semibold tracking-tight">
              {isNew ? "新建模板" : "编辑模板"}
            </h2>
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={onCancel}
                disabled={saving}
                className="inline-flex items-center gap-1.5 rounded border border-zinc-300 bg-white px-3 py-1.5 text-sm font-medium text-zinc-700 hover:bg-zinc-50 disabled:opacity-50"
              >
                <X size={14} />
                取消
              </button>
              <button
                type="button"
                onClick={handleSave}
                disabled={!canSave}
                className="inline-flex items-center gap-1.5 rounded bg-amber-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-amber-700 disabled:opacity-50"
              >
                <Save size={14} />
                {saving ? "保存中…" : "保存模板"}
              </button>
            </div>
          </div>

          <div className="space-y-3">
            <Section title="显示名">
              <input
                type="text"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                autoFocus
                placeholder="例如：邮箱"
                className="w-full rounded border border-zinc-300 px-3 py-2 text-sm focus:border-zinc-500 focus:outline-none"
              />
            </Section>

            <Section title="标签">
              <TagInput tags={tags} onChange={setTags} />
            </Section>

            <Section title="正文">
              <textarea
                value={bodyDisplay}
                onChange={handleBodyChange}
                rows={6}
                placeholder="模板正文。引用变量用 {显示名}，例如 {Language}。"
                className="w-full rounded border border-zinc-300 px-3 py-2 font-mono text-sm focus:border-zinc-500 focus:outline-none"
              />
            </Section>

            <Section>
              <VariableList
                variables={variables}
                errors={variableErrors}
                onChange={handleVariablesChange}
                onMutexTransfer={(name) => onMutexTransfer?.(name)}
                onDelete={handleVariableDelete}
              />
            </Section>

            <div className="text-xs text-zinc-500">
              Cmd/Ctrl+Enter 保存 · Esc 取消
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Section({
  title,
  children,
}: {
  title?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="rounded border border-zinc-200 bg-white p-3">
      {title && (
        <div className="mb-1.5 text-xs font-medium uppercase tracking-wide text-zinc-500">
          {title}
        </div>
      )}
      {children}
    </div>
  );
}

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
