import { Trash2 } from "lucide-react";
import type { Variable, VariableType } from "./lib/bindings/Variable";
import { OptionsInput } from "./OptionsInput";

interface Props {
  variable: Variable;
  error?: string;
  onChange: (v: Variable) => void;
  onDelete: () => void;
}

export function VariableEditor({ variable, error, onChange, onDelete }: Props) {
  const setField = <K extends keyof Variable>(key: K, value: Variable[K]) => {
    onChange({ ...variable, [key]: value });
  };

  const handleTypeChange = (newType: VariableType) => {
    if (newType === variable.type) return;
    // B3: type change clears staticDefault. Initialize options for enum.
    onChange({
      ...variable,
      type: newType,
      staticDefault: null,
      options: newType === "enum" ? variable.options ?? [] : null,
    });
  };

  const handleOptionsChange = (newOptions: string[]) => {
    // B2: if current staticDefault is no longer in options, clear it.
    let staticDefault = variable.staticDefault;
    if (staticDefault !== null && !newOptions.includes(staticDefault)) {
      staticDefault = null;
    }
    onChange({ ...variable, options: newOptions, staticDefault });
  };

  return (
    <div className="rounded border border-zinc-200 bg-white p-3 dark:border-zinc-700 dark:bg-zinc-800">
      <div className="mb-2 flex items-start gap-2">
        <div className="flex-1">
          <label className="mb-1 block text-[10px] font-medium uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
            显示名
          </label>
          <input
            type="text"
            value={variable.displayName}
            onChange={(e) => setField("displayName", e.target.value)}
            placeholder="例如：Language"
            className={
              "w-full rounded border px-2 py-1 text-sm focus:outline-none dark:bg-zinc-900 dark:text-zinc-100 " +
              (error
                ? "border-red-400 focus:border-red-500 dark:border-red-500"
                : "border-zinc-300 focus:border-zinc-500 dark:border-zinc-600 dark:focus:border-zinc-400")
            }
          />
        </div>
        <div className="w-24">
          <label className="mb-1 block text-[10px] font-medium uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
            类型
          </label>
          <select
            value={variable.type}
            onChange={(e) => handleTypeChange(e.target.value as VariableType)}
            className="w-full rounded border border-zinc-300 px-2 py-1 text-sm focus:border-zinc-500 focus:outline-none dark:border-zinc-600 dark:bg-zinc-900 dark:text-zinc-100 dark:focus:border-zinc-400"
          >
            <option value="text">text</option>
            <option value="enum">enum</option>
          </select>
        </div>
        <button
          type="button"
          title="删除变量"
          onClick={onDelete}
          className="mt-5 rounded p-1 text-zinc-500 hover:bg-red-50 hover:text-red-600 dark:text-zinc-400 dark:hover:bg-red-950/40 dark:hover:text-red-400"
        >
          <Trash2 size={14} />
        </button>
      </div>

      {error && (
        <div className="mb-2 rounded bg-red-50 px-2 py-1 text-xs text-red-600 dark:bg-red-950/40 dark:text-red-300">
          {error}
        </div>
      )}

      {variable.type === "enum" && (
        <div className="mb-2">
          <label className="mb-1 block text-[10px] font-medium uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
            选项
          </label>
          <OptionsInput
            options={variable.options ?? []}
            onChange={handleOptionsChange}
          />
        </div>
      )}

      <div className="mb-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-zinc-700 dark:text-zinc-300">
        <Checkbox
          checked={variable.required}
          onChange={(v) => setField("required", v)}
          label="必填"
        />
        <Checkbox
          checked={variable.fillFromClipboard}
          onChange={(v) => setField("fillFromClipboard", v)}
          label="从剪贴板填充"
        />
        <Checkbox
          checked={variable.rememberLastUsed}
          onChange={(v) => setField("rememberLastUsed", v)}
          label="记住上次值"
        />
      </div>

      <div>
        <label className="mb-1 block text-[10px] font-medium uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
          静态默认值
        </label>
        {variable.type === "enum" ? (
          <select
            value={variable.staticDefault ?? ""}
            onChange={(e) =>
              setField(
                "staticDefault",
                e.target.value === "" ? null : e.target.value
              )
            }
            className="w-full rounded border border-zinc-300 px-2 py-1 text-sm focus:border-zinc-500 focus:outline-none dark:border-zinc-600 dark:bg-zinc-900 dark:text-zinc-100 dark:focus:border-zinc-400"
          >
            <option value="">（无）</option>
            {(variable.options ?? []).map((opt) => (
              <option key={opt} value={opt}>
                {opt}
              </option>
            ))}
          </select>
        ) : (
          <input
            type="text"
            value={variable.staticDefault ?? ""}
            onChange={(e) =>
              setField(
                "staticDefault",
                e.target.value === "" ? null : e.target.value
              )
            }
            placeholder="（无）"
            className="w-full rounded border border-zinc-300 px-2 py-1 text-sm focus:border-zinc-500 focus:outline-none dark:border-zinc-600 dark:bg-zinc-900 dark:text-zinc-100 dark:focus:border-zinc-400"
          />
        )}
      </div>
    </div>
  );
}

function Checkbox({
  checked,
  onChange,
  label,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  label: string;
}) {
  return (
    <label className="inline-flex cursor-pointer items-center gap-1.5">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="rounded border-zinc-300 text-zinc-900 focus:ring-zinc-500 dark:border-zinc-600"
      />
      <span>{label}</span>
    </label>
  );
}
