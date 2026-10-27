# Forge Design Principles

Design principles for Forge Studio and developer tool interfaces.

---

## Core Pillars

1. **Dense but calm:**
   - Base 13px typography with clean hierarchy.
   - Clean spacing with no superfluous padding that pushes content offscreen.
   - Information density tailored for power creators.

2. **Precise alignment on a 4px grid:**
   - Spacing scales strictly in 4px multiples: 4px, 8px, 12px, 16px, 20px, 24px, 32px.
   - Control heights:
     - Compact (default): 22px (sm), 26px (md), 30px (lg).
     - Comfortable: 26px (sm), 30px (md), 34px (lg).

3. **Restrained color:**
   - Surfaces are deep near-black (`#0a0b0e`).
   - One **Ember** accent (`#ff5c26`) reserved strictly for primary actions, selections, and confirmations.
   - One **Arc Violet** accent (`#8b5cf6`) reserved exclusively for AI reasoning, suggestions, and activity.
   - Never mix accent colors arbitrarily.

4. **Crisp 1px borders over heavy shadows:**
   - Thin 1px subtle borders (`var(--forge-border)`) create structural separation without visual clutter.
   - Depth and shadows are used sparingly, only for elevated floating layers (tooltips, popovers, command palettes).

5. **Text is the hero:**
   - Typography uses **Geist Sans** for UI and **JetBrains Mono** for code and numeric inspectors.
   - Inspector values use tabular numbers (`tabular-nums`) to prevent jitter during scrubbing or animation.

6. **Short, purposeful motion:**
   - Micro-transitions are snappy: 100ms (`motion-fast`), 150ms (`motion-normal`), 200ms (`motion-slow`).
   - Automatically collapses to instant (0ms) when `prefers-reduced-motion` is active.

7. **Keyboard first:**
   - Every primary action exhibits its keyboard shortcut hint (`<kbd>`).
   - Intentional focus rings on keyboard navigation (`:focus-visible`).

---

## Do and Do Not Examples

| Do | Do Not |
|---|---|
| Use `var(--forge-accent)` (Ember) for user action buttons | Do not use Ember for badges, AI indicators, or random links |
| Use `var(--forge-ai)` (Arc Violet) for AI agent panels and triggers | Do not use violet for primary editor actions |
| Use 1px borders (`var(--forge-border)`) for panels and dividers | Do not use heavy 4px or blur-heavy drop shadows on docked panels |
| Use tabular figures for coordinates, velocities, and rotation inputs | Do not allow proportional numbers that shift input box widths |
| Use 22px/26px/30px control heights for high density toolbars | Do not use generic 40px+ web form button heights |
| Provide explicit `:focus-visible` outline on interactive elements | Do not disable browser outline without replacement |
