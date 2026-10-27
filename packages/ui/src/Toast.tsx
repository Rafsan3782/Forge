import React, { createContext, useContext, useState } from "react";
import { Icon } from "./Icon.js";

export interface ToastMessage {
  id: string;
  title: string;
  description?: string;
  type?: "info" | "success" | "warn" | "error";
  duration?: number;
}

interface ToastContextValue {
  toast: (msg: Omit<ToastMessage, "id">) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

export const ToastProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [toasts, setToasts] = useState<ToastMessage[]>([]);

  const toast = (msg: Omit<ToastMessage, "id">) => {
    const id = Math.random().toString(36).substring(2, 9);
    const newToast: ToastMessage = { ...msg, id };
    setToasts((prev) => [...prev, newToast]);

    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, msg.duration || 3000);
  };

  return (
    <ToastContext.Provider value={{ toast }}>
      {children}
      <div
        style={{
          position: "fixed",
          bottom: 16,
          right: 16,
          zIndex: "var(--forge-z-toast)",
          display: "flex",
          flexDirection: "column",
          gap: 8,
          pointerEvents: "none",
        }}
      >
        {toasts.map((t) => (
          <div
            key={t.id}
            style={{
              pointerEvents: "auto",
              background: "var(--forge-surface-overlay)",
              border: "1px solid var(--forge-border-strong)",
              borderRadius: "var(--forge-radius-sm)",
              padding: "8px 12px",
              boxShadow: "var(--forge-elevation-medium)",
              color: "var(--forge-text-primary)",
              minWidth: 220,
              maxWidth: 320,
              display: "flex",
              alignItems: "flex-start",
              gap: 8,
              fontSize: "var(--forge-font-size-xs)",
            }}
          >
            <Icon
              name={
                t.type === "success"
                  ? "check-circle"
                  : t.type === "warn"
                  ? "alert-triangle"
                  : t.type === "error"
                  ? "alert-circle"
                  : "info"
              }
              size={14}
              color={
                t.type === "success"
                  ? "var(--forge-success)"
                  : t.type === "warn"
                  ? "var(--forge-warn)"
                  : t.type === "error"
                  ? "var(--forge-danger)"
                  : "var(--forge-info)"
              }
            />
            <div style={{ flex: 1 }}>
              <div style={{ fontWeight: 600 }}>{t.title}</div>
              {t.description && <div style={{ color: "var(--forge-text-muted)" }}>{t.description}</div>}
            </div>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
};

export const useToast = (): ToastContextValue => {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    return {
      toast: () => {},
    };
  }
  return ctx;
};
