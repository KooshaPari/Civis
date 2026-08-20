# research

> R&D proposal validator + replay-safe cache.

## Overview

The `research` crate acts as the gatekeeper for technological advancement in Civis. It validates "TechCards" (LLM-proposed technology developments) against a versioned `LawDb` (physics/logic database) to ensure consistency and prevent paradoxical advancements. It includes a hash-keyed cache for replay safety and deterministic results.

It optionally integrates with the FirepassKimi LLM client for proposing new cards, but its primary role is validation and caching. This ensures that the simulation remains consistent even when driven by external, potentially hallucinating agents.

## Features

- Typed `TechCard` validation against `LawDb`
- Hash-keyed replay-safe cache
- Rejection reasoning via `RejectReason` enum
- Optional FirepassKimi LLM client integration
- Deterministic validation outcomes

## Usage

```rust
use research::*;
```

## Architecture

The crate is centered around the `ValidationOutcome` type, which is produced by validating a `TechCard`. The cache maps input hashes to these outcomes, preventing redundant computations. The `LawDb` provides the ground truth against which all cards are measured.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
