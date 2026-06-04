# Cross-Repo Docs Audit — 2026-05-30

Audited README + docs across the user's local git repos for **accuracy**, **completeness**, and **richness** (Mermaid diagrams / screenshots / demo media — ties to rich-embeds #966). Scored 0–5 per axis, flagged gaps, and applied local-only fixes. No pushes.

## Repo enumeration

`find C:\Users\koosh -maxdepth 3 -name .git` returned **80+** git repos (most under `Dev/`). This audit prioritized the named + user-facing/deployed repos:

| Repo | Path | Remote |
|------|------|--------|
| DINOForge | `C:\Users\koosh\Dino` | github.com/KooshaPari/Dino |
| AgilePlus | `C:\Users\koosh\agileplus` | github.com/kooshapari/agileplus |
| phenotype-journeys | `C:\Users\koosh\phenotype-journeys` | github.com/KooshaPari/phenotype-journeys |
| bare-cua (playcua) | `C:\Users\koosh\playcua_ci_test` | github.com/KooshaPari/playcua |
| Civis / CivLab | `C:\Users\koosh\Dev\Civis` | github.com/KooshaPari/Civis |

**HWLedger**: not present locally (referenced only as a planned consumer in phenotype-journeys). Skipped — no local clone to audit.
The broader `Dev/*` repos (phenotype-* family, BytePort, OmniRoute, etc.) were out of scope for this pass given the named-repo priority; recommend a follow-up sweep.

## Scorecard (0–5)

| Repo | Accuracy | Completeness | Richness | Notes |
|------|:--------:|:------------:|:--------:|-------|
| **DINOForge** | 5 | 5 | 4 → 5 | Excellent: badges, milestone table, Mermaid arch, full sections. Gap: no demo/media embed despite many screenshots. |
| **AgilePlus** | 2 → 5 | 2 → 5 | 1 → 4 | **Major gap**: README described only the `proto/` subset; repo is a full polyglot monorepo (Rust core/CLI, Python MCP, Go pheno-cli, VitePress docs). Rewrote to match reality. |
| **phenotype-journeys** | 5 | 5 | 3 → 4 | Accurate, detailed (assert/OCR/sentinel docs). Gap: no diagram, no badges. |
| **bare-cua (playcua)** | 5 | 5 | 4 → 5 | Very rich ASCII arch + method/platform tables. Gap: no badges, no demo media. |
| **Civis / CivLab** | 5 | 5 | 4 → 5 | Very rich: badges, deep WS JSON-RPC API docs, status. Gap: ASCII-only, no architecture Mermaid. |

### Gap detail

- **AgilePlus (most severe):** title was `agileplus-proto`; described 3 gRPC services as the whole repo. Actual repo: `crates/` (20+ Rust crates incl. domain/api/grpc/graph/cli), `agileplus-mcp/` (Python MCP server), `agileplus-agents/` (Rust), `pheno-cli/` (Go), `docs/` (51-page VitePress site, spec-driven dev engine), `proto/`, `python/`, `rust/`. A consumer reading the old README would entirely miss the product. **Fixed.**
- **DINOForge:** README is accurate to code/milestones; only richness gap was absence of a demo embed (screenshots exist under `docs/screenshots/`). Added embed stub.
- **phenotype-journeys / bare-cua / Civis:** content accurate and complete; richness limited by no Mermaid (Civis/bare-cua used ASCII) and missing demo media for what are visual/interactive tools.

## Fixes applied (local commits only)

| Repo | Change | Commit (branch) |
|------|--------|--------|
| AgilePlus | Full README rewrite: accurate monorepo description, component table, **2 Mermaid diagrams** (architecture + 9-phase pipeline), proto section retained, EMBED stub | `130d125` (docs/readme-audit-20260530) |
| phenotype-journeys | Added **Mermaid** record→verify→assert flow + EMBED stub | `c509493` (docs/readme-audit-20260530) |
| bare-cua (playcua) | Added license/rust/transport badges, **Mermaid** stdio sequence diagram, EMBED demo stub | `6eeae50` (docs/readme-audit-20260530) |
| Civis | Added **Mermaid** multi-client architecture diagram + EMBED stub | `62fb8aa3` (ci/local-first-manifest-verify) |
| DINOForge | Cross-repo audit doc + README Demo section with EMBED stub | `0bdd4938` (feat/unityexplorer-devtools-20260528) |
| DINOForge | Committed #958 model-preview gallery (contact sheet + 9 real-model PNGs) + `RenderModelPreviews.cs`; README **Model Preview Gallery** EMBED stub → **HAVE** | (this commit) |

All `> [!EMBED] STUB ...` boxes are placed where demo/media belongs so the #966 rich-embed pipeline can fill them. The DINOForge **Model Preview Gallery** embed is now **HAVE** — backed by the committed `docs/screenshots/model-previews/` gallery (#958). The remaining DINOForge in-game mod-menu/live-swap embed stays STUB pending a recording.

## Documented follow-up (not done this pass)

- Sweep the remaining `Dev/*` repos (phenotype-* family, BytePort, OmniRoute, Authvault, etc.) for README accuracy — large surface, deferred.
- Once #966 pipeline lands, replace EMBED stubs with rendered recordings.
- HWLedger README audit when/if cloned locally.
- AgilePlus: consider per-component sub-READMEs (`agileplus-mcp/`, `pheno-cli/` already has one) linked from root.
