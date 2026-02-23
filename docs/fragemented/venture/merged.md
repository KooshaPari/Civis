# Merged Fragmented Markdown

## Source: venture/api-events-spec.md

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


---

## Source: venture/data-model-db-spec.md

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


---

## Source: venture/implementation-roadmap.md

# Venture Implementation Roadmap

Source mirror: `./venture/IMPLEMENTATION_ROADMAP.md`

# Venture Implementation Roadmap

## Phase 1: Sandbox Baseline
1. Freeze IR schema family and publish schema versions.
2. Implement deterministic build keys and provenance records.
3. Enable default-deny treasury with reasoned decisions.
4. Stand up event envelope validation and policy bundle pinning.

## Phase 2: Limited Autopilot
1. Enable scoped role/tool allowlists by workflow class.
2. Activate reconciliation and compliance attestation cadence.
3. Add incident class routing and freeze control tests.
4. Gate rollout with deterministic replay pass rate.

## Phase 3: Governed Autonomy
1. Expand workflow classes under budget envelopes.
2. Tighten policy drift detection and monthly governance reviews.
3. Enforce privacy ops SLAs (DSAR/deletion/suppression).
4. Validate operational resilience with incident drills.

## Exit Criteria
1. No unapproved side-effect action path.
2. 100% event envelope compliance on external effects.
3. Reconciliation and audit checks pass at required cadence.


---

## Source: venture/ops-compliance-spec.md

# Venture Ops Compliance Spec

Source mirror: `./venture/OPS_COMPLIANCE_SPEC.md`

# Venture-Autonomy Ops/Compliance Spec

## Operations Controls
1. Budget ceilings by workflow, agent role, and time window.
2. Retry and timeout doctrine by task class.
3. Freeze mode and global kill-switch behavior.
4. Incident classes and escalation paths.

## Compliance Controls
1. Outreach/legal policy engine (channel/jurisdiction checks).
2. Tax/reporting event capture and retention.
3. Vendor trust and milestone proof checks.
4. Data-sharing classification and suppression defaults.

## Security Controls
1. Workload identity, mTLS, and credential rotation.
2. Tool capability restrictions and egress policy.
3. Prompt-injection resistant action gates.
4. Tamper-evident logs and periodic integrity attestations.

## Audits
1. Daily reconciliation checks.
2. Weekly policy drift review.
3. Monthly governance and control attestation.
4. Quarterly compliance tabletop and incident drill.


---

## Source: venture/role-tool-allowlist-matrix.md

# Venture Role Tool Allowlist Matrix

Source mirror: `./venture/ROLE_TOOL_ALLOWLIST_MATRIX.md`

# Venture Role Tool Allowlist Matrix

1. `orchestrator`: `workflow.dispatch`, `policy.evaluate`, `event.publish`, `io.read`.
2. `researcher`: `web.fetch`, `io.read`, `artifact.render`.
3. `solver`: `code.exec`, `io.read`, `io.write`, `event.publish`.
4. `finance-controller`: `money.intent.create`, `money.authorize`, `ledger.reconcile`, `event.publish`.
5. `ops-auditor`: `io.read`, `event.query`, `compliance.case.review`.

Policy rules:
1. Default deny across all roles.
2. Budget and TTL envelopes required for money tools.
3. Sensitive tools require policy bundle pin + trace ID.


---

## Source: venture/schema-pack.md

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


---

## Source: venture/technical-spec.md

# Venture Technical Spec

Source mirror: `./venture/TECHNICAL_SPEC.md`

# Venture-Autonomy Technical Spec (v1)

## Control Plane Components
1. Workflow orchestrator (durable execution).
2. Agent runtime/router.
3. Tool permission and capability policy engine.
4. Money API (authorization/limits/ledger boundary).
5. Schema registry and event bus.
6. Compliance policy evaluator.

## Core Architectural Pattern
1. Intent -> validated task schema -> workflow DAG.
2. Agent actions only through constrained tools.
3. External effects require policy checks and signed events.
4. Every state transition emits auditable events.

## Artifact Compiler Subsystem
1. IR specs: `SlideSpec`, `DocSpec`, `TimelineSpec`, `AudioSpec`, `BoardSpec`.
2. Build pipeline: spec -> render -> validate -> export.
3. Provenance metadata attached to outputs.

## Money Control Subsystem
1. Default-deny spend model.
2. Intent-scoped authorization and caps.
3. Merchant/category and velocity controls.
4. Idempotent payment events and reconciliation.

## Security Requirements
1. Workload identity and short-lived credentials.
2. Strict tool allowlists per agent role.
3. External-content isolation and prompt-injection defenses.
4. Tamper-evident event logs.


---

## Source: venture/track-a.md

# Venture Track A Artifact + Determinism

Source mirror: `./venture/TRACK_A_ARTIFACT_DETERMINISM_SPEC.md`

# Venture Track A Artifact IR and Determinism Closure

## Scope
Closes Track A gaps for artifact contracts, deterministic build/replay, and Veo/NanoBanana compiler behavior.

## Artifact IR Family
1. `SlideSpec`: deck layout, slide graph, style tokens, source references.
2. `DocSpec`: sections, constraints, citations, output channels.
3. `TimelineSpec`: scenes, timing, transitions, narration anchors.
4. `AudioSpec`: voice config, script segments, timing, loudness profile.
5. `BoardSpec`: whiteboard objects, connectors, layering, animation steps.

All IR objects require: `schema_version`, `content_hash`, `inputs_hash`, `policy_bundle_id`, `created_at`.

## Deterministic Build/Replay Contract
1. Idempotency key: hash of `(ir_hash, toolchain_version, policy_bundle_id, target_surface)`.
2. Cache key equals idempotency key plus explicit renderer version.
3. Provenance signature emitted for each export artifact.
4. Replay must reproduce byte-identical outputs when toolchain and dependencies are pinned.

## Veo/NanoBanana Scene Compiler Contract
1. `TimelineSpec -> scene plan -> provider prompt pack -> render jobs -> verification`.
2. Provider fallback order is policy-driven by quality tier and budget envelope.
3. All provider calls emit signed provenance and event records.
4. Non-deterministic providers require artifact fingerprint plus semantic-equivalence validator.

## Data/DB Additions
1. `artifact_ir(id, ir_type, schema_version, content_hash, payload_json, created_at)`
2. `artifact_builds(id, ir_id, idempotency_key, toolchain_version, status, created_at)`
3. `artifact_provenance(id, build_id, provider, model, signature, created_at)`

## Events
1. `artifact.ir.registered.v1`
2. `artifact.build.started.v1`
3. `artifact.build.completed.v1`
4. `artifact.provenance.attested.v1`
5. `artifact.replay.verified.v1`

## Acceptance Checks
1. Schema validation passes for all IR families.
2. Deterministic replay passes for pinned toolchain builds.
3. Fallback routing obeys policy tier constraints and budget caps.

## Related Specs

- `ARTIFACT_COMPILER_SPEC.md` — Full Artifact Compiler System specification including IR schemas, compiler pipeline, validation engine, headless execution model, and multi-format export
- `TECHNICAL_SPEC.md` — Venture-Autonomy Control Plane (artifact compiler is a subsystem)
- `API_EVENTS_SPEC.md` — Event topics and envelope format for artifact build completion, validation, and export events


---

## Source: venture/track-b.md

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


---

## Source: venture/track-c.md

# Venture Track C Control Plane

Source mirror: `./venture/TRACK_C_CONTROL_PLANE.md`

# Venture Track C Control-Plane Closure

## Overview
Track C binds the Venture autonomy control plane into a closed system: authenticated identities, explicit isolation boundaries, policy-based tool access, tamper-evident events, and governed rollout/incident behavior. The deliverable below stitches the Venture specification sources with the civ/infra agent-workspace scaffolds so every decision is grounded in an auditable schema.

## Identity, Scoped Isolation, and Tool Allowlists
- **Workload identity**: follow the Technical Spec’s requirement for short-lived credentials and mTLS ("Workload identity and short-lived credentials" plus "Workload identity, mTLS, and credential rotation"). Every agent session must assert a pinned `agent_role` or `workspace_id` inside the Task Envelope schema (`task_id`, `workflow_id`, `agent_role`, `input`, `created_at`).
- **Isolation boundary**: enclave each agent in the role-specific workspace defined in `workspaces/default.yaml` (e.g., `civ-default`) with `max_concurrency` guardrails, `retry_policy`, and capped budgets (`global_eau_cap`, `per_workflow_eau_cap`). This ensures every run is time-boxed, budget-limited, and traceable before it touches external tools.
- **Tool allowlists**: enforce the civ/infra policy at `policy/tool-allowlists.yaml`. Each named role (or an overtime-managed alias) can only use the declared tools (`workflow.dispatch`, `policy.evaluate`, `event.publish`, `io.read`, `web.fetch`, `artifact.render`, etc.). Align any additional permissible tool with a change request that updates this YAML, accompanied by a new policy version in the Product/Operations specs.
- **Composition**: Combine schema validation with policy enforcement before handing tasks to agents: validate `task_type` against schemas, inject `trace_id` and `workflow_id`, shape `input` via the Task Envelope, then check the requested tool set against the role’s allowlist.

## Event Bus + FSM Pack
- **Event toxicity**: Use the Venture API Events list as your topic catalog (`policy.published.v1`, `workflow.started.v1`, etc.) and honor the Event Envelope schema (`event_id`, `event_type`, `trace_id`, `workflow_id`, `task_id`, `payload`, `created_at`). Emit every state transition from finite state machines (FSMs) as one of these events so the audit chain is uniform.
- **FSM pack**: model orchestration steps (workflow → task → agent action → artifact generation) as FSM states that fire events on entry/exit. Keep `trace_id` and `workflow_id` identical across `task.completed.v1`, `artifact.generated.v1`, `money.intent.created.v1`, etc., by stuffing them into the event payload per the schema and reusing them for retrieve/replay.
- **Schema binding**: before publishing events, assert the payload shape with the same policy-driven schema registry that the control plane uses (per Technical Spec’s "Schema registry and event bus"). Use `policy_bundle` versions (DB spec) to tie every emitted event to a known policy version for provenance.
- **Event consumer contracts**: document each listener's expected `payload` subset and any FSM guard transitions (e.g., the treasury FSM waits for `money.authorization.decided.v1` before dispatching `ledger.entry.created.v1`). Keep this doc synchronized with the API events spec to avoid silent drift.

## Observability and Audit
- **Mandatory context**: enforce `require_trace_id` and `require_event_envelope` as dictated by `workspaces/default.yaml` so no action is recorded without a unique `trace_id` and the `EventEnvelopeV1` context.
- **Tamper-evident logs**: mirror the Ops Compliance Spec’s requirement for tamper-evident logs and periodic integrity attestations. Pipe all events and DB writes through an append-only log (+ checksum) that records `created_at` (ISO 8601) and policy bundle version.
- **Audit cadence**: schedule daily reconciliation checks, weekly policy-drift reviews, monthly governance attestations, and quarterly tabletop incident drills (Ops Compliance Spec’s Audits section). Capture proof artifacts (log snapshot, policy version, event ID ranges) in a retrievable bucket for each cadence.
- **Observability layers**: instrument the FSM pack to emit metrics/alerts for state duration, tool invocation counts, policy evaluation latency, event publish success, and budget consumption (per workspace budget caps). Integrate with the Ops spec’s fiscal oversight surfaces (Treasury/Compliance dashboards) so the gaps are visible to Finance Controllers and Operations Auditors.

## Rollout Stages
1. **Sandbox verification** (pre-policy lock): run new policies and FSM transitions in a staging workspace, ensure `tool-allowlists` and event payloads validate against the schemas, and confirm budgets/trace IDs persist.
2. **Gate** (policy publish + policy bundle pinning): once compliance/finance owners sign-off, publish the policy via `POST /policies/publish` (API spec) and pin the bundle ID on the workflow. This stage still only uses orchestrator/researcher/solver roles under a manual approval gate.
3. **Controlled ramp** (limited workspace concurrency, monitoring, alerting): allow a subset of workflows (e.g., low-risk `task_type`s) to execute with the full FSM pack, event bus, audit logging, and budget enforcement. Observe incident metrics and escalate if thresholds exceed.
4. **Full launch** (policy fully enforced, budgets tuned): remove manual gates, allow necessary roles, and run the entire product model surface stack (control-plane editor, treasury firewall, compliance cockpit). Keep a shadow copy of budgets and audit proofs for retroactive compliance.
- Each stage should be recorded as an FSM transition and event; stage gates trigger `workflow.started.v1` or bespoke `rollout.stage.changed.v1` events.

## Incident Doctrine
- **Incident classes & escalation**: follow Ops controls—classify incidents into predefined buckets (example: policy violation, treasury drift, tool misuse, compliance/legal). Map each class to an escalation path (on-call, compliance SME, founder governor) and include predefined playbooks.
- **Freeze/kill switch**: align with Ops controls ("Freeze mode and global kill-switch behavior"). Upon a Class 1 breach (budget overrun, root policy violation), immediately pause orchestrator execution (retaining event logs), flip the kill switch, and inform the compliance channel.
- **Evidence capture**: when incidents fire, record at least one `event_id` per impacted FSM transition, the implicated `policy_bundle` version, and the workspace/budget context. These become the starting point for the audit drill schedule described above.
- **Postmortem**: require compliance/audit review and a policy bundle re-evaluation before reactivating any halted FSM flows. Document lessons learned via the `policy.evaluate` workflow, tie them to updated `policy_bundle` IDs, and disseminate to Controllers/Auditors.

## References & Next Steps
- Venture specs: `TECHNICAL_SPEC.md`, `API_EVENTS_SPEC.md`, `DATA_MODEL_DB_SPEC.md`, `OPS_COMPLIANCE_SPEC.md`, `PRODUCT_MODEL.md`, `USER_SPEC.md`.
- Scaffolds: `../civ/infra/agent-workspace/policy/tool-allowlists.yaml`, `../civ/infra/agent-workspace/schemas/task-envelope.v1.json`, `../civ/infra/agent-workspace/schemas/event-envelope.v1.json`, `../civ/infra/agent-workspace/workspaces/default.yaml`.
- Next: validate FSM transitions against tool allowlist updates, codify stage-gate event metrics, and bake incident playbooks into the policy bundle review flow.


---
