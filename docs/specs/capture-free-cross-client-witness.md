# Capture-free cross-client witness

Status: a narrow in-process bridge test, not an installed-client or full gameplay acceptance.

An isolated loopback bridge starts with seed 42, pauses its simulation, and connects two independently opened WebSocket clients. The observer records the civilian-pin projection; the actor spawns a civilian at a coordinate absent from that projection. The observer must see exactly one additional matching pin. The observer disconnects and a newly connected observer must see the same paused tick, population, and pins. Each RPC must receive a matching request ID under a deadline, with interleaved notifications and binary frames ignored.

The pre-action absence, exact pin-count delta, and post-reconnect equality prevent an accepted response or a stale, pre-action projection from passing. Suppressing the spawn must fail the postcondition. This witness binds observations to one in-process bridge lifetime and endpoint; it does not provide persistent world identity, prove client rendering or controls, verify authorization, establish save/load continuity, or cover a separately launched server.
