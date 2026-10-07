# Civis SOTA pass 2 — component identity vs runtime guest state

Date2026-09-30. Targeted after reproduced mod/persistence failures.

## TUF snapshot consistency — LEARN FROM

TUF snapshot metadata binds versions/hashes so clients cannot combine metadata from inconsistent repository times. Civis should borrow the consistency principle for a save's semantic component set: world/policy/research/mod identity must belong to one save generation/world/tick.

Do not adopt TUF as the save format. Save integrity/authenticity and software-update trust are distinct problems.

## Wasmtime serialization — important rejection

Wasmtime officially supports serializing **compiled Module/Component artifacts** to avoid recompilation. That is code/compiled-artifact persistence, not a general snapshot of an instantiated guest's mutable linear memory, host resources, capabilities or product-level mod lifecycle.

Wasmtime component resources also carry dynamic ownership/borrow state tied to instances/stores. Persisting a numeric/resource handle across a save boundary is therefore not a valid substitute for a Civis-owned semantic resource identity/reconstruction contract.

Consequence:
- Civis should persist mod artifact identity + compatible guest-owned state under its own versioned contract;
- do not infer that `Component::serialize` solves mod runtime state persistence;
- reinstantiate/resolve compatible mod artifacts, then restore explicitly supported guest memory/state.

Sources: official Wasmtime Component/serialization/resource docs, reviewed2026-09-30.

## Existing ecosystem fit

Current Civis mod host uses Wasmtime-style guest state plus host-managed memory. The mature save contract should keep host ownership explicit: save what Civis semantically owns, reconstruct ephemeral Wasmtime instance resources, and reject/migrate incompatible identities.
