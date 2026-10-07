import React, { useEffect, useRef } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";

export const TerminalPanel: React.FC = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    // Initialize xterm instance with Forge dark theme tokens
    const term = new Terminal({
      cursorBlink: true,
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      fontSize: 12,
      lineHeight: 1.2,
      theme: {
        background: "#0A0B0D",
        foreground: "#E8EAED",
        cursor: "#FF6A2B",
        selectionBackground: "rgba(124, 92, 255, 0.3)",
        black: "#121418",
        red: "#FF4D4F",
        green: "#3DD68C",
        yellow: "#F5A524",
        blue: "#7C5CFF",
        magenta: "#FF6A2B",
        cyan: "#3DD68C",
        white: "#E8EAED",
      },
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);

    term.open(containerRef.current);
    try {
      fitAddon.fit();
    } catch {
      // Container not laid out yet
    }

    term.writeln("\x1b[1;38;2;255;106;43mForge Studio Embedded Terminal\x1b[0m");
    term.writeln("Local developer toolchain activated (\x1b[38;2;61;214;140m.tools ready\x1b[0m)");
    term.writeln(
      "Try: \x1b[38;2;124;92;255mforge call engine.ping {}\x1b[0m or \x1b[38;2;124;92;255mcargo check\x1b[0m\r\n",
    );
    term.write("\x1b[38;2;255;106;43mforge\x1b[0m \x1b[38;2;139;145;156m>\x1b[0m ");

    term.onData((data) => {
      // Echo UI only
      if (data === "\r") {
        term.write("\r\n\x1b[38;2;255;106;43mforge\x1b[0m \x1b[38;2;139;145;156m>\x1b[0m ");
      } else if (data === "\u007F") {
        term.write("\b \b");
      } else {
        term.write(data);
      }
    });

    termRef.current = term;

    const handleResize = () => {
      try {
        fitAddon.fit();
      } catch {
        // Container resized
      }
    };
    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      term.dispose();
    };
  }, []);

  return (
    <div className="h-full w-full bg-[#0A0B0D] p-1 flex flex-col overflow-hidden">
      <div ref={containerRef} className="h-full w-full overflow-hidden" />
    </div>
  );
};
