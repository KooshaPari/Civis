# Concurrent source drift and falsification receipt

Observed2026-09-29 after draft PR1567 opened. Program `gaming-pair-20260929`.

## Identity — do not silently rebase the assessment

- Original analyzed source remains **b3cd62a7394878cc64d024fbfcfd398b8bb88bf1**.
- Newly observed `main` / PR base: **54d5758970249c8d1f24688ea45920b530e77299**.
- GitHub compare: new tip **15 commits ahead**, zero behind original source.
- Specification branch remains based on the original snapshot. No implementation candidate was executed by this program; no rebase/merge occurred.
- The new comparison touches save/engine/model/agent/mod-host/protocol/MCP state and a large trace-audit/test corpus. Full drift review is NOT complete. Detailed claims below are bound to their own revisions.

## New source materially changes the integrity assessment

At the new tip, `crates/engine/src/save_bundle.rs`, blob **274a63c9e973ad9fc4e2a2c135ef1540052bad83**, ranges **260-450 and590-790** were read. Unlike the earlier snapshot, save_dir now writes an integrity manifest after components; load_dir checks a v5+ manifest and its spec ID, then calls verify_integrity_manifest before world-state migration/replay payload deserialization. The verifier checks plain component names, duplicates, lengths/digests, unlisted files and the listing root. **Do not repeat the old source assessment as though this added verification did not exist.** Native effects and all call paths remain unqualified.

The remaining C-F02 question is more precise, not automatically closed:

1. load_dir first reads `metadata.json`; missing metadata still selects version1.
2. Manifest verification runs only when that metadata-selected file_version is >=5.
3. The manifest verification code excludes metadata.json from listed-file enforcement; the accompanying audit describes metadata as derivative.
4. Therefore metadata's role in selecting whether verification runs must be reconciled with the claim that it is derivative/not trusted. A missing/downgraded metadata fixture tests a materially different path from a changed body protected by an intact v5 declaration.

This is a **source-observed control-flow distinction**, not a native demonstration that every damaged bundle will successfully load. Genuine supported legacy saves must still work under an explicit classification rule. Do not claim keyed authenticity: the imported audit itself correctly limits unkeyed BLAKE3 to corruption checking, not protection from an actor able to rewrite the manifest.

The audit's prose mentions `SaveBundle`, `verify_save`, `load_save` and MessagePack, while this inspected path is `CivSaveBundle`, verify_integrity_manifest and JSON components. Those names/layout descriptions require source reconciliation; do not bind requirement evidence by a nearby-looking symbol or narrative alone.

## Additional original-snapshot trace recovered

At **b3cd62a7394878cc64d024fbfcfd398b8bb88bf1**, save_bundle.rs **340-535 and620-880** were inspected again/extended. `load` dispatches to directory/archive entrypoints; `save_to_slot` directly calls save_archive, `load_from_slot` directly calls load_archive. Those wrappers do not themselves add an atomic-save transaction. Higher mounted UI/server/storage callers remain to inspect. This narrows C-F03's caller gap without falsely claiming a reproduced crash defect or full application reachability.

## Imported native reports — not this worker's native execution

Read at new tip54d57589:

- `docs/audits/validation-2026-09-29.md`, full, blob **f3fa389ba6905813f1cf432e138f95698ac2e612**. Author reports validation against short revision2253de28, **5731 passed,15 failed,32 ignored**, and excludes `ten_thousand_ticks_under_budget`. The report says failures also reproduce at baselineb3cd62a7. Raw command logs, environment identity, full test candidate revision and independent rerun were NOT acquired here. A baseline-relative no-new-regression claim is not an all-green product result. The report's1423-ID/74.3% coverage calculation is not this program's accepted mature-contract denominator.
- `docs/audits/save006-mutation-results.md`, full, blob **dfac1660f95f2b20e127a4812a029805f5fcf29f**. Author reports save suite39 passed/0 failed/1 ignored and five guard mutations killed; a length-check mutation initially survived, motivating an isolated negative test. This is useful imported evidence and a concrete testing lead, not an independently reproduced result. Referenced mutation runner is an owner-local Windows path; its bytes/full candidate binding/raw results were not retrieved.
- `docs/audits/fr-save-006-audit.md:1-155`, blob **89870b484edecd403951a7227218e0f6711cda83**, read as an author audit. It records spec/layout deviations, hash/authentication distinction and guard claims. Requirement text and audit assertions still need semantic/source/candidate reconciliation.

These reports remain IMPORTED ASSERTIONS until independently qualified against exact subjects/configurations. No skipped or failed case becomes green because it is pre-existing. Some reported failing tests concern determinism or multiplayer-like behavior and need authority/scope classification against the charter rather than blanket deletion or reinstatement.

## Receipt consequence

Current-state source snapshot is unchanged. Add **DRIFT_REVIEW_OPEN** to the pending work: reconcile the new integrity manifest, version selector, actual save callers and imported audits with original findings. Both specification/design gates remain incomplete. Registry cross-link: https://github.com/KooshaPari/PhenoRegistry/pull/596 . Paired Dino draft: https://github.com/KooshaPari/Dino/pull/490 .
