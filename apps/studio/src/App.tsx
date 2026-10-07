import React, { useRef, useCallback } from "react";
import {
  DockviewReact,
  type DockviewReadyEvent,
  type DockviewApi,
  type IDockviewPanelProps,
} from "dockview-react";
import { TitleBar } from "./components/TitleBar";
import { CommandPalette } from "./components/CommandPalette";
import { SceneTreePanel } from "./components/panels/SceneTreePanel";
import { ViewportPanel } from "./components/panels/ViewportPanel";
import { CodeEditorPanel } from "./components/panels/CodeEditorPanel";
import { InspectorPanel } from "./components/panels/InspectorPanel";
import { AgentChatPanel } from "./components/panels/AgentChatPanel";
import { ConsolePanel } from "./components/panels/ConsolePanel";
import { TerminalPanel } from "./components/panels/TerminalPanel";
import { PlaytestPanel } from "./components/panels/PlaytestPanel";
import { useLayoutStore } from "./stores/layoutStore";

// Dockview Component Wrapper Map
const components = {
  scene_tree: (_props: IDockviewPanelProps) => <SceneTreePanel />,
  viewport: (_props: IDockviewPanelProps) => <ViewportPanel />,
  code_editor: (_props: IDockviewPanelProps) => <CodeEditorPanel />,
  inspector: (_props: IDockviewPanelProps) => <InspectorPanel />,
  agent_chat: (_props: IDockviewPanelProps) => <AgentChatPanel />,
  console: (_props: IDockviewPanelProps) => <ConsolePanel />,
  terminal: (_props: IDockviewPanelProps) => <TerminalPanel />,
  playtest: (_props: IDockviewPanelProps) => <PlaytestPanel />,
};

export const App: React.FC = () => {
  const apiRef = useRef<DockviewApi | null>(null);
  const { layout, setLayout, resetLayout } = useLayoutStore();

  // Setup default dockview panels and proportions
  const buildDefaultLayout = useCallback((api: DockviewApi) => {
    api.clear();

    // 1. Center Viewport Panel
    const viewportPanel = api.addPanel({
      id: "viewport",
      component: "viewport",
      title: "Viewport",
    });

    // 2. Add Code Editor in the same center group (tabbed)
    api.addPanel({
      id: "code_editor",
      component: "code_editor",
      title: "hero_controller.ts",
      position: { referencePanel: viewportPanel, direction: "within" },
    });

    // 3. Left: Scene Tree
    api.addPanel({
      id: "scene_tree",
      component: "scene_tree",
      title: "Scene Tree",
      position: { referencePanel: viewportPanel, direction: "left" },
      initialWidth: 260,
    });

    // 4. Right: Inspector & Agent Chat (tabbed)
    const inspectorPanel = api.addPanel({
      id: "inspector",
      component: "inspector",
      title: "Inspector",
      position: { referencePanel: viewportPanel, direction: "right" },
      initialWidth: 320,
    });

    api.addPanel({
      id: "agent_chat",
      component: "agent_chat",
      title: "Agent Chat",
      position: { referencePanel: inspectorPanel, direction: "within" },
    });

    // 5. Bottom: Terminal, Console, Playtest (tabbed)
    const terminalPanel = api.addPanel({
      id: "terminal",
      component: "terminal",
      title: "Terminal",
      position: { referencePanel: viewportPanel, direction: "below" },
      initialHeight: 220,
    });

    api.addPanel({
      id: "console",
      component: "console",
      title: "Console",
      position: { referencePanel: terminalPanel, direction: "within" },
    });

    api.addPanel({
      id: "playtest",
      component: "playtest",
      title: "Playtest",
      position: { referencePanel: terminalPanel, direction: "within" },
    });

    // Activate default visible tabs
    viewportPanel.api.setActive();
    terminalPanel.api.setActive();
    inspectorPanel.api.setActive();
  }, []);

  const onReady = useCallback(
    (event: DockviewReadyEvent) => {
      const api = event.api;
      apiRef.current = api;

      let restored = false;
      if (layout) {
        try {
          api.fromJSON(layout);
          restored = true;
        } catch {
          // Layout migration / deserialization fallback
        }
      }

      if (!restored) {
        buildDefaultLayout(api);
      }

      // Track layout changes with debounce to persist
      let saveTimer: ReturnType<typeof setTimeout> | undefined;
      const saveDisposable = api.onDidLayoutChange(() => {
        clearTimeout(saveTimer);
        saveTimer = setTimeout(() => {
          if (apiRef.current) {
            setLayout(apiRef.current.toJSON());
          }
        }, 300);
      });

      return () => {
        clearTimeout(saveTimer);
        saveDisposable.dispose();
      };
    },
    [layout, setLayout, buildDefaultLayout],
  );

  const handleResetLayout = useCallback(() => {
    resetLayout();
    if (apiRef.current) {
      buildDefaultLayout(apiRef.current);
    }
  }, [resetLayout, buildDefaultLayout]);

  const handleTogglePanel = useCallback((panelId: string) => {
    if (!apiRef.current) return;
    const panel = apiRef.current.getPanel(panelId);
    if (panel) {
      if (panel.api.isVisible) {
        panel.api.close();
      } else {
        panel.api.setActive();
      }
    } else {
      // Re-add panel if closed
      apiRef.current.addPanel({
        id: panelId,
        component: panelId,
        title: panelId.replace("_", " ").toUpperCase(),
      });
    }
  }, []);

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-[#0A0B0D] text-[#E8EAED]">
      <TitleBar projectName="templates/hello" />

      <main className="flex-1 w-full h-[calc(100vh-2.5rem)] relative overflow-hidden">
        <DockviewReact
          components={components}
          onReady={onReady}
          className="dockview-theme-dark h-full w-full"
        />
      </main>

      <CommandPalette onTogglePanel={handleTogglePanel} onResetLayout={handleResetLayout} />
    </div>
  );
};

export default App;
