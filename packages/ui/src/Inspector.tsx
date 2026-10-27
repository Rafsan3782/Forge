import React, { useState } from "react";
import { Icon } from "./Icon.js";

export interface PropertyRowProps {
  label: string;
  tooltip?: string;
  actions?: React.ReactNode;
  children: React.ReactNode;
}

export const PropertyRow: React.FC<PropertyRowProps> = ({
  label,
  tooltip,
  actions,
  children,
}) => {
  const [hovered, setHovered] = useState(false);

  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      className="forge-property-row"
    >
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <span className="forge-property-label" title={tooltip || label}>
          {label}
        </span>
        {hovered && actions && <div style={{ display: "flex", gap: 2 }}>{actions}</div>}
      </div>
      <div style={{ minWidth: 0 }}>{children}</div>
    </div>
  );
};

export interface PropertyGridProps {
  title?: string;
  defaultExpanded?: boolean;
  children: React.ReactNode;
}

export const PropertyGrid: React.FC<PropertyGridProps> = ({
  title,
  defaultExpanded = true,
  children,
}) => {
  const [expanded, setExpanded] = useState(defaultExpanded);

  return (
    <div
      style={{
        border: "1px solid var(--forge-border)",
        borderRadius: "var(--forge-radius-sm)",
        marginBottom: 8,
        overflow: "hidden",
        background: "var(--forge-surface-raised)",
      }}
    >
      {title && (
        <button
          type="button"
          onClick={() => setExpanded(!expanded)}
          style={{
            width: "100%",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "4px 8px",
            background: "var(--forge-surface-overlay)",
            border: "none",
            borderBottom: expanded ? "1px solid var(--forge-border)" : "none",
            color: "var(--forge-text-primary)",
            fontSize: "var(--forge-font-size-xs)",
            fontWeight: 600,
            cursor: "pointer",
            textAlign: "left",
          }}
        >
          <span>{title}</span>
          <Icon
            name="chevron-right"
            size={12}
            style={{
              transform: expanded ? "rotate(90deg)" : "rotate(0deg)",
              transition: "transform 100ms",
            }}
          />
        </button>
      )}
      {expanded && <div>{children}</div>}
    </div>
  );
};

export interface BreadcrumbsProps {
  items: { label: string; onClick?: () => void }[];
}

export const Breadcrumbs: React.FC<BreadcrumbsProps> = ({ items }) => {
  return (
    <nav
      aria-label="Breadcrumb"
      style={{
        display: "flex",
        alignItems: "center",
        gap: 4,
        fontSize: "var(--forge-font-size-xs)",
        color: "var(--forge-text-muted)",
      }}
    >
      {items.map((item, idx) => {
        const isLast = idx === items.length - 1;
        return (
          <React.Fragment key={idx}>
            <span
              onClick={item.onClick}
              style={{
                color: isLast ? "var(--forge-text-primary)" : "var(--forge-text-muted)",
                cursor: item.onClick ? "pointer" : "default",
                fontWeight: isLast ? 600 : 400,
              }}
            >
              {item.label}
            </span>
            {!isLast && <Icon name="chevron-right" size={10} color="var(--forge-text-disabled)" />}
          </React.Fragment>
        );
      })}
    </nav>
  );
};

export interface EmptyStateProps {
  icon?: string;
  title: string;
  description?: string;
  action?: React.ReactNode;
}

export const EmptyState: React.FC<EmptyStateProps> = ({
  icon = "box",
  title,
  description,
  action,
}) => {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        padding: 32,
        textAlign: "center",
        color: "var(--forge-text-muted)",
      }}
    >
      <div
        style={{
          width: 36,
          height: 36,
          borderRadius: "var(--forge-radius-md)",
          background: "var(--forge-surface-active)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          marginBottom: 12,
          color: "var(--forge-text-muted)",
        }}
      >
        <Icon name={icon} size={20} />
      </div>
      <div style={{ fontSize: "var(--forge-font-size-sm)", fontWeight: 600, color: "var(--forge-text-primary)", marginBottom: 4 }}>
        {title}
      </div>
      {description && (
        <div style={{ fontSize: "var(--forge-font-size-xs)", color: "var(--forge-text-muted)", maxWidth: 280, marginBottom: 12 }}>
          {description}
        </div>
      )}
      {action && <div>{action}</div>}
    </div>
  );
};

export interface SearchFieldProps {
  value: string;
  onChange: (val: string) => void;
  placeholder?: string;
  onClear?: () => void;
}

export const SearchField: React.FC<SearchFieldProps> = ({
  value,
  onChange,
  placeholder = "Search...",
  onClear,
}) => {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 6,
        padding: "0 8px",
        height: "var(--forge-control-height-sm)",
        background: "var(--forge-surface-sunken)",
        border: "1px solid var(--forge-border)",
        borderRadius: "var(--forge-radius-sm)",
        width: "100%",
        boxSizing: "border-box",
      }}
    >
      <Icon name="search" size={12} color="var(--forge-text-muted)" />
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        style={{
          flex: 1,
          background: "transparent",
          border: "none",
          outline: "none",
          color: "var(--forge-text-primary)",
          fontSize: "var(--forge-font-size-xs)",
        }}
      />
      {value && (
        <button
          onClick={() => {
            onChange("");
            onClear?.();
          }}
          style={{ background: "transparent", border: "none", color: "var(--forge-text-muted)", cursor: "pointer", padding: 0 }}
        >
          <Icon name="x" size={10} />
        </button>
      )}
    </div>
  );
};

export interface ScrollAreaProps {
  children: React.ReactNode;
  maxHeight?: string | number;
  className?: string;
}

export const ScrollArea: React.FC<ScrollAreaProps> = ({
  children,
  maxHeight = "100%",
  className = "",
}) => {
  return (
    <div className={`forge-scrollarea ${className}`} style={{ maxHeight, overflow: "auto" }}>
      {children}
    </div>
  );
};

export interface ResizablePanelsProps {
  left: React.ReactNode;
  right: React.ReactNode;
  initialSplit?: number; // percentage
}

export const ResizablePanels: React.FC<ResizablePanelsProps> = ({
  left,
  right,
  initialSplit = 50,
}) => {
  const [split, setSplit] = useState(initialSplit);
  const isDragging = React.useRef(false);

  const handleMouseDown = () => {
    isDragging.current = true;
    document.body.style.cursor = "col-resize";
  };

  React.useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current) return;
      const pct = (e.clientX / window.innerWidth) * 100;
      setSplit(Math.max(15, Math.min(85, pct)));
    };
    const handleMouseUp = () => {
      isDragging.current = false;
      document.body.style.cursor = "";
    };
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, []);

  return (
    <div style={{ display: "flex", width: "100%", height: "100%", overflow: "hidden" }}>
      <div style={{ width: `${split}%`, height: "100%", overflow: "auto" }}>{left}</div>
      <div
        onMouseDown={handleMouseDown}
        style={{
          width: 4,
          background: "var(--forge-border)",
          cursor: "col-resize",
          transition: "background-color 100ms",
        }}
        onMouseEnter={(e) => (e.currentTarget.style.background = "var(--forge-accent)")}
        onMouseLeave={(e) => (e.currentTarget.style.background = "var(--forge-border)")}
      />
      <div style={{ width: `${100 - split}%`, height: "100%", overflow: "auto" }}>{right}</div>
    </div>
  );
};

export interface DiffViewProps {
  original: string;
  modified: string;
}

export const DiffView: React.FC<DiffViewProps> = ({ original, modified }) => {
  const origLines = original.split("\n");
  const modLines = modified.split("\n");

  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "1fr 1fr",
        gap: 8,
        fontFamily: "var(--forge-font-mono)",
        fontSize: "var(--forge-font-size-xs)",
        background: "var(--forge-surface-sunken)",
        border: "1px solid var(--forge-border)",
        borderRadius: "var(--forge-radius-sm)",
        padding: 8,
      }}
    >
      <div>
        <div style={{ color: "var(--forge-danger)", fontWeight: 600, marginBottom: 4 }}>- Original</div>
        {origLines.map((line, i) => (
          <div key={i} style={{ color: "var(--forge-text-muted)" }}>
            {line}
          </div>
        ))}
      </div>
      <div>
        <div style={{ color: "var(--forge-success)", fontWeight: 600, marginBottom: 4 }}>+ Modified</div>
        {modLines.map((line, i) => (
          <div key={i} style={{ color: "var(--forge-text-primary)" }}>
            {line}
          </div>
        ))}
      </div>
    </div>
  );
};
