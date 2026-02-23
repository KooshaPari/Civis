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
