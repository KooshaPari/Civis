# Journey Traceability

Implements the [phenotype-infra journey-traceability standard](https://github.com/kooshapari/phenotype-infra/blob/main/docs/governance/journey-traceability-standard.md) for the DINOForge mod platform workspace.

> **Note:** This file lives at `docs/journeys/traceability.md` (next to `manifests/`) to avoid clobber churn with tooling that targets `docs/operations/journey-traceability.md`.

## Traceability Model

Every developer-facing or operator-facing flow in DINOForge should be traceable across:

1. **FR/NFR** — requirement ID and user story.
2. **Spec** — acceptance criteria and non-regression constraints (including the .NET 11 preview / BepInEx Mono CLR constraints in `CLAUDE.md`).
3. **Docs** — operator/user documentation and rich media placeholders.
4. **Code** — CLI tool, SDK, plugin, content loader, validator, or pack surface implementing the flow.
5. **Tests/Gates** — unit, integration, BDD, lint, and journey verification acting as autograders.
6. **Evidence** — journey manifest, recording/keyframes, and evaluation verdict.

## Developer-Facing and Operator-Facing Flows

| Flow | Requirement | Implementation surface | Autograder gates | Evidence status |
| --- | --- | --- | --- | --- |
| Operator runs `task` build and verifies deterministic task DAG | FR-DINO-BUILD-001, NFR-DINO-REPRODUCIBILITY-001 | `Taskfile.yml`, build helper | task fixture tests, dry-run checks, BDD journey, eval verdict | Stubbed |
| Mod author validates a domain plugin against SDK schemas | FR-DINO-SDK-001, NFR-DINO-CONTRACT-001 | SDK validators, registries, `ContentLoader` | schema negative tests, fixture manifests, journey manifest | Stubbed |
| Mod author compiles a pack via PackCompiler | FR-DINO-PACK-001, NFR-DINO-CONTRACT-001 | `PackCompiler` (net11.0), pack schema | pack fixture tests, roundtrip tests, eval verdict | Stubbed |
| Mod author exercises runtime/plugin bridge end-to-end | FR-DINO-RUNTIME-001, NFR-DINO-STABILITY-001 | `DINOForge.Runtime` (netstandard2.0), BepInEx plugin | runtime fixture tests, plugin isolation tests, BDD journey | Stubbed |
| Operator queries/invokes platform tools via MCP server | FR-DINO-MCP-001, NFR-DINO-OPERABILITY-001 | `McpServer` (net11.0), tool surface | MCP contract tests, tool smoke tests, journey manifest | Stubbed |

## Rich Media Stubs

<!-- RICH-MEDIA-STUB type="animated-gif" subject="Task DAG build and dry-run" journey="dino-task-build-dry-run" status="TODO" -->
![DINOForge task build — task list, dry-run graph, build output, and evidence log](../assets/rich-media/dino/task-build-dry-run.gif)

*Expected capture: run `task --list`, perform a deterministic task dry-run, execute a fixture build, and capture evidence links for the build DAG.*

<!-- RICH-MEDIA-STUB type="annotated-screenshot" subject="Domain plugin schema validation" journey="dino-domain-plugin-validation" status="TODO" -->
![DINOForge plugin validation — fixture manifest, validator output, error report, and SDK link](../assets/rich-media/dino/domain-plugin-validation.png)

*Expected capture: validate a deterministic plugin manifest, show valid/invalid cases, and link each report back to the SDK validator used.*

<!-- RICH-MEDIA-STUB type="journey-eval" subject="Pack compile roundtrip verdict" journey="dino-pack-compile-roundtrip" status="TODO" -->
![DINOForge pack compile — input pack, compiled output, roundtrip diff, and eval verdict](../assets/rich-media/dino/pack-compile-roundtrip.png)

*Expected capture: run PackCompiler against a fixture pack, roundtrip the output, and attach a pass/fail verdict for FR-DINO-PACK-001 and NFR-DINO-CONTRACT-001.*

<!-- RICH-MEDIA-STUB type="journey-eval" subject="Runtime plugin bridge verdict" journey="dino-runtime-plugin-bridge" status="TODO" -->
![DINOForge runtime — plugin loaded, BepInEx lifecycle, bridge call, and eval verdict](../assets/rich-media/dino/runtime-plugin-bridge.png)

*Expected capture: load a fixture plugin into the runtime, exercise a bridge call, and attach a pass/fail verdict honoring the netstandard2.0-only runtime constraint.*

## Journey Manifests

Journey manifests should live in `docs/journeys/manifests/` and include:

- FR/NFR IDs covered by the journey;
- CLI command, `task` target, `PackCompiler` invocation, or MCP tool used to reproduce the flow;
- deterministic fixture pack/manifest/bridge payload required for replay;
- expected screenshots/GIFs/keyframes;
- tests and gates that must pass before the journey is accepted;
- eval verdict schema and pass/fail criteria.

## Autograder Gates

Minimum gates before marking a journey complete:

- `task` fixture tests for build/dry-run flows;
- SDK schema negative and drift tests for plugin validation;
- PackCompiler roundtrip and contract tests;
- runtime isolation tests honoring the netstandard2.0-only BepInEx constraint;
- MCP contract tests for the platform tool surface;
- doc link validation for every referenced rich media asset;
- journey manifest validation via `phenotype-journey verify` when available;
- eval verdict linked to the FR/NFR IDs in the manifest.

## Status

- [x] Identify initial build, SDK, pack, runtime, and MCP flows
- [x] Stub rich media embeds for expected screenshots/GIFs/evals
- [ ] Author manifests in `docs/journeys/manifests/`
- [ ] Record journey captures for each flow
- [ ] Run `phenotype-journey verify` in CI
