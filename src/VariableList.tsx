import { Plus } from "lucide-react";
import type { Variable } from "./lib/bindings/Variable";
import { VariableEditor } from "./VariableEditor";

interface Props {
  variables: Variable[];
  errors: Map<string, string>;
  onChange: (vars: Variable[]) => void;
  onMutexTransfer: (fromName: string) => void;
  // Receives the deleted variable's GUID (not displayName) so the parent can
  // strip placeholders from body in storage form unambiguously, even when
  // names collide.
  onDelete: (deletedGuid: string) => void;
}

export function VariableList({
  variables,
  errors,
  onChange,
  onMutexTransfer,
  onDelete,
}: Props) {
  const handleAdd = () => {
    const newVar: Variable = {
      guid: crypto.randomUUID(),
      displayName: uniqueName("变量", variables),
      type: "text",
      options: null,
      required: true,
      fillFromClipboard: false,
      rememberLastUsed: false,
      staticDefault: null,
    };
    onChange([...variables, newVar]);
  };

  const handleVariableChange = (idx: number, updated: Variable) => {
    const old = variables[idx];
    let result = [...variables];
    result[idx] = updated;

    // SPEC §5.4: at most one variable per template can have fillFromClipboard.
    // When the flag is newly set on B, silently clear it on whoever had it (A).
    if (updated.fillFromClipboard && !old.fillFromClipboard) {
      let clearedName: string | null = null;
      result = result.map((v, i) => {
        if (i !== idx && v.fillFromClipboard) {
          clearedName = v.displayName;
          return { ...v, fillFromClipboard: false };
        }
        return v;
      });
      if (clearedName) {
        onMutexTransfer(clearedName);
      }
    }

    onChange(result);
  };

  const handleDelete = (idx: number) => {
    const removed = variables[idx];
    onChange(variables.filter((_, i) => i !== idx));
    onDelete(removed.guid);
  };

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <div className="text-xs font-medium uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
          变量（{variables.length}）
        </div>
        <button
          type="button"
          onClick={handleAdd}
          className="inline-flex items-center gap-1 rounded border border-zinc-300 bg-white px-2 py-0.5 text-xs font-medium text-zinc-700 hover:bg-zinc-50 dark:border-zinc-600 dark:bg-zinc-800 dark:text-zinc-300 dark:hover:bg-zinc-700"
        >
          <Plus size={12} />
          添加变量
        </button>
      </div>

      {variables.length === 0 ? (
        <div className="rounded border border-dashed border-zinc-300 p-4 text-center text-sm text-zinc-500 dark:border-zinc-600 dark:text-zinc-400">
          暂无变量。在 body 中用{" "}
          <code className="rounded bg-zinc-100 px-1 py-0.5 text-xs dark:bg-zinc-700 dark:text-zinc-300">
            {"{显示名}"}
          </code>{" "}
          引用变量。
        </div>
      ) : (
        variables.map((v, idx) => (
          <VariableEditor
            key={v.guid}
            variable={v}
            error={errors.get(v.guid)}
            onChange={(updated) => handleVariableChange(idx, updated)}
            onDelete={() => handleDelete(idx)}
          />
        ))
      )}
    </div>
  );
}

function uniqueName(base: string, existing: Variable[]): string {
  let n = 1;
  let candidate = base;
  while (existing.some((v) => v.displayName === candidate)) {
    n++;
    candidate = `${base}${n}`;
  }
  return candidate;
}
