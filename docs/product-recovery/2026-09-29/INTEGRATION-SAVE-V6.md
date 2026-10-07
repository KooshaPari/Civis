# Civis integration design — save format v6 semantic state component

Date 2026-09-30. Based on three reproduced CivSaveBundle semantic losses. Design only; production save path unchanged.

## Preserve v5 lower layers

Do not replace:
- componentized folder/archive;
- v5 integrity manifest;
- explicit environment/stocks/institutions sidecars;
- replay audit/history.

Add semantic completeness above them.

## Proposed v6 additions

`semantic_state.json`:
- schema_version;
- effective economy policy;
- research cache/progression;
- scenario/profile binding identity where recoverable;
- active mod-set compatibility identities;
- future component declarations only after state-denominator review.

The existing integrity manifest automatically hashes the new component if the generic component enumeration remains authoritative; verify this in prototype.

Metadata format_version becomes6 only when semantic_state.json is successfully written and included in integrity publication.

## Mod identity limits discovered

Current mod-host ModMeta provides id, version, api_version, type, author and optional author public key, but no persisted artifact digest. The mature save contract needs stronger identity than id/version alone.

Prototype can start with id/version/api_version for compatibility classification, but production acceptance should add artifact/content digest or signed package identity where available. Do not pretend semver strings uniquely identify bytes.

Guest state restore order:
1. resolve required/optional active mod identities;
2. load/validate compatible artifacts;
3. only then restore matching guest memory;
4. orphan/incompatible memory yields explicit degraded/refused/migration outcome.

## Economy policy distinction

There are two separate policy concepts:
- PolicyInput: mutable scenario economy consumption knobs — reproduced lost and should be persisted or explicitly rebound.
- Box<dyn Policy>: high-level control policy kind — separate state owner, currently not covered by the first red.

The v6 experiment must not claim "policy persistence complete" after fixing PolicyInput alone. Persist/rebind policy kind/config separately once its accepted lifecycle is resolved.

## Research

Engine ResearchCache is serializable and directly affects snapshots/victory. It is suitable for a semantic component. Do not confuse it with the separate civ-research crate's hash cache, which is a different type with the same name.

## Load ordering

1. classify format and integrity;
2. load/reconstruct base Simulation/replay;
3. validate semantic-state component and external bindings;
4. resolve mod set;
5. apply durable semantic state;
6. restore compatible guest memory;
7. derive mirrors/projections;
8. run cross-component consistency checks;
9. expose loaded world.

No user-visible "load succeeded" before step8.

## Migration

v5 -> v6:
- preserve original bytes;
- missing policy/research/mod-set identity cannot be invented;
- use explicit migration defaults only when product semantics authorize them;
- otherwise mark degraded legacy state and surface what was unavailable.

A migration receipt binds source save digest, adapter version, resulting v6 candidate digest and unresolved losses.
