import React, { useState, useRef, useEffect } from "react";
import { Icon } from "./Icon.js";

export interface SelectOption {
  value: string;
  label: string;
  icon?: string;
}

export interface SelectProps {
  value: string;
  onChange: (value: string) => void;
  options: SelectOption[];
  sizeVariant?: "sm" | "md";
  disabled?: boolean;
  className?: string;
}

export const Select: React.FC<SelectProps> = ({
  value,
  onChange,
  options,
  sizeVariant = "md",
  disabled = false,
  className = "",
}) => {
  const [open, setOpen] = useState(false);
  const selected = options.find((o) => o.value === value) || options[0];
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
    <div
      ref={ref}
      className={`forge-select-wrapper forge-size-${sizeVariant} ${className}`}
      style={{ position: "relative", width: "100%" }}
    >
      <button
        type="button"
        disabled={disabled}
        onClick={() => setOpen(!open)}
        className="forge-control"
        style={{
          width: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "0 8px",
          height: "100%",
          background: "var(--forge-surface-sunken)",
          borderRadius: "var(--forge-radius-sm)",
          color: "var(--forge-text-primary)",
          fontSize: "var(--forge-font-size-xs)",
          cursor: disabled ? "not-allowed" : "pointer",
        }}
      >
        <span style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
          {selected?.icon && <Icon name={selected.icon} size={14} />}
          <span>{selected?.label ?? value}</span>
        </span>
        <Icon name="chevron-down" size={12} color="var(--forge-text-muted)" />
      </button>

      {open && (
        <div
          style={{
            position: "absolute",
            top: "100%",
            left: 0,
            right: 0,
            marginTop: 4,
            background: "var(--forge-surface-overlay)",
            border: "1px solid var(--forge-border-strong)",
            borderRadius: "var(--forge-radius-sm)",
            boxShadow: "var(--forge-elevation-medium)",
            zIndex: "var(--forge-z-dropdown)",
            maxHeight: 180,
            overflowY: "auto",
            padding: 2,
          }}
        >
          {options.map((opt) => (
            <button
              key={opt.value}
              type="button"
              onClick={() => {
                onChange(opt.value);
                setOpen(false);
              }}
              style={{
                width: "100%",
                display: "flex",
                alignItems: "center",
                gap: 6,
                padding: "4px 8px",
                border: "none",
                background: opt.value === value ? "var(--forge-surface-active)" : "transparent",
                color: "var(--forge-text-primary)",
                fontSize: "var(--forge-font-size-xs)",
                borderRadius: "calc(var(--forge-radius-sm) - 2px)",
                cursor: "pointer",
                textAlign: "left",
              }}
            >
              {opt.icon && <Icon name={opt.icon} size={14} />}
              <span style={{ flex: 1 }}>{opt.label}</span>
              {opt.value === value && <Icon name="check" size={12} color="var(--forge-accent)" />}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};

export interface ComboboxProps extends SelectProps {
  placeholder?: string;
}

export const Combobox: React.FC<ComboboxProps> = ({
  value,
  onChange,
  options,
  placeholder = "Search...",
  sizeVariant = "md",
  disabled = false,
}) => {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const ref = useRef<HTMLDivElement>(null);

  const filtered = options.filter((o) =>
    o.label.toLowerCase().includes(query.toLowerCase())
  );

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
    <div ref={ref} style={{ position: "relative", width: "100%" }}>
      <div
        className="forge-control"
        style={{
          display: "flex",
          alignItems: "center",
          gap: 6,
          padding: "0 8px",
          height: `var(--forge-control-height-${sizeVariant})`,
          background: "var(--forge-surface-sunken)",
          borderRadius: "var(--forge-radius-sm)",
        }}
      >
        <Icon name="search" size={14} color="var(--forge-text-muted)" />
        <input
          value={open ? query : options.find((o) => o.value === value)?.label || value}
          onFocus={() => {
            setOpen(true);
            setQuery("");
          }}
          onChange={(e) => {
            setQuery(e.target.value);
            if (!open) setOpen(true);
          }}
          placeholder={placeholder}
          disabled={disabled}
          style={{
            flex: 1,
            background: "transparent",
            border: "none",
            outline: "none",
            color: "var(--forge-text-primary)",
            fontSize: "var(--forge-font-size-xs)",
          }}
        />
        <Icon name="chevrons-up-down" size={12} color="var(--forge-text-muted)" />
      </div>

      {open && (
        <div
          style={{
            position: "absolute",
            top: "100%",
            left: 0,
            right: 0,
            marginTop: 4,
            background: "var(--forge-surface-overlay)",
            border: "1px solid var(--forge-border-strong)",
            borderRadius: "var(--forge-radius-sm)",
            boxShadow: "var(--forge-elevation-medium)",
            zIndex: "var(--forge-z-dropdown)",
            maxHeight: 180,
            overflowY: "auto",
            padding: 2,
          }}
        >
          {filtered.length === 0 ? (
            <div style={{ padding: "8px", fontSize: "11px", color: "var(--forge-text-muted)" }}>
              No results found
            </div>
          ) : (
            filtered.map((opt) => (
              <button
                key={opt.value}
                type="button"
                onClick={() => {
                  onChange(opt.value);
                  setOpen(false);
                }}
                style={{
                  width: "100%",
                  display: "flex",
                  alignItems: "center",
                  gap: 6,
                  padding: "4px 8px",
                  border: "none",
                  background: opt.value === value ? "var(--forge-surface-active)" : "transparent",
                  color: "var(--forge-text-primary)",
                  fontSize: "var(--forge-font-size-xs)",
                  borderRadius: "calc(var(--forge-radius-sm) - 2px)",
                  cursor: "pointer",
                  textAlign: "left",
                }}
              >
                {opt.icon && <Icon name={opt.icon} size={14} />}
                <span style={{ flex: 1 }}>{opt.label}</span>
                {opt.value === value && <Icon name="check" size={12} color="var(--forge-accent)" />}
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
};
