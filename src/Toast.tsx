import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { AlertTriangle, Check } from "lucide-react";
import { DURATION, EASE } from "./lib/motion";

export type ToastVariant = "success" | "error" | "warning";

const DEFAULT_DURATION: Record<ToastVariant, number> = {
  success: 2500,
  error: 4000,
  warning: 4000,
};

const ICON_CLASS: Record<ToastVariant, string> = {
  success: "text-green-400 dark:text-green-600",
  error: "text-red-400 dark:text-red-500",
  warning: "text-amber-400 dark:text-amber-500",
};

interface Props {
  message: string;
  onDismiss: () => void;
  variant?: ToastVariant;
  durationMs?: number;
}

export function Toast({ message, onDismiss, variant = "success", durationMs }: Props) {
  const resolvedDuration = durationMs ?? DEFAULT_DURATION[variant];
  const [exiting, setExiting] = useState(false);
  const exitDurationMs = DURATION.fast * 1000;

  useEffect(() => {
    const fadeOutAt = Math.max(resolvedDuration - exitDurationMs, 0);
    const t1 = setTimeout(() => setExiting(true), fadeOutAt);
    const t2 = setTimeout(onDismiss, resolvedDuration);
    return () => {
      clearTimeout(t1);
      clearTimeout(t2);
    };
  }, [onDismiss, resolvedDuration, exitDurationMs]);

  const Icon = variant === "success" ? Check : AlertTriangle;

  return (
    <div className="pointer-events-none fixed bottom-6 left-1/2 z-50 -translate-x-1/2">
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: exiting ? 0 : 1, y: exiting ? -8 : 0 }}
        transition={{ duration: DURATION.fast, ease: EASE.out }}
        className="pointer-events-auto inline-flex items-center gap-2 rounded-lg bg-zinc-900 px-4 py-2 text-sm text-white shadow-lg dark:bg-zinc-100 dark:text-zinc-900"
      >
        <Icon size={14} className={ICON_CLASS[variant]} />
        {message}
      </motion.div>
    </div>
  );
}
