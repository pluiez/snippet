import { useMemo, useState } from "react";
import { Copy, Unlock, X } from "lucide-react";
import type { FillDialogState } from "./lib/bindings/FillDialogState";
import type { Variable } from "./lib/bindings/Variable";
import { render } from "./lib/render";
import { useColorMaps, variableColor } from "./lib/colors";

interface Props {
  state: FillDialogState;
  onApply: (values: Record<string, string>) => Promise<void>;
  onUnlock: (values: Record<string, string>) => void;
  onCancel: () => void;
}

export function TemplateFillDialog({
  state,
  onApply,
  onUnlock,
  onCancel,
}: Props) {
  const [values, setValues] = useState<Record<string, string>>(
    state.initialValues
  );
  const [submitting, setSubmitting] = useState(false);

  const allRequiredFilled = useMemo(
    () =>
      state.orderedVariables.every(
        (v) => !v.required || (values[v.guid] ?? "").trim().length > 0
      ),
    [state.orderedVariables, values]
  );
  const canSubmit = allRequiredFilled && !submitting;

  const preview = useMemo(
    () => render(state.template.body, values),
    [state.template.body, values]
  );

  const handleSubmit = async () => {
    if (!canSubmit) return;
    setSubmitting(true);
    try {
      await onApply(values);
    } catch (e) {
      console.error("apply failed", e);
      setSubmitting(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.nativeEvent.isComposing) return;
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      handleSubmit();
    } else if (e.key === "Escape") {
      e.preventDefault();
      onCancel();
    }
  };

  return (
    <div onKeyDown={handleKeyDown}>
      <div className="border-b border-zinc-200 bg-zinc-50 px-6 py-1.5 text-xs uppercase tracking-wide text-zinc-600">
        填充模式
      </div>

      <div className="p-6">
        <div className="mx-auto max-w-5xl">
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-base font-semibold tracking-tight">
              试用 ·{" "}
              <span className="text-zinc-700">
                {state.template.displayName}
              </span>
            </h2>
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={() => onUnlock(values)}
                disabled={submitting}
                className="inline-flex items-center gap-1.5 rounded border border-zinc-300 bg-white px-3 py-1.5 text-sm font-medium text-zinc-700 hover:bg-zinc-50 disabled:opacity-50"
              >
                <Unlock size={14} />
                解锁编辑
              </button>
              <button
                type="button"
                onClick={onCancel}
                disabled={submitting}
                className="inline-flex items-center gap-1.5 rounded border border-zinc-300 bg-white px-3 py-1.5 text-sm font-medium text-zinc-700 hover:bg-zinc-50 disabled:opacity-50"
              >
                <X size={14} />
                取消
              </button>
              <button
                type="button"
                onClick={handleSubmit}
                disabled={!canSubmit}
                className="inline-flex items-center gap-1.5 rounded bg-zinc-900 px-3 py-1.5 text-sm font-medium text-white hover:bg-zinc-800 disabled:opacity-50"
              >
                <Copy size={14} />
                {submitting ? "复制中…" : "复制"}
              </button>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-4 rounded border border-zinc-200 bg-white p-5">
              <div className="text-xs font-medium uppercase tracking-wide text-zinc-500">
                填充变量
              </div>
              {state.orderedVariables.length === 0 ? (
                <div className="text-sm text-zinc-500">该模板没有变量</div>
              ) : (
                state.orderedVariables.map((v) => (
                  <VariableField
                    key={v.guid}
                    variable={v}
                    value={values[v.guid] ?? ""}
                    onChange={(val) =>
                      setValues((s) => ({ ...s, [v.guid]: val }))
                    }
                  />
                ))
              )}
            </div>

            <div className="rounded border border-zinc-200 bg-white p-5">
              <div className="mb-3 text-xs font-medium uppercase tracking-wide text-zinc-500">
                实时预览
              </div>
              <pre className="whitespace-pre-wrap break-words font-mono text-sm text-zinc-900">
                {preview || (
                  <span className="italic text-zinc-400">（空）</span>
                )}
              </pre>
            </div>
          </div>

          <div className="mt-4 text-xs text-zinc-500">
            Cmd/Ctrl+Enter 复制 · Esc 取消
          </div>
        </div>
      </div>
    </div>
  );
}

function VariableField({
  variable,
  value,
  onChange,
}: {
  variable: Variable;
  value: string;
  onChange: (v: string) => void;
}) {
  const maps = useColorMaps();
  const color = variableColor(variable.displayName, maps);
  return (
    <div>
      <label
        htmlFor={`field-${variable.guid}`}
        className="mb-1.5 flex items-center gap-1.5 text-xs font-medium uppercase tracking-wide text-zinc-500"
      >
        {/* SPEC §4.5: label color matches the variable color. Small dot avoids
            low contrast issues with colored text on white. */}
        <span
          className="inline-block h-2 w-2 shrink-0 rounded-full"
          style={{ backgroundColor: color }}
        />
        <span>{variable.displayName}</span>
        {variable.required && <span className="text-red-500">*</span>}
      </label>
      {variable.type === "enum" ? (
        <select
          id={`field-${variable.guid}`}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="w-full rounded border border-zinc-300 px-3 py-2 text-sm focus:border-zinc-500 focus:outline-none"
        >
          {value === "" && <option value="">（请选择）</option>}
          {(variable.options ?? []).map((opt) => (
            <option key={opt} value={opt}>
              {opt}
            </option>
          ))}
        </select>
      ) : (
        <textarea
          id={`field-${variable.guid}`}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          rows={3}
          className="w-full rounded border border-zinc-300 px-3 py-2 font-mono text-sm focus:border-zinc-500 focus:outline-none"
        />
      )}
    </div>
  );
}
