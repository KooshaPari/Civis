# Experimental world save witness

This isolated Cargo workspace patches hecs to the adjacent experimental fork
only within this workspace. It does not change Civis production dependencies.
It combines the existing strict seed-world component snapshot with a versioned
allocator image and an integrity-checked envelope. Restore builds a separate
world before returning it; no live world is changed on failure. The digest is
unkeyed corruption detection, not authentication.

This witness covers only the component combinations accepted by
`SeedWorldSnapshot`, and not the rest of `Simulation`, its RNG, save/load RPC,
or continued gameplay. It is not a production save format. Input size is
bounded before parsing, output serialization is capped, and allocator capture
checks its slot limit before cloning. JSON parsing can still allocate multiples
of the input size. A production format needs tighter per-field limits,
fuzzing, schema migration, concurrency review, provenance, and an audited
fork or upstream API. A successful allocation witness is not game continuity.

Run `cargo test -j 1` in this directory with an external `CARGO_TARGET_DIR`.
