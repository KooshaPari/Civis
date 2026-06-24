# Venture Data Model DB Spec

Source mirror: `./venture/DATA_MODEL_DB_SPEC.md`

# Venture-Autonomy Data/DB Spec

## Core Entities
1. `policy_bundle`
2. `workflow`
3. `task`
4. `agent_action`
5. `money_intent`
6. `authorization_decision`
7. `ledger_entry`
8. `compliance_case`
9. `privacy_request`

## Relational Model (proposed)
1. `policy_bundles(id, version, content_hash, status, created_at)`
2. `workflows(id, objective, policy_bundle_id, status, created_at)`
3. `tasks(id, workflow_id, type, schema_version, status, retries, created_at)`
4. `agent_actions(id, task_id, agent_role, tool, input_hash, output_hash, created_at)`
5. `money_intents(id, workflow_id, amount_cents, currency, merchant_scope, ttl, status, created_at)`
6. `authorization_decisions(id, money_intent_id, decision, reason_code, created_at)`
7. `ledger_entries(id, workflow_id, entry_type, amount_cents, external_ref, created_at)`
8. `compliance_cases(id, workflow_id, policy_rule, severity, status, created_at)`
9. `privacy_requests(id, subject_ref, request_type, status, due_at, created_at)`

## Invariants
1. Every external side effect maps to an event and DB record.
2. Money actions require prior `money_intent` + decision chain.
3. Immutable history for ledger and compliance decisions.
4. Policy bundle version pinned for each workflow run.
