# Venture Track B Treasury + Compliance

Source mirror: `./venture/TRACK_B_TREASURY_COMPLIANCE_SPEC.md`

# Venture Track B Treasury, Ledger, and Compliance Closure

## Scope
Closes Track B gaps for money policy, ledger/reconciliation model, and compliance/privacy controls.

## Money Authorization Model
1. Default-deny for all spend attempts.
2. Spend envelope requires `money_intent` with TTL, vendor/MCC scope, budget class, workflow binding.
3. Authorization decision requires policy bundle evaluation and reason code.
4. Revocation is immediate and emits freeze events for downstream workflows.

## Ledger and Reconciliation
1. Every approved spend creates immutable ledger entries.
2. External processor references are linked to internal ledger IDs.
3. Daily reconciliation compares internal ledger, processor exports, and bank statements.
4. Drift above threshold opens compliance case automatically.

## Compliance and Privacy Controls
1. Jurisdiction gates on outreach, payment, and data export actions.
2. Machine-readable policy packs for tax, retention, suppression, and do-not-sell/share.
3. DSAR/deletion workflows tracked with due dates and status state machine.
4. Audit trail is append-only and includes policy version on every decision.

## Data/DB Additions
1. `money_limits(id, scope_type, scope_id, cap_amount, window, status, created_at)`
2. `reconciliation_runs(id, period_start, period_end, drift_amount, status, created_at)`
3. `policy_attestations(id, policy_bundle_id, attestor, result, created_at)`
4. `privacy_classifications(id, entity_ref, class, sharing_default, created_at)`

## Events
1. `money.limit.updated.v1`
2. `money.authorization.revoked.v1`
3. `ledger.reconciliation.completed.v1`
4. `compliance.policy.attested.v1`
5. `privacy.classification.updated.v1`

## Acceptance Checks
1. Unauthorized spend attempts are denied and logged with reason code.
2. Reconciliation drift is detected deterministically and creates case records.
3. Policy pack updates propagate to authorization/compliance decisions without silent fallback.
