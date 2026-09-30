# Developer-agent handoff — Civis semantic save integration

Date 2026-09-30. Status: CONDITIONAL READY. Execute only after the current exact semantic candidate run completes green; otherwise repair/classify that run first.

## Evidence baseline

Frozen source: `b3cd62a7394878cc64d024fbfcfd398b8bb88bf1`.
Concurrent implementation used for state archaeology: `54d5758970249c8d1f24688ea45920b530e77299`.

Canonical recovery/spec PR: #1567.
Implementation experiment PR: #1569, branch `experiment/semantic-save-manifest-20260930`.

State denominator currently includes 125 Simulation fields / 48 WorldState fields. It is not closed.

Production recovery has nine separately tracked persistence/classification reds, including economy policy, research, control policy, market, mod identity/guest-state semantics, metadata downgrade, tutorial, religion and active caravans.

## Accepted implementation direction

Componentized semantic state + explicit outer format/generation identity. Default CivSaveBundle remains comparison baseline until vNext gates close. Replay is not authoritative persistence.

Candidate architecture includes:
- SemanticStateManifest;
- compatibility-first SemanticSaveAdapter;
- opt-in SemanticBundleBridge;
- staged generations;
- CURRENT-last publication;
- reconciliation that does not auto-promote CURRENT.next.

## Developer-agent assignment after candidate green

1. Keep vNext opt-in.
2. Preserve original v5 bytes during migration experiments.
3. Resolve scenario/profile and exact active mod artifacts before guest-memory import.
4. Bind semantic component to world/tick/generation.
5. Expand semantic components only from the 125/48 authority ledger; do not blindly serialize every field.
6. Establish filesystem capability policy for publication:
   - Linux target;
   - Windows/NTFS target;
   - file + directory durability requirements;
   - rename/replacement behavior;
   - recovery after orphan staged generation.
7. Define filesystem vs SQLite save-index authority and reconciliation.
8. Close metadata-deletion downgrade without rejecting genuine legacy imports.

## Required controls

- all nine current production reds remain as comparison controls;
- opt-in bridge turns each applicable semantic red green;
- missing semantic component fails closed for vNext;
- future schema fails closed;
- mixed world/generation rejected;
- missing/incompatible mod rejected/degraded before guest import;
- v5 original remains intact;
- disk-full/permission/interruption before publication retains prior generation;
- orphan CURRENT.next does not promote;
- invalid CURRENT is explicit recovery mode;
- successful restart/load continues semantic world state.

## Forbidden shortcuts

- no default CivSaveBundle replacement yet;
- no global deterministic replay resurrection;
- no "serialize Simulation" blanket dump;
- no automatic defaulting of missing durable state without a documented migration invariant;
- no treating guest-memory bytes as active mod restoration;
- no completion percentage until denominator closes.

## Completion evidence

Return exact candidate, filesystem/OS, save generation IDs, component digests, migration source/target, mod identities, fault point, CURRENT before/after, raw logs and post-restart semantic checks. Update #1569/#1567 and PhenoRegistry.
