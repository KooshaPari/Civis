# ADR-012: Keycap Palette Design System

## Status
Accepted

## Context
Civis needs a unified design language across all UI surfaces (Bevy HUD, web dashboard, MCP tooling). Multiple color systems existed in parallel.

## Decision
Adopt Keycap Palette as the authoritative design language: teal #7ebab5 (ACCENT), midnight #090a0c (BG), Montserrat headings, Bricolage subheadings, JetBrains Mono code, 5-layer glassmorphism panels.

## Rationale
- Consistent brand across all surfaces
- WCAG AA compliant contrast ratios
- Defined token set in ui_theme.rs exports

## Consequences
All new UI components must reference ui_theme.rs constants. Legacy amber/cyan references to be migrated.
