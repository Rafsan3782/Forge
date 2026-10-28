import React, { useState } from "react";
import {
  ThemeProvider,
  useTheme,
  Button,
  IconButton,
  ToggleButton,
  SegmentedControl,
  Input,
  NumberInput,
  Vector2Input,
  ColorInput,
  Checkbox,
  Switch,
  Slider,
  Select,
  Combobox,
  Tooltip,
  Popover,
  DropdownMenu,
  Dialog,
  Sheet,
  TreeView,
  Table,
  Badge,
  Chip,
  StatusDot,
  Spinner,
  Skeleton,
  ProgressBar,
  PropertyGrid,
  PropertyRow,
  Breadcrumbs,
  EmptyState,
  SearchField,
  DiffView,
  CommandPalette,
  ToastProvider,
  useToast,
  Icon,
} from "@forge/ui";

interface TableRowData {
  id: string;
  name: string;
  type: string;
  status: string;
}

function GalleryContent() {
  const { theme, density, toggleTheme, toggleDensity } = useTheme();
  const { toast } = useToast();

  const [search, setSearch] = useState("");
  const [togglePressed, setTogglePressed] = useState(false);
  const [segmentVal, setSegmentVal] = useState("translate");
  const [textInput, setTextInput] = useState("PlayerEntity");
  const [numVal, setNumVal] = useState(42);
  const [vecVal, setVecVal] = useState({ x: 120, y: 240 });
  const [colorVal, setColorVal] = useState("#ff5c26");
  const [checkVal, setCheckVal] = useState(true);
  const [switchVal, setSwitchVal] = useState(false);
  const [sliderVal, setSliderVal] = useState(75);
  const [selectVal, setSelectVal] = useState("dynamic");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [sheetOpen, setSheetOpen] = useState(false);
  const [cmdOpen, setCmdOpen] = useState(false);

  const treeNodes = [
    {
      id: "scene-root",
      label: "Main Scene",
      icon: "scene",
      children: [
        {
          id: "player",
          label: "Player",
          icon: "entity",
          children: [
            { id: "sprite", label: "SpriteRenderer", icon: "sprite" },
            { id: "collider", label: "BoxCollider2D", icon: "collider" },
            { id: "script", label: "PlayerController.ts", icon: "script" },
          ],
        },
        { id: "tilemap", label: "Ground Tilemap", icon: "tilemap" },
        { id: "checkpoint", label: "Checkpoint 1", icon: "checkpoint" },
      ],
    },
  ];

  const tableData: TableRowData[] = [
    { id: "1", name: "PlayerEntity", type: "Entity", status: "Active" },
    { id: "2", name: "EnemyPatrol", type: "Entity", status: "Inactive" },
    { id: "3", name: "CoinSpawner", type: "Script", status: "Active" },
  ];

  const tableCols = [
    { key: "name", header: "Name", render: (r: TableRowData) => <strong>{r.name}</strong> },
    { key: "type", header: "Type", render: (r: TableRowData) => <Badge variant="accent">{r.type}</Badge> },
    { key: "status", header: "Status", render: (r: TableRowData) => <Badge variant="default">{r.status}</Badge> },
  ];

  const customIcons = [
    "entity", "component", "scene", "tilemap", "sprite", "collider", "script",
    "play", "stop", "step", "gizmo-move", "gizmo-rotate", "gizmo-scale", "checkpoint", "agent"
  ];

  return (
    <div
      style={{
        minHeight: "100vh",
        background: "var(--forge-surface-base)",
        color: "var(--forge-text-primary)",
        fontFamily: "var(--forge-font-sans)",
        fontSize: "var(--forge-font-size-base)",
      }}
    >
      {/* Top Header */}
      <header
        style={{
          height: 48,
          borderBottom: "1px solid var(--forge-border)",
          background: "var(--forge-surface-raised)",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "0 16px",
          position: "sticky",
          top: 0,
          zIndex: "var(--forge-z-header)",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, fontWeight: 700, letterSpacing: "-0.02em" }}>
            <span style={{ color: "var(--forge-accent)" }}>FORGE</span>
            <span style={{ fontSize: "11px", color: "var(--forge-text-muted)", fontWeight: 500 }}>STUDIO DESIGN SYSTEM V2</span>
          </div>
          <Badge variant="ai">Arc AI</Badge>
          <Badge variant="accent">Ember V2</Badge>
        </div>

        <div style={{ width: 260 }}>
          <SearchField value={search} onChange={setSearch} placeholder="Filter components..." />
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <Button variant="default" size="sm" onClick={toggleDensity}>
            Density: {density}
          </Button>
          <Button variant="default" size="sm" onClick={toggleTheme}>
            Theme: {theme}
          </Button>
          <Button variant="ai" size="sm" icon="agent" onClick={() => setCmdOpen(true)} shortcut="Ctrl+K">
            Palette
          </Button>
        </div>
      </header>

      {/* Main Container */}
      <div style={{ maxWidth: 1200, margin: "0 auto", padding: 24, display: "flex", flexDirection: "column", gap: 32 }}>
        
        {/* Section: Custom Engine Icons */}
        <section>
          <h2 style={{ fontSize: "16px", fontWeight: 600, borderBottom: "1px solid var(--forge-border)", paddingBottom: 6, marginBottom: 16 }}>
            Engine Concept Icons (16px, 1.5px stroke)
          </h2>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(130px, 1fr))", gap: 10 }}>
            {customIcons.map((ic) => (
              <div
                key={ic}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  padding: "8px 10px",
                  background: "var(--forge-surface-sunken)",
                  border: "1px solid var(--forge-border)",
                  borderRadius: "var(--forge-radius-sm)",
                  fontSize: "12px",
                }}
              >
                <Icon name={ic} size={16} color="var(--forge-accent)" />
                <span style={{ color: "var(--forge-text-muted)" }}>{ic}</span>
              </div>
            ))}
          </div>
        </section>

        {/* Section: Buttons & Controls */}
        <section>
          <h2 style={{ fontSize: "16px", fontWeight: 600, borderBottom: "1px solid var(--forge-border)", paddingBottom: 6, marginBottom: 16 }}>
            Buttons & Segmented Controls
          </h2>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 12, alignItems: "center" }}>
            <Button variant="default">Default</Button>
            <Button variant="primary" icon="play">Primary Ember</Button>
            <Button variant="ai" icon="agent">AI Assistant</Button>
            <Button variant="ghost">Ghost</Button>
            <Button variant="danger">Danger</Button>
            <Button variant="default" disabled>Disabled</Button>
            <Button variant="primary" loading>Building</Button>
            <Button variant="default" shortcut="⌘S">Save</Button>
            <IconButton icon="settings" label="Settings" />
            <ToggleButton pressed={togglePressed} onPressedChange={setTogglePressed} icon="lock">
              {togglePressed ? "Locked" : "Unlocked"}
            </ToggleButton>
            <SegmentedControl
              value={segmentVal}
              onChange={setSegmentVal}
              items={[
                { value: "translate", icon: "gizmo-move", label: "Move" },
                { value: "rotate", icon: "gizmo-rotate", label: "Rotate" },
                { value: "scale", icon: "gizmo-scale", label: "Scale" },
              ]}
            />
          </div>
        </section>

        {/* Section: Inputs & Scrubbing */}
        <section>
          <h2 style={{ fontSize: "16px", fontWeight: 600, borderBottom: "1px solid var(--forge-border)", paddingBottom: 6, marginBottom: 16 }}>
            Inputs, Drag Scrubbing & Pickers
          </h2>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))", gap: 16 }}>
            <div>
              <label style={{ fontSize: "12px", color: "var(--forge-text-muted)", display: "block", marginBottom: 4 }}>Text Input</label>
              <Input value={textInput} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setTextInput(e.target.value)} icon="entity" />
            </div>
            <div>
              <label style={{ fontSize: "12px", color: "var(--forge-text-muted)", display: "block", marginBottom: 4 }}>Number Input (Drag label to scrub)</label>
              <NumberInput label="Speed" value={numVal} onChange={setNumVal} unit="px/s" step={1} />
            </div>
            <div>
              <label style={{ fontSize: "12px", color: "var(--forge-text-muted)", display: "block", marginBottom: 4 }}>Vector2 Input</label>
              <Vector2Input x={vecVal.x} y={vecVal.y} onChange={setVecVal} />
            </div>
            <div>
              <label style={{ fontSize: "12px", color: "var(--forge-text-muted)", display: "block", marginBottom: 4 }}>Color Input with Picker</label>
              <ColorInput value={colorVal} onChange={setColorVal} label="Tint" />
            </div>
          </div>
        </section>

        {/* Section: Select, Combobox & Controls */}
        <section>
          <h2 style={{ fontSize: "16px", fontWeight: 600, borderBottom: "1px solid var(--forge-border)", paddingBottom: 6, marginBottom: 16 }}>
            Selection, Checkboxes & Switches
          </h2>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))", gap: 16, alignItems: "center" }}>
            <Select
              value={selectVal}
              onChange={setSelectVal}
              options={[
                { value: "dynamic", label: "Dynamic Rigidbody", icon: "box" },
                { value: "static", label: "Static Collider", icon: "collider" },
                { value: "kinematic", label: "Kinematic Body", icon: "play" },
              ]}
            />
            <Combobox
              value={selectVal}
              onChange={setSelectVal}
              options={[
                { value: "dynamic", label: "Dynamic Rigidbody" },
                { value: "static", label: "Static Collider" },
                { value: "kinematic", label: "Kinematic Body" },
                { value: "tilemap", label: "Tilemap Layer" },
              ]}
            />
            <Checkbox checked={checkVal} onChange={setCheckVal} label="Enable Physics Simulation" />
            <Switch checked={switchVal} onChange={setSwitchVal} label="Headless Playtest" />
            <div style={{ width: "100%" }}>
              <div style={{ fontSize: "11px", color: "var(--forge-text-muted)", marginBottom: 2 }}>Volume: {sliderVal}%</div>
              <Slider value={sliderVal} onChange={setSliderVal} />
            </div>
          </div>
        </section>

        {/* Section: Inspector PropertyGrid & Hierarchy Tree */}
        <section>
          <h2 style={{ fontSize: "16px", fontWeight: 600, borderBottom: "1px solid var(--forge-border)", paddingBottom: 6, marginBottom: 16 }}>
            Inspector PropertyGrid & Scene Hierarchy
          </h2>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
            <div>
              <Breadcrumbs items={[{ label: "Project" }, { label: "Scenes" }, { label: "Level_01.scene" }]} />
              <div style={{ marginTop: 8 }}>
                <TreeView nodes={treeNodes} selectedId="player" onSelect={(n: { id: string; label: string }) => toast({ title: `Selected ${n.label}`, type: "info" })} />
              </div>
            </div>

            <div>
              <PropertyGrid title="Transform Component">
                <PropertyRow label="Position" tooltip="Entity local transform coordinates">
                  <Vector2Input x={vecVal.x} y={vecVal.y} onChange={setVecVal} />
                </PropertyRow>
                <PropertyRow label="Rotation">
                  <NumberInput label="Deg" value={0} onChange={() => {}} unit="°" />
                </PropertyRow>
                <PropertyRow label="Scale">
                  <Vector2Input x={1} y={1} onChange={() => {}} />
                </PropertyRow>
              </PropertyGrid>

              <PropertyGrid title="Sprite Renderer">
                <PropertyRow label="Color Tint">
                  <ColorInput value={colorVal} onChange={setColorVal} />
                </PropertyRow>
                <PropertyRow label="Visible">
                  <Switch checked={checkVal} onChange={setCheckVal} />
                </PropertyRow>
              </PropertyGrid>
            </div>
          </div>
        </section>

        {/* Section: Overlays & Modals */}
        <section>
          <h2 style={{ fontSize: "16px", fontWeight: 600, borderBottom: "1px solid var(--forge-border)", paddingBottom: 6, marginBottom: 16 }}>
            Overlays, Menus & Tooltips
          </h2>
          <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
            <Tooltip content="Step one frame forward in playtest" shortcut="F10">
              <Button variant="default" icon="step">Hover for Tooltip</Button>
            </Tooltip>

            <Popover
              trigger={<Button variant="default">Open Popover</Button>}
              content={
                <div>
                  <div style={{ fontWeight: 600, fontSize: "13px", marginBottom: 4 }}>Inspector Settings</div>
                  <div style={{ fontSize: "12px", color: "var(--forge-text-muted)" }}>Configure grid snapping and viewport guides.</div>
                </div>
              }
            />

            <DropdownMenu
              trigger={<Button variant="default" icon="chevron-down">Options Menu</Button>}
              items={[
                { id: "edit", label: "Edit Component", icon: "script", action: () => toast({ title: "Edit", type: "info" }) },
                { id: "dup", label: "Duplicate", shortcut: "Ctrl+D", action: () => toast({ title: "Duplicated", type: "success" }) },
                { id: "del", label: "Delete Entity", icon: "trash", danger: true, action: () => toast({ title: "Deleted", type: "error" }) },
              ]}
            />

            <Button variant="default" onClick={() => setDialogOpen(true)}>Open Dialog</Button>
            <Button variant="default" onClick={() => setSheetOpen(true)}>Open Side Sheet</Button>
            <Button variant="default" onClick={() => toast({ title: "Checkpoint Saved", description: "All entity states recorded", type: "success" })}>
              Trigger Toast
            </Button>
          </div>
        </section>

        {/* Section: Data Table, DiffView & Resizable Panels */}
        <section>
          <h2 style={{ fontSize: "16px", fontWeight: 600, borderBottom: "1px solid var(--forge-border)", paddingBottom: 6, marginBottom: 16 }}>
            Data Table & Code Diff
          </h2>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
            <Table columns={tableCols} data={tableData} />
            <DiffView
              original={'const speed = 100;\nplayer.velocity.x = speed;'}
              modified={'const speed = 150; // Boosted\nplayer.velocity.x = speed;\nplayer.playAnimation("run");'}
            />
          </div>
        </section>

        {/* Section: Feedback & Skeletons */}
        <section>
          <h2 style={{ fontSize: "16px", fontWeight: 600, borderBottom: "1px solid var(--forge-border)", paddingBottom: 6, marginBottom: 16 }}>
            Feedback, Badges & Loading States
          </h2>
          <div style={{ display: "flex", gap: 16, alignItems: "center" }}>
            <StatusDot status="online" />
            <StatusDot status="busy" />
            <StatusDot status="ai" />
            <Spinner size={16} />
            <div style={{ width: 140 }}>
              <ProgressBar value={64} />
            </div>
            <div style={{ width: 160 }}>
              <Skeleton height={20} />
            </div>
            <Chip label="Tag: Player" onRemove={() => toast({ title: "Removed tag", type: "info" })} />
          </div>
        </section>
      </div>

      {/* Modals & Dialogs */}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen} title="Project Settings" description="Engine and build target configuration">
        <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          <Input placeholder="Game Title" defaultValue="Cyber Platformer" />
          <NumberInput label="Framerate" value={60} onChange={() => {}} unit="fps" />
          <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 12 }}>
            <Button variant="ghost" onClick={() => setDialogOpen(false)}>Cancel</Button>
            <Button variant="primary" onClick={() => setDialogOpen(false)}>Save Changes</Button>
          </div>
        </div>
      </Dialog>

      <Sheet open={sheetOpen} onOpenChange={setSheetOpen} title="Agent Inspection Console">
        <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          <EmptyState icon="agent" title="Autonomous Agent Ready" description="Arc agent is standing by for scene generation prompts." />
        </div>
      </Sheet>

      <CommandPalette
        open={cmdOpen}
        onOpenChange={setCmdOpen}
        commands={[
          { id: "play", title: "Play Game", category: "Engine", icon: "play", shortcut: "F5", action: () => toast({ title: "Playing", type: "success" }) },
          { id: "save", title: "Save Scene", category: "File", icon: "save", shortcut: "Ctrl+S", action: () => toast({ title: "Saved", type: "info" }) },
          { id: "agent", title: "Generate Platformer Tilemap", category: "Agent", icon: "agent", action: () => toast({ title: "Agent running", type: "info" }) },
        ]}
      />
    </div>
  );
}

export default function App() {
  return (
    <ThemeProvider>
      <ToastProvider>
        <GalleryContent />
      </ToastProvider>
    </ThemeProvider>
  );
}
