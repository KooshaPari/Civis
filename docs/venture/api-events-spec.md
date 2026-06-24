# Venture API Events Spec

Source mirror: `./venture/API_EVENTS_SPEC.md`

# Venture-Autonomy API & Event Spec

## API Domains
1. Policy management.
2. Workflow orchestration.
3. Artifact generation.
4. Treasury authorization.
5. Compliance/privacy operations.

## Representative Endpoints
1. `POST /policies/publish`
2. `POST /workflows`
3. `POST /workflows/{id}/run`
4. `POST /money/intents`
5. `POST /money/authorize`
6. `GET /compliance/cases`
7. `POST /privacy/requests`

## Event Topics
1. `policy.published.v1`
2. `workflow.started.v1`
3. `task.completed.v1`
4. `artifact.generated.v1`
5. `money.intent.created.v1`
6. `money.authorization.decided.v1`
7. `ledger.entry.created.v1`
8. `compliance.violation.detected.v1`
9. `privacy.request.received.v1`

## Event Envelope
1. `event_id`
2. `event_type`
3. `workflow_id`
4. `policy_version`
5. `payload`
6. `trace_id`
7. `created_at`
