"use client";

import React, {
  createContext,
  useCallback,
  useContext,
  useState,
  useRef,
  type ReactNode,
} from "react";

/* ------------------------------------------------------------------ */
/*  Types                                                              */
/* ------------------------------------------------------------------ */

export type ToastSeverity = "success" | "error" | "info" | "warning";

export interface ToastItem {
  id: string;
  severity: ToastSeverity;
  title: string;
  detail?: string;
  duration?: number;
}

interface ToastContextValue {
  toasts: ToastItem[];
  addToast: (toast: Omit<ToastItem, "id">) => string;
  removeToast: (id: string) => void;
}

/* ------------------------------------------------------------------ */
/*  Context                                                            */
/* ------------------------------------------------------------------ */

const ToastContext = createContext<ToastContextValue | undefined>(undefined);

let toastCounter = 0;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const timers = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
    const timer = timers.current.get(id);
    if (timer) {
      clearTimeout(timer);
      timers.current.delete(id);
    }
  }, []);

  const addToast = useCallback(
    (toast: Omit<ToastItem, "id">) => {
      const id = `toast-${++toastCounter}`;
      const duration = toast.duration ?? 4000;

      setToasts((prev) => [...prev.slice(-4), { ...toast, id }]);

      if (duration > 0) {
        const timer = setTimeout(() => removeToast(id), duration);
        timers.current.set(id, timer);
      }

      return id;
    },
    [removeToast],
  );

  return (
    <ToastContext.Provider value={{ toasts, addToast, removeToast }}>
      {children}
      <ToastContainer toasts={toasts} onDismiss={removeToast} />
    </ToastContext.Provider>
  );
}

export function useToast() {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error("useToast must be used within ToastProvider");
  return ctx;
}

/* ------------------------------------------------------------------ */
/*  Visual helpers                                                     */
/* ------------------------------------------------------------------ */

const SEVERITY_CONFIG: Record<
  ToastSeverity,
  { icon: string; border: string; bg: string; titleColor: string }
> = {
  success: {
    icon: "\u2714",
    border: "border-emerald-500/40",
    bg: "from-emerald-500/15 to-emerald-600/5",
    titleColor: "text-emerald-400",
  },
  error: {
    icon: "\u2716",
    border: "border-red-500/40",
    bg: "from-red-500/15 to-red-600/5",
    titleColor: "text-red-400",
  },
  info: {
    icon: "\u2139",
    border: "border-blue-500/40",
    bg: "from-blue-500/15 to-blue-600/5",
    titleColor: "text-blue-400",
  },
  warning: {
    icon: "\u26A0",
    border: "border-yellow-500/40",
    bg: "from-yellow-500/15 to-yellow-600/5",
    titleColor: "text-yellow-400",
  },
};

/* ------------------------------------------------------------------ */
/*  Container                                                          */
/* ------------------------------------------------------------------ */

function ToastContainer({
  toasts,
  onDismiss,
}: {
  toasts: ToastItem[];
  onDismiss: (id: string) => void;
}) {
  if (toasts.length === 0) return null;

  return (
    <div
      className="fixed bottom-4 right-4 z-[100] flex flex-col-reverse gap-2 max-w-sm w-full pointer-events-none"
      aria-live="polite"
    >
      {toasts.map((toast) => {
        const cfg = SEVERITY_CONFIG[toast.severity];
        return (
          <div
            key={toast.id}
            className={`pointer-events-auto animate-toast-in bg-gradient-to-r ${cfg.bg} backdrop-blur-md border ${cfg.border} rounded-xl px-4 py-3 shadow-2xl flex items-start gap-3 transition-all duration-300`}
            role="alert"
          >
            <span
              className={`text-lg leading-none mt-0.5 ${cfg.titleColor}`}
            >
              {cfg.icon}
            </span>
            <div className="flex-1 min-w-0">
              <p className={`text-sm font-semibold ${cfg.titleColor}`}>
                {toast.title}
              </p>
              {toast.detail && (
                <p className="text-xs text-gray-300 mt-0.5 leading-relaxed">
                  {toast.detail}
                </p>
              )}
            </div>
            <button
              onClick={() => onDismiss(toast.id)}
              className="text-gray-500 hover:text-white transition-colors text-sm leading-none mt-0.5"
              aria-label="Dismiss notification"
            >
              \u2715
            </button>
          </div>
        );
      })}
    </div>
  );
}
