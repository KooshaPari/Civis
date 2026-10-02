# Civis pass 13 — reproduced semantic persistence failures

Candidate `eb74d0724163f9958e93c16c4deeb18344882246`; recovery run `36628556544`; artifact `11062202963`, sha256 `9d76dede0df4b2da29a2fbb8f17220f8950157ee22b7490972235852af7738fb`.

Compile succeeded. Three targeted recovery tests executed; all three failed.

## C-BEH-01 CONFIRMED — economy policy resets

Saved Simulation policy:
- base_consumption_joules =123456
- scarcity_multiplier =2.75

Loaded value at first assertion:
- base_consumption_joules =5,000,000,000, matching DEFAULT_ECONOMY_POLICY.

The product decision remains whether economy policy is durable world state or external scenario/profile configuration. Current direct CivSaveBundle load silently defaults it and does not bind an external scenario/profile identity.

## C-BEH-02 CONFIRMED — research cache is lost

Saved researched set: pottery, masonry; queued: writing.
Loaded researched set: empty.

ResearchCache is user-visible in snapshots and contributes directly to Age of Enlightenment victory conditions. This is not merely internal cache loss unless product semantics are changed to reconstruct it from another authoritative source—which the active replay path does not currently do.

## C-BEH-03 CONFIRMED — guest memory is independent of loaded mod identity

The test proves recovery-orphan-mod is not loaded, imports guest bytes, saves/loads, proves those bytes survived, then checks loaded-mod identity. The loaded-mod assertion fails.

Thus mod_state.json is guest-memory persistence, not active-mod-set restoration.

## Architecture decision now justified

The next save format revision/prototype should add an explicit authoritative-state manifest and compatibility identity rather than adding isolated fields blindly.

At minimum decide/persist or explicitly rebind:
- scenario/profile identity and mutable economy policy;
- research state;
- active mod set with version/artifact/capability identity plus guest memory compatibility;
- every remaining durable Simulation/ECS owner identified by the state denominator.

Then add migration/version rules and negative controls for missing/incompatible external configuration. Preserve current v5 byte-integrity manifest as a lower layer; state completeness is a separate predicate.

No production fix committed yet.
