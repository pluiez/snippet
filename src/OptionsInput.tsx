import { useState } from "react";
import { X } from "lucide-react";

interface Props {
  options: string[];
  onChange: (options: string[]) => void;
}

export function OptionsInput({ options, onChange }: Props) {
  const [input, setInput] = useState("");

  const handleAdd = () => {
    const trimmed = input.trim();
    if (!trimmed) return;
    if (options.includes(trimmed)) {
      setInput("");
      return;
    }
    onChange([...options, trimmed]);
    setInput("");
  };

  const handleRemove = (idx: number) => {
    onChange(options.filter((_, i) => i !== idx));
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.nativeEvent.isComposing) return;
    if (e.key === "Enter") {
      e.preventDefault();
      handleAdd();
    } else if (e.key === "Backspace" && input === "" && options.length > 0) {
      handleRemove(options.length - 1);
    }
  };

  return (
    <div className="flex flex-wrap items-center gap-1.5 rounded border border-zinc-300 bg-white px-2 py-1 dark:border-zinc-600 dark:bg-zinc-900">
      {options.map((opt, idx) => (
        <span
          key={`${opt}-${idx}`}
          className="inline-flex items-center gap-1 rounded bg-blue-50 px-1.5 py-0.5 text-xs text-blue-700 dark:bg-blue-950/40 dark:text-blue-300"
        >
          {opt}
          <button
            type="button"
            onClick={() => handleRemove(idx)}
            className="text-blue-400 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-200"
          >
            <X size={10} />
          </button>
        </span>
      ))}
      <input
        type="text"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={handleKeyDown}
        onBlur={handleAdd}
        placeholder={options.length === 0 ? "输入选项后按 Enter" : ""}
        className="min-w-[80px] flex-1 bg-transparent text-sm outline-none dark:text-zinc-100 dark:placeholder:text-zinc-500"
      />
    </div>
  );
}
