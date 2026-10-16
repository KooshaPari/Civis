# Civis pass 17 — expanded reproduced persistence failures

Date 2026-09-30.

Candidate `ed8d8bbbffc4883ecbb7fa382c28aeb43ab60b14`; run `36687813616`; artifact `11085881640`; sha256 `6ccfa7439c3f75158a4fb68133500179c5365a2978d3271600979f4df335d84f`.

Compile succeeded. Recovery execution: 10 tests; 4 passed, 6 failed, 864 filtered out.

## Prototype greens

Both test-only semantic state-manifest prototype cases passed:
- policy + research capture/serialize/decode/apply restores the tested semantic state;
- orphan guest memory is detected when no active mod identity declares it.

These are architecture-prototype greens only, not production save qualification.

## Production reds retained

Previously reproduced:
- C-BEH-01 economy PolicyInput resets to default;
- C-BEH-02 research cache resets;
- C-BEH-03 guest memory survives without loaded mod identity.

New reproduced failures:

### C-BEH-04 control policy kind resets

Fixture begins with control policy kind `capitalist`; after save/load, `policy.name()` is `noop`.

This is distinct from economy `PolicyInput`. The authoritative-state manifest needs a separate control-policy identity/configuration row.

### C-BEH-05 market state is lost

Fixture stores a market price entry expected as `Some(777)`; loaded state returns `None`.

Market state is exposed to clients and affects future economic behavior. If it is intended derivable, that derivation contract must be explicit; current direct load does not restore the tested value.

### C-BEH-06 current-format metadata removal silently downgrades

A freshly written v5 bundle has metadata removed. The recovery oracle expected current-format corruption/downgrade detection, but load did not return an error. This confirms the source-derived version-selector bypass: metadata absence selects legacy behavior before v5 integrity policy can protect the bundle.

Genuine supported legacy saves remain a separate compatibility requirement; the fix cannot simply reject every metadata-less directory without classification.

## Architecture consequence

Expand the semantic manifest prototype to:
- economy PolicyInput;
- control policy kind + accepted configuration;
- research;
- market durable state or explicit reconstruction identity;
- active mod set/artifact identity + guest compatibility.

Separately, strengthen the **outer format classifier** so current-format integrity policy cannot be bypassed by deleting the metadata that selects it. This outer classifier is distinct from semantic state completeness.

Do not conflate these six failures into one "save broken" claim; each has separate authority and remediation.
