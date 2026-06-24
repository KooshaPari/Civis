# Spec Completion TODOs #2 (No-HITL Venture/Artifact Stack)

1. Define canonical no-HITL architecture document (`control plane`, `artifact compilers`, `treasury`, `compliance`, `event bus`).
2. Define artifact IR contracts and versioning/migration policy.
3. Define deterministic build/replay contract (idempotency keys, cache keys, provenance signatures).
4. Define Veo/NanoBanana scene compiler schema and fallback policy by target quality tier.
5. Define Money API policy model (default-deny, spend envelopes, MCC/vendor lock, TTL, revocation).
6. Define minimal financial ledger + reconciliation/event model.
7. Define compliance policy pack (tax, outreach, data retention, suppression, jurisdiction gates).
8. Define identity model (workload identity + capability credentials + revocation flow).
9. Define tool permission model per agent role with hard constraints and budget limits.
10. Define core event schema pack + FSM pack (approval, dispute, payout, freeze/shutdown).
11. Define privacy ops schema pack (DSAR, deletion, do-not-sell/share, third-party classification).
12. Define verification harness (schema validation, policy tests, adversarial prompt-injection tests).
13. Define staged rollout path (sandbox -> limited autopilot -> governed autonomy -> full no-HITL).
14. Define hard kill-switch and incident response doctrine for autonomous spend/outreach/deploy actions.
15. Define monthly governance cadence (spec changes, risk review, model drift review, compliance attestations).

## Closure Mapping (2026-02-21)
1. Canonical no-HITL architecture and control plane: `./venture/TRACK_C_CONTROL_PLANE.md` and `./venture/IMPLEMENTATION_ROADMAP.md`.
2. Artifact IR contracts + deterministic build/replay + Veo/NanoBanana schema: `./venture/TRACK_A_ARTIFACT_DETERMINISM_SPEC.md`.
3. Money API, ledger, reconciliation, compliance, privacy ops: `./venture/TRACK_B_TREASURY_COMPLIANCE_SPEC.md`.
4. Identity/tool permission/event+FSM/privacy/verification/rollout/incident/governance cadence: `./venture/TRACK_C_CONTROL_PLANE.md`, `./venture/ROLE_TOOL_ALLOWLIST_MATRIX.md`, `./venture/SCHEMA_PACK.md`, and `./venture/OPS_COMPLIANCE_SPEC.md`.
