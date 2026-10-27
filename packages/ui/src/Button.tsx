import React, { forwardRef } from "react";
import { Icon } from "./Icon.js";

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "default" | "primary" | "ai" | "ghost" | "danger";
  size?: "sm" | "md" | "lg";
  loading?: boolean;
  icon?: string;
  iconRight?: string;
  shortcut?: string;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = "default",
      size = "md",
      loading = false,
      disabled = false,
      icon,
      iconRight,
      shortcut,
      children,
      className = "",
      ...rest
    },
    ref
  ) => {
    return (
      <button
        ref={ref}
        disabled={disabled || loading}
        className={`forge-control forge-button forge-button-${variant} forge-size-${size} ${className}`}
        {...rest}
      >
        {loading ? (
          <span className="forge-spinner-sm" aria-hidden="true" />
        ) : icon ? (
          <Icon name={icon} size={size === "sm" ? 14 : 16} />
        ) : null}

        {children && <span>{children}</span>}

        {iconRight && !loading && (
          <Icon name={iconRight} size={size === "sm" ? 14 : 16} />
        )}

        {shortcut && (
          <kbd className="forge-kbd" style={{ marginLeft: 4 }}>
            {shortcut}
          </kbd>
        )}
      </button>
    );
  }
);
Button.displayName = "Button";

export interface IconButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  icon: string;
  variant?: "default" | "primary" | "ai" | "ghost" | "danger";
  size?: "sm" | "md" | "lg";
  loading?: boolean;
  label: string; // for accessibility
}

export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(
  (
    {
      icon,
      variant = "ghost",
      size = "md",
      loading = false,
      disabled = false,
      label,
      className = "",
      ...rest
    },
    ref
  ) => {
    return (
      <button
        ref={ref}
        aria-label={label}
        title={label}
        disabled={disabled || loading}
        className={`forge-control forge-button forge-button-${variant} forge-size-${size} ${className}`}
        style={{ padding: 0, width: "var(--forge-control-height-" + size + ")" }}
        {...rest}
      >
        {loading ? (
          <span className="forge-spinner-sm" aria-hidden="true" />
        ) : (
          <Icon name={icon} size={size === "sm" ? 14 : 16} />
        )}
      </button>
    );
  }
);
IconButton.displayName = "IconButton";

export interface ToggleButtonProps extends ButtonProps {
  pressed?: boolean;
  onPressedChange?: (pressed: boolean) => void;
}

export const ToggleButton = forwardRef<HTMLButtonElement, ToggleButtonProps>(
  ({ pressed = false, onPressedChange, onClick, className = "", ...rest }, ref) => {
    return (
      <Button
        ref={ref}
        aria-pressed={pressed}
        variant={pressed ? "primary" : "default"}
        onClick={(e) => {
          onPressedChange?.(!pressed);
          onClick?.(e);
        }}
        className={`${pressed ? "forge-button-active" : ""} ${className}`}
        {...rest}
      />
    );
  }
);
ToggleButton.displayName = "ToggleButton";

export interface SegmentedControlItem<T extends string = string> {
  value: T;
  label?: React.ReactNode;
  icon?: string;
}

export interface SegmentedControlProps<T extends string = string> {
  value: T;
  onChange: (value: T) => void;
  items: SegmentedControlItem<T>[];
  size?: "sm" | "md";
  className?: string;
}

export function SegmentedControl<T extends string = string>({
  value,
  onChange,
  items,
  size = "md",
  className = "",
}: SegmentedControlProps<T>) {
  return (
    <div
      role="radiogroup"
      className={`forge-segmented-control forge-size-${size} ${className}`}
      style={{
        display: "inline-flex",
        background: "var(--forge-surface-sunken)",
        border: "1px solid var(--forge-border)",
        borderRadius: "var(--forge-radius-sm)",
        padding: 2,
        gap: 2,
      }}
    >
      {items.map((item) => {
        const isSelected = item.value === value;
        return (
          <button
            key={item.value}
            role="radio"
            aria-checked={isSelected}
            onClick={() => onChange(item.value)}
            className={`forge-segmented-item ${isSelected ? "active" : ""}`}
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 4,
              padding: "0 8px",
              height: "calc(100% - 2px)",
              fontSize: "var(--forge-font-size-xs)",
              background: isSelected ? "var(--forge-surface-active)" : "transparent",
              color: isSelected ? "var(--forge-text-primary)" : "var(--forge-text-muted)",
              border: isSelected ? "1px solid var(--forge-border-strong)" : "1px solid transparent",
              borderRadius: "calc(var(--forge-radius-sm) - 2px)",
              cursor: "pointer",
            }}
          >
            {item.icon && <Icon name={item.icon} size={14} />}
            {item.label && <span>{item.label}</span>}
          </button>
        );
      })}
    </div>
  );
}
