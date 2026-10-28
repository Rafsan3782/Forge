import React from "react";
import * as LucideIcons from "lucide-react";

export type CustomIconName =
  | "entity"
  | "component"
  | "scene"
  | "tilemap"
  | "sprite"
  | "collider"
  | "script"
  | "play"
  | "stop"
  | "step"
  | "gizmo-move"
  | "gizmo-rotate"
  | "gizmo-scale"
  | "checkpoint"
  | "agent";

export type IconName = keyof typeof LucideIcons | CustomIconName;

export interface IconProps extends React.SVGProps<SVGSVGElement> {
  name: string;
  size?: number | string;
  color?: string;
  strokeWidth?: number;
  className?: string;
}

export const CustomIcons: Record<
  CustomIconName,
  React.FC<React.SVGProps<SVGSVGElement>>
> = {
  entity: (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <polygon points="8 1.5 14 5 14 11 8 14.5 2 11 2 5 8 1.5" />
      <line x1="8" y1="1.5" x2="8" y2="14.5" />
      <line x1="14" y1="5" x2="2" y2="11" />
      <line x1="14" y1="11" x2="2" y2="5" />
    </svg>
  ),
  component: (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <rect x="2.5" y="2.5" width="5" height="5" rx="1" />
      <rect x="8.5" y="2.5" width="5" height="5" rx="1" />
      <rect x="2.5" y="8.5" width="5" height="5" rx="1" />
      <rect x="8.5" y="8.5" width="5" height="5" rx="1" />
    </svg>
  ),
  scene: (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <rect x="2" y="2.5" width="12" height="11" rx="1.5" />
      <circle cx="5.5" cy="6" r="1.2" />
      <path d="M14 10.5l-3.5-3.5-4.5 4.5" />
    </svg>
  ),
  tilemap: (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <rect x="2" y="2" width="12" height="12" rx="1" />
      <line x1="6" y1="2" x2="6" y2="14" />
      <line x1="10" y1="2" x2="10" y2="14" />
      <line x1="2" y1="6" x2="14" y2="6" />
      <line x1="2" y1="10" x2="14" y2="10" />
    </svg>
  ),
  sprite: (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <rect x="2" y="2" width="12" height="12" rx="2" />
      <path d="M5 8h6M8 5v6" />
    </svg>
  ),
  collider: (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeDasharray="2 2" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <rect x="2.5" y="2.5" width="11" height="11" rx="2" />
      <circle cx="8" cy="8" r="2.5" />
    </svg>
  ),
  script: (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <path d="M5 4.5l-2.5 2.5 2.5 2.5" />
      <path d="M11 4.5l2.5 2.5-2.5 2.5" />
      <line x1="9.5" y1="3" x2="6.5" y2="13" />
    </svg>
  ),
  play: (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <polygon points="4 2.5 13 8 4 13.5 4 2.5" fill="currentColor" fillOpacity="0.2" />
    </svg>
  ),
  stop: (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <rect x="3.5" y="3.5" width="9" height="9" rx="1.5" fill="currentColor" fillOpacity="0.2" />
    </svg>
  ),
  step: (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <polygon points="3.5 3 9.5 8 3.5 13 3.5 3" />
      <line x1="12.5" y1="3" x2="12.5" y2="13" />
    </svg>
  ),
  "gizmo-move": (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <path d="M8 2v12M2 8h12M8 2l-2 2m2-2l2 2M14 8l-2-2m2 2l-2 2M8 14l-2-2m2 2l2-2M2 8l2-2m-2 2l2 2" />
    </svg>
  ),
  "gizmo-rotate": (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <path d="M13.5 8a5.5 5.5 0 1 1-1.6-3.9L14 6" />
      <polyline points="14 2 14 6 10 6" />
    </svg>
  ),
  "gizmo-scale": (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <rect x="2.5" y="2.5" width="4" height="4" />
      <rect x="9.5" y="9.5" width="4" height="4" />
      <line x1="6.5" y1="6.5" x2="9.5" y2="9.5" />
    </svg>
  ),
  checkpoint: (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <circle cx="8" cy="8" r="6" />
      <circle cx="8" cy="8" r="2.5" fill="currentColor" />
    </svg>
  ),
  agent: (props) => (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <path d="M8 1.5l1.6 3.8 4.1.4-3.1 2.8.9 4-3.5-2.1-3.5 2.1.9-4-3.1-2.8 4.1-.4L8 1.5z" />
    </svg>
  ),
};

export const Icon: React.FC<IconProps> = ({
  name,
  size = 16,
  color = "currentColor",
  strokeWidth = 1.5,
  className = "",
  ...rest
}) => {
  if (name in CustomIcons) {
    const CustomComp = CustomIcons[name as CustomIconName];
    return (
      <CustomComp
        width={size}
        height={size}
        stroke={color}
        className={`forge-icon ${className}`}
        {...rest}
      />
    );
  }

  // Check Lucide Icons
  const lucideKey = (name.charAt(0).toUpperCase() + name.slice(1)) as keyof typeof LucideIcons;
  const LucideComp = (LucideIcons[lucideKey] || LucideIcons[name as keyof typeof LucideIcons]) as
    | React.ComponentType<{ size?: number | string; color?: string; strokeWidth?: number; className?: string }>
    | undefined;

  if (LucideComp) {
    return (
      <LucideComp
        size={size}
        color={color}
        strokeWidth={strokeWidth}
        className={`forge-icon ${className}`}
        {...rest}
      />
    );
  }

  // Fallback box icon
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      stroke={color}
      strokeWidth={strokeWidth}
      className={`forge-icon ${className}`}
      {...rest}
    >
      <rect x="2" y="2" width="12" height="12" rx="2" />
    </svg>
  );
};
