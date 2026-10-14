# Save.slot diplomacy tuple-key regression

Directive: Instinct authorized a separate candidate save PR via
`relaySentinel_64c310c5f20c81919c93df5677014708`, review before merge.
This candidate is based on main `80c84264520daaa1674c3993a76744033743b107`.
It has no code dependency on fixture PR1593, whose evidence head is
`4d1260a51f09844025bbae1fc0238a8c9c1500cb`.

## Reproduction and synthesis

PR1593's owned loopback fixture used seed42, speed0, reset tick0, four
operator-input civilian spawns, and private save/replay directories. Phases1–7
and six denials passed; phase8 save ID2040 returned -32603
`json: key must be a string`. Its same-world control saved slot-1 (ID2014),
offered trade 0→1 (ID2015), then failed saving slot-2 (ID2016).
See https://github.com/KooshaPari/Civis/pull/1593 and its
[CI live job](https://github.com/KooshaPari/Civis/actions/runs/38504290087/job/115658103589).
These are prior live observations, not a new live run of this candidate.

The standalone regression uses seed42/tick0 without borrowed fixture files.
`seed42_tick0_diplomacy_archive_roundtrip` successfully saves/loads a pristine
world, then saves/loads after trade, first conflict and second conflict and
compares complete relation records. On baseline it exits101 at the first
populated save: `Json(Error("key must be a string", line: 0, column: 0))`.

The server regression invokes the actual JSON-RPC handler and save.slot effect.
IDs2014/2015/2016 use the same valid slot/trade/slot requests as the causal
control. On baseline the first save and trade succeed, then ID2016 returns
`{"jsonrpc":"2.0","id":2016,"error":{"code":-32603,"message":"json: key must be a string","data":null}}`.
Expected: saved=true/tick0 and an archive retaining the relation records.
Both before-fix regressions were executed with the original diplomacy.rs.

## Repair and compatibility

Only FactionRelations.rows receives a serde adapter. Populated human-readable
maps encode as ordered `[key,value]` entries, keeping tuple direction, score
and sample count. Empty maps retain `{}` and legacy empty objects still load.
Duplicate pairs, malformed keys and nonempty legacy objects fail explicitly.
Binary map bytes stay unchanged, verified against the previous bincode layout.
Existing JSON could not contain populated tuple-key maps, so no populated
legacy JSON migration is needed. Older engines cannot read the newly populated
entry representation; this is a forward writer fix, not a backward-reader claim.
The bundle version stays5 because existing readable saves retain their shape.

Other populated tuple maps, ECS civilian restoration, fixture transport,
Wasmtime and quality-policy changes remain outside this candidate. No full
persistence, full playthrough, platform matrix or authenticated identity
acceptance is claimed. No production authorization changes are included.

## Exact focused commands

From `.worktrees/save-slot-tuple-keys`:

```bash
source /workspace/.onboarding/env.sh
export CARGO_TARGET_DIR=/workspace/Civis/target
cargo test --locked --offline -p civ-engine --lib relation_rows_tests -j2
cargo test --locked --offline -p civ-engine --test save_diplomacy_json -j2
cargo test --locked --offline -p civ-server --lib jsonrpc_save_slot -j2
git diff --check
```

Rust/Cargo1.99.0; two jobs; incremental, dev/test debug symbols off. The focused
CI workflow runs the same named tests with locked dependencies on Ubuntu24.04.
Local result: four compatibility tests, one archive regression and two save.slot
handler tests pass. The two new persistence regressions fail before the repair
and pass afterward. Local raw logs are under `/tmp/civis-save-evidence/`; CI
provides independent receipts after publication. Full workspace checks were not
run for this scoped serialization adapter; no RPC or snapshot shape is changed.
