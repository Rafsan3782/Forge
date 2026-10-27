import React from "react";
import { Icon } from "./Icon.js";

export interface BadgeProps {
  variant?: "default" | "accent" | "ai" | "success" | "warn" | "danger";
  children: React.ReactNode;
  icon?: string;
}

export const Badge: React.FC<BadgeProps> = ({ variant = "default", children, icon }) => {
  return (
    <span className={`forge-badge forge-badge-${variant}`}>
      {icon && <Icon name={icon} size={12} />}
      <span>{children}</span>
    </span>
  );
};

export interface ChipProps {
  label: string;
  onRemove?: () => void;
  icon?: string;
}

export const Chip: React.FC<ChipProps> = ({ label, onRemove, icon }) => {
  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 4,
        padding: "2px 6px",
        background: "var(--forge-surface-active)",
        border: "1px solid var(--forge-border)",
        borderRadius: "var(--forge-radius-full)",
        fontSize: "var(--forge-font-size-xs)",
        color: "var(--forge-text-primary)",
      }}
    >
      {icon && <Icon name={icon} size={12} />}
      <span>{label}</span>
      {onRemove && (
        <button
          onClick={onRemove}
          style={{
            background: "transparent",
            border: "none",
            color: "var(--forge-text-muted)",
            cursor: "pointer",
            padding: 0,
            display: "inline-flex",
          }}
        >
          <Icon name="x" size={10} />
        </button>
      )}
    </span>
  );
};

export interface KbdProps {
  children: React.ReactNode;
}

export const Kbd: React.FC<KbdProps> = ({ children }) => {
  return <kbd className="forge-kbd">{children}</kbd>;
};

export interface StatusDotProps {
  status?: "online" | "busy" | "offline" | "ai";
}

export const StatusDot: React.FC<StatusDotProps> = ({ status = "online" }) => {
  const colorMap = {
    online: "var(--forge-success)",
    busy: "var(--forge-warn)",
    offline: "var(--forge-text-disabled)",
    ai: "var(--forge-ai)",
  };

  return (
    <span
      className="forge-status-dot"
      style={{ backgroundColor: colorMap[status] }}
    />
  );
};

export interface SpinnerProps {
  size?: number;
  color?: string;
}

export const Spinner: React.FC<SpinnerProps> = ({ size = 16, color = "var(--forge-accent)" }) => {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 16 16"
      style={{ animation: "forge-spin 0.8s linear infinite" }}
    >
      <circle
        cx="8"
        cy="8"
        r="6"
        fill="none"
        stroke="var(--forge-border)"
        strokeWidth="2"
      />
      <circle
        cx="8"
        cy="8"
        r="6"
        fill="none"
        stroke={color}
        strokeWidth="2"
        strokeDasharray="28"
        strokeDashoffset="18"
      />
      <style>{`
        @keyframes forge-spin {
          100% { transform: rotate(360deg); }
        }
      `}</style>
    </svg>
  );
};

export interface SkeletonProps {
  width?: string | number;
  height?: string | number;
  borderRadius?: string;
  className?: string;
}

export const Skeleton: React.FC<SkeletonProps> = ({
  width = "100%",
  height = 16,
  borderRadius = "var(--forge-radius-sm)",
  className = "",
}) => {
  return (
    <div
      className={className}
      style={{
        width,
        height,
        borderRadius,
        background: "var(--forge-surface-active)",
        opacity: 0.6,
        animation: "forge-pulse 1.5s ease-in-out infinite",
      }}
    >
      <style>{`
        @keyframes forge-pulse {
          0%, 100% { opacity: 0.4; }
          50% { opacity: 0.8; }
        }
      `}</style>
    </div>
  );
};

export interface ProgressBarProps {
  value: number; // 0 to 100
  max?: number;
  color?: string;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  value,
  max = 100,
  color = "var(--forge-accent)",
}) => {
  const percentage = Math.max(0, Math.min(100, (value / max) * 100));
  return (
    <div
      style={{
        width: "100%",
        height: 4,
        background: "var(--forge-surface-active)",
        borderRadius: "var(--forge-radius-full)",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          width: `${percentage}%`,
          height: "100%",
          background: color,
          transition: "width var(--forge-motion-fast)",
        }}
      />
    </div>
  );
};
