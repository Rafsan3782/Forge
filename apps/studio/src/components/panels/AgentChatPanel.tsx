import React, { useState } from "react";
import { Bot, Send, Sparkles } from "lucide-react";

export const AgentChatPanel: React.FC = () => {
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<Array<{ role: "user" | "agent"; text: string }>>([]);

  const handleSend = () => {
    if (!input.trim()) return;
    setMessages((prev) => [
      ...prev,
      { role: "user", text: input.trim() },
      {
        role: "agent",
        text: `Connected to Forge Bridge runtime. Ready to execute engine actions for: "${input.trim()}"`,
      },
    ]);
    setInput("");
  };

  return (
    <div className="h-full w-full bg-[#121418] text-[#E8EAED] flex flex-col select-none text-xs">
      {/* Header */}
      <div className="p-2 border-b border-[#23262D] flex items-center justify-between bg-[#0A0B0D]/50">
        <div className="flex items-center gap-1.5 font-medium text-[#8B919C]">
          <Bot className="w-3.5 h-3.5 text-[#7C5CFF]" />
          <span>Agent Chat</span>
        </div>
        <div className="flex items-center gap-1 text-[10px] text-[#7C5CFF] font-mono">
          <span className="w-1.5 h-1.5 rounded-full bg-[#7C5CFF] animate-pulse" />
          <span>Gemini Agent</span>
        </div>
      </div>

      {/* Messages / Empty State */}
      <div className="flex-1 overflow-y-auto p-3 flex flex-col">
        {messages.length === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center text-center p-4">
            <div className="w-9 h-9 rounded-full bg-[#7C5CFF]/15 border border-[#7C5CFF]/30 flex items-center justify-center mb-3">
              <Sparkles className="w-4 h-4 text-[#7C5CFF]" />
            </div>
            <span className="font-semibold text-sm text-[#E8EAED]">Autonomous Engine Agent</span>
            <p className="text-[11px] text-[#8B919C] mt-1 max-w-[220px]">
              Describe game mechanics, scene tweaks, or asset generation to run in real-time.
            </p>

            <div className="mt-4 space-y-1.5 w-full text-left">
              {[
                "Add jump physics to Hero entity",
                "Create a rotating coin item at (100, 50)",
                "Capture screenshot and inspect layout",
              ].map((prompt) => (
                <button
                  key={prompt}
                  onClick={() => setInput(prompt)}
                  className="w-full text-[11px] px-2.5 py-1.5 rounded bg-[#181B20] hover:bg-[#20232B] border border-[#23262D] text-[#8B919C] hover:text-[#E8EAED] transition-colors text-left"
                >
                  "{prompt}"
                </button>
              ))}
            </div>
          </div>
        ) : (
          <div className="space-y-3">
            {messages.map((m, idx) => (
              <div
                key={idx}
                className={`p-2.5 rounded-lg border text-xs ${
                  m.role === "user"
                    ? "bg-[#181B20] border-[#23262D] text-[#E8EAED] ml-4"
                    : "bg-[#7C5CFF]/10 border-[#7C5CFF]/30 text-[#E8EAED] mr-4"
                }`}
              >
                <div className="flex items-center gap-1.5 font-medium mb-1 text-[10px] uppercase tracking-wider text-[#8B919C]">
                  {m.role === "user" ? "Creator" : "Forge Agent"}
                </div>
                <div>{m.text}</div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Chat Input Box */}
      <div className="p-2 border-t border-[#23262D] bg-[#0A0B0D]/60">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            handleSend();
          }}
          className="flex items-center gap-1.5 bg-[#181B20] border border-[#23262D] focus-within:border-[#7C5CFF] rounded-lg px-2.5 py-1.5 transition-colors duration-150"
        >
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Ask agent to modify scene or code..."
            className="flex-1 bg-transparent text-xs text-[#E8EAED] placeholder-[#8B919C] outline-none"
          />
          <button
            type="submit"
            disabled={!input.trim()}
            className="p-1 rounded bg-[#7C5CFF] hover:bg-[#6A4BE0] disabled:opacity-40 disabled:hover:bg-[#7C5CFF] text-white transition-colors cursor-pointer"
          >
            <Send className="w-3 h-3" />
          </button>
        </form>
      </div>
    </div>
  );
};
