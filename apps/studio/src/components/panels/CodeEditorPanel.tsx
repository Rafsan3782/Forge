import React, { useRef } from "react";
import Editor, { type OnMount } from "@monaco-editor/react";

const INITIAL_CODE = `import { Entity, Transform2D, Sprite } from "@forge/core";

/**
 * Hero character movement controller system
 */
export function updateHeroController(world: any, deltaSec: number) {
  const query = world.query([Transform2D, Sprite]);

  for (const [entity, transform, sprite] of query) {
    // Smooth pulse animation
    transform.scale.x = 1.0 + Math.sin(Date.now() * 0.003) * 0.05;
    transform.scale.y = 1.0 + Math.sin(Date.now() * 0.003) * 0.05;
  }
}
`;

export const CodeEditorPanel: React.FC = () => {
  const editorRef = useRef<Parameters<OnMount>[0] | null>(null);

  const handleEditorDidMount: OnMount = (editor, monaco) => {
    editorRef.current = editor;

    // Define tokens-based custom Monaco theme
    monaco.editor.defineTheme("forge-dark", {
      base: "vs-dark",
      inherit: true,
      rules: [
        { token: "", background: "121418", foreground: "E8EAED" },
        { token: "comment", foreground: "8B919C", fontStyle: "italic" },
        { token: "keyword", foreground: "FF6A2B" },
        { token: "identifier", foreground: "E8EAED" },
        { token: "string", foreground: "3DD68C" },
        { token: "number", foreground: "F5A524" },
        { token: "type", foreground: "7C5CFF" },
      ],
      colors: {
        "editor.background": "#121418",
        "editor.foreground": "#E8EAED",
        "editorLineNumber.foreground": "#565B64",
        "editorLineNumber.activeForeground": "#E8EAED",
        "editorCursor.foreground": "#FF6A2B",
        "editor.selectionBackground": "#282C36",
        "editor.inactiveSelectionBackground": "#181B20",
        "editorIndentGuide.background1": "#1C1E24",
        "editorIndentGuide.activeBackground1": "#23262D",
        "editorGutter.background": "#121418",
      },
    });

    monaco.editor.setTheme("forge-dark");
  };

  return (
    <div className="h-full w-full bg-[#121418] flex flex-col overflow-hidden">
      <div className="h-7 px-3 bg-[#0A0B0D]/50 border-b border-[#23262D] flex items-center justify-between text-xs text-[#8B919C] font-mono">
        <span>src/scripts/hero_controller.ts</span>
        <span className="text-[11px] text-[#7C5CFF]">TypeScript (Monaco)</span>
      </div>

      <div className="flex-1 w-full h-full relative">
        <Editor
          height="100%"
          defaultLanguage="typescript"
          defaultValue={INITIAL_CODE}
          theme="forge-dark"
          onMount={handleEditorDidMount}
          options={{
            fontSize: 13,
            fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            automaticLayout: true,
            tabSize: 2,
            lineNumbersMinChars: 3,
            folding: true,
            glyphMargin: false,
            renderLineHighlight: "all",
          }}
        />
      </div>
    </div>
  );
};
