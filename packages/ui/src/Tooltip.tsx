import React from "react";
import * as RadixTooltip from "@radix-ui/react-tooltip";

export interface TooltipProps {
  content: React.ReactNode;
  shortcut?: string;
  side?: "top" | "right" | "bottom" | "left";
  children: React.ReactElement;
}

export const Tooltip: React.FC<TooltipProps> = ({
  content,
  shortcut,
  side = "top",
  children,
}) => {
  return (
    <RadixTooltip.Provider delayDuration={200}>
      <RadixTooltip.Root>
        <RadixTooltip.Trigger asChild>{children}</RadixTooltip.Trigger>
        <RadixTooltip.Portal>
          <RadixTooltip.Content
            side={side}
            sideOffset={4}
            className="forge-tooltip-content"
          >
            <span>{content}</span>
            {shortcut && <kbd className="forge-kbd">{shortcut}</kbd>}
            <RadixTooltip.Arrow style={{ fill: "var(--forge-border-strong)" }} />
          </RadixTooltip.Content>
        </RadixTooltip.Portal>
      </RadixTooltip.Root>
    </RadixTooltip.Provider>
  );
};
