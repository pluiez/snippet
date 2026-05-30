import { useEffect } from "react";
import { Check } from "lucide-react";

interface Props {
  message: string;
  onDismiss: () => void;
  durationMs?: number;
}

export function Toast({ message, onDismiss, durationMs = 2500 }: Props) {
  useEffect(() => {
    const t = setTimeout(onDismiss, durationMs);
    return () => clearTimeout(t);
  }, [onDismiss, durationMs]);

  return (
    <div className="pointer-events-none fixed bottom-6 left-1/2 -translate-x-1/2">
      <div className="pointer-events-auto inline-flex items-center gap-2 rounded-lg bg-zinc-900 px-4 py-2 text-sm text-white shadow-lg">
        <Check size={14} className="text-green-400" />
        {message}
      </div>
    </div>
  );
}
