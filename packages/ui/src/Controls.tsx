import React from "react";
import { Icon } from "./Icon.js";

export interface CheckboxProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: React.ReactNode;
  disabled?: boolean;
  className?: string;
}

export const Checkbox: React.FC<CheckboxProps> = ({
  checked,
  onChange,
  label,
  disabled = false,
  className = "",
}) => {
  return (
    <label
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 6,
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.5 : 1,
        userSelect: "none",
        fontSize: "var(--forge-font-size-sm)",
        color: "var(--forge-text-primary)",
      }}
      className={className}
    >
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.target.checked)}
        style={{ position: "absolute", opacity: 0, width: 0, height: 0 }}
      />
      <span
        style={{
          width: 14,
          height: 14,
          borderRadius: "var(--forge-radius-xs)",
          border: checked ? "1px solid var(--forge-accent)" : "1px solid var(--forge-border)",
          background: checked ? "var(--forge-accent)" : "var(--forge-surface-sunken)",
          display: "inline-flex",
          alignItems: "center",
          justifyContent: "center",
          color: "#ffffff",
          transition: "all var(--forge-motion-fast)",
        }}
      >
        {checked && <Icon name="check" size={10} strokeWidth={2.5} />}
      </span>
      {label && <span>{label}</span>}
    </label>
  );
};

export interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: React.ReactNode;
  disabled?: boolean;
}

export const Switch: React.FC<SwitchProps> = ({
  checked,
  onChange,
  label,
  disabled = false,
}) => {
  return (
    <label
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 8,
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.5 : 1,
        userSelect: "none",
        fontSize: "var(--forge-font-size-sm)",
        color: "var(--forge-text-primary)",
      }}
    >
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        style={{
          width: 28,
          height: 16,
          borderRadius: "var(--forge-radius-full)",
          background: checked ? "var(--forge-accent)" : "var(--forge-surface-active)",
          border: "1px solid var(--forge-border)",
          position: "relative",
          cursor: "pointer",
          padding: 1,
          transition: "background-color var(--forge-motion-fast)",
        }}
      >
        <span
          style={{
            display: "block",
            width: 12,
            height: 12,
            borderRadius: "50%",
            background: "#ffffff",
            transform: checked ? "translateX(12px)" : "translateX(0px)",
            transition: "transform var(--forge-motion-fast)",
          }}
        />
      </button>
      {label && <span>{label}</span>}
    </label>
  );
};

export interface SliderProps {
  value: number;
  onChange: (val: number) => void;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  className?: string;
}

export const Slider: React.FC<SliderProps> = ({
  value,
  onChange,
  min = 0,
  max = 100,
  step = 1,
  disabled = false,
  className = "",
}) => {
  const percentage = Math.max(0, Math.min(100, ((value - min) / (max - min)) * 100));

  return (
    <div
      className={className}
      style={{
        display: "inline-flex",
        alignItems: "center",
        width: "100%",
        position: "relative",
        opacity: disabled ? 0.5 : 1,
      }}
    >
      <input
        type="range"
        value={value}
        min={min}
        max={max}
        step={step}
        disabled={disabled}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        style={{
          width: "100%",
          accentColor: "var(--forge-accent)",
          cursor: disabled ? "not-allowed" : "pointer",
        }}
      />
    </div>
  );
};
