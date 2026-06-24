# `civ-ai` Crate Design — Provider-Trait + 5 SLM Features Architecture

**Status:** Design (docs-only, PLANNER stance — architecture / trait-sketch / pseudocode, **no implementation**).
**Owner:** AI/ML R&D Lead.
**Governing inputs (read-only):**
- [`docs/research/ai-rnd.md`](../research/ai-rnd.md) — the R&D plan this design realizes (§4 architecture, §5 top-5).
- `crates/research/src/lib.rs` — `LlmClient`, `ResearchCache`, `LlmEvent`, `ReplayMode`, `replay_advance_llm_event`, `DummyLlmClient`, `validate`, `run_research_cycle` (the template to **generalize, not duplicate**).
- `crates/research/src/firepass.rs` — `FirepassKimiClient` (cloud, OpenAI-compatible, `KIMI_API_KEY` / `FIREPASS_BASE_URL`).
**Governing constraints:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) (AI is flavor/advisory over the emergent substrate, never authority); `CLAUDE.md` "Optionality and failure behavior" (loud failure), "Extend, Never Duplicate", "local OSS/free only". Determinism **not** required (charter §"Determinism is NOT a requirement").

> **Planner note.** All Rust below is *trait sketch / pseudocode* to communicate shape and contracts to implementer agents. It is intentionally non-compiling, signature-level only, per `CLAUDE.md` "Planner agents: no code in docs or plans". Implementer agents own real code.

---

## 1. Thesis & scope

`crates/research::LlmClient` is **already the right port** — async, `Send + Sync`, typed result, paired with a blake3 hash-keyed `ResearchCache`, an `LlmEvent` provenance record, and `ReplayMode` rules. It is only *too specific*: it returns `TechCard` and exposes a single `propose_tech_card` method.

**This design extracts the generic provider / cache / event / worker-pool machinery into a new `crates/ai` (`civ-ai`) crate.** `civ-research` becomes a **consumer** of `civ-ai` that adds the tech-card prompt + `civ-laws` validation on top. No new abstraction is invented; the existing one is widened. This satisfies "Extend, Never Duplicate" and is a flagged Phenotype-org reuse candidate (§9).

**In scope:** the `AiProvider` trait (`generate` + `embed`), the five concrete providers, the cache + `LlmEvent` provenance reuse, the tokio worker-pool so the sim never awaits a token, loud-failure preflight, `.env` config, and the five adopt-now features each wired as an async/cached/off-hot-path service.

**Out of scope (design only):** implementation, model weights fetching, fine-tuning, deep-RL (deferred per research §1.5).

---

## 2. Requirements (FR-CIV-AI-*)

Traceability anchors for AgilePlus/Tracera (requirement → code → test → PR). Each maps to research §.

| ID | Requirement | Source § | Acceptance signal |
|---|---|---|---|
| **FR-CIV-AI-001** | A generic `AiProvider` port with `generate` + `embed`, `Send + Sync`, async, `model_id`/`model_version` for provenance. | §4.1 | `civ-research::LlmClient` reframed as a consumer of this port; trait object usable behind `Arc`. |
| **FR-CIV-AI-002** | `LocalSlmProvider` (mistral.rs, GGUF Q4_K_M) as the **default in-game** generator; in-process, no sidecar. | §2.1, §6 | Generates a legends paragraph from a digest; honors model path from `.env`. |
| **FR-CIV-AI-003** | `OllamaDevProvider` — dev-only, OpenAI-compatible, reuses the cloud HTTP client; **not** a shipping dependency. | §2.1 | Compiled behind a `dev`/feature flag; never selected in release config. |
| **FR-CIV-AI-004** | `FirepassKimiProvider` — wrap existing `FirepassKimiClient` as an `AiProvider`; cloud heavy-reasoning **fallback only**. | §2.3 | Behind `CIVAI_ENABLE_CLOUD`; missing `KIMI_API_KEY` → loud unavailable at call site. |
| **FR-CIV-AI-005** | `EmbedProvider` — fastembed-rs / `ort` (MiniLM, 384-dim) for `embed`; generation unsupported (loud error). | §1.4, §2.1 | Returns batched vectors; `generate` returns `Unsupported`. |
| **FR-CIV-AI-006** | `DummyAiProvider` — deterministic, test-only; mirrors `DummyLlmClient`. | §4.1 | Stable output for the same input; used by all feature unit tests. |
| **FR-CIV-AI-007** | Reuse blake3 hash-keyed cache + `AiEvent` provenance (generalized `LlmEvent`); `ReplayMode` carried unchanged. | §4.2 | Repeated identical request is a cache hit; `AiEvent.cache_key()` composite holds. |
| **FR-CIV-AI-008** | Async worker pool (dedicated tokio runtime off sim/render threads); sim **never awaits**; bounded queue + backpressure + LOD/coalesce. | §4.3 | Sim tick advances with no in-flight result; result lands later via channel. |
| **FR-CIV-AI-009** | Loud-failure preflight: required model artifact missing → named startup failure; no silent "AI off". | §4.4 | `civ-ai preflight failed: missing model <name> at <path>`. |
| **FR-CIV-AI-010** | `.env`-driven config: model paths, provider selection, token/concurrency budgets; committed `.env.example`. | §4.5 | No hardcoded paths/keys; `.env.example` lists all `CIVAI_*` keys. |
| **FR-CIV-AI-011** | **Naming** service (grammar+Markov inline; SLM seeds a per-culture grammar once, batch). | §1.2 / top-5 #1 | Zero per-name model calls; grammar cached on the culture record. |
| **FR-CIV-AI-012** | **Legends narration** service (epoch-digest → SLM prose; digest-hash cached). | §1.1 / top-5 #2 | One ≤1.5B call per epoch; unchanged epoch is a cache hit on reload. |
| **FR-CIV-AI-013** | **Culture/meme drift** service (embeddings → cosine drift → speciation threshold). | §1.4 / top-5 #3 | Speciation event fires past threshold; vectors stored on meme record. |
| **FR-CIV-AI-014** | **Chatter/headlines** service (fixed-persona SLM, event-triggered, rate-limited, LOD-gated, `(persona,event)`-cached). | §1.3 / top-5 #4 | Generates only on state-change near camera/feed; hard concurrency cap. |
| **FR-CIV-AI-015** | **Balance analyst** dev-assist (heuristic anomaly detection → SLM triage; offline/CI batch). | §3 / top-5 #5 | Runs headless; emits a human-readable balance report; never in shipping sim. |
| **NFR-CIV-AI-001** | No AI call ever blocks a sim or render tick (frame-time isolation). | §0.3, §4.3 | Worker-pool isolation verified; no `await` on the sim thread. |
| **NFR-CIV-AI-002** | Local-first/OSS/free default; cloud is explicit opt-in only. | §0.1 | Default config selects `LocalSlmProvider`; cloud gated off by default. |
| **NFR-CIV-AI-003** | Caching mandatory for cost/latency even though determinism is optional. | §0.5, §4.2 | Every provider call routes through the cache layer. |

---

## 3. Crate layout

```
crates/ai/                         # civ-ai
├── Cargo.toml                     # deps: tokio, blake3, serde, thiserror; mistral-rs, fastembed (feature-gated); reqwest (cloud/ollama)
├── .env.example                   # CIVAI_* keys (also merged into repo-root .env.example)
└── src/
    ├── lib.rs                     # re-exports; AiProvider trait; AiError; GenRequest/GenOutput; EmbedRequest
    ├── provenance.rs              # AiEvent (generalized LlmEvent) + cache_key(); ReplayMode re-exported/shared
    ├── cache.rs                   # AiCache (generalized ResearchCache, blake3-keyed, value-generic)
    ├── pool.rs                    # AiWorkerPool, AiTask, AiHandle, result channel, backpressure
    ├── preflight.rs               # required-artifact check → loud named failure
    ├── config.rs                  # AiConfig from env (.env); provider selection + budgets
    └── providers/
        ├── local_slm.rs           # LocalSlmProvider (mistral.rs, GGUF)        [feature "local"]
        ├── ollama_dev.rs          # OllamaDevProvider (OpenAI-compatible)       [feature "dev"]
        ├── firepass_kimi.rs       # FirepassKimiProvider (wraps civ-research::FirepassKimiClient) [feature "cloud"]
        ├── embed.rs               # EmbedProvider (fastembed-rs / ort)          [feature "embed"]
        └── dummy.rs               # DummyAiProvider (deterministic, tests)
```

The five **feature services** (naming, legends, drift, chatter, balance) live in their **owning sim crates** (`civ-species`, event-log/`civ-engine`, `civ-engine` ideology, clusters/diplomacy, headless dev tooling) and depend **on** `civ-ai`. `civ-ai` stays domain-agnostic — it knows providers, cache, pool, provenance; it does **not** know about cultures or epochs. (Hexagonal: `civ-ai` = the port + adapters; feature services = application logic.)

`civ-research` migration: delete its bespoke `LlmClient`/`ResearchCache`/`DummyLlmClient`; re-implement `propose_tech_card` as a thin wrapper that calls `AiProvider::generate` with a tech-card prompt and parses the JSON into a `TechCard`, then runs the existing `validate(card, db)`. `LlmEvent` becomes a type alias or newtype over `AiEvent<TechCard>`. Forward-only migration per "Extend, Never Duplicate".

---

## 4. The `AiProvider` trait surface (FR-CIV-AI-001)

### 4.1 Core port (sketch)

```rust
/// Generic AI provider port. Generalizes civ-research::LlmClient.
/// All impls are Arc-shared across the worker pool; methods are pure w.r.t. the cache layer above them.
#[allow(async_fn_in_trait)]
pub trait AiProvider: Send + Sync {
    /// Free-form text generation for flavor (legends, chatter, headlines, lore)
    /// and one-shot batch jobs (naming-grammar seeding). Returns Unsupported for embed-only providers.
    async fn generate(&self, req: &GenRequest) -> Result<GenOutput, AiError>;

    /// Embeddings for drift/speciation (§1.4) and log triage (§3).
    /// Returns Unsupported for generate-only providers.
    async fn embed(&self, req: &EmbedRequest) -> Result<Vec<Vec<f32>>, AiError>;

    /// Stable provider model identifier — flows into AiEvent provenance + cache key.
    fn model_id(&self) -> &str;
    /// Provider model version — flows into AiEvent provenance + cache key.
    fn model_version(&self) -> &str;

    /// Declared capabilities so callers/pool route correctly without a failed round-trip.
    fn capabilities(&self) -> Capabilities; // { generate: bool, embed: bool, cloud: bool }
}

/// Request for generate(). Carries everything needed to form a deterministic cache key.
pub struct GenRequest {
    pub prompt: String,                 // fully-rendered prompt (template + variables)
    pub max_tokens: u32,                // tight budget: 60–600 typical
    pub temperature: f32,               // randomness welcome (determinism not required)
    pub json_schema: Option<String>,    // when JSON output is required (tech cards, structured headlines)
    pub input_snapshot_hash: [u8; 32],  // blake3 of the sim region observed → cache key + provenance
    pub seed: Option<u64>,              // optional; recorded for provenance, not required for replay
}

/// Output of generate().
pub struct GenOutput {
    pub text: String,
    pub output_hash: [u8; 32],          // blake3(text) for AiEvent.output_hash
    pub from_cache: bool,
}

/// Request for embed(). Batched by construction.
pub struct EmbedRequest {
    pub texts: Vec<String>,
    pub input_snapshot_hash: [u8; 32],
}

/// Error taxonomy (generalizes civ-research::LlmError; loud and named).
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("provider unavailable: {0}")] Unavailable(String),     // cloud key missing, server down — LOUD at call site
    #[error("operation unsupported by provider {0}")] Unsupported(String), // embed-only called for generate, etc.
    #[error("rate limited")] RateLimited,
    #[error("invalid response: {0}")] InvalidResponse(String),
    #[error("model artifact missing: {0}")] ModelMissing(String),   // surfaced by preflight
}
```

**Why `generate` + `embed` on one trait (not two traits):** keeps a single registry/pool and a single provenance/cache path; providers that only do one declare the other `Unsupported` and advertise via `capabilities()`. This mirrors the research doc's §4.1 sketch exactly while avoiding a failed round-trip — the pool consults `capabilities()` before dispatch.

### 4.2 The five providers (FR-CIV-AI-002..006)

| Provider | `generate` | `embed` | Selection | Notes |
|---|---|---|---|---|
| `LocalSlmProvider` | ✅ mistral.rs, GGUF Q4_K_M, in-process | ✗ Unsupported | **DEFAULT in-game** | `model_id`=Qwen2.5-1.5B-Instruct default; llama.cpp bindings held as a perf escape hatch behind the same trait. Loads once at preflight. |
| `OllamaDevProvider` | ✅ OpenAI-compatible HTTP | (optional) | dev feature only | Reuses the same `reqwest` chat-completions client shape as `FirepassKimiClient`; **never** in release. |
| `FirepassKimiProvider` | ✅ wraps `FirepassKimiClient` | ✗ | cloud feature, `CIVAI_ENABLE_CLOUD=1` | Heavy-reasoning fallback only (sagas, tech R&D, summit set-pieces). Missing `KIMI_API_KEY` → `AiError::Unavailable` (loud). |
| `EmbedProvider` | ✗ Unsupported | ✅ fastembed-rs / `ort`, MiniLM 384-dim | embed feature | Drives drift (§1.4) + log triage (§3). |
| `DummyAiProvider` | ✅ deterministic stub | ✅ fixed vectors | tests only | Mirrors `DummyLlmClient`; enables all feature tests with no weights. |

A small **provider registry** (`HashMap<ProviderKind, Arc<dyn AiProvider>>` built from `AiConfig`) lets feature services request a *role* ("narrator", "embedder", "heavy") rather than a concrete provider — config-driven, per "primitives first / provider interface + registry > N classes".

---

## 5. Cache + provenance reuse (FR-CIV-AI-007 / NFR-CIV-AI-003)

Generalize **without reinventing**:

- **`AiCache`** = `ResearchCache` widened from "value = `TechCard`" to a generic value (`AiCache<V>` or a `Bytes`/`String`-valued cache for prose; `TechCard` becomes one `V`). Same blake3 64-byte key, same `insert`/`get`/`len`/`is_empty` surface.
- **`AiEvent`** = `LlmEvent` generalized over the output type. The composite **cache key is reused verbatim**:
  `cache_key = prompt_hash ‖ input_snapshot_hash ‖ model_id ‖ model_version` (see `LlmEvent::cache_key`). Extended key inputs per feature:
  - legends → `prompt_hash` derived from the **epoch-digest hash** (`FR-CIV-AI-012`);
  - chatter → `prompt_hash` derived from `(persona_hash, event_hash)` (`FR-CIV-AI-014`);
  - naming-grammar → keyed by `(culture_id, language_params_hash)` (`FR-CIV-AI-011`).
- **`ReplayMode` + `replay_advance_*`** carry over unchanged. Cosmetic flavor (legends prose, chatter) need **not** be replay-gated; any advisory/sim-affecting AI records an `AiEvent` and honors `Canonical` (refuse) / `Hybrid`/`Free` (require cache hit) exactly as today. Determinism is not required, but the cache is **mandatory** for cost/latency/reload.

The cache layer wraps every provider: `cached_generate(provider, req)` computes the key, returns on hit, else calls `provider.generate`, stores, and (for opted-in features) appends an `AiEvent`. Providers themselves stay cache-agnostic.

---

## 6. Worker-pool model (FR-CIV-AI-008 / NFR-CIV-AI-001)

**The single most important runtime property: the simulation never `await`s a token.** Flavor attaches whenever it is ready — this tick, three ticks later, or instantly from cache on reload.

```
        SIM THREAD(s)                         AI WORKER POOL (own tokio runtime, own OS threads)
        ─────────────                         ────────────────────────────────────────────────
  end-of-tick:                                 N worker tasks, hard-capped concurrency
   enqueue AiTask ──► bounded mpsc ───────────► dequeue ─► cached_generate / cached_embed
   (NEVER awaits)        (backpressure)              │            │
        ▲                                            │       provider.generate/embed (Arc<dyn AiProvider>)
        │                                            ▼
   drain results ◄──── bounded mpsc (results) ◄── post AiResult { task_id, entity_ref, payload }
   at safe point
   (attach to entity / event log)
```

**Components (sketch):**

```rust
pub struct AiWorkerPool {
    runtime: tokio::runtime::Runtime,    // dedicated runtime, OFF sim/render threads
    task_tx: mpsc::Sender<AiTask>,       // bounded → backpressure
    result_rx: mpsc::Receiver<AiResult>, // sim drains at a safe point (end-of-tick)
    inflight: AtomicUsize,               // hard cap on concurrent generations
}

pub enum AiTask {
    Generate { id: TaskId, who: EntityRef, role: ProviderRole, req: GenRequest, priority: Priority },
    Embed    { id: TaskId, req: EmbedRequest },
}

pub struct AiResult { pub id: TaskId, pub who: EntityRef, pub payload: AiPayload } // payload = Text | Vectors | Err
```

**Discipline (research §4.3):**
1. **Dedicated runtime, off the hot path** — `AiWorkerPool` owns a multi-thread tokio runtime on its own OS threads; the sim/render threads share none of it. `NFR-CIV-AI-001` holds by construction.
2. **Sim enqueues, never blocks** — `try_send` onto the bounded queue; if full, the task is **dropped or coalesced** (see below), never awaited.
3. **Bounded queue + hard concurrency cap** — `CIVAI_MAX_CONCURRENT_GEN` caps in-flight generations (VRAM/latency budget); excess queues.
4. **Backpressure = coalesce, newest-wins per entity** — for chatter/headlines, a newer event for the same cluster supersedes a queued older one (per-entity dedup key); over-budget cosmetic tasks drop with a **logged warning** (visible, not silent).
5. **LOD gating** — only near-camera / notification-relevant entities generate (research §1.3). Embed batches and epoch digests are cadence-gated (per-epoch / on meme-mutation), never per-tick.
6. **Result drain at a safe point** — the sim drains `result_rx` at end-of-tick and attaches payloads to entities / the event log; partial/late results are normal and expected.

**Cadence summary:** naming = inline Rust (no pool) + one batch grammar-seed per culture; legends = one generate per in-game epoch; drift = embed batch on meme-mutation events; chatter = event-triggered + rate-limited + LOD-gated; balance = offline/CI batch (its own headless runner, same pool API).

---

## 7. Loud failure + config (FR-CIV-AI-009/010)

**Preflight (`preflight.rs`).** At startup, `civ-ai` verifies every **required** model artifact named by `AiConfig` exists on disk. Missing → **named, loud failure**, no silent "AI off" (`CLAUDE.md` "Optionality and failure behavior"):

```
civ-ai preflight failed: missing model 'qwen2.5-1.5b-instruct-q4_k_m.gguf'
  at E:/models/qwen2.5-1.5b-instruct-q4_k_m.gguf; run Tools/fetch-models.ps1
```

Multiple missing artifacts are listed semicolon-separated (matches the repo's "named items" failure style). **Degrade rules:**
- *Required* (sim-advisory) providers missing → fail preflight, hard.
- *Cosmetic optional upgrade* missing (e.g. SmolLM3-3B narrator absent but Qwen2.5-1.5B present) → **visible logged warning** + announced degrade to the smaller/base model or a **templated fallback string**, never a hidden silent drop.
- *Cloud* missing `KIMI_API_KEY` with `CIVAI_ENABLE_CLOUD=1` → `AiError::Unavailable` **at the call site that requested cloud**; local providers keep serving all in-game flavor.

**Config (`config.rs`, FR-CIV-AI-010).** All paths/budgets/selection from `.env` + committed `.env.example` (never hardcode — `feedback_secrets_config`). Keys:

| Key | Purpose | Default |
|---|---|---|
| `CIVAI_LOCAL_MODEL_PATH` | GGUF path for `LocalSlmProvider` | (required if local enabled) |
| `CIVAI_NARRATOR_MODEL` | model id for legends/chatter role | `qwen2.5-1.5b-instruct` |
| `CIVAI_EMBED_MODEL` | embedding model id | `all-MiniLM-L6-v2` |
| `CIVAI_MAX_CONCURRENT_GEN` | hard cap on in-flight generations | `2` |
| `CIVAI_GEN_TOKEN_BUDGET` | default `max_tokens` ceiling | `600` |
| `CIVAI_ENABLE_CLOUD` | opt-in cloud fallback | `0` |
| `KIMI_API_KEY` / `FIREPASS_BASE_URL` | cloud creds (already in `.env.example`) | — |
| `CIVAI_OLLAMA_URL` | dev provider endpoint | `http://localhost:11434` (dev only) |

---

## 8. The five adopt-now features (each async, cached, off-hot-path)

Each is a **service in its owning sim crate** consuming `civ-ai` (the port). None lives in `civ-ai` itself. All route through `cached_generate`/`cached_embed` + the worker pool; none touches a sim/render tick.

| # | Feature (FR) | Owning crate | Provider role | Trigger / cadence | Cache key | Failure behavior |
|---|---|---|---|---|---|---|
| 1 | **Naming** (FR-CIV-AI-011) | `civ-species` / `civ-agents` | heavy/narrator, **batch once per culture** | culture-birth (grammar seed); names minted **inline in Rust**, zero model calls | `(culture_id, language_params_hash)` | grammar-seed failure → templated phoneme grammar fallback (announced) |
| 2 | **Legends narration** (FR-CIV-AI-012) | event-log / `civ-engine` | narrator (≤1.5B) | once per in-game **epoch**; off hot path | `epoch_digest_hash` | missing model → templated chronicle line + warning |
| 3 | **Culture/meme drift** (FR-CIV-AI-013) | `civ-engine` ideology | **embedder** | on meme-mutation events; cluster at culture-tick cadence | text/meme hash → vector | embed provider required for the feature → loud if missing |
| 4 | **Chatter / headlines** (FR-CIV-AI-014) | clusters + diplomacy (CIV-0105) | narrator/chatter | **event-triggered**, rate-limited, LOD-gated near camera/feed | `(persona_hash, event_hash)` | over-budget → coalesce/drop with warning; missing model → templated headline |
| 5 | **Balance analyst** (FR-CIV-AI-015) | headless dev tooling | heuristic-first → narrator/heavy (cloud OK) | **offline / CI batch**, never in shipping sim | `(telemetry_window_hash)` | dev-only; cloud `Unavailable` falls back to heuristic-only report |

**Design invariants across all five (research §0):**
- **Flavor/advisory, never authority** — AI re-describes outcomes the sim already produced; it never writes a hardcoded enum, never bypasses `civ-laws::validate`, never replaces physics/genetics.
- **Map-reduce summarization, not generation-from-nothing** — legends/balance summarize a structured digest (bounded hallucination); naming uses **grammar+Markov first**, SLM only to seed a grammar once (the worst place for per-item LLM calls is avoided).
- **Embeddings, not generation, for drift** — meme/dialect speciation by cosine threshold, directly analogous to the genomic Hamming speciation already blessed by the charter.

**Explicitly deferred** (research §1.5 / §5): deep RL for routine agent behavior (keep utility-AI/GOAP; extend the `civ-tactics` GA). If ever pursued, a frozen ONNX policy plugs in as **just another `AiProvider`** behind this same trait (`ort` backend) — no new abstraction.

---

## 9. Phenotype-AI reuse note (Cross-Project Reuse Protocol)

The generic substrate in `civ-ai` — **`AiProvider` trait + provider registry + blake3 hash-keyed `AiCache` + `AiEvent` provenance + the async worker pool + loud preflight + `.env` config** — is **domain-agnostic** and a strong **Phenotype-org shared-crate candidate** (`phenotype-ai`). It carries zero Civis concepts (no culture/epoch/tech-card types); the five feature services and `civ-research` are the only Civis-specific consumers.

**Recommendation:** ship `civ-ai` inside this repo first (forward-only extraction from `civ-research`), prove it across the five features, then propose promotion to a shared `phenotype-ai` crate for sibling repos that need local-SLM-first generation with a cloud fallback. **Per the Cross-Project Reuse Protocol, the cross-repo extraction destination and rollout require user confirmation before execution** — flagged here, not auto-executed. Migration order when greenlit: (1) extract `civ-ai` from `civ-research` in-repo; (2) wire the 5 features; (3) lift the provider/cache/pool/provenance core to `phenotype-ai`, leaving Civis feature services behind; (4) update sibling repos to consume.

---

## 10. Phased WBS (DAG)

| Phase | Task ID | Description | Depends On | Effort (aggressive) |
|---|---|---|---|---|
| P1 Foundation | A1 | Extract `AiProvider` + `AiCache` + `AiEvent` + `ReplayMode` from `civ-research` into `crates/ai`; `civ-research` consumes it | — | 3–5 parallel subagents / ~15–20 min |
| P1 | A2 | `LocalSlmProvider` (mistral.rs GGUF) + preflight model check + `.env` config | A1 | 2–3 subagents / ~8 min |
| P1 | A3 | `AiWorkerPool` + bounded queue + result channel + backpressure/coalesce | A1 | 2–3 subagents / ~8 min |
| P1 | A4 | `EmbedProvider` (fastembed-rs/`ort`, MiniLM) | A1 | 2 subagents / ~5 min |
| P1 | A5 | `OllamaDevProvider`, `FirepassKimiProvider` wrap, `DummyAiProvider` + registry | A1 | 2 subagents / ~6 min |
| P2 Features | B1 | Naming grammar+Markov + SLM grammar-seed (FR-011) | A2, A5 | 2–3 subagents / ~6 min |
| P2 | B2 | Epoch-digest aggregator + legends narration (FR-012) | A2, A3 | 2–3 subagents / ~8 min |
| P2 | B3 | Meme/dialect embed + drift/speciation clustering (FR-013) | A4 | 2 subagents / ~6 min |
| P2 | B4 | Fixed-persona chatter + headlines, LOD/rate-gated (FR-014) | A2, A3 | 2–3 subagents / ~8 min |
| P3 Dev-assist | C1 | Headless balance analyst (heuristics → SLM triage) (FR-015) | A2 | 1–2 subagents / ~6 min |
| P4 Optional | D1 | Offline RL/ML policy export → `ort` provider (only on a measured bottleneck) | A1 | deferred |
| P5 Reuse | E1 | Lift core to `phenotype-ai` (after user confirmation) | A1–A5 | gated on user |

---

## 11. Sources

Inherits the source set in [`docs/research/ai-rnd.md`](../research/ai-rnd.md) §8 (mistral.rs, candle, fastembed-rs, `ort`, arXiv 2511.10277 fixed-persona SLMs, PCG+LLM survey, SLM leaderboards, HF model cards). No new external sources introduced by this design.
