# FR-SAVE-006 Mutation Results

**Date:** 2026-09-29
**Target:** `crates/engine/src/save_bundle.rs`
**Command:** `python C:\Users\koosh\agents\sandbox\mutation_save006.py`
**Suite:** `cargo test -p civ-engine --lib save_bundle`
**Baseline:** 39 passed, 0 failed, 1 ignored

## Method

Each guard is disabled by a single source substitution, the crate is rebuilt,
and the suite is re-run. A guard is **killed** when at least one test fails
without it. A guard that survives is untested by construction, and any claim
that the guard is covered must be withdrawn until a test is added.

| # | Mutation | Source substitution | Verdict |
|---|----------|---------------------|---------|
| 1 | no path-containment check | `if !is_plain_component_name(&expected.component) {` → `if false {` | **KILLED** (1 test failed) |
| 2 | no duplicate-entry check | `if !listed.insert(expected.component.as_str()) {` → `if false {` | **KILLED** (27 tests failed) |
| 3 | no unlisted-file scan | `if !listed.contains(name.as_str()) {` → `if false {` | **KILLED** (1 test failed) |
| 4 | no length comparison | `if actual != expected.blake3 \|\| bytes.len() as u64 != expected.len {` → `if actual != expected.blake3 {` | **KILLED** (1 test failed) |
| 5 | no root-digest recheck | `if recomputed_root != manifest.root {` → `if false {` | **KILLED** (1 test failed) |

All five guards killed. The source file was restored and verified byte-identical
after the run.

## What mutation testing found

Mutation 4 **survived on the first run**. Removing the byte-length comparison
from the digest guard left the entire suite green. The length check was, at that
point, claimed coverage with no test behind it: every existing test that altered a
component's size also altered its content, so the digest check caught the change
and the length half of the condition was never the thing that fired.

That is precisely the failure mode mutation testing exists to catch. A guard that
is never independently exercised is a guard that can be deleted without any test
complaint, and a reviewer reading only the test names would have had no way to
notice.

The fix was
`fr_save_006_rejects_a_component_whose_length_disagrees_with_the_manifest`, which
isolates length from content: it appends four bytes to a real component,
recomputes the digest over the enlarged file so the digest matches perfectly, and
leaves the recorded `len` stale. Only the length check can reject that bundle. The
test then repairs the length and asserts the same bundle loads, which proves the
rejection came from the length check and not from some other difference.

After that addition, mutation 4 is killed.

## Tooling notes

Two Windows-specific problems had to be solved before any verdict could be
observed, and both are worth recording because a silent wrong answer is worse than
a failure.

**`cargo test` output is not capturable through a pipe on this machine.** The
libtest harness writes its per-test lines and `test result:` summary straight to
the console handle. Under `subprocess.run(..., capture_output=True)` the build
output arrives and then nothing, so a script that scans for `test result:` finds
nothing and cannot tell "tests passed" from "tests never ran". The script
therefore builds with `cargo test --no-run`, locates the test binary from
`--message-format=json` (`kind: ["lib"]` with a non-null `executable` is the
test-profile artifact; the plain build reports the same kind with a null
executable), and invokes the binary directly.

**The result regex was wrong in a way that reported "did not run" for a passing
suite.** The harness prints `test result: ok. 38 passed; 0 failed; ...` and
`test result: FAILED. 37 passed; 1 failed; ...`. A pattern anchored directly on
`test result: (\d+)` does not match either, because the `ok.` / `FAILED.` token
sits between the colon and the counts. The working pattern is
`test result:\D*?(\d+) passed;\s*(\d+) failed`.

Both of these initially manifested as "the suite did not run", which is the same
message a genuine compile error produces. The distinction matters: one is a
tooling bug and one is a real result, and conflating them would have produced a
report claiming the tests could not be verified when in fact they were passing.

## Reproducing

```
python C:\Users\koosh\agents\sandbox\mutation_save006.py
```

The script restores `save_bundle.rs` from an in-memory copy in a `finally` block
and asserts the restoration is byte-identical before reporting, so an interrupted
run cannot leave a mutated guard behind in the working tree.
