# Civis AI / LLM / SLM / ML R&D Plan

**Status:** R&D proposal (docs-only). Owner: AI/ML R&D Lead.
**Governing constraint:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — only physical/environmental/genomic laws are authored; everything else EMERGES. **Determinism is NOT required** (charter §"Determinism is NOT a requirement", 2026-05-29): LLMs, floats, and real randomness are welcome where they enrich emergent variety.
**Gap map:** [`docs/specs/feature-matrix.md`](../specs/feature-matrix.md).
**Existing assets this plan extends (do NOT duplicate):**
- `crates/research` — the `LlmClient` trait, `ResearchCache` (hash-keyed), `LlmEvent` (blake3-keyed event record), `ReplayMode`, and `run_research_cycle` pipeline already exist and are the template for everything below.
- `crates/research/src/firepass.rs` — `FirepassKimiClient` (cloud, OpenAI-compatible chat-completions via `KIMI_API_KEY` / `FIREPASS_BASE_URL`, `json_object` response format). This is the heavy-reasoning fallback.

The single most important architectural conclusion: **we already have the right port (`LlmClient` + cache + event log). The work is to *generalize* it into a `civ-ai` crate with a local SLM impl alongside the cloud impl, not to invent a new abstraction.**

---

## 0. Design principles (binding)

1. **Local-first, OSS, free.** Per repo stance (`CLAUDE.md` "local OSS/free only"): a local GGUF SLM is the default in-game generator; Firepass/Kimi cloud is an explicit, opt-in heavy-reasoning fallback, never the default path.
2. **Flavor, not authority.** AI output is *narration and naming* over the emergent substrate, or *advisory* input to existing utility-AI — it never becomes a hardcoded enum, never bypasses `civ-laws` validation, never replaces the physics/genetics simulation. The charter's "model the rule, not the outcome" still holds: AI describes outcomes the sim already produced.
3. **Never block the sim.** Every AI call is async, runs on a dedicated worker pool off the sim/render threads, and the simulation tick advances regardless. Results land later via a channel and are attached to entities/log when ready. No tick ever waits on a token.
4. **Loud failure, not silent degrade** (`CLAUDE.md` "Optionality and failure behavior"). If a *required* model file is missing, fail at preflight with the named missing artifact (`civ-ai: model 'qwen2.5-1.5b-instruct-q4_k_m.gguf' not found at <path>`). Cosmetic features (legends prose) degrade *visibly* to a templated fallback with a logged warning; advisory features that the sim depends on are required and fail loud.
5. **Cache everything; replay-safe by construction.** Reuse the existing blake3 hash-keyed cache (`ResearchCache` / `LlmEvent.cache_key`). Determinism is not required, but caching is still mandatory for cost, latency, and the optional event-log/replay path.

---

## 1. IN-GAME AI

Each row maps a feature to a Civis system, a recommended model/approach, and a perf/cost/caching strategy. Priority column feeds the top-5 in §5.

### 1.1 Emergent-history / legends narration (Dwarf-Fortress-style chronicle)

- **Civis system:** the event log (already exists per feature-matrix §2 "Emergent histories/legends — BLIND, event log only"); consumes `civ-engine` metrics + `civ-agents` lifecycle events (birth/death/sickness, recent commit), `civ-tactics` battles, `civ-economy` booms/busts.
- **Approach:** **SLM summarization/chronicling**, NOT generation-from-nothing. The sim is the source of truth; the SLM *renders* a window of structured events into prose. Pipeline: (a) deterministic event aggregator buckets events by epoch/region/lineage into a compact JSON "epoch digest"; (b) SLM turns the digest into a legends paragraph; (c) store prose keyed by digest-hash. This is map-reduce summarization — exactly what 1–3B SLMs are good at, and it keeps hallucination bounded because the model only re-describes facts in the digest.
- **Recommended model:** **Qwen2.5-1.5B-Instruct** (GGUF Q4_K_M) as default; **SmolLM3-3B** or **Gemma-2-2B-it** when more narrative fluency is wanted and headroom exists. Summarization tolerates small models well.
- **Perf/cost/caching:** Runs **off the hot path entirely** — fire once per in-game epoch (minutes of wall-clock), not per tick. Digest-hash → prose cache means an unchanged epoch is free on replay/reload. Budget: ~300–600 output tokens/epoch. On a 3090 Ti a 1.5B Q4 model emits this in well under a second; even CPU-only is acceptable at epoch cadence. **Cloud (Kimi) tier** only for "grand saga" end-of-age recaps the player explicitly requests.

### 1.2 Procedural NAMING (agents, cultures, languages, places, factions)

- **Civis system:** `civ-species` / `civ-agents` (agent + lineage names), emergent culture/language layer (`civ-engine` ideology metrics), `civ-protocol-3d` building/settlement graph (place names), emergent polity clusters (faction names — note charter: faction is an *emergent cluster*, not `faction:u32`, so names attach to cluster IDs).
- **Approach: grammar/Markov FIRST, SLM as garnish.** Naming is the worst possible use of a per-name LLM call (thousands of agents, latency-sensitive, repetitive). Use a **phoneme-grammar + Markov generator** seeded by emergent language parameters (the culture's drifted phoneme inventory, syllable structure). This is effectively Tracery/Markov-style generation but driven by the *emergent* language state, so names speciate as dialects drift — fully on-brand for the charter. Reserve the SLM for **one-shot batch jobs**: at culture-birth, ask the SLM to invent a *naming grammar* (phoneme set + morphology rules) for that culture once; thereafter the cheap local grammar mints unlimited names with zero model calls. Sources confirm hybrid grammar+LLM beats pure-LLM for naming.
- **Recommended model/crate:** grammar generator hand-rolled or via a small Rust crate (Markov order-2/3 over phoneme tables); SLM for grammar-seeding = **Qwen2.5-0.5B/1.5B** (tiny, batch, rare).
- **Perf/cost/caching:** **~zero runtime cost** — the grammar runs inline in Rust, no model on the hot path. Per-culture grammar generated once and cached with the culture record. This is the cheapest high-impact feature; see top-5.

### 1.3 Leader/faction "personality" + diplomacy chatter + news headlines + lore

- **Civis system:** emergent polity clusters + diplomacy shadow (`CIV-0105`), psyche layer (feature-matrix §2 "Psyche — BLIND", BehaviorWeights today). Personality should be *derived from* the agent's emergent `BehaviorWeights`/temperament/memory, then *expressed* as text — the AI reads the numbers, it does not invent the personality.
- **Approach: Fixed-persona SLM with modular memory.** A leader's `BehaviorWeights` + recent memory + current diplomatic stance form a compact persona prompt; the SLM produces in-character chatter, a diplomatic line, or a one-line news headline. The recent paper *"Fixed-Persona SLMs with Modular Memory: Scalable NPC Dialogue on Consumer Hardware"* (arXiv 2511.10277) validates exactly this design — fine-tuned/persona-prompted SLMs with runtime-swappable memory modules for many characters on consumer GPUs. Headlines and diplomacy lines are short (1–2 sentences) → cheap.
- **Recommended model:** **Llama-3.2-3B-Instruct** or **Gemma-2-2B-it** (GGUF Q4_K_M) for chatter quality; **Qwen2.5-1.5B** for headlines. Optional future: a single LoRA fine-tune over a small persona-dialogue set (TRL/Unsloth) to bake "in-world voice" cheaply — but persona-prompting is enough to ship.
- **Perf/cost/caching:** **Event-triggered, rate-limited, never per-tick.** Generate chatter only on state-change events (war declared, trade signed, leader dies) and only for clusters near the camera / on the player's notification feed. Cache by `(persona_hash, event_hash)` so the same situation reuses prose. Hard cap N concurrent generations; queue the rest. Cloud tier for rare "summit speech"-grade set-pieces.

### 1.4 Cultural/language drift assist (embeddings for similarity / speciation of memes & dialects)

- **Civis system:** emergent ideology/culture/language drift (feature-matrix §2 "INCOMPLETE", `civ-engine` ideology metrics). Mirrors the existing **Hamming-distance speciation** in `civ-genetics` — but for *memes/dialects* in continuous semantic space instead of DNA bitvectors.
- **Approach: embeddings, NOT generative LLM.** Represent a belief/meme/dialect as a vector; measure drift as cosine distance; declare a *cultural speciation event* when clusters separate past a threshold (directly analogous to the genomic speciation threshold the charter already endorses). This is a measured emergent pattern — exactly what the charter wants ("a measured, emergent pattern over the substrate"). Embeddings are cheap, deterministic-enough, and run locally with no generation.
- **Recommended crate/model:** **fastembed-rs** or **EmbedAnything** (both Rust, Candle/ONNX backends, load HF models, local) with a small embedding model (**all-MiniLM-L6-v2**, 384-dim, ~22M params) or **bge-small-en**. ONNX Runtime path gives 3–5× faster inference / 60–80% less memory than Python equivalents.
- **Perf/cost/caching:** Embeddings are tiny and batchable; compute on meme-mutation events, not per tick. Store vectors with the meme/dialect record; clustering (k-means / DBSCAN over the vector set) runs on a worker at culture-tick cadence. Effectively free at game scale.

### 1.5 Optional ML/RL for agent behavior (vs the existing utility-AI/GOAP backbone)

- **Civis system:** `civ-agents` utility-AI (Needs: food/shelter/safety/belonging) + `civ-tactics` doctrine GA (already SOLID — doctrines evolve via genetic algorithm).
- **Verdict: mostly NOT worth it; keep utility-AI/GOAP as the backbone.** Recommendation by case:
  - **Keep utility-AI/GOAP** for per-agent need-satisfaction. It is cheap, debuggable, designer-legible, and runs at 100k-agent LOD scale — RL cannot match that cost/latency/transparency for routine behavior, and RL's opacity fights the charter's "measured, emergent" legibility goal.
  - **The GA in `civ-tactics` already IS the right ML.** Evolving doctrines via GA over fitness is the sanctioned pattern — extend it (more doctrine genes, online fitness) rather than bolting on deep RL.
  - **Narrow RL only where reward is crisp and offline:** e.g. desire-path/road formation (feature-matrix §4 "Roads — BLIND") can use a lightweight RL/flow-field or simply emergent path-cost reinforcement (cheap, classic) rather than a neural policy. Logistics/maneuver (warfare operational layer) is a candidate for offline-trained policies *if* it ever becomes a bottleneck — train offline (TRL/HF Jobs/Unsloth or a small custom loop), ship a frozen `ort`/ONNX policy for inference. Never train online in the shipping sim.
- **Recommended approach if pursued:** offline training (HF Jobs / local 3090 Ti) → export ONNX → infer via **`ort` (ONNX Runtime)** on a worker. Treat a trained policy as just another `civ-ai` provider behind a trait.
- **Perf/cost:** RL inference per-agent at scale is the danger — gate behind LOD (only Hot-tier agents, and only if utility-AI proves insufficient). Default: **do not adopt**; revisit only with a measured bottleneck.

---

## 2. LOCAL-FIRST INFERENCE STACK (Rust-native preferred)

### 2.1 Runtime choice

| Runtime | Role in Civis | Notes |
|---|---|---|
| **mistral.rs** (`EricLBuehler/mistral.rs`) | **Primary in-process generator.** | Pure-Rust on Candle; format-agnostic (HF, GGUF, UQFF); CPU + CUDA + Metal; easy embed. Best fit for a Rust game that wants in-process, no sidecar. |
| **llama.cpp** via **`llama-cpp-2`/`llama_cpp` bindings** | **Performance fallback / GGUF gold path.** | Fastest measured generation for Mistral-7B Q4 GGUF; the most battle-tested quant kernels. Use if mistral.rs perf/coverage falls short for a given model. |
| **candle** (huggingface/candle) | Underlying tensor lib + **embeddings**. | We get it transitively via mistral.rs/fastembed; also direct for BERT-class embedding models. |
| **`ort` / ONNX Runtime** | **Embeddings + any frozen ML policy.** | 3–5× faster, 60–80% less memory than Python; ideal for MiniLM embeddings and exported RL/ML policies. |
| **Ollama** | **Dev server only.** | Convenient local OpenAI-compatible endpoint for iterating on prompts/models during development; NOT a shipping dependency (avoid the sidecar in the released game). The cloud `FirepassKimiClient` is already OpenAI-compatible, so an Ollama dev provider is a near-zero-cost reuse of the same client. |
| **fastembed-rs / EmbedAnything** | **Embeddings convenience layer.** | Rust, local, loads HF models; wraps Candle/ONNX. Use for §1.4. |

**Decision:** **mistral.rs as the default local generation runtime** (Rust-native, in-process, GGUF + GPU), `ort`/fastembed-rs for embeddings, llama.cpp bindings held as a perf escape hatch, Ollama for dev-loop only, Firepass/Kimi for cloud heavy-reasoning.

### 2.2 SLM picks (sized for 3090 Ti / M1 with the GAME also running)

The game owns most VRAM (voxel meshes, render targets). Budget the SLM to a **small slice** (≈1–3 GB VRAM at Q4). Hence default to **≤1.5B**, step up to 2–3B only for narrative set-pieces.

| Model (HF card) | Params | Role | GGUF quant | Approx VRAM (Q4_K_M) |
|---|---|---|---|---|
| **Qwen2.5-0.5B-Instruct** (`Qwen/Qwen2.5-0.5B-Instruct`) | 0.5B | Naming-grammar seeding, ultra-cheap headlines | Q4_K_M / Q5 | ~0.4–0.6 GB |
| **Qwen2.5-1.5B-Instruct** (`Qwen/Qwen2.5-1.5B-Instruct`) | 1.5B | **Default** legends + headlines | Q4_K_M | ~1.1–1.4 GB |
| **Llama-3.2-1B-Instruct** (`meta-llama/Llama-3.2-1B-Instruct`) | 1B | Alt default; strong instruction following | Q4_K_M | ~0.9 GB |
| **Llama-3.2-3B-Instruct** (`meta-llama/Llama-3.2-3B-Instruct`) | 3B | Leader chatter / diplomacy quality | Q4_K_M | ~2.2–2.6 GB |
| **Gemma-2-2B-it** (`google/gemma-2-2b-it`) | 2B | Narrative fluency, lore prose | Q4_K_M | ~1.8 GB |
| **SmolLM3-3B** (`HuggingFaceTB/SmolLM3-3B`) | 3B | Best-in-class small narrator; beats Llama-3.2-3B/Qwen2.5-3B on 12 benchmarks | Q4_K_M | ~2.2 GB |
| **all-MiniLM-L6-v2** (`sentence-transformers/all-MiniLM-L6-v2`) | ~22M | Embeddings for culture/meme drift (§1.4) | ONNX / fp16 | <0.1 GB |

**Quantization:** GGUF **Q4_K_M** is the default (best size/quality trade for ≤3B); **Q5_K_M** when a feature is quality-sensitive and headroom exists; avoid below Q4 for narrative coherence. **Context budget:** keep prompts tight (epoch digest / persona card ≤ ~1–2k tokens; outputs 60–600 tokens). Small context = low latency and low KV-cache VRAM.

**Latency expectation:** ≤1.5B Q4 on a 3090 Ti generates a headline/paragraph in sub-second; M1 is slower but fine at epoch/event cadence (never per-tick). All generation is async and off the critical path, so latency affects *freshness of flavor*, never frame time.

### 2.3 Cloud heavy-reasoning fallback (Firepass/Kimi)

Already implemented (`FirepassKimiClient`, `kimi-k2.6-turbo`, OpenAI-compatible, `json_object`). Use **only** for: (a) grand end-of-age sagas the player requests; (b) the existing tech-card R&D proposals (`run_research_cycle`); (c) rare summit-grade set-piece dialogue. **Cache + replay-safe:** every cloud call already records a blake3-keyed `LlmEvent`; honor `ReplayMode` (Hybrid/Free require cache hits on replay). Requires user-supplied `KIMI_API_KEY` (per `project_civis_infra_stack` — user supplies keys); absent key → cloud provider unavailable (loud), local provider still serves all in-game flavor.

---

## 3. DEV-ASSIST AI

| Use | Approach | Stack | Notes |
|---|---|---|---|
| **Automated balance / playtesting** | Headless bots drive the sim; an LLM/heuristic "analyst" reads telemetry and flags imbalances (runaway faction, economic collapse, dead-end tech, starvation spirals). Start **heuristic** (statistical anomaly detection over timeseries) — cheap, deterministic-enough, no model — then layer an SLM to *narrate/triage* anomalies into human-readable balance reports. | Headless sim runner + `civ-engine` timeseries (CIV-0103) → heuristic detectors → SLM summary (local Qwen2.5-1.5B or Kimi for depth). | Run as a nightly/CI batch job, not in-game. Highest dev-leverage AI use. |
| **Procgen tuning** | Treat worldgen/economy/drift knobs as a parameter search; score generated worlds against target metrics (biome variety, settleability, economic liveliness). Optimize with a GA / Bayesian search (CMA-ES, optuna-style); LLM only to *propose* knob hypotheses from failure reports. | Offline search loop + scoring harness; reuse the GA pattern already proven in `civ-tactics`. | Mostly classic optimization; LLM is advisory. |
| **Telemetry / log analysis** | Embed log lines / events → cluster → surface novel failure signatures; SLM explains clusters. | fastembed-rs + clustering + SLM summary. Reuses the §1.4 embedding stack. | Doubles as crash-pattern triage. |
| **Test generation** | LLM proposes property-test cases / edge inputs for `civ-laws` validators, conservation invariants, speciation thresholds; humans/agents review before commit. | Kimi/cloud for breadth; output is *reviewed*, never auto-merged (per `CLAUDE.md` quality gates, ≥90% coverage, no auto-merge on red). | Pairs with existing `proptest` dev-dep. |

All dev-assist runs **offline/CI**, never in the shipping sim, so model size/cost constraints relax (cloud Kimi is fine here).

---

## 4. ARCHITECTURE: the `civ-ai` crate

**Thesis:** generalize the existing `civ-research` port. `civ-research::LlmClient` is already the right shape (async, `Send + Sync`, returns a typed result, has a cache + event log + replay rules). Extract the *generic* provider/cache/event machinery into a new **`crates/ai`** (`civ-ai`) crate; `civ-research` becomes a *consumer* that adds the tech-card-specific prompt + `civ-laws` validation on top. This is "extend, never duplicate" (`CLAUDE.md`) and an org-reuse candidate (the provider+cache+event-log substrate is sharable across Phenotype repos).

### 4.1 Core trait (sketch — pseudocode, not implementation)

```
// crates/ai — generic AI provider port (generalizes civ-research::LlmClient)
trait AiProvider: Send + Sync {
    // Free-form text generation for flavor (legends, chatter, headlines, lore).
    async fn generate(&self, req: &GenRequest) -> Result<GenOutput, AiError>;
    // Embeddings for drift/speciation + log analysis.
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AiError>;
    fn model_id(&self) -> &str;       // for LlmEvent provenance
    fn model_version(&self) -> &str;
}

// GenRequest carries: prompt, max_tokens, optional json schema, snapshot/persona hash.
// Implementations:
//   LocalSlmProvider   — mistral.rs / llama.cpp, GGUF, in-process (DEFAULT in-game)
//   OllamaDevProvider  — dev-only, OpenAI-compatible (reuses the cloud HTTP client)
//   FirepassKimiProvider — existing cloud impl, heavy reasoning fallback
//   EmbedProvider      — fastembed-rs / ort for embed()
//   DummyAiProvider    — deterministic, for tests (mirror existing DummyLlmClient)
```

### 4.2 Caching + replay-safe (reuse, do not reinvent)

- Reuse `ResearchCache` (blake3 hash-keyed) and `LlmEvent` (`cache_key = prompt_hash ++ input_snapshot_hash ++ model_id ++ model_version`). Generalize key to cover `(persona_hash, event_hash)` and `(epoch_digest_hash)`.
- Determinism not required, but the cache is mandatory for cost/latency; the `ReplayMode` (Canonical/Hybrid/Free) machinery already in `civ-research` carries over unchanged for any feature that opts into the event log. Cosmetic flavor (legends prose) need not be replay-gated; advisory/sim-affecting AI (if any) records `LlmEvent`s.

### 4.3 Async / multi-tick scheduler (never blocks the sim)

- A dedicated **AI worker pool** (tokio runtime on its own threads, off the sim/render threads). The sim enqueues `AiTask`s (epoch digest, persona+event, embed batch) onto a bounded channel; workers run them and post results back via a channel the sim drains at a safe point (e.g., end-of-tick). The sim **never awaits** a task — flavor attaches whenever it's ready (this turn, three turns later, or after a reload from cache).
- **Backpressure:** bounded queue + hard cap on concurrent generations; over-budget tasks are dropped or coalesced (newest-wins per entity), with a logged warning. LOD-gate generation to near-camera / notification-relevant entities.

### 4.4 Failure behavior (loud, per `CLAUDE.md`)

- **Preflight:** on startup, `civ-ai` verifies required model artifacts exist (path from `.env` / config). Missing required model → **named, loud failure** (`civ-ai preflight failed: missing model <name> at <path>; run Tools/fetch-models.ps1`). No silent fallback to "AI off".
- **Cosmetic features** (legends, chatter): if the *optional* high-quality model is absent but a base model is present, log a visible warning and degrade to the smaller model or a templated string — but the degrade is *announced*, not hidden.
- **Cloud:** missing `KIMI_API_KEY` → cloud provider reports unavailable (loud at the call site that requested it); local providers continue serving all in-game flavor. Never make a required dependency "optional" to dodge a failure.

### 4.5 Config / secrets

- All model paths, provider selection, token budgets, concurrency caps via `.env` + committed `.env.example` (per `feedback_secrets_config` / `CLAUDE.md` — never hardcode). `KIMI_API_KEY` / `FIREPASS_BASE_URL` already in `.env.example`. Add `CIVAI_LOCAL_MODEL_PATH`, `CIVAI_NARRATOR_MODEL`, `CIVAI_EMBED_MODEL`, `CIVAI_MAX_CONCURRENT_GEN`, `CIVAI_ENABLE_CLOUD`.

### 4.6 Cross-project reuse opportunity

The generic `AiProvider` + hash-keyed cache + async worker pool + `LlmEvent` provenance is a **Phenotype-org shared substrate** (candidate shared crate, e.g. `phenotype-ai`), reusable by sibling projects that need local-SLM-first generation with a cloud fallback. Flagged per the Cross-Project Reuse Protocol; destination/rollout to be confirmed with the user before extraction.

---

## 5. ADOPT-NOW TOP-5 AI FEATURES (highest game-impact / lowest cost)

Ranked by impact ÷ cost. All are off-hot-path, cached, and local-first.

| # | Feature | Why now (impact) | Cost | System / §ref |
|---|---|---|---|---|
| **1** | **Procedural naming (grammar + Markov, SLM-seeded grammars)** | Turns anonymous agents/places/cultures/factions into a *named, legible world* — the single biggest "this feels alive" win. Fills a BLIND-adjacent gap across §2/§4 of the matrix. | **~Zero runtime** (grammar runs inline in Rust; SLM called once per culture, batch). | §1.2 → `civ-species`/`civ-agents`/clusters |
| **2** | **Emergent-history / legends narration (SLM over event-log digests)** | Directly fills the gold-standard DF "legends" gap (matrix §2 BLIND). Converts the existing event log into the chronicle that makes a zero-player sim *readable and shareable*. | **Low** — one ≤1.5B Q4 call per epoch, digest-hash cached. | §1.1 → event log + `civ-engine` |
| **3** | **Cultural/language drift via embeddings (speciation of memes/dialects)** | Makes the "ideology/culture/language drift — INCOMPLETE" gap *measurable* using the same speciation-threshold pattern already blessed for genomics. Pure measurement, on-charter. | **Very low** — tiny embedding model, ONNX, batched on events. | §1.4 → `civ-engine` ideology |
| **4** | **Leader/faction chatter + news headlines (fixed-persona SLM)** | Surfaces the emergent psyche/diplomacy layers (BLIND/INCOMPLETE) as a notification feed + headlines — high perceived life, validated by arXiv 2511.10277. | **Low-med** — event-triggered, rate-limited, persona+event cached, LOD-gated. | §1.3 → clusters + diplomacy (CIV-0105) |
| **5** | **Dev-assist balance analyst (heuristic + SLM triage)** | Multiplies *every other system's* iteration speed by auto-finding imbalances from headless playtests. Pure dev win, no in-game cost. | **Low** — offline/CI batch; heuristics first, SLM for narration; cloud OK. | §3 → headless sim + timeseries |

**Explicitly deferred:** deep RL for routine agent behavior (§1.5 — keep utility-AI/GOAP; extend the `civ-tactics` GA instead). Cloud generation as a *default* path (keep it fallback-only).

---

## 6. Recommended local SLM (headline pick)

**Default in-game narrator/namer: `Qwen2.5-1.5B-Instruct`, GGUF Q4_K_M, served by mistral.rs.** Rationale: best quality-per-VRAM at ~1.1–1.4 GB (leaves the 3090 Ti's VRAM to the game), strong instruction-following and summarization, runs fine on M1 at epoch/event cadence. Step-up to **SmolLM3-3B** or **Gemma-2-2B-it** (Q4_K_M) only for narrative set-pieces when headroom exists; **Qwen2.5-0.5B** for batch naming-grammar seeding; **all-MiniLM-L6-v2** (ONNX via `ort`/fastembed-rs) for the embedding/drift features. Cloud `kimi-k2.6-turbo` via the existing `FirepassKimiClient` stays the heavy-reasoning fallback only.

---

## 7. Phased WBS (DAG)

| Phase | Task ID | Description | Depends On |
|---|---|---|---|
| P1 Foundation | A1 | Extract generic `AiProvider`+cache+`LlmEvent` from `civ-research` into `crates/ai` (`civ-ai`); `civ-research` consumes it | — |
| P1 | A2 | `LocalSlmProvider` (mistral.rs, GGUF Q4_K_M) + preflight model check + `.env` config | A1 |
| P1 | A3 | Async AI worker pool + bounded queue + result channel (never blocks sim) | A1 |
| P1 | A4 | `EmbedProvider` (fastembed-rs/`ort`, MiniLM) | A1 |
| P2 Top-5 build | B1 | Naming grammar + Markov; SLM grammar-seeding (top-5 #1) | A2 |
| P2 | B2 | Event-log epoch-digest aggregator + legends narration (top-5 #2) | A2, A3 |
| P2 | B3 | Meme/dialect embedding + drift/speciation clustering (top-5 #3) | A4 |
| P2 | B4 | Fixed-persona chatter + headlines, LOD/rate-gated (top-5 #4) | A2, A3 |
| P3 Dev-assist | C1 | Headless balance analyst (heuristics → SLM triage) (top-5 #5) | A2 |
| P3 | C2 | Telemetry/log embedding+cluster triage | A4 |
| P4 Optional | D1 | Offline RL/ML policy export → `ort` provider (only if a measured bottleneck) | A1 |

Aggressive agent-effort estimate: P1 ≈ 3–5 parallel subagents / ~15–20 min wall; each P2 feature ≈ 2–3 subagents / ~5–10 min wall; P3 ≈ 1–2 subagents each.

---

## 8. Sources

- mistral.rs — fast, flexible Rust LLM inference (GGUF/UQFF/HF, CPU+CUDA+Metal): https://github.com/EricLBuehler/mistral.rs
- huggingface/candle — minimalist Rust ML framework (tensor backend, embeddings): https://github.com/huggingface/candle
- Rust LLM inference comparison (llama.cpp fastest Q4 GGUF; Candle close; format/GGUF support notes): https://medium.com/@zaiinn440/apple-mlx-vs-llama-cpp-vs-hugging-face-candle-rust-for-lightning-fast-llms-locally-5447f6e9255a
- Building LLM apps in Rust (candle + llm crates, GGUF/GGMLv3 state 2026): https://dasroot.net/posts/2026/01/building-llm-applications-rust-candle-llm-crates/
- fastembed-rs — local Rust embeddings + reranking: https://github.com/Anush008/fastembed-rs
- EmbedAnything — Rust embeddings (Candle + ONNX backends): https://github.com/StarlightSearch/EmbedAnything
- sentence-transformers in Rust (Burn/ONNX/Candle; ONNX 3–5× faster, 60–80% less mem): https://dev.to/mayu2008/building-sentence-transformers-in-rust-a-practical-guide-with-burn-onnx-runtime-and-candle-281k
- "Fixed-Persona SLMs with Modular Memory: Scalable NPC Dialogue on Consumer Hardware" (arXiv 2511.10277): https://arxiv.org/pdf/2511.10277
- Procedural Content Generation in Games — survey w/ LLM integration (hybrid grammar/symbolic + LLM): https://arxiv.org/html/2410.15644v1
- Survey on LLM-Based Game Agents (ACM CSUR): https://github.com/git-disl/awesome-LLM-game-agent-papers
- Small Language Model leaderboard (≤10B; Q4 8GB-RAM viability): https://awesomeagents.ai/leaderboards/small-language-model-leaderboard/
- Best open-source SLMs 2026 (Phi/SmolLM3/Qwen2.5/Llama-3.2/Gemma-2 comparisons): https://www.bentoml.com/blog/the-best-open-source-small-language-models
- Model cards (HF): `Qwen/Qwen2.5-0.5B-Instruct`, `Qwen/Qwen2.5-1.5B-Instruct`, `meta-llama/Llama-3.2-1B-Instruct`, `meta-llama/Llama-3.2-3B-Instruct`, `google/gemma-2-2b-it`, `HuggingFaceTB/SmolLM3-3B`, `sentence-transformers/all-MiniLM-L6-v2`.
