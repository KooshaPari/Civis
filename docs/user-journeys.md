---
title: User Story Journeys
description: Automated video proof of DINOForge features
---

# User Story Journeys

Each journey demonstrates a complete user workflow with automated screenshots, Claude verification, and visual annotations.

## US-F1.1: Game Launch & Mod Verification

Demonstrates launching the game and verifying DINOForge runtime is loaded.

**Status**: Manifest created, awaiting gameplay capture via MCP bridge.

- **Intent**: Launch game → Verify DINOForge loads → Confirm ECS world ready
- **Keyframes**: 3 (game launch, mod verification, world ready)
- **Expected Duration**: ~15 seconds
- **Requirements**: Game instance, BepInEx + DINOForge plugin

### Interactive Journey Viewer

(Journey viewer component would render here with @phenotype/journey-viewer)

---

## US-F2.1: Unit Spawn & Asset Swap

Demonstrates spawning units and verifying asset swaps apply correctly.

**Status**: Planned

---

## US-F3.1: Debug Overlay Toggle

Demonstrates toggling F9/F10 debug overlay.

**Status**: Planned

---

## US-F4.1: Menu Navigation

Demonstrates navigating menus with keyboard input.

**Status**: Planned

---

## How Journeys Are Recorded

All journeys are recorded via the **MCP Bridge** automation layer:

1. **Game Launch** — `game_launch` tool
2. **Automated Screenshots** — `game_screenshot` tool with timing
3. **User Interactions** — `game_input` tool for keyboard/mouse automation
4. **Verification** — Claude AI analyzes screenshots for assertions
5. **Manifest Creation** — Metadata + step references → JSON manifest
6. **Documentation** — Embedded journey-viewer component in VitePress

Zero manual game interaction required. All proofs are fully automated and CI/CD-integrated.

---

**Last Updated**: 2026-04-20  
**Manifest Source**: `docs/journeys/manifests/`  
**CI Quality Gates**: `.github/workflows/journey-quality-gates.yml`
