# Voxel/Falling-Sand Baselines — fork/borrow before hand-rolling

**User directive (2026-05-29):** "https://github.com/dmitriy-shmilo/sio2 — this and finding other similar items may be highly relevant as baselines." + "don't forget wrap-over-handroll! fork/borrows are included in this."

## Binding rule: WRAP > HANDROLL (forks/borrows count)
Before any Lead hand-rolls a subsystem (falling-sand CA, chunk streaming, meshing, overlays, pathfinding), it MUST first survey existing OSS to **fork, borrow, depend on, or learn from** — and justify in its report why it wrapped vs hand-rolled. Forking/vendoring/borrowing an algorithm is the preferred path; writing from scratch is the exception that needs justification. Reuse the ecosystem ([[feedback_use_existing_ecosystem]], [[feedback_quality_charter]] wrap>handroll).

## Primary baseline: sio2
- **`dmitriy-shmilo/sio2`** — a Bevy falling-sand / cellular-automata sandbox. Directly in our domain (voxel material-fluid CA on Bevy). Study/borrow: its chunk layout, cell update scheduling, dirty-rect tracking, material/element model, and Bevy integration patterns. Candidate to fork or borrow CA-scheduling + dirty-tracking from, rather than re-deriving ours.

## Other baselines to survey (the "similar items")
The Research/Scale/Material Leads should sweep and shortlist (fork/borrow-or-justify):
- **Falling-sand / powder:** Noita (GDC talk on its CA), The Powder Toy (TPT element + reaction model — our material taxonomy target), sandspiel / sandspiel-studio, Bevy `bevy_falling_sand`, `MaciekTalaska`/various Rust sand sims.
- **Voxel terrain + streaming (Rust/Bevy):** `bonsairobo/building-blocks` & `block-mesh-rs` & `ilattice` (greedy/quad meshing — borrow the mesher), `feldspar`, `bevy_voxel_world`, `Vinatorul/bevy_voxel`, Veloren's voxel/chunk-streaming + LOD, Hexaffett `bevy_meshem`, `splashdust/block_mesh`.
- **SVO / sparse:** `OpenVDB`/NanoVDB concepts, Veloren terrain, `svo`-style crates.
- **Overlays/info-views, ECS perf:** Bevy ecosystem plugins (`bevy_egui` overlays, gizmo layers).

## How Leads apply this
- **Scale & Streaming Lead:** evaluate building-blocks/block-mesh-rs/bevy_voxel_world/Veloren streaming BEFORE hand-rolling the chunk-stream + mesher; borrow the proven mesher/LOD if it fits the deterministic + fixed-point constraints, else document why ours must differ.
- **Material/Voxel Lead:** borrow TPT's element/reaction table shape + sio2's CA scheduling/dirty-tracking for the WorldBox×PowderToy taxonomy.
- Every Lead: a "baselines surveyed / wrapped vs hand-rolled / why" line in its report.

See [[feedback_quality_charter]], [[civis-voxel-fluid-vision]], [[project_phenotype_voxel]].
