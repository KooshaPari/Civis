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
