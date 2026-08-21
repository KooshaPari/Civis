# ai

> Generic AI provider port with blake3 cache, event provenance tracking, and async worker pool.

## Overview

The `ai` crate is a domain-agnostic AI substrate that provides a pluggable provider interface for text generation, embedding, and goal-directed reasoning. It knows nothing about cultures, epochs, or tech cards — it is pure infrastructure consumed by higher-level crates.

Requests flow through an `AiWorkerPool` that manages concurrency, retries, and backpressure. An `AiCache` backed by blake3 hashing deduplicates identical requests and stores results with full provenance metadata so every generated output can be traced back to its inputs.

The provider trait is implemented by multiple backends ranging from a deterministic dummy provider for testing to real SLM and third-party API integrations.

## Features

- `AiProvider` trait for pluggable LLM/embedding backends
- blake3-keyed `AiCache` with provenance metadata
- `AiWorkerPool` for async concurrency, retries, and backpressure
- Request/response typing for generation and embedding flows
- Provider registry for runtime backend selection
- Domain-agnostic — no simulation-specific knowledge

## Usage

```rust
use ai::*;
```

## Architecture

- **AiProvider** — Trait defining `generate`, `embed`, and `health_check` methods
- **AiCache** — Blake3-keyed content-addressable cache with event provenance
- **AiWorkerPool** — Async task pool managing provider concurrency and rate limits
- **GenRequest / EmbedRequest** — Typed request envelopes carrying prompt, parameters, and metadata
- **Goal** — Structured objective descriptor for goal-directed generation
- **AiConfig** — Provider configuration (timeouts, retries, model selection)
- **ProviderRegistry** — Runtime map from provider names to instantiated backends

Providers ship as separate structs: `DummyAiProvider` (deterministic test), `FirepassKimiProvider` (third-party API), and `LocalSlmProvider` (on-device small language model).

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
