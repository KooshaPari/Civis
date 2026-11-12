# hecs allocator image experiment

This is an isolated test-only experiment against the registry source of hecs
0.10.5, under its MIT OR Apache-2.0 license; both license files are retained in
the source copy. The Civis dependency graph is unchanged. The image format is
not a production format and has no stability or compatibility guarantee.

The experiment records all existing slot generations, the ordered pending list,
the signed reservation cursor and the allocated count. Restore validates the
image against an already reconstructed allocator, retains its live locations,
and rejects malformed images before mutating it. Tests exercise free-list and
new-ID reservations and several malformed inputs.

The experiment operates on private hecs `Entities` internals. It does not restore
component storage, public `World` state, or a Civis game; does not prove
continuity; and does not include resource limits, a wire format, schema
versioning, fuzzing, concurrency tests, or an upstream maintenance strategy.
All of those are prerequisites for any production integration. A production
implementation would also need to audit entity mutation entry points and save
atomicity, and distinguish an unflushed reservation from a live component.

Run from `research/hecs-allocator-prototype` with `cargo test --lib allocator_image`.
