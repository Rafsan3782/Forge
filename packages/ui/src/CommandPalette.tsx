import React, { useState } from "react";
import { Icon } from "./Icon.js";

export interface CommandItem {
  id: string;
  title: string;
  category?: string;
  shortcut?: string;
  icon?: string;
  action: () => void;
}

export interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  commands: CommandItem[];
  placeholder?: string;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  open,
  onOpenChange,
  commands,
  placeholder = "Type a command or search...",
}) => {
  const [search, setSearch] = useState("");

  if (!open) return null;

  const filtered = commands.filter((cmd) =>
    cmd.title.toLowerCase().includes(search.toLowerCase()) ||
    (cmd.category && cmd.category.toLowerCase().includes(search.toLowerCase()))
  );

  return (
    <div
      onClick={() => onOpenChange(false)}
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0, 0, 0, 0.75)",
        zIndex: "var(--forge-z-modal)",
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "center",
        paddingTop: "15vh",
      }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          width: 540,
          background: "var(--forge-surface-raised)",
          border: "1px solid var(--forge-border-strong)",
          borderRadius: "var(--forge-radius-md)",
          boxShadow: "var(--forge-elevation-floating)",
          overflow: "hidden",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            padding: "10px 14px",
            borderBottom: "1px solid var(--forge-border)",
            gap: 8,
          }}
        >
          <Icon name="search" size={16} color="var(--forge-text-muted)" />
          <input
            autoFocus
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={placeholder}
            style={{
              flex: 1,
              background: "transparent",
              border: "none",
              outline: "none",
              color: "var(--forge-text-primary)",
              fontSize: "var(--forge-font-size-sm)",
            }}
          />
          <kbd className="forge-kbd">ESC</kbd>
        </div>

        <div style={{ maxHeight: 280, overflowY: "auto", padding: 4 }}>
          {filtered.length === 0 ? (
            <div style={{ padding: "12px", textAlign: "center", fontSize: "12px", color: "var(--forge-text-muted)" }}>
              No commands found
            </div>
          ) : (
            filtered.map((cmd) => (
              <button
                key={cmd.id}
                onClick={() => {
                  cmd.action();
                  onOpenChange(false);
                }}
                style={{
                  width: "100%",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  padding: "6px 10px",
                  border: "none",
                  background: "transparent",
                  color: "var(--forge-text-primary)",
                  borderRadius: "var(--forge-radius-xs)",
                  fontSize: "var(--forge-font-size-xs)",
                  cursor: "pointer",
                  textAlign: "left",
                }}
              >
                <span style={{ display: "inline-flex", alignItems: "center", gap: 8 }}>
                  {cmd.icon && <Icon name={cmd.icon} size={14} color="var(--forge-text-muted)" />}
                  <span>{cmd.title}</span>
                  {cmd.category && (
                    <span style={{ fontSize: "10px", color: "var(--forge-text-disabled)" }}>
                      {cmd.category}
                    </span>
                  )}
                </span>
                {cmd.shortcut && <kbd className="forge-kbd">{cmd.shortcut}</kbd>}
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
