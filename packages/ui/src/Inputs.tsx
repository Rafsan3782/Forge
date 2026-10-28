import React, { forwardRef, useState, useRef, useEffect } from "react";
import { Icon } from "./Icon.js";

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  sizeVariant?: "sm" | "md" | "lg";
  icon?: string;
  error?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ sizeVariant = "md", icon, error, className = "", ...rest }, ref) => {
    return (
      <div
        className={`forge-input-wrapper forge-size-${sizeVariant}`}
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: 6,
          background: "var(--forge-surface-sunken)",
          border: error ? "1px solid var(--forge-danger)" : "1px solid var(--forge-border)",
          borderRadius: "var(--forge-radius-sm)",
          padding: "0 8px",
          width: "100%",
          boxSizing: "border-box",
        }}
      >
        {icon && <Icon name={icon} size={14} color="var(--forge-text-muted)" />}
        <input
          ref={ref}
          className={`forge-input ${className}`}
          style={{
            flex: 1,
            background: "transparent",
            border: "none",
            outline: "none",
            color: "var(--forge-text-primary)",
            fontSize: "var(--forge-font-size-sm)",
            width: "100%",
            padding: 0,
          }}
          {...rest}
        />
      </div>
    );
  }
);
Input.displayName = "Input";

export interface NumberInputProps {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
  label?: string;
  sizeVariant?: "sm" | "md";
  className?: string;
  disabled?: boolean;
}

export const NumberInput: React.FC<NumberInputProps> = ({
  value,
  onChange,
  min = -Infinity,
  max = Infinity,
  step = 1,
  unit,
  label,
  sizeVariant = "md",
  className = "",
  disabled = false,
}) => {
  const [isDragging, setIsDragging] = useState(false);
  const startX = useRef(0);
  const startVal = useRef(value);

  const handleMouseDown = (e: React.MouseEvent) => {
    if (disabled) return;
    setIsDragging(true);
    startX.current = e.clientX;
    startVal.current = value;
    document.body.style.cursor = "ew-resize";
  };

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      const deltaX = e.clientX - startX.current;
      const speed = e.shiftKey ? step * 10 : e.altKey ? step * 0.1 : step;
      let next = startVal.current + Math.round(deltaX) * speed;
      if (next < min) next = min;
      if (next > max) next = max;
      // Round to 2 decimals to avoid JS floating point quirks
      next = Math.round(next * 100) / 100;
      onChange(next);
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      document.body.style.cursor = "";
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isDragging, min, max, step, onChange]);

  return (
    <div
      className={`forge-number-input forge-size-${sizeVariant} ${className}`}
      style={{
        display: "inline-flex",
        alignItems: "center",
        background: "var(--forge-surface-sunken)",
        border: "1px solid var(--forge-border)",
        borderRadius: "var(--forge-radius-sm)",
        overflow: "hidden",
        width: "100%",
        opacity: disabled ? 0.5 : 1,
      }}
    >
      {label && (
        <span
          onMouseDown={handleMouseDown}
          className="forge-number-label"
          title="Drag to scrub value (Shift for 10x, Alt for 0.1x)"
        >
          {label}
        </span>
      )}
      <input
        type="number"
        value={Number.isNaN(value) ? "" : value}
        disabled={disabled}
        min={min}
        max={max}
        step={step}
        onChange={(e) => {
          const val = parseFloat(e.target.value);
          if (!Number.isNaN(val)) {
            let clamped = val;
            if (clamped < min) clamped = min;
            if (clamped > max) clamped = max;
            onChange(clamped);
          }
        }}
        className="forge-number-field"
        style={{
          flex: 1,
          background: "transparent",
          border: "none",
          outline: "none",
          padding: "0 6px",
          color: "var(--forge-text-primary)",
        }}
      />
      {unit && (
        <span
          style={{
            paddingRight: 6,
            fontSize: "var(--forge-font-size-xs)",
            color: "var(--forge-text-muted)",
            userSelect: "none",
          }}
        >
          {unit}
        </span>
      )}
    </div>
  );
};

export interface Vector2InputProps {
  x: number;
  y: number;
  onChange: (val: { x: number; y: number }) => void;
  step?: number;
  sizeVariant?: "sm" | "md";
}

export const Vector2Input: React.FC<Vector2InputProps> = ({
  x,
  y,
  onChange,
  step = 1,
  sizeVariant = "sm",
}) => {
  return (
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 4, width: "100%" }}>
      <NumberInput
        label="X"
        value={x}
        onChange={(val) => onChange({ x: val, y })}
        step={step}
        sizeVariant={sizeVariant}
      />
      <NumberInput
        label="Y"
        value={y}
        onChange={(val) => onChange({ x, y: val })}
        step={step}
        sizeVariant={sizeVariant}
      />
    </div>
  );
};

export interface ColorInputProps {
  value: string;
  onChange: (color: string) => void;
  label?: string;
  sizeVariant?: "sm" | "md";
}

export const ColorInput: React.FC<ColorInputProps> = ({
  value,
  onChange,
  label,
  sizeVariant: _sizeVariant = "sm",
}) => {
  return (
    <div
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 6,
        background: "var(--forge-surface-sunken)",
        border: "1px solid var(--forge-border)",
        borderRadius: "var(--forge-radius-sm)",
        padding: "2px 6px",
        width: "100%",
        boxSizing: "border-box",
      }}
    >
      <input
        type="color"
        value={value.startsWith("#") ? value : "#ffffff"}
        onChange={(e) => onChange(e.target.value)}
        style={{
          width: 18,
          height: 18,
          padding: 0,
          border: "none",
          background: "transparent",
          cursor: "pointer",
          borderRadius: 2,
        }}
      />
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="forge-tabular-nums"
        style={{
          flex: 1,
          background: "transparent",
          border: "none",
          outline: "none",
          color: "var(--forge-text-primary)",
          fontSize: "var(--forge-font-size-xs)",
          fontFamily: "var(--forge-font-mono)",
        }}
      />
      {label && <span style={{ fontSize: "11px", color: "var(--forge-text-muted)" }}>{label}</span>}
    </div>
  );
};
