import { useEffect } from "react";

interface Props {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  destructive?: boolean;
  onConfirm: () => Promise<void> | void;
  onCancel: () => void;
}

export function ConfirmDialog({
  title,
  message,
  confirmText = "确定",
  cancelText = "取消",
  destructive,
  onConfirm,
  onCancel,
}: Props) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
      <div className="w-full max-w-sm rounded-lg bg-white p-5 shadow-xl dark:bg-zinc-800">
        <h3 className="mb-2 text-base font-semibold text-zinc-900 dark:text-zinc-100">{title}</h3>
        <p className="mb-5 text-sm leading-relaxed text-zinc-600 dark:text-zinc-400">{message}</p>
        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="rounded border border-zinc-300 bg-white px-3 py-1.5 text-sm font-medium text-zinc-700 hover:bg-zinc-50 dark:border-zinc-600 dark:bg-zinc-700 dark:text-zinc-300 dark:hover:bg-zinc-600"
          >
            {cancelText}
          </button>
          <button
            type="button"
            onClick={onConfirm}
            autoFocus
            className={
              "rounded px-3 py-1.5 text-sm font-medium text-white " +
              (destructive
                ? "bg-red-600 hover:bg-red-700 dark:bg-red-500 dark:hover:bg-red-600"
                : "bg-zinc-900 hover:bg-zinc-800 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-200")
            }
          >
            {confirmText}
          </button>
        </div>
      </div>
    </div>
  );
}
