# Evidence, ownership and decision gates

Pinned source: [main2bcb231c](https://github.com/KooshaPari/Civis/tree/2bcb231c7f3e11835a49e3d467fb2b91634aa988). This is an owned non-fork product (`isFork=false`, parent=null), not an upstream patch staging branch. Owned release owner must reconcile product/server/client/profile versions and artifact/checkpoint compatibility. Fork-specific upstream delta/version suffix is not applicable to the product itself; dependencies and renderer adapters retain their own base/version/license/update ownership.

| Evidence | What it supports | Limits |
|---|---|---|
| [3D FR matrix](../../traceability/fr-3d-matrix.md) | Authoritative requirement/source/test map | Implementation labels are not a new execution or installed proof |
| [Playthrough](../../playthrough.md) | Existing journey/harness contract | Historical profile, determinism and capture instructions cannot override current game intent or capture ban |
| [Save specification](../../specs/CIV-1000-save-load-persistence-spec.md) | Full resumable state contract | Specification is not proof every field/runtime path is complete |
| [Save source](https://github.com/KooshaPari/Civis/blob/2bcb231c7f3e11835a49e3d467fb2b91634aa988/crates/engine/src/save_bundle.rs) | Actual engine save surface | Full culture/social/AI/mod round-trip not newly tested |
| [PR1598](https://github.com/KooshaPari/Civis/pull/1598), [1593](https://github.com/KooshaPari/Civis/pull/1593) | Existing E2E binary/framing repairs | Open candidates; no duplicate implementation or merge here |
| [PR1596](https://github.com/KooshaPari/Civis/pull/1596), [1569](https://github.com/KooshaPari/Civis/pull/1569) | Populated faction tuple and semantic-save work | Reuse and integrate only after acceptance receipts |
| [PR1595](https://github.com/KooshaPari/Civis/pull/1595), [1594](https://github.com/KooshaPari/Civis/pull/1594), [1597](https://github.com/KooshaPari/Civis/pull/1597) | Existing advisory/manifest/quality lanes | Gate status and actual fix need exact-head qualification |
| [PR1567](https://github.com/KooshaPari/Civis/pull/1567) | Mature emergence/save oracle recovery | Draft specification, not current gameplay proof |
| [Issue1610](https://github.com/KooshaPari/Civis/issues/1610) | Current-main playthrough regression | Failure type must be classified, not inferred as universal simulation failure |
| [Issues1602-1609](https://github.com/KooshaPari/Civis/issues/1602) | All-features compilation/lint/annotation debt family | Multiple reports may share causes; not an independent defect count |
| [Main gameE2E](https://github.com/KooshaPari/Civis/actions/runs/38938174755), [playthrough](https://github.com/KooshaPari/Civis/actions/runs/38938174749) | Dated failed native product gates | No fresh run here; successful library/manifest gates separate |

Read-only protection refresh at cutoff: required reviews null; required status strict=true but named contexts=[] and checks=[]; enforce_admins=false. Review removal is observed, meaningful check retention is not. Existing seed intent requires both appropriate review removal and retained status gates. Sampled prior quality/game jobs used GitHub-hosted runners, so desk execution remains unqualified. No settings or workflows were changed here.

Dossier source defines installed persistent game-first scope, controls/authority, living settlement, restart and unattended pilot. Private user/harness exports are summarized only; no raw private transcript is published. Canonical dirty branch and its modified manifest/census/API doc were preserved by an isolated docs worktree. No product tests/builds, services, images or captures in this lane.

| Decision | Owner / required evidence | Failure response |
|---|---|---|
| D1 finite supported game profile | Product/integration owner beforeC1; selected client, content, roles, world and FR denominator | Resolve boundary, preserve wider intent without inventing completion |
| D2 engine/adapters | Product/renderer owner using same real checkpoint/content tasks | Adopt/wrap engine capabilities where better; no mandatory Bevy/Godot/Unreal winner |
| D3 evolved persistence authority | Simulation/save owners with independent oracle, C3 | Do not call seed regeneration a resumed world |
| D4 actual play value | Unfamiliar player + assurance, C5/M5 | Narrow or reshape experience from feedback; do not equate build success with game value |
| D5 release | Release owner afterMVP; enforced profiles, install/update/save/restore receipts | Internal experimental status until reliable journey; no release-readiness percentage |

No sunset is selected. If the persistent multi-role experience does not create value, propose an explicit narrower product/adapter with preserved saves and consumer impact rather than silently erase accepted goals. Future scaling and feedback may change scope; estimates are finite assumptions, not a perpetual roadmap ceiling.
