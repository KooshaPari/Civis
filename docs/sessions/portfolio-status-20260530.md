# Portfolio Status Dashboard — 2026-05-30

Manager-level read-only survey of the wider KooshaPari portfolio (beyond DINOForge), focused on the explicitly named families: **Helios**, **PlusPlus** (agentapi + cliproxy), **OmniRoute**, **BytePort**, plus context on the broader product set.

- **Remote total:** 177 repos under `KooshaPari` (gh). The vast majority are `Pheno*`/`phenotype-*` ecosystem libraries — out of scope for this pass except as context.
- **Local clones found:** 69 git repos across `C:\Users\koosh` and `C:\Users\koosh\Dev` (many `Dev/*`).
- **Method:** `gh repo list` + `gh run list` + `gh pr list` for CI/PR; local `git log/status/rev-list` where cloned. Read-only — no other repos modified.

Legend: 🟢 healthy · 🟡 warning/stale-ish · 🔴 broken/failing · — none/N/A.

---

## Health Matrix

| Repo | Family | What it is | Last activity | Build | Deploy | Docs | Open PRs | State |
|------|--------|-----------|---------------|:----:|:-----:|:---:|:-------:|-------|
| OmniRoute | OmniRoute | AI gateway: OpenAI-compat endpoint, smart routing/LB/retries/fallbacks, policies+observability | 2026-05-30 | 🟡 | — | 🟢 | 1 | Most active in portfolio (1038 commits/30d). CI only runs Claude/Copilot review (mostly "skipped"); no real build/test gate visible. On feature branch `feat/step10-upstream-tracking-adr`, clean tree, in sync. |
| BytePort | BytePort | "Phenotype-org infrastructure tooling" | remote 2026-05-28 / local 2025-01-06 | 🔴 | 🟡 | 🟢 | 12 | Local clone is STALE (~16 months behind, on `bytesolar`, 1 uncommitted file, 0 commits/30d). Remote CI failing: github_actions update + TruffleHog secrets scan fail. 12 open PRs (mostly dependabot-style updates). Landing `byteport.kooshapari.com` referenced. |
| byteport-landing | BytePort | Landing page for byteport.kooshapari.com | 2026-05-28 | 🟡 | 🟡 | 🟡 | — | Companion landing page; deploy URL referenced, live status not verified (#968). |
| agentapi-plusplus | PlusPlus | HTTP API for Claude Code, Goose, Aider, Gemini, Amp, Codex | 2026-05-28 | 🔴 | — | 🟢 | 5 | Active. CI `Alert sync issues` workflow startup_failure (repeated); coderabbit retry succeeded. 5 open PRs. |
| cliproxyapi-plusplus | PlusPlus | "Plus" version of CLIProxyAPI | 2026-05-29 | 🔴 | — | 🟢 | 9 | Active. CI: repeated `Alert sync issues` startup_failure. 9 open PRs — highest PR backlog in PlusPlus. |
| agentapi | PlusPlus | Original AgentAPI — unified gateway for agent orchestration (private) | 2026-05-07 | 🟡 | — | 🟡 | — | Predecessor to agentapi-plusplus; ~3 weeks idle, likely superseded by the ++ fork. |
| vibeproxy | PlusPlus | Deprecated fork: macOS menu-bar app routing AI tools via CLIProxyAPIPlus | 2026-05-29 | 🟡 | — | 🟢 | — | Self-described "Deprecated fork" but recently pushed. Consumes cliproxyapi-plusplus. |
| helios-cli | Helios | Phenotype-org multi-runtime CLI | 2026-05-29 | 🔴 | — | 🟢 | 5 | Active. `codeql-rust` workflow failing; stale-PR/CLA workflows skipped. 5 open PRs. Canonical Helios CLI (note backup says "use HexaKit/helios-cli" — possible source-of-truth ambiguity). |
| heliosApp | Helios | "Internal tool/component" (generic desc) | 2026-05-29 | 🔴 | — | 🔴 | 15 | Active but unhealthy: repeated `Alert sync issues` startup_failure, **15 open PRs** (highest in Helios), placeholder description. Needs triage. |
| helioscope | Helios | "Helios CLI - Command-line interface" | 2026-05-29 | 🟡 | — | 🟡 | 0 | `alert-sync-issues` failing but `PR Babysit Watch` succeeds; 0 open PRs. Name/desc overlaps helios-cli — clarify relationship. |
| heliosBench | Helios | "Internal tool/component" (generic desc) | 2026-05-28 | 🔴 | — | 🔴 | 8 | OpenSSF Scorecard + Dependency Review pass, but `Journey Gate` failing. 8 open PRs, placeholder description. |
| HeliosLab | Helios | Phenotype-org research lab | 2026-05-29 | 🔴 | — | 🟢 | 2 | Security (SAST) + OpenSSF Scorecard failing. 2 open PRs. Research/experimental. |
| helios-router | Helios | Streamlit dashboard for Pareto analysis of LLM provider/model selection (private) | 2026-05-28 | 🟡 | 🟡 | 🟡 | — | Streamlit app — deploy likely manual/local. Not cloned locally. |
| helios-cli-backup | Helios | DEPRECATED backup of helios-cli | 2026-05-03 | — | — | 🟢 | — | ARCHIVED. Self-labels deprecated → use HexaKit/helios-cli. Retire/confirm. |

---

## Families grouped

### Helios family (7 active + 1 archived)
`helios-cli` (canonical CLI), `helioscope` (CLI, overlapping), `heliosApp` (app), `heliosBench` (benchmarking), `HeliosLab` (research), `helios-router` (Streamlit Pareto dashboard, private), `helios-cli-backup` (archived/deprecated). All Rust/Python-flavored Phenotype-org components.
- **Systemic CI rot:** the shared `Alert sync issues` / `alert-sync-issues.yml` workflow `startup_failure`s across heliosApp + helioscope (and the PlusPlus repos). This is one broken reusable workflow hitting many repos — fix once.
- **PR backlog:** heliosApp (15) + heliosBench (8) + helios-cli (5) + HeliosLab (2) = 30 open PRs in the family.
- **Naming ambiguity:** `helios-cli`, `helioscope`, and `helios-cli-backup` all describe a "Helios CLI." Source-of-truth unclear (backup points to HexaKit).

### PlusPlus family (agentapi + cliproxy)
`agentapi-plusplus` (5 PRs), `cliproxyapi-plusplus` (9 PRs), original `agentapi` (private, idle since 2026-05-07), `vibeproxy` (deprecated macOS consumer of cliproxyapi-plusplus). Same `Alert sync issues` CI startup_failure affects both ++ repos.

### OmniRoute (standalone product)
Single repo, by far the **most active** in the entire portfolio (1038 commits in 30d). Clean local tree, in sync with origin, on a feature branch. The gap: CI is only AI-review (Claude/Copilot) with results "skipped" — no evident build/test gate enforcing correctness on such a high-velocity repo.

### BytePort (standalone product)
`BytePort` (infra tooling, 12 PRs, failing CI), `byteport-landing` (byteport.kooshapari.com), `Byteport-Portfolio` (local-only, see "not located"). Local BytePort clone is badly stale (Jan 2025).

### Other product-level repos (context, not deep-assessed this pass)
Notable non-Pheno products in the portfolio: **Tracera** (requirements traceability/PM), **Authvault** (auth framework, multiple worktrees locally), **AgilePlus** (spec-driven PM, the active PM engine), **Civis** (governance, private + local game variant), **forgecode** (AI pair programmer), **bifrost** (AI gateway — overlaps OmniRoute thematically), **Planify** (deprecated Plane.so fork), **PlayCua** (computer-use agent), **WorldSphereMod**/**Compound-Spheres-3D** (game mods), **Tokn** (LLM cost tracking), **kwality**/**KodeVibe** (code-quality, protected personal projects). Many `STRICTLY DO NOT DELETE` personal projects (AppGen, kwality, KodeVibeGo, KlipDot, KVirtualStage) are intentionally preserved.

---

## Most broken / stale

1. **BytePort (local clone)** — ~16 months stale (Jan 2025), on `bytesolar`, uncommitted file; remote CI failing (TruffleHog + actions update) with 12 open PRs. Biggest divergence in the named set.
2. **heliosApp** — 15 open PRs, CI startup_failure, placeholder description. Largest PR backlog.
3. **`Alert sync issues` reusable workflow** — startup_failure across heliosApp, helioscope, agentapi-plusplus, cliproxyapi-plusplus. One root cause, multiple red repos.
4. **heliosBench** — `Journey Gate` failing, 8 PRs, generic description.
5. **HeliosLab / helios-cli** — SAST/Scorecard/codeql-rust failures.

## Most active

1. **OmniRoute** — 1038 commits/30d, clean, in-sync. Clearly the current primary product focus.
2. **PlusPlus (cliproxyapi-plusplus, agentapi-plusplus)** — pushed 05-28/29, steady PR flow.
3. **helios-cli / heliosApp / helioscope** — all pushed 05-29.

## Recommended next actions per family

- **Helios:** (a) Fix the shared `alert-sync-issues` reusable workflow once — clears red across the family. (b) Triage/merge heliosApp's 15 PRs. (c) Resolve helios-cli vs helioscope vs HexaKit/helios-cli source-of-truth. (d) Replace the two "internal tool/component" placeholder descriptions (heliosApp, heliosBench).
- **PlusPlus:** Same workflow fix; burn down cliproxyapi-plusplus's 9 PRs; decide formally whether original `agentapi` is retired in favor of the ++ fork.
- **OmniRoute:** Add a real build/test CI gate (it currently only runs AI review) given its very high velocity; merge the single open PR. Note thematic overlap with `bifrost` — confirm intentional.
- **BytePort:** Refresh the stale local clone; fix TruffleHog secret-scan + actions-update CI failures; sweep the 12 PRs; fill in the generic description.

## Named repos NOT located (point me to these)

- **"PlusPlus" as a single repo / org** — there is no repo literally named `plusplus`/`plus-plus`. The family exists only as the `*-plusplus` suffix convention (`agentapi-plusplus`, `cliproxyapi-plusplus`, with `vibeproxy` as a consumer). Confirm whether a parent/umbrella repo is expected.
- **"cliproxy"** — closest match is `cliproxyapi-plusplus` (the ++ fork); there is no standalone `cliproxy` repo under KooshaPari (the upstream `CLIProxyAPI` is a different org's project). Confirm if you mean the ++ fork.
- **`Byteport-Portfolio`** — exists as a **local-only** clone (`C:\Users\koosh\Dev\Byteport-Portfolio`) with **no matching remote** in `KooshaPari`. Also `BytePort-TestPortfolio` is local-only. Confirm origin / whether these should be pushed.
- All explicitly named families (Helios, agentapi, OmniRoute, BytePort) were otherwise located.
