# Mod Platform + AI Agent Bridge SOTA

Training-only synthesis, no live verification.

## What DINOForge already does well

- `43-tool` MCP surface is already large enough to expose real capability, not just a toy demo.
- HMAC-signed `BridgeReceipts` with per-session keys and a world-frame replay counter is stronger than most mod bridges on integrity and anti-replay.
- Named-pipe JSON-RPC with NDJSON framing is a good fit for low-latency local agent control.
- Pack-based YAML content and a declarative ECS bridge are the right split: data in packs, behavior in code.

## What to adopt

### Mod platforms

- **BepInEx 6**: adopt the trajectory, not the alpha label. The value is cleaner plugin boundaries, more modern .NET alignment, and a future-facing path for Unity modding. Keep 5.4 as the stable baseline until 6 is operationally boring.
- **MelonLoader**: adopt only the fast bootstrap and mod UX lessons. Do not copy its tendency toward opaque runtime magic if it weakens bridge contracts.
- **Forge / NeoForge / Fabric**: adopt lifecycle hooks, registry-first design, and datagen-style separation. Their best idea is “content declares itself through registries and events,” not “patch everything.”
- **SMAPI**: adopt the event bus pattern. It is the cleanest reference for mod extensibility without forcing every extension through patching.
- **Thunderstore**: adopt package metadata, version pinning, profile-based deployment, and dependency manifests. Distribution ergonomics matter as much as runtime architecture.

### AI-agent bridges

- **Voyager**: adopt planner/executor separation, skill memory, and observational summaries. The main lesson is not GPT-4 specifically; it is breaking long-horizon play into stable sub-policies.
- **MineDojo / MineRL**: adopt structured observation/action vocabularies and replayable trajectories. Good agents need a typed interface to the world.
- **Anthropic computer-use**: adopt small, explicit tools, action confirmation, and “observe -> plan -> act -> verify” loops. Prefer narrowly scoped tools over one giant browser/game driver.
- **LangGraph**: adopt graph-based orchestration for multi-step agent state, retries, checkpoints, and branching recovery.
- **SWE-agent / OpenDevin**: adopt file/diff/command tool patterns, especially the habit of making edits auditable and separable from reasoning.

### MCP ecosystem

- Adopt streaming and push events where state changes are meaningful.
- Adopt capability scoping so high-tool-count servers expose fewer tools per task.
- Adopt tool namespaces, resource handles, and schema-typed tool outputs.
- For 43 tools, prefer a few high-level composite tools plus a smaller set of primitive tools that remain stable.

### Unity tooling

- **Unity SOTA**: adopt editor/runtime split, typed inspection data, and locator-like query semantics.
- **Playwright/VHS patterns**: adopt `wait/expect/actionability` semantics for game UI and bridge calls. Deterministic waits beat polling spaghetti.

## What to drop

- Drop “one tool per tiny action” proliferation unless the tool adds a distinct capability.
- Drop implicit, untyped bridge responses. Every agent-facing payload should be versioned and schema-backed.
- Drop patch-only mod architecture. Patching is a mechanism, not a platform.
- Drop single-shot agent loops that cannot recover from partial failure.
- Drop hidden side effects in tool calls; agents need auditable, replayable actions.

## Bottom line

The winning pattern is: **event bus + typed observations + capability-scoped tools + auditable actions + replayable state**. DINOForge is already close on transport, integrity, and data modeling. The main upgrades to steal from the field are mod-platform event semantics, agent planner/executor structure, and MCP scoping/streaming discipline.
