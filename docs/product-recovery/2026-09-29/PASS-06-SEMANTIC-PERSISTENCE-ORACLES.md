# Civis pass 6 — semantic persistence oracle gaps

Date 2026-09-29. Concurrent source `54d5758970249c8d1f24688ea45920b530e77299`.

## Existing save tests are strong in the wrong dimension for completeness

The active save_bundle suite has meaningful mutation-resistant integrity tests. Preserve them.

The two mod guest-state roundtrip tests, however, directly create guest memory for IDs `test-mod` and `archive-mod` without first loading those mods. After load they assert only the guest bytes. This proves blob persistence independent of loaded-mod identity.

That is not merely absence of a broader test; it is an existing executable model where orphan mod memory is accepted as a successful round trip.

## C-F20 — product-level mod persistence needs compatibility identity

A faithful world cannot claim "mods restored" from guest memory alone. The persistence contract needs either:
- save embeds exact active mod-set identity/versions/digests and loader resolves them before guest memory import; or
- profile/scenario identity is authoritative and the save binds to that external mod set; or
- explicit degraded load semantics for missing/incompatible mods.

Replay ModLoaded events are informational during replay and cannot satisfy this.

## C-F21 — policy and research omissions have no discovered semantic save tests

Repository search found economy-policy behavior tests and research snapshot/behavior tests, but no save/load tests for those states.

Economy policy is mutable through a mounted server operation and directly affects future economy behavior. Active save code has no observed policy component/event.

ResearchCache affects snapshot fields and victory conditions. Active replay's ResearchOutcome handler does not rebuild the cache.

These should be the first expected-failure semantic persistence tests.

## Exact oracle cases

### policy continuity
1. mutate policy through mounted `sim.set_policy`;
2. observe non-default policy and one behavior influenced by it;
3. save through mounted slot/autosave path;
4. replace/restart Simulation;
5. load;
6. require either same policy or an explicit accepted scenario/profile rebinding that deterministically re-establishes it;
7. next economy behavior must correspond to declared restored configuration.

### research continuity
1. establish researched and in-progress technology through real mechanics or a narrowly identified setup hook;
2. record snapshot and outcome-progress state;
3. save/load;
4. compare researched set, in-progress state, victory/outcome projection and next research transition.

### mod compatibility
1. load exact mod artifact and establish guest state;
2. save;
3. same mod artifact load → behavior/memory restored;
4. missing mod → explicit compatibility result, not silent full success;
5. changed version → migration/compatibility result;
6. orphan guest memory fixture must not count as active-mod restoration.

## C-F22 — integrity manifest test prose overclaims adversarial tamper resistance

One test comment says a manifest edited to match tampered bytes is "still caught" because the root is recomputed. The implemented test updates the component digest but deliberately leaves the root stale. An actor able to edit the component and manifest can also recompute the unkeyed root. Existing audit prose elsewhere already recognizes BLAKE3 here is corruption detection, not authenticity.

Keep the test as accidental/stale-root detection. Do not use it as evidence against a malicious writer. Product authenticity requires a separate trust/signature model if that threat is accepted.

No implementation fix or native execution in this pass.
