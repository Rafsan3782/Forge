import React, { useState, useRef, useEffect } from "react";
import { Icon } from "./Icon.js";

export interface MenuItem {
  id: string;
  label: string;
  icon?: string;
  shortcut?: string;
  danger?: boolean;
  disabled?: boolean;
  action: () => void;
}

export interface DropdownMenuProps {
  trigger: React.ReactNode;
  items: MenuItem[];
  align?: "left" | "right";
}

export const DropdownMenu: React.FC<DropdownMenuProps> = ({
  trigger,
  items,
  align = "left",
}) => {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    window.addEventListener("mousedown", handleClickOutside);
    return () => window.removeEventListener("mousedown", handleClickOutside);
  }, []);

  return (
    <div ref={ref} style={{ position: "relative", display: "inline-block" }}>
      <div onClick={() => setOpen(!open)}>{trigger}</div>
      {open && (
        <div
          style={{
            position: "absolute",
            top: "100%",
            marginTop: 4,
            [align === "right" ? "right" : "left"]: 0,
            background: "var(--forge-surface-overlay)",
            border: "1px solid var(--forge-border-strong)",
            borderRadius: "var(--forge-radius-sm)",
            boxShadow: "var(--forge-elevation-medium)",
            zIndex: "var(--forge-z-dropdown)",
            padding: 4,
            minWidth: 160,
          }}
        >
          {items.map((item) => (
            <button
              key={item.id}
              disabled={item.disabled}
              onClick={() => {
                item.action();
                setOpen(false);
              }}
              style={{
                width: "100%",
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                gap: 8,
                padding: "4px 8px",
                border: "none",
                background: "transparent",
                color: item.danger ? "var(--forge-danger)" : "var(--forge-text-primary)",
                fontSize: "var(--forge-font-size-xs)",
                borderRadius: "calc(var(--forge-radius-sm) - 2px)",
                cursor: item.disabled ? "not-allowed" : "pointer",
                opacity: item.disabled ? 0.45 : 1,
                textAlign: "left",
              }}
            >
              <span style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
                {item.icon && <Icon name={item.icon} size={14} />}
                <span>{item.label}</span>
              </span>
              {item.shortcut && <kbd className="forge-kbd">{item.shortcut}</kbd>}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};

export interface ContextMenuProps {
  items: MenuItem[];
  children: React.ReactNode;
}

export const ContextMenu: React.FC<ContextMenuProps> = ({ items, children }) => {
  const [coords, setCoords] = useState<{ x: number; y: number } | null>(null);

  useEffect(() => {
    const handleGlobalClick = () => setCoords(null);
    window.addEventListener("click", handleGlobalClick);
    return () => window.removeEventListener("click", handleGlobalClick);
  }, []);

  return (
    <div
      onContextMenu={(e) => {
        e.preventDefault();
        setCoords({ x: e.clientX, y: e.clientY });
      }}
    >
      {children}
      {coords && (
        <div
          style={{
            position: "fixed",
            top: coords.y,
            left: coords.x,
            background: "var(--forge-surface-overlay)",
            border: "1px solid var(--forge-border-strong)",
            borderRadius: "var(--forge-radius-sm)",
            boxShadow: "var(--forge-elevation-medium)",
            zIndex: "var(--forge-z-popover)",
            padding: 4,
            minWidth: 160,
          }}
        >
          {items.map((item) => (
            <button
              key={item.id}
              disabled={item.disabled}
              onClick={() => {
                item.action();
                setCoords(null);
              }}
              style={{
                width: "100%",
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                gap: 8,
                padding: "4px 8px",
                border: "none",
                background: "transparent",
                color: item.danger ? "var(--forge-danger)" : "var(--forge-text-primary)",
                fontSize: "var(--forge-font-size-xs)",
                borderRadius: "calc(var(--forge-radius-sm) - 2px)",
                cursor: item.disabled ? "not-allowed" : "pointer",
                opacity: item.disabled ? 0.45 : 1,
                textAlign: "left",
              }}
            >
              <span style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
                {item.icon && <Icon name={item.icon} size={14} />}
                <span>{item.label}</span>
              </span>
              {item.shortcut && <kbd className="forge-kbd">{item.shortcut}</kbd>}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};
