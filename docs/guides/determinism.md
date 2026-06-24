# Simulation Determinism

Civis simulation work must be reproducible from the same inputs, seed, scenario,
and tick sequence.

Prefer fixed-step execution, deterministic ordering, explicit seeded randomness,
and state hashing at replay boundaries. Any nondeterministic dependency needs an
ADR and a test that proves replay stability for the affected system.
