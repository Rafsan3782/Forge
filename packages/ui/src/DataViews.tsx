import React, { useState } from "react";
import { Icon } from "./Icon.js";

export interface TreeNode {
  id: string;
  label: string;
  icon?: string;
  children?: TreeNode[];
}

export interface TreeViewProps {
  nodes: TreeNode[];
  selectedId?: string;
  onSelect?: (node: TreeNode) => void;
  className?: string;
}

export const TreeView: React.FC<TreeViewProps> = ({
  nodes,
  selectedId,
  onSelect,
  className = "",
}) => {
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});

  const toggle = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setExpanded((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  const renderNode = (node: TreeNode, depth = 0) => {
    const isExpanded = !!expanded[node.id];
    const isSelected = selectedId === node.id;
    const hasChildren = node.children && node.children.length > 0;

    return (
      <div key={node.id} style={{ userSelect: "none" }}>
        <div
          onClick={() => onSelect?.(node)}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 4,
            paddingLeft: depth * 14 + 6,
            paddingRight: 6,
            height: 24,
            background: isSelected ? "var(--forge-surface-active)" : "transparent",
            color: isSelected ? "var(--forge-text-primary)" : "var(--forge-text-muted)",
            borderRadius: "var(--forge-radius-xs)",
            cursor: "pointer",
            fontSize: "var(--forge-font-size-xs)",
          }}
        >
          {hasChildren ? (
            <span
              onClick={(e) => toggle(node.id, e)}
              style={{
                display: "inline-flex",
                transform: isExpanded ? "rotate(90deg)" : "rotate(0deg)",
                transition: "transform 100ms",
              }}
            >
              <Icon name="chevron-right" size={12} />
            </span>
          ) : (
            <span style={{ width: 12 }} />
          )}

          {node.icon && <Icon name={node.icon} size={14} color={isSelected ? "var(--forge-accent)" : "currentColor"} />}
          <span style={{ flex: 1, textOverflow: "ellipsis", overflow: "hidden", whiteSpace: "nowrap" }}>
            {node.label}
          </span>
        </div>
        {hasChildren && isExpanded && (
          <div>{node.children!.map((child) => renderNode(child, depth + 1))}</div>
        )}
      </div>
    );
  };

  return (
    <div
      className={`forge-scrollarea ${className}`}
      style={{
        maxHeight: 400,
        overflowY: "auto",
        border: "1px solid var(--forge-border)",
        borderRadius: "var(--forge-radius-sm)",
        padding: 4,
        background: "var(--forge-surface-sunken)",
      }}
    >
      {nodes.map((node) => renderNode(node, 0))}
    </div>
  );
};

export interface ListProps<T> {
  items: T[];
  renderItem: (item: T, index: number) => React.ReactNode;
  selectedItem?: T;
  onSelect?: (item: T) => void;
  className?: string;
}

export function List<T>({ items, renderItem, onSelect, className = "" }: ListProps<T>) {
  return (
    <div
      className={`forge-scrollarea ${className}`}
      style={{
        border: "1px solid var(--forge-border)",
        borderRadius: "var(--forge-radius-sm)",
        background: "var(--forge-surface-sunken)",
        maxHeight: 300,
        overflowY: "auto",
        padding: 2,
      }}
    >
      {items.map((item, idx) => (
        <div
          key={idx}
          onClick={() => onSelect?.(item)}
          style={{
            padding: "4px 8px",
            fontSize: "var(--forge-font-size-xs)",
            color: "var(--forge-text-primary)",
            cursor: "pointer",
            borderRadius: "var(--forge-radius-xs)",
          }}
        >
          {renderItem(item, idx)}
        </div>
      ))}
    </div>
  );
}

export interface TableColumn<T> {
  key: string;
  header: React.ReactNode;
  render: (row: T) => React.ReactNode;
  width?: string | number;
}

export interface TableProps<T> {
  columns: TableColumn<T>[];
  data: T[];
  className?: string;
}

export function Table<T>({ columns, data, className = "" }: TableProps<T>) {
  return (
    <div
      className={`forge-scrollarea ${className}`}
      style={{
        border: "1px solid var(--forge-border)",
        borderRadius: "var(--forge-radius-sm)",
        background: "var(--forge-surface-sunken)",
        overflow: "auto",
      }}
    >
      <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "var(--forge-font-size-xs)" }}>
        <thead>
          <tr style={{ background: "var(--forge-surface-overlay)", borderBottom: "1px solid var(--forge-border)" }}>
            {columns.map((col) => (
              <th
                key={col.key}
                style={{
                  textAlign: "left",
                  padding: "6px 8px",
                  color: "var(--forge-text-muted)",
                  fontWeight: 600,
                  width: col.width,
                }}
              >
                {col.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {data.map((row, idx) => (
            <tr
              key={idx}
              style={{
                borderBottom: "1px solid var(--forge-border-subtle)",
                transition: "background-color var(--forge-motion-fast)",
              }}
            >
              {columns.map((col) => (
                <td key={col.key} style={{ padding: "6px 8px", color: "var(--forge-text-primary)" }}>
                  {col.render(row)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
