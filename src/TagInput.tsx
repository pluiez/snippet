import { useState } from "react";
import { X } from "lucide-react";
import { tagColor, useColorMaps } from "./lib/colors";

interface Props {
  tags: string[];
  onChange: (tags: string[]) => void;
}

export function TagInput({ tags, onChange }: Props) {
  const maps = useColorMaps();
  const [input, setInput] = useState("");

  const handleAdd = () => {
    const trimmed = input.trim();
    if (!trimmed) return;
    // SPEC §2.3: tag comparison is case-insensitive (lowercased), display preserves original.
    if (tags.some((t) => t.toLowerCase() === trimmed.toLowerCase())) {
      setInput("");
      return;
    }
    onChange([...tags, trimmed]);
    setInput("");
  };

  const handleRemove = (idx: number) => {
    onChange(tags.filter((_, i) => i !== idx));
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.nativeEvent.isComposing) return;
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      handleAdd();
    } else if (e.key === "Backspace" && input === "" && tags.length > 0) {
      handleRemove(tags.length - 1);
    }
  };

  return (
    <div className="flex flex-wrap items-center gap-1.5 rounded border border-zinc-300 bg-white px-2 py-1.5">
      {tags.map((tag, idx) => (
        <span
          key={`${tag}-${idx}`}
          style={{ backgroundColor: tagColor(tag, maps) }}
          className="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs text-white"
        >
          {tag}
          <button
            type="button"
            onClick={() => handleRemove(idx)}
            className="text-white/70 hover:text-white"
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
        placeholder={tags.length === 0 ? "输入标签后按 Enter" : ""}
        className="min-w-[100px] flex-1 bg-transparent text-sm outline-none"
      />
    </div>
  );
}
