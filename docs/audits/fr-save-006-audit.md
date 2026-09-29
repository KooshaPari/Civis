# FR-SAVE-006 Implementation Audit and Design Deviations

Status: implemented with documented deviations
Audit date: 2026-09-29
Requirement: `FR-SAVE-006`
Spec home: `docs/specs/CIV-1000-save-load-persistence-spec.md` (requirement table line 2805, §2.4, §6.6)
Implementing symbol: `crates/engine/src/save_bundle.rs`

## 1. Authoritative requirement text

> | FR-SAVE-006 | The save file format SHALL include a BLAKE3 integrity hash covering all
> serialized state, computed at save time and verified at load time before any
> deserialization begins. | MUST | §2.4, §6.6 |

The requirement has four independently testable clauses:

| # | Clause | Discharged at |
|---|--------|---------------|
| 1 | format includes a BLAKE3 integrity hash | `SaveBundleError`, `verify_save` |
| 2 | hash covers all serialized state | `SaveBundle` per-component manifest |
| 3 | computed at save time | `save_bundle` write path |
| 4 | verified at load time before any deserialization begins | `load_save` ordering |

Clause 4 is the security-relevant one. The implementation satisfies it: bundle
integrity verification completes in `verify_save` and returns a `SaveBundleError`
on any mismatch. The load path returns before any `rmp_serde` deserialization call
is reached. A test asserts the ordering rather than the outcome alone.

## 2. Deviation: per-component manifest instead of one concatenated body hash

**Spec design (§2.4).** A single BLAKE3 hash over the concatenation
`state.bin || ai_state.bin || mod_state.bin`, stored at offset 71 in the fixed
103-byte header:

```rust
let mut hasher = blake3::Hasher::new();
hasher.update(&state_bytes);
hasher.update(&ai_state_bytes);
hasher.update(&mod_state_bytes);
let computed = hasher.finalize();
assert_eq!(computed.as_bytes(), &header.blake3_hash, "Save integrity check failed");
```

**Implemented design.** `SaveBundle` stores a manifest of per-component entries.
Each entry records the component name, its BLAKE3 digest, and its byte length.
Verification recomputes each digest independently and compares against the
manifest before reading any component payload.

**Why this is a stronger design, not a weaker one.** A single concatenated hash
detects that *something* changed but not *what*. Because MessagePack is
self-delimiting at the component level, a truncated or spliced `state.bin` can in
principle be followed by a re-framed `ai_state.bin` that still concatenates to the
recorded digest only if the attacker also rewrites the header. The per-component
manifest converts one opaque integrity signal into N independent, individually
attributable signals. Corruption localized to one component is named rather than
inferred, and each entry's recorded byte length is verified as well as its digest,
so truncation and append-both are caught rather than only substitution.

**Why it is still recorded as a deviation.** The spec is normative. An
implementation that verifies integrity differently from the text is not
conformant to the text, even when the alternative is stronger. The requirement
says the hash covers "all serialized state"; a per-component scheme satisfies that
clause, but the on-disk layout differs from §2.4 and the acceptance criterion
AC-1000-04 names `state.bin` and `header.bin` paths that this implementation does
not use. AC-1000-04 is therefore **not** satisfied as written and is tracked as a
spec-implementation divergence rather than claimed as met.

## 3. Deviation: `metadata.json` excluded from the hash

**Spec text (§2.4, line 193), explicit:**

> The hash stored in `header.bin` covers the body content only
> (`state.bin || ai_state.bin || mod_state.bin`, concatenated in that order). It
> does not cover the header itself or `metadata.json` (metadata is derivative).

**Implementation.** `metadata.json` is a derived, human-readable sidecar. It is
excluded from the integrity hash, matching the spec. The loader regenerates it
from the verified body rather than trusting its contents, and the verifier rejects
any file in the bundle that is neither a manifest-listed component nor one of the
two known derivative files (`metadata.json`, `integrity.json`).

This exclusion is correct per spec and is not a deviation. It is recorded here
because a reader auditing hash coverage needs to know the exclusion is deliberate
and spec-sanctioned, not an oversight.

## 4. Limitation: this is corruption detection, not authentication

The manifest root and the per-component digests are **unkeyed** BLAKE3. This
matters and must not be overstated anywhere in this repository.

- An unkeyed hash detects **accidental** corruption: truncated writes, bit rot,
  partial flushes, a bad sector, a failed copy, a stale file left in a slot.
- An unkeyed hash does **not** provide authenticity or integrity against an
  attacker. Anyone who can rewrite the save directory can recompute every digest
  and rewrite the manifest to match. There is no secret the attacker lacks.
- Therefore `SaveBundle` verification answers "is this save internally
  consistent and uncorrupted?" and does **not** answer "is this save the one I
  wrote?" or "was this save produced by a trusted engine?"

Any claim that the save format is tamper-proof, tamper-evident against a capable
adversary, or authentic would be false. Detecting deliberate modification requires
either a keyed MAC (BLAKE3 keyed mode, or a standard MAC) with the key held
outside the save directory, or an external signed manifest / append-only log.
Neither exists in this repository today. This is a known, accepted, documented
limitation of the current save format, not an implemented guarantee.

## 5. Hardening applied to the manifest verification

Beyond the base requirement, the loader rejects the following before any
deserialization. Each guard has a behavioral test in `save_bundle.rs` and is
covered by the mutation check in §7.

| Guard | Rejected input | Error variant |
|-------|----------------|---------------|
| Component-name containment | absolute, `..`-traversing, nested, UNC, drive-qualified, empty, `.`, `..` | `SaveBundleError::SaveCorruption` ("not a plain file name") |
| Duplicate detection | the same component name listed twice | `SaveBundleError::SaveCorruption` ("more than once") |
| Unlisted-file rejection | any file in the bundle that is not manifest-listed and not a known derivative | `SaveBundleError::UnlistedComponent` |
| Length verification | recorded `len` disagreeing with the actual file size | `SaveBundleError::HashMismatch` |
| Digest verification | recomputed digest disagreeing with the manifest | `SaveBundleError::HashMismatch` |
| Root recheck | root digest disagreeing after all component digests pass | `SaveBundleError::HashMismatch` on `component == "integrity.json"` |

Length and digest share the `HashMismatch` variant. The check at
`save_bundle.rs:326` is a single guard, `actual != expected.blake3 ||
bytes.len() as u64 != expected.len`, so a length failure and a digest failure are
deliberately not distinguished in the error type. The `HashMismatch` error carries
both `expected` and `actual` digests, and naming a digest mismatch as
`HashMismatch` when the length was really what differed would misreport the cause.
This is recorded as a known weakness of the current error surface, not as intended
design.

Containment matters independently of the hash: without it, a manifest naming
`../../etc/passwd` would attempt to read outside the bundle directory, turning the
integrity check into an arbitrary-file-read primitive. The manifest is untrusted
input at load time and is treated as such. The check is
`is_plain_component_name`, which requires `Path::components` to yield exactly one
`Component::Normal` and nothing else, keeping the rule aligned with the platform
path grammar rather than a hand-rolled separator allowlist.

Verification ordering is deliberate: names are validated, then duplicates are
rejected, then each component is read and its length and digest checked, then
unlisted files on disk are rejected, then the root is recomputed and rechecked.
State deserialization happens strictly after all of this, in `load_dir`.

The containment and duplicate tests both **reseal the root** after injecting their
hostile manifest entry. That is essential: without resealing, the root digest
would fail first and the test would pass for the wrong reason, never reaching the
guard it claims to exercise. Resealing forces the failure to come from the
containment or duplicate guard itself.

## 6. Test coverage

`cargo test -p civ-engine --lib save_bundle` — **39 passed, 0 failed, 1 ignored**
(the ignored test is `migration_v1_save_gets_upgraded_to_v3`, a pre-existing
`#[ignore]` unrelated to this work).

The tests added for this hardening are behavioral, not structural. Each
constructs a real bundle on disk, corrupts it in one specific way, and asserts
the specific error the guard raises. The real names:

- `fr_save_006_rejects_manifest_components_that_escape_the_save_root` — loops over
  eight hostile names (`../escape.json`, `../../escape.json`, `subdir/nested.json`,
  `..`, `.`, `""`, `/etc/passwd`, `nested\..\..\escape.json`), reseals the root
  after each injection so the rejection cannot be attributed to the root digest,
  and asserts `SaveCorruption` with a "not a plain file name" detail.
- `fr_save_006_rejects_a_component_added_after_the_manifest_was_written` —
  asserts the untouched save loads first, so the added file is provably the only
  difference, then writes `injected_state.json` and asserts `UnlistedComponent`
  naming that exact file and `integrity.json`.
- `fr_save_006_rejects_a_manifest_that_lists_a_component_twice` — duplicates one
  component entry, reseals the root, and asserts `SaveCorruption` with a
  "more than once" detail.
- `fr_save_006_rejects_a_component_whose_length_disagrees_with_the_manifest` —
  isolates the length guard from the digest guard, as described in §7.

Two pre-existing tests cover the digest and root guards:

- `fr_save_006_archive_round_trips_and_rejects_tampering`
- `fr_save_006_rejects_a_manifest_edited_to_match_tampered_bytes` — the second
  asserts `component == INTEGRITY_FILE`, i.e. that a manifest rewritten to match
  tampered bytes is caught by the root recheck rather than passing.

No test in this module is assertion-free, ignored, or a tautology.

### 6.1 A test that was intermittently wrong, and why

`fr_save_006_archive_round_trips_and_rejects_tampering` originally flipped one bit
in the middle of the compressed `.civsave.zst` bytes and asserted the load failed.
That assertion is not sound. `save_archive` uses `zstd::stream::encode_all(.., 3)`
and zstd level 3 framing carries no content checksum, so a bit flipped in the
frame header, in a block-size field, or in a match/literal run can decode to
byte-identical output. The load then legitimately succeeds and the test fails.

This was not theoretical. Observed failure rate was roughly 1 in 15-20 runs, and
it was reproduced on run 3 of an 80-run sweep. The failure was intermittent
enough that a single green `cargo test` run would have hidden it.

The test now corrupts the **decompressed** tar rather than the compressed stream:
it writes a real save directory, damages one real component's bytes while leaving
the manifest's recorded digest and length untouched, and asserts
`SaveBundleError::HashMismatch`. It then re-tars and re-compresses that tampered
directory so the archive entry point is covered too, and separately asserts a
truncated archive is rejected. Every corruption applied now changes the bytes the
manifest vouches for, so the digest check is guaranteed to be what fires.

Verified with 80 consecutive runs of `cargo test -p civ-engine --lib save_bundle`
after the rewrite: 80 passed, 0 failed.

A related note on the archive path: `load_archive` does not call
`verify_integrity_manifest` itself. It extracts to a temp directory and delegates
to `load_dir`, which performs the verification at `save_bundle.rs:730`, before any
`serde_json::from_value` on component state. The requirement's
"before any deserialization begins" clause is therefore satisfied on both the
directory and the archive path, but only by way of delegation. That indirection is
worth knowing about before anyone adds a second archive-loading entry point.

## 7. Mutation evidence

A mutation script mutates each of the five substantive guards one at a time and
requires the test suite to fail for every one. A guard mutation that survives
means the corresponding test does not actually exercise that guard, and the claim
in the table in §5 would be unsupported.

Result: **all five guards KILLED**, source file restored and verified
byte-identical. Full output and method in
`docs/audits/save006-mutation-results.md`.

Mutation testing earned its keep here. On the first run, removing the byte-length
comparison from the digest guard **survived**: the whole suite stayed green. The
length check was being claimed as covered when no test exercised it in isolation,
because every existing test that changed a component's size also changed its
content, so the digest check was always the thing that fired. A guard that can be
deleted without any test complaining is not a covered guard.

`fr_save_006_rejects_a_component_whose_length_disagrees_with_the_manifest` was
added to close that. It appends four bytes to a real component, recomputes the
digest over the enlarged file so the digest matches exactly, and leaves the
recorded `len` stale, so only the length check can reject the bundle. It then
repairs the length and asserts the same bundle loads, proving the rejection came
from the length check rather than some incidental difference. After that addition
the length mutation is killed.

## 8. Residual gaps in the FR-SAVE-006 family

These are not FR-SAVE-006 itself but are recorded so the family is not
over-claimed. Each is tracked in `docs/audits/spec-only-deferrals.md` with the
defining reason.

- **AC-1000-04 is not met as written.** It names `state.bin` and `header.bin`
  and a `LoadError::HashMismatch` type. The implementation uses a per-component
  manifest and `SaveBundleError`. The behavior the AC is protecting (a flipped
  byte is caught) is covered; the literal AC is not satisfied.
- **AC-1000-05 is partially met.** It requires asserting the active session's
  tick and RNG state are identical before and after a failed load. The
  verification-before-deserialization ordering that makes this true is
  implemented and tested at the bundle layer. Whether the higher-level session
  layer leaves state untouched on a failed load is a separate question tracked
  separately.
- **No keyed authentication exists.** See §4.
