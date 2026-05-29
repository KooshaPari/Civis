# v0.27.0 — True Full-Conversion Experience: Story Index

**Epic**: [EPIC-027](../v0.27.0-full-conversion-epic.md)
**Target release**: v0.27.0
**Total points (excl. stretch)**: 131 — **Total (incl. stretch)**: 144

---

## Sprint Plan

Sprints are ordered by dependency depth: foundation first, then identity, then asset
completeness, then mechanical expansion, then stretch. Each sprint is designed to be
independently shippable: every story has an in-game verification gate (screenshot or external
judge receipt) before it is considered done.

---

### Sprint 1 — Foundation (24 points)

Goal: unblock every downstream story. No identity or asset work starts until these are green.

| ID | Story | Points | Key dependency |
|---|---|---|---|
| [SW-001](native-mods-page-visible.md) | Native Mods Page Visible | 8 | iter-146 button |
| [SW-002](dinoforge-active-indicator.md) | DINOForge Active Indicator | 3 | Plugin.Awake |
| [SW-003](real-asset-bundles.md) | Real Asset Bundles (kill #101) | 13 | Unity 2021.3 build |

**Sprint 1 exit criteria**:
- `dinoforge status` shows both packs Active.
- Mods page opens and lists packs (no blank state).
- Window title shows `| DINOForge v0.27.0`.
- `dinoforge verify-mod --pack warfare-starwars` exits 0 (0 stub bundles).
- `dotnet test` green.

---

### Sprint 2 — Identity (42 points)

Goal: the game reads as a completely different product from the title screen forward.

| ID | Story | Points | Key dependency |
|---|---|---|---|
| [SW-004](loading-screen-takeover.md) | Loading Screen Takeover | 13 | SW-006 Phase 0 schema |
| [SW-005](mod-brand-identity.md) | Mod Brand Identity (SW + Modern) | 8 | SW-006 Phase 1 |
| [SW-006](full-ui-ux-reskin.md) | Full UI/UX Reskin Engine | 21 | MainMenuThemer refactor |

**SW-006 must start first** (Phase 0 schema + ThemeEngine foundation). SW-004 and SW-005 can
progress in parallel once Phase 1 (MainMenuReskinner) of SW-006 is merged.

**Sprint 2 exit criteria**:
- DINOForge loading screen visible during plugin init (screenshot proof).
- SW main menu: deep-space background, gold title, Republic palette (external judge receipt).
- Modern main menu: satellite-pass background, stencil title, amber palette (external judge receipt).
- HUD (Phase 2): SW + Modern palette applied in gameplay (screenshot proof).
- `dotnet test` green; `PackCompiler validate` exits 0 for both packs.

---

### Sprint 3 — Assets (26 points)

Goal: no vanilla 2D art visible in either mod during a full play session.

| ID | Story | Points | Key dependency |
|---|---|---|---|
| [SW-007](ingame-2d-asset-takeover.md) | In-Game 2D Asset Takeover | 13 | SW-006 Phase 1, key-discovery |
| [SW-008](mod-asset-completeness.md) | Mod Asset Completeness | 13 | SW-003 bundles |

Both stories can run in parallel. SW-007 Strategy A (Addressables intercept) requires the
key-discovery pass — unblock SW-007 with Strategy B first, then add Strategy A.

**Sprint 3 exit criteria**:
- In-game: every spawnable unit shows a mod portrait (screenshot proof per mod).
- Faction emblems replaced in HUD (external judge receipt).
- `PackCompiler validate` exits 0 for both packs.
- Unit count: SW ≥ 22/28, Modern ≥ 18/28.

---

### Sprint 4 — Mechanics (42 points)

Goal: new unit types (naval, aerial) functional; combat feels thematic.

| ID | Story | Points | Key dependency |
|---|---|---|---|
| [SW-009](blaster-projectile-support.md) | Blaster / Projectile Support | 8 | SW-003 bundles |
| [SW-010](naval-combat.md) | Naval Combat | 13 | SW-009 (soft) |
| [SW-011](aerial-combat-complete.md) | Aerial Combat Complete | 13 | SW-009 (soft) |
| [SW-012](audio-takeover.md) | Audio Takeover | 8 | SW-006 ThemeEngine |

All four stories are parallel. SW-009 is a soft dependency for SW-010 and SW-011 (they fall
back to default projectiles if SW-009 is not merged first).

**Sprint 4 exit criteria**:
- Themed projectiles visible during combat (screenshot proof).
- Naval units build, move on water, and attack (screenshot proof).
- Aerial units fly over all terrain and engage air + ground (screenshot proof).
- Mod music plays on main menu and in gameplay for both mods.
- `dotnet test` green.

---

### Sprint 5 — Stretch (13 points)

Goal: R&D decision on space combat; prototype if time allows.

| ID | Story | Points | Key dependency |
|---|---|---|---|
| [SW-013](sw-space-combat.md) | SW Space Combat (R&D / stretch) | 13 | SW-006, SW-011 |

This sprint is not a v0.27.0 release blocker. SW-013's minimum deliverable is the
R&D document — implementation is bonus.

---

## Cross-Story Dependencies

```
SW-003 (real bundles)
  └── SW-008 (asset completeness)
  └── SW-009 (projectiles)
      └── SW-010 (naval) [soft]
      └── SW-011 (aerial) [soft]

SW-006 (ThemeEngine — all phases)
  └── SW-004 (loading screen)
  └── SW-005 (brand identity)
  └── SW-007 (2D asset takeover, Strategy B)
  └── SW-012 (audio, scene-change hook)
  └── SW-013 (stretch HUD)

SW-007 (2D asset takeover)
  └── SW-008 (portrait loading)
```

---

## Definition of Done (Epic-level, v0.27.0 tag)

All Sprint 1–4 stories must be done. Sprint 5 is optional.

- [ ] `dotnet build src/DINOForge.sln -c Release` exits 0.
- [ ] `dotnet test src/DINOForge.sln` all green, no regressions from v0.26.0.
- [ ] `PackCompiler validate` exits 0 for both packs.
- [ ] External judge receipts in `docs/proof/judge-receipts/` for every story.
- [ ] CHANGELOG.md updated for v0.27.0.
- [ ] All new public APIs carry XML doc comments.
- [ ] `dinoforge status` reports both packs Active.

---

## Suggested Agent Dispatch Order

1. **Sprint 1**: 3 agents in parallel — one per story (SW-001, SW-002, SW-003).
   SW-003 requires Unity 2021.3.45f2 build step — schedule for early in sprint.
2. **Sprint 2**: Start SW-006 Phase 0 first (schema + tests, 1 agent). Once schema lands,
   dispatch SW-004, SW-005, and SW-006 Phase 1–6 in parallel (3 agents).
3. **Sprint 3**: SW-007 + SW-008 in parallel (2 agents). SW-007 Strategy A unblocked when
   key-discovery pass finishes.
4. **Sprint 4**: All 4 stories in parallel (4 agents). SW-009 starts first by 1–2 days.
5. **Sprint 5**: 1 agent for R&D spike only.

Minimum concurrent dispatches per project rules: ≥ 5 agents (see `feedback_scale_concurrency_10_15.md`).
