# Cross-client persistence regression

Status: known failing regression at base f37f945. This document specifies a required outcome; it does not claim the save path currently satisfies it.

With an invocation-owned, in-process loopback bridge, seed 42, isolated save directory, and paused simulation, one client spawns a civilian. A second client observes that civilian, disconnects, and reconnects. The first client saves, explicitly advances one tick, and loads the saved slot. The second client must observe the saved tick and the spawned civilian after the successful load; the load must return a scene generation. The full saved civilian projection must also match after load. An accepted load response alone is not a continuity witness.

The regression currently fails because the spawned civilian is absent after successful load. The server mutates the ECS world for this spawn without recording a replay event; the save bundle restores from a replay and selected sidecars. Replay tick application advances the tick field without replaying full simulation evolution. Therefore logging only the spawn would not establish persistence for previously evolved ECS state or RNG continuity.

Acceptance of a repair additionally requires preserved stable entity identities and relevant components, future deterministic evolution under the same inputs and RNG state, schema and integrity validation, legacy-save behavior, and failure without replacing the active world for invalid or incomplete state. Exercise missing or corrupt data, stale generation, altered action, multiple saves, and independent restored instances. Validate an owned standalone server and installed-client journey separately. Never use screenshots or recording in this witness.
