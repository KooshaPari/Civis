# Strict seed-world ECS snapshot

The opt-in snapshot accepts the four seed-42 archetypes: civilian bundles, standalone citizens, standalone buildings, and standalone military units. It preserves every supported component and current entity identity, rejects unknown components and unsupported combinations, and validates duplicate or excessive identities before constructing an isolated world. The source world is not changed on rejection.

This is an ECS component snapshot, not a production save format. It does not restore allocator history, external entity references, non-ECS simulation fields, scene generation, or the RNG. Seeded-world and authored-spawn component equality are necessary controls, not proof of post-load simulation continuity. The existing save path and legacy behavior remain unchanged until integrated persistence and independent validation are complete.

An experimental extension also recognizes four observed evolved civilian components: cluster membership, psyche, social graph, and psyche-driven behavior. This is a bounded extension, not an exhaustive inventory of evolved worlds or proof of save continuity.

Compatibility is one-way: this reader accepts canonical v1 snapshots lacking these fields, but older strict v1 readers reject records containing them. Version negotiation or a new schema version is required before persisted or cross-version use.
