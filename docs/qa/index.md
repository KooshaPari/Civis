# QA Index

This page collects the quality gates that matter for the current v0.26 wave.

## Gate Map

| Gate | Purpose | Canonical check |
|---|---|---|
| Test isolation | Prevent process-global test flake. | `docs/qa/test-isolation-policy.md` |
| Pattern coverage | Keep quality patterns aligned with enforcement. | `docs/qa/PATTERN_INDEX.md` |
| Schema validation | Catch invalid pack and schema edits early. | `docs/reference/schema-governance.md` |
| Release readiness | Ensure release notes match actual gates. | `docs/release/process.md` |

## Expected QA Artifacts

| Artifact | Use |
|---|---|
| Policy page | Defines the rule and why it exists. |
| Allowlist | Documents explicit exceptions. |
| Report | Captures the current audit result. |
| Test fixture | Proves the rule is enforced. |

## v0.26 Minimum Bar

| Area | Minimum evidence |
|---|---|
| Tests | A named gate or fixture exists for each claimed rule. |
| Schemas | Parse, reject, and round-trip cases are covered where relevant. |
| Release | No release step depends on undocumented manual knowledge. |
| Research | Any open research note is linked to a downstream owner. |

