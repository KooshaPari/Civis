# Venture Schema Pack

Source mirror: `./venture/SCHEMA_PACK.md`

# Venture Core Schema Pack

## Event Envelope v1
1. `event_id` (UUID)
2. `event_type` (topic string)
3. `workflow_id` (UUID)
4. `policy_version` (string)
5. `trace_id` (string)
6. `payload` (object)
7. `created_at` (ISO-8601)

## Task Envelope v1
1. `task_id` (UUID)
2. `workflow_id` (UUID)
3. `agent_role` (enum)
4. `task_type` (enum)
5. `input` (object)
6. `created_at` (ISO-8601)

## FSM Pack (minimum)
1. Approval FSM: `pending -> approved|denied|expired`.
2. Payout FSM: `intent_created -> authorized -> executed -> reconciled|disputed`.
3. Compliance FSM: `open -> investigating -> remediated|escalated|waived`.
4. Kill-switch FSM: `active -> frozen -> recovering -> active`.

## Validation Rules
1. Reject unknown schema versions.
2. Reject events without trace/workflow linkage.
3. Reject side-effect transitions without prior authorization state.
