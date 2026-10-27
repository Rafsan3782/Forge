import React from "react";
import * as RadixDialog from "@radix-ui/react-dialog";
import { Icon } from "./Icon.js";

export interface DialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: string;
  children: React.ReactNode;
}

export const Dialog: React.FC<DialogProps> = ({
  open,
  onOpenChange,
  title,
  description,
  children,
}) => {
  return (
    <RadixDialog.Root open={open} onOpenChange={onOpenChange}>
      <RadixDialog.Portal>
        <RadixDialog.Overlay
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0, 0, 0, 0.75)",
            zIndex: "var(--forge-z-modal)",
            backdropFilter: "blur(2px)",
          }}
        />
        <RadixDialog.Content
          style={{
            position: "fixed",
            top: "50%",
            left: "50%",
            transform: "translate(-50%, -50%)",
            background: "var(--forge-surface-raised)",
            border: "1px solid var(--forge-border-strong)",
            borderRadius: "var(--forge-radius-md)",
            boxShadow: "var(--forge-elevation-floating)",
            width: "90vw",
            maxWidth: 480,
            maxHeight: "85vh",
            padding: 20,
            zIndex: "calc(var(--forge-z-modal) + 1)",
            color: "var(--forge-text-primary)",
          }}
        >
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
            <RadixDialog.Title style={{ margin: 0, fontSize: "var(--forge-font-size-md)", fontWeight: 600 }}>
              {title}
            </RadixDialog.Title>
            <RadixDialog.Close asChild>
              <button
                style={{
                  background: "transparent",
                  border: "none",
                  color: "var(--forge-text-muted)",
                  cursor: "pointer",
                }}
              >
                <Icon name="x" size={16} />
              </button>
            </RadixDialog.Close>
          </div>
          {description && (
            <RadixDialog.Description
              style={{
                fontSize: "var(--forge-font-size-sm)",
                color: "var(--forge-text-muted)",
                marginBottom: 16,
              }}
            >
              {description}
            </RadixDialog.Description>
          )}
          {children}
        </RadixDialog.Content>
      </RadixDialog.Portal>
    </RadixDialog.Root>
  );
};

export interface SheetProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  side?: "right" | "left";
  children: React.ReactNode;
}

export const Sheet: React.FC<SheetProps> = ({
  open,
  onOpenChange,
  title,
  side = "right",
  children,
}) => {
  return (
    <RadixDialog.Root open={open} onOpenChange={onOpenChange}>
      <RadixDialog.Portal>
        <RadixDialog.Overlay
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0, 0, 0, 0.6)",
            zIndex: "var(--forge-z-modal)",
          }}
        />
        <RadixDialog.Content
          style={{
            position: "fixed",
            top: 0,
            bottom: 0,
            [side]: 0,
            width: "360px",
            background: "var(--forge-surface-raised)",
            borderLeft: side === "right" ? "1px solid var(--forge-border)" : "none",
            borderRight: side === "left" ? "1px solid var(--forge-border)" : "none",
            boxShadow: "var(--forge-elevation-high)",
            padding: 16,
            zIndex: "calc(var(--forge-z-modal) + 1)",
            display: "flex",
            flexDirection: "column",
          }}
        >
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
            <RadixDialog.Title style={{ margin: 0, fontSize: "var(--forge-font-size-md)", fontWeight: 600 }}>
              {title}
            </RadixDialog.Title>
            <RadixDialog.Close asChild>
              <button
                style={{
                  background: "transparent",
                  border: "none",
                  color: "var(--forge-text-muted)",
                  cursor: "pointer",
                }}
              >
                <Icon name="x" size={16} />
              </button>
            </RadixDialog.Close>
          </div>
          <div style={{ flex: 1, overflowY: "auto" }}>{children}</div>
        </RadixDialog.Content>
      </RadixDialog.Portal>
    </RadixDialog.Root>
  );
};
