# Civis UI Design System

This document is the shared UI/UX specification for all Civis clients:
`web` (Three.js overlay), `Bevy` (`egui`), and `Godot` (`Control` nodes).

It defines the common layout, visual language, interaction patterns, and accessibility
rules that each client must implement against. Clients may adapt spacing and widget
implementation details to fit their runtime, but they must preserve the same information
hierarchy and control semantics.

## Design Goals

- Make the simulation readable at a glance.
- Keep core controls discoverable without obscuring the world.
- Support fast selection, inspection, and camera control.
- Preserve a consistent mental model across all clients.
- Scale from compact web overlays to richer desktop clients without changing the
  underlying UI contract.

## Reference Games UI

These references define the intended feel and control grammar. Civis should borrow the
pattern, not the art assets.

| Reference | What to borrow | Civis application |
|-----------|----------------|-------------------|
| WorldBox bottom bar | Compact, icon-led tool strip; immediate world editing; clear mode switching | Bottom bar is the primary action surface for Select, Spawn, Build, Terraform, Destroy, Weather, and Diplomacy |
| Cities: Skylines 2 radial menu | Fast contextual branching from a selected target; low-friction command groups | Right-click opens a context menu whose entries depend on selection type and current tool |
| Manor Lords contextual UI | World-first, lightly framed panels that feel attached to the scene | Inspector and contextual panels should be restrained, high-contrast, and close to the selected entity or panel edge |
| Age of Empires sidebar | Dense command and status presentation in a vertical rail | Right-side inspector and left-side minimap/faction rail should prioritize dense, scannable information |

## Layout

The UI is organized into five stable regions.

### Top Bar

Purpose: high-level state and time control.

Contains:

- Game clock
- Population
- Key resources
- Current era
- Speed controls

Rules:

- The top bar is always visible unless the client is in a dedicated full-screen cinematic
  mode.
- Speed controls must include pause/resume and at least 1x, 2x, and 4x states.
- The clock and era should be visually dominant enough to anchor the simulation state.

### Bottom Bar

Purpose: primary action palette.

Contains:

- Select
- Spawn
- Build
- Terraform
- Destroy
- Weather
- Diplomacy

Rules:

- This is the main world-editing surface for all clients.
- The active tool must be clearly highlighted.
- Tool groups may expand into submenus, but the top-level categories must remain stable.
- If a tool is unavailable in a client, it must be disabled rather than removed from the
  layout contract.

### Right Panel

Purpose: entity inspector.

Behavior:

- Click an entity in the world to select it.
- The right panel updates to show details for the selected entity.
- If nothing is selected, the panel shows summary state or remains collapsed, depending
  on client capacity.

Contents:

- Entity name and type
- Ownership / faction
- Current status
- Core stats relevant to the selected type
- Contextual actions where applicable

Rules:

- The inspector is the authoritative location for per-entity details.
- Details must update immediately on selection change.
- Do not use the inspector as a second action palette unless those actions are
  selection-specific and explicitly contextual.

### Left Panel

Purpose: world overview.

Contains:

- Minimap
- Faction list

Rules:

- The minimap should support click-to-focus where the client can map world position back
  to camera focus.
- The faction list should surface faction identity, color, and current state at a glance.
- The left panel must remain legible when compressed into a narrow docked rail.

### Center Toast Region

Purpose: transient notifications.

Behavior:

- Show short-lived alerts in the center of the screen.
- Use this region for state changes that need immediate attention but do not warrant a
  modal interrupt.

Examples:

- Tool unavailable
- Entity selected
- Speed changed
- Action completed
- Diplomacy state changed

Rules:

- Toasts must not block camera control or selection.
- Toast stacking should prefer vertical grouping with recent messages near the center.
- Messages should auto-dismiss unless they require user acknowledgment.

## Visual Language

### Overall Tone

- Functional, tactical, and legible.
- Framed like a control room, not a consumer app.
- The world view remains primary; chrome should support it, not overpower it.

### UI Chrome

Use a restrained chrome language:

- Dark translucent panels with subtle borders
- Soft shadows, not heavy drop shadows
- Thin separators for dense data tables
- Rounded corners kept modest and consistent

Recommended chrome tokens:

- `ui.panel.bg`: `rgba(15, 20, 28, 0.82)`
- `ui.panel.bg-strong`: `rgba(12, 16, 22, 0.94)`
- `ui.panel.border`: `rgba(255, 255, 255, 0.08)`
- `ui.panel.border-active`: `rgba(255, 255, 255, 0.16)`
- `ui.shadow`: subtle blur, low opacity, never a hard outline

### Color Palette

The palette must support faction identity, biome reading, and UI hierarchy.

#### Faction Colors

Use at least four distinct, color-blind-aware faction colors.

Suggested default set:

- `Faction A`: teal `#2EC4B6`
- `Faction B`: gold `#E9C46A`
- `Faction C`: coral `#E76F51`
- `Faction D`: blue `#4D96FF`

Guidelines:

- Avoid relying on red/green alone to distinguish factions.
- Pair each faction color with a shape, icon, or label.
- Use faction colors consistently in the minimap, faction list, entity chips, and
  relation/diplomacy indicators.

#### Biome Colors

Biome colors should be subdued enough to read the terrain without competing with faction
identity.

Suggested biome family:

- Water: deep blue `#2A5D8F`
- Grassland: muted green `#4F7D4A`
- Forest: darker green `#2F5B3A`
- Desert: sand `#C9B27B`
- Tundra: pale gray-blue `#B7C7D6`
- Mountain: slate `#6D7683`
- Swamp: olive `#5B6B4F`

Guidelines:

- Biomes are background geography, not primary UI accents.
- Biome markers must remain distinguishable under reduced saturation.

#### Text Colors

- Primary text: `#F2F5F8`
- Secondary text: `#B8C0CC`
- Disabled text: `#6F7A88`
- Warning text: amber family
- Error text: warm red family

#### Accent Colors

Use a small set of accents for state emphasis:

- Positive / success: green-cyan family
- Warning / active alert: amber family
- Critical / destructive: red family
- Interactive focus: bright cyan or blue family

Rules:

- Accent color should convey meaning, not decoration.
- Never use the same accent color for both destructive and constructive states.

## Typography

The game should use a readable, UI-optimized font family with a compact, tactical feel.

Recommended font stack:

- Primary: a geometric or humanist sans optimized for UI legibility
- Fallbacks: system sans, then platform default

Typography rules:

- Use a single primary font family across clients when possible.
- Prefer tabular numerals for resources, time, and counters.
- Keep labels short and scannable.

Suggested sizing scale:

- Header / title: `20-24 px`
- Section heading: `16-18 px`
- Body: `13-15 px`
- Label / metadata: `11-12 px`
- Micro / helper text: `10-11 px`

Usage rules:

- Clock, population, and era should use strong numeric clarity.
- Entity names should be visually stronger than secondary stats.
- Dense inspector data should rely on spacing and weight, not only on size changes.

## Interaction Patterns

All clients must preserve these input semantics.

| Input | Behavior |
|-------|----------|
| Left click | Select entity |
| Right click | Open context menu |
| Scroll | Zoom camera |
| WASD | Pan camera |
| Number keys | Speed control |
| Space | Pause / resume |
| Tab | Cycle between selected entities |
| Escape | Deselect or open menu |

Rules:

- Selection is the primary interaction primitive.
- Right-click menus should be contextual, not generic tool dumps.
- Zoom should not steal focus from UI widgets when the pointer is over a panel.
- Keyboard controls must remain usable without mouse input.
- Escape should follow a predictable priority: close transient UI, then deselect, then open
  the menu if nothing else is active.

### Context Menu Rules

- The menu contents should change based on selected entity, active tool, or hovered
  object.
- Keep the menu shallow enough for fast use.
- If a command requires multiple steps, the menu should launch a workflow rather than
  embedding a long form inline.

### Selection Rules

- A selected entity must have one clear highlight state in the world and one matching
  state in the inspector.
- Multi-selection is allowed only if the client can render it clearly and the selection
  model remains explicit.
- Tab cycling should respect selection order and proximity where applicable.

## Responsive Behavior

The same design must work across the three client families:

### Web

- Use a Three.js overlay for the world and HTML/CSS or canvas-based panels for chrome.
- Keep panels docked and compact.
- Favor explicit layout constraints over auto-flow-heavy UI that can drift across screen sizes.

### Bevy

- Use `egui` panels and overlays.
- Preserve the same region ordering and color tokens.
- Favor keyboard-first navigation and quick inspector updates.

### Godot

- Use `Control` nodes, anchors, and theme overrides.
- Preserve the same dock positions and interaction semantics.
- Prefer native scene controls for menus and inspectors, but keep content and state model
  aligned with the other clients.

Responsive rules:

- On wide screens, all four side regions may be visible simultaneously.
- On narrow screens, preserve top bar and bottom bar first, then collapse left and right
  panels into toggles or drawers.
- The entity inspector must remain reachable without hiding the primary world view.
- The UI must not depend on a fixed pixel-perfect window size.

## Accessibility

Accessibility is a first-class requirement.

### Color and Contrast

- Faction colors must remain distinguishable for common color-vision deficiencies.
- Do not use color alone to communicate state.
- Maintain strong contrast for text, icons, and selection outlines.
- Prefer luminance contrast and shape cues for important distinctions.

### Keyboard Navigation

- Menus must be keyboard-navigable.
- Focus order must be deterministic.
- Every actionable control should have a keyboard path or shortcut.
- Tooltips should expose the corresponding shortcut when one exists.

### Readability

- Avoid tiny text in core HUD elements.
- Keep numeric counters aligned and easy to scan.
- Ensure entity selection and state changes have a visible, non-color cue.

### Motion and Alerts

- Animations should help orientation, not distract from gameplay.
- Toasts and panel transitions should be brief and readable.
- Allow critical feedback to persist long enough for keyboard-only users to perceive it.

## Client Implementation Contract

Each client must implement the same semantic UI model:

- Same top-level regions
- Same tool names
- Same selection and context menu behaviors
- Same core keyboard shortcuts
- Same palette and typography intent
- Same accessibility rules

Clients may differ in rendering technology and widget implementation, but they must not
change the meaning of the controls or the information hierarchy.

## Non-Goals

- Recreating the exact art style of the reference games
- Hardcoding a single resolution
- Making the web client match desktop feature-for-feature when the platform cannot support
  it
- Introducing a second, client-specific UI vocabulary

## Open Notes

- If a client cannot support a specific tool or panel, it should degrade by disabling the
  feature, not by renaming or relocating it.
- The design system should remain compatible with future FR additions for modding,
  diplomacy depth, and richer entity inspection.
