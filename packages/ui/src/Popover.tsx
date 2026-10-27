import React, { useState, useRef, useEffect } from "react";

export interface PopoverProps {
  trigger: React.ReactNode;
  content: React.ReactNode;
  align?: "start" | "center" | "end";
}

export const Popover: React.FC<PopoverProps> = ({ trigger, content, align = "start" }) => {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    window.addEventListener("mousedown", handleClickOutside);
    return () => window.removeEventListener("mousedown", handleClickOutside);
  }, []);

  return (
    <div ref={ref} style={{ position: "relative", display: "inline-block" }}>
      <div onClick={() => setOpen(!open)}>{trigger}</div>
      {open && (
        <div
          style={{
            position: "absolute",
            top: "100%",
            marginTop: 4,
            [align === "end" ? "right" : "left"]: 0,
            background: "var(--forge-surface-overlay)",
            border: "1px solid var(--forge-border-strong)",
            borderRadius: "var(--forge-radius-md)",
            boxShadow: "var(--forge-elevation-high)",
            zIndex: "var(--forge-z-popover)",
            padding: 12,
            minWidth: 200,
          }}
        >
          {content}
        </div>
      )}
    </div>
  );
};
