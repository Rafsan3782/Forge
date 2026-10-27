import React from "react";
import * as RadixTabs from "@radix-ui/react-tabs";
import { Icon } from "./Icon.js";

export interface TabItem {
  id: string;
  label: React.ReactNode;
  icon?: string;
  content: React.ReactNode;
}

export interface TabsProps {
  items: TabItem[];
  defaultValue?: string;
  value?: string;
  onValueChange?: (val: string) => void;
  className?: string;
}

export const Tabs: React.FC<TabsProps> = ({
  items,
  defaultValue,
  value,
  onValueChange,
  className = "",
}) => {
  return (
    <RadixTabs.Root
      defaultValue={defaultValue || items[0]?.id}
      value={value}
      onValueChange={onValueChange}
      className={`forge-tabs ${className}`}
      style={{ display: "flex", flexDirection: "column", width: "100%", height: "100%" }}
    >
      <RadixTabs.List
        style={{
          display: "flex",
          borderBottom: "1px solid var(--forge-border)",
          background: "var(--forge-surface-sunken)",
          padding: "0 4px",
          gap: 2,
        }}
      >
        {items.map((tab) => (
          <RadixTabs.Trigger
            key={tab.id}
            value={tab.id}
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 6,
              padding: "6px 12px",
              fontSize: "var(--forge-font-size-xs)",
              fontWeight: 500,
              color: "var(--forge-text-muted)",
              background: "transparent",
              border: "none",
              borderBottom: "2px solid transparent",
              cursor: "pointer",
            }}
          >
            {tab.icon && <Icon name={tab.icon} size={14} />}
            <span>{tab.label}</span>
          </RadixTabs.Trigger>
        ))}
      </RadixTabs.List>
      {items.map((tab) => (
        <RadixTabs.Content
          key={tab.id}
          value={tab.id}
          style={{ flex: 1, padding: 8, overflow: "auto" }}
        >
          {tab.content}
        </RadixTabs.Content>
      ))}
    </RadixTabs.Root>
  );
};
