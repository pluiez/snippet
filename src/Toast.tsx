import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { Check } from "lucide-react";
import { DURATION, EASE } from "./lib/motion";

interface Props {
  message: string;
  onDismiss: () => void;
  durationMs?: number;
}

export function Toast({ message, onDismiss, durationMs = 2500 }: Props) {
  const [exiting, setExiting] = useState(false);
  const exitDurationMs = DURATION.fast * 1000;

  useEffect(() => {
    const fadeOutAt = Math.max(durationMs - exitDurationMs, 0);
    const t1 = setTimeout(() => setExiting(true), fadeOutAt);
    const t2 = setTimeout(onDismiss, durationMs);
    return () => {
      clearTimeout(t1);
      clearTimeout(t2);
    };
  }, [onDismiss, durationMs, exitDurationMs]);

  return (
    <div className="pointer-events-none fixed bottom-6 left-1/2 z-50 -translate-x-1/2">
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: exiting ? 0 : 1, y: exiting ? -8 : 0 }}
        transition={{ duration: DURATION.fast, ease: EASE.out }}
        className="pointer-events-auto inline-flex items-center gap-2 rounded-lg bg-zinc-900 px-4 py-2 text-sm text-white shadow-lg dark:bg-zinc-100 dark:text-zinc-900"
      >
        <Check size={14} className="text-green-400 dark:text-green-600" />
        {message}
      </motion.div>
    </div>
  );
}
