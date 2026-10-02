# Civis pass 16 — save-manifest integration boundary

Date 2026-09-30.

The test-only semantic manifest prototype deliberately covers only the three reproduced losses. Production integration must preserve ownership distinctions.

## Economy policy has two policy concepts

`policy.rs` explicitly distinguishes:
1. scenario economy `PolicyInput` consumed by phase_economy;
2. control `Policy` trait producing ControlSignals.

The reproduced red concerns `economy_policy: PolicyInput`. Persisting only that value does not qualify the active control-policy object or last_control_signals. Those remain separate state-manifest rows.

## Mod identity must come before guest-memory import

Current ModHost exposes loaded manifests with:
- stable id;
- version string;
- api_version;
- type/author and dependency/permission data.

The prototype records id/version/api_version only as the minimal compatibility identity. Mature production identity should also bind the exact artifact/content digest and relevant capability/dependency resolution. A version string alone is not immutable evidence.

Load order should be:
1. parse outer save/component manifest;
2. resolve required mod artifacts/profile;
3. validate API/version/dependency/capability compatibility;
4. establish active ModHost set;
5. only then import matching guest-memory blobs;
6. orphan/incompatible memory becomes explicit degraded/refused/migration state.

## Research state

Engine ResearchCache is already Serialize/Deserialize and semantically small (researched + queued). That makes it a good independent component rather than burying it inside a generic WorldState dump.

## Migration boundary

Prototype should become a versioned semantic component only after:
- red controls remain red on old path;
- prototype green proves policy/research restoration + orphan detection;
- v5 adapter behavior is specified;
- missing component, incompatible mod and mixed-world fixtures exist;
- interrupted publication leaves original v5 bytes recoverable.

No production CivSaveBundle change yet.
