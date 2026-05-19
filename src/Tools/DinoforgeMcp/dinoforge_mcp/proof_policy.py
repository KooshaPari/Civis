"""Policy file loader + validator + evaluator (smart-contract proof system, spec section 5).

Three layers:

1. ``load_policy(yaml_path)``  -> :class:`ProofPolicy`             — parse + shape-validate YAML.
2. ``is_judge_forbidden(...)``                                     — single-judge glob check.
3. ``evaluate(bundle, policy, *, now=None)`` -> :class:`EvaluateResult`
                                                                   — full per-feature gate evaluation.

The evaluator is what ``.github/workflows/proof-gate.yml`` calls in strict mode. It loads a
BundleManifest dict (already parsed) and produces an aggregate pass/fail plus per-feature
detail. Signature verification is delegated to :mod:`proof_signing.verify_blob`.

CLI entrypoint::

    python -m dinoforge_mcp.proof_policy evaluate <bundle.json> <policy.yaml>

Exit codes for the CLI: 0 = PASS, 1 = FAIL (any feature failed), 2 = usage / IO error.
"""
from __future__ import annotations
import fnmatch
import json
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable, Optional

import yaml  # PyYAML — already a transitive dep via existing tooling


@dataclass
class FeaturePolicy:
    name: str
    description: str = ""
    required_judge: str = ""
    forbidden_judges: list[str] = field(default_factory=list)
    required_artifacts: list[str] = field(default_factory=list)
    require_bridge_receipt: bool = True
    require_external_judge: bool = True
    max_age_seconds: int = 86400


@dataclass
class ProofPolicy:
    version: str
    policy_id: str
    description: str = ""
    forbidden_judges_global: list[str] = field(default_factory=list)
    features: dict[str, FeaturePolicy] = field(default_factory=dict)


def load_policy(yaml_path: Path) -> ProofPolicy:
    """Load policy YAML. Raises ValueError on malformed shape."""
    yaml_path = Path(yaml_path)
    with yaml_path.open("r", encoding="utf-8") as f:
        data = yaml.safe_load(f)

    if not isinstance(data, dict):
        raise ValueError(f"policy yaml is not a mapping: {yaml_path}")
    if "version" not in data or "policy_id" not in data or "features" not in data:
        raise ValueError(f"policy yaml missing required keys: {yaml_path}")

    features = {}
    for name, raw in (data.get("features") or {}).items():
        features[name] = FeaturePolicy(
            name=name,
            description=raw.get("description", ""),
            required_judge=raw.get("required_judge", ""),
            forbidden_judges=list(raw.get("forbidden_judges") or []),
            required_artifacts=list(raw.get("required_artifacts") or []),
            require_bridge_receipt=bool(raw.get("require_bridge_receipt", True)),
            require_external_judge=bool(raw.get("require_external_judge", True)),
            max_age_seconds=int(raw.get("max_age_seconds", 86400)),
        )

    return ProofPolicy(
        version=data["version"],
        policy_id=data["policy_id"],
        description=data.get("description", ""),
        forbidden_judges_global=list(data.get("forbidden_judges_global") or []),
        features=features,
    )


def is_judge_forbidden(judge_model: str, policy: ProofPolicy, feature_name: Optional[str] = None) -> bool:
    """Check whether a judge model is on the forbidden list (global or feature-specific).

    Glob-matches: 'claude-*' matches 'claude-haiku-4-5'.
    """
    if not judge_model:
        return True
    patterns = list(policy.forbidden_judges_global)
    if feature_name and feature_name in policy.features:
        patterns.extend(policy.features[feature_name].forbidden_judges)
    for pat in patterns:
        if fnmatch.fnmatchcase(judge_model.lower(), pat.lower()):
            return True
    return False


# ---------------------------------------------------------------------------
# Evaluator (Phase 2: spec section 8)
# ---------------------------------------------------------------------------

@dataclass
class FeatureResult:
    feature: str
    passed: bool
    violations: list[str] = field(default_factory=list)
    matched_judge: Optional[str] = None
    bridge_receipts_seen: int = 0


@dataclass
class EvaluateResult:
    passed: bool
    bundle_id: str
    policy_id: str
    violations: list[str] = field(default_factory=list)
    feature_results: dict[str, FeatureResult] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {
            "ok": self.passed,
            "bundle_id": self.bundle_id,
            "policy_id": self.policy_id,
            "violations": list(self.violations),
            "features": {
                name: {
                    "passed": fr.passed,
                    "violations": list(fr.violations),
                    "matched_judge": fr.matched_judge,
                    "bridge_receipts_seen": fr.bridge_receipts_seen,
                }
                for name, fr in self.feature_results.items()
            },
        }


def _parse_iso8601(s: str) -> Optional[datetime]:
    """Parse a timestamp string. Accepts trailing 'Z' (UTC). Returns None on failure."""
    if not s:
        return None
    try:
        # Python <3.11 doesn't accept 'Z' suffix in fromisoformat
        if s.endswith("Z"):
            s = s[:-1] + "+00:00"
        dt = datetime.fromisoformat(s)
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return dt
    except ValueError:
        return None


def _judge_endpoint_is_external(endpoint: Optional[str]) -> bool:
    """An 'external' judge is anything not hosted on anthropic.com.

    Empty / missing endpoint => NOT external (be conservative; missing data is a violation).
    """
    if not endpoint:
        return False
    lower = endpoint.lower()
    # Reject anthropic family hosts
    bad_hosts = ("anthropic.com", "api.anthropic.com")
    return not any(host in lower for host in bad_hosts)


def _coerce_judges(bundle: dict) -> list[dict]:
    """Pull JudgeReceipt-shaped dicts out of a bundle.

    A JudgeReceipt has a top-level ``kind == 'JudgeReceipt'`` and a ``subject`` mapping
    with at least ``feature_id`` and ``judge_model``. The bundle may carry receipts under
    either ``judges`` (denormalized list of receipt dicts) or via ``judge_receipts`` (list
    of relative paths — caller must hydrate). The evaluator accepts the denormalized form
    for self-contained verification.
    """
    raw = bundle.get("judges") or bundle.get("judge_receipts_inline") or []
    out: list[dict] = []
    for entry in raw:
        if isinstance(entry, dict):
            out.append(entry)
    return out


def _coerce_bridges(bundle: dict) -> list[dict]:
    raw = bundle.get("bridges") or bundle.get("bridge_receipts_inline") or []
    out: list[dict] = []
    for entry in raw:
        if isinstance(entry, dict):
            out.append(entry)
    return out


def _required_judge_matches(judge_model: str, required: str) -> bool:
    """The policy's ``required_judge`` is a single string (per current schema).

    A bundle judge_model "passes" the requirement if either:
      - the names are equal (case-insensitive), OR
      - the bundle name STARTS WITH the required name (case-insensitive)
        — e.g. ``required_judge: moonshot`` is satisfied by ``moonshot-v1-128k``.
    """
    if not required or not judge_model:
        return False
    j = judge_model.lower()
    r = required.lower()
    return j == r or j.startswith(r + "-") or j.startswith(r)


def evaluate(bundle: dict, policy: ProofPolicy, *, now: Optional[datetime] = None) -> EvaluateResult:
    """Evaluate a parsed BundleManifest dict against a loaded policy.

    Per spec section 8:
      - For every feature in policy:
          - At least one JudgeReceipt with subject.feature_id == feature must exist.
          - That receipt's judge_model must satisfy ``required_judge``.
          - That receipt's judge_model must NOT match any ``forbidden_judges`` glob
            (global or feature-specific).
          - If ``require_external_judge``: judge_endpoint must NOT be anthropic.com.
          - Receipt timestamp must be within ``max_age_seconds`` of *now*.
          - If ``require_bridge_receipt``: at least one BridgeReceipt with the same
            ``feature_id`` must exist.
          - Every path in ``required_artifacts`` must appear in ``manifest.leaves``.

    Bundle shape (relevant fields)::

        {
          "bundle_id": "...",
          "policy_id": "...",
          "leaves": [{"path": "validate_f9.png", "sha256": "..."}, ...],
          "judges":  [JudgeReceipt, ...],          # inline JudgeReceipts (subject contains feature_id)
          "bridges": [BridgeReceipt, ...],         # inline BridgeReceipts (subject contains feature_id)
        }

    *now* defaults to ``datetime.now(timezone.utc)``. Unparseable receipt timestamps
    fail the freshness check.
    """
    if now is None:
        now = datetime.now(timezone.utc)

    bundle_id = str(bundle.get("bundle_id", ""))
    policy_id = policy.policy_id
    bundle_policy_id = bundle.get("policy_id")

    violations: list[str] = []
    if bundle_policy_id and bundle_policy_id != policy_id:
        violations.append(
            f"bundle.policy_id ({bundle_policy_id!r}) does not match active policy ({policy_id!r})"
        )

    leaf_paths: set[str] = {
        str(l.get("path"))
        for l in (bundle.get("leaves") or [])
        if isinstance(l, dict) and l.get("path")
    }
    judges = _coerce_judges(bundle)
    bridges = _coerce_bridges(bundle)

    feature_results: dict[str, FeatureResult] = {}
    for name, fp in policy.features.items():
        fr = FeatureResult(feature=name, passed=False)

        # 1. required artifacts
        for art in fp.required_artifacts:
            if art not in leaf_paths:
                fr.violations.append(f"required artifact missing: {art}")

        # 2. find candidate judge receipts for this feature
        candidates = [
            j for j in judges
            if isinstance(j.get("subject"), dict)
            and j["subject"].get("feature_id") == name
        ]
        if not candidates:
            fr.violations.append(f"no JudgeReceipt found for feature {name!r}")
        else:
            matched = None
            for j in candidates:
                subject = j.get("subject", {})
                model = subject.get("judge_model") or ""
                endpoint = subject.get("judge_endpoint") or ""
                ts_str = j.get("timestamp_utc") or subject.get("timestamp_utc") or ""

                # forbidden glob match
                if is_judge_forbidden(model, policy, feature_name=name):
                    fr.violations.append(
                        f"judge {model!r} matches forbidden_judges (feature {name})"
                    )
                    continue

                # required name match
                if not _required_judge_matches(model, fp.required_judge):
                    fr.violations.append(
                        f"judge {model!r} does not match required_judge {fp.required_judge!r}"
                    )
                    continue

                # external judge
                if fp.require_external_judge and not _judge_endpoint_is_external(endpoint):
                    fr.violations.append(
                        f"judge endpoint {endpoint!r} is not external (require_external_judge=true)"
                    )
                    continue

                # freshness
                ts = _parse_iso8601(ts_str)
                if ts is None:
                    fr.violations.append(f"judge receipt missing/unparseable timestamp_utc: {ts_str!r}")
                    continue
                age = (now - ts).total_seconds()
                if age < 0:
                    fr.violations.append(
                        f"judge receipt timestamp is in the future: {ts_str}"
                    )
                    continue
                if age > fp.max_age_seconds:
                    fr.violations.append(
                        f"judge receipt stale: age={int(age)}s > max_age_seconds={fp.max_age_seconds}"
                    )
                    continue

                matched = model
                break

            if matched:
                fr.matched_judge = matched
            else:
                if not any("matches forbidden_judges" in v or "does not match required_judge" in v
                           or "not external" in v or "stale" in v
                           or "missing/unparseable timestamp" in v
                           or "timestamp is in the future" in v
                           for v in fr.violations):
                    fr.violations.append(f"no acceptable judge receipt for feature {name!r}")

        # 3. bridge receipt
        if fp.require_bridge_receipt:
            matching_bridges = [
                b for b in bridges
                if isinstance(b.get("subject"), dict)
                and b["subject"].get("feature_id") == name
            ]
            fr.bridge_receipts_seen = len(matching_bridges)
            if not matching_bridges:
                fr.violations.append(
                    f"require_bridge_receipt=true but no BridgeReceipt found for feature {name!r}"
                )

        fr.passed = len(fr.violations) == 0
        feature_results[name] = fr

    overall = (not violations) and all(fr.passed for fr in feature_results.values())
    return EvaluateResult(
        passed=overall,
        bundle_id=bundle_id,
        policy_id=policy_id,
        violations=violations,
        feature_results=feature_results,
    )


# ---------------------------------------------------------------------------
# CLI: `python -m dinoforge_mcp.proof_policy evaluate <bundle.json> <policy.yaml>`
# Returns exit 0 on PASS, 1 on FAIL, 2 on IO/usage error.
# JSON is written to stdout regardless.
# ---------------------------------------------------------------------------
def _cli_evaluate(bundle_path: Path, policy_path: Path) -> int:
    try:
        bundle = json.loads(Path(bundle_path).read_text(encoding="utf-8"))
    except FileNotFoundError as e:
        print(json.dumps({"ok": False, "error": f"bundle not found: {e}"}), file=sys.stderr)
        return 2
    except json.JSONDecodeError as e:
        print(json.dumps({"ok": False, "error": f"bundle JSON parse error: {e}"}), file=sys.stderr)
        return 2

    try:
        policy = load_policy(policy_path)
    except (FileNotFoundError, ValueError) as e:
        print(json.dumps({"ok": False, "error": f"policy load failed: {e}"}), file=sys.stderr)
        return 2

    result = evaluate(bundle, policy)
    payload = result.to_dict()
    print(json.dumps(payload, indent=2, sort_keys=True))
    # Exit code: this line is what tells the workflow to fail on violation.
    return 0 if result.passed else 1


def _main(argv: list[str]) -> int:
    import argparse

    parser = argparse.ArgumentParser(prog="dinoforge_mcp.proof_policy")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p_eval = sub.add_parser("evaluate", help="evaluate a bundle against the policy")
    p_eval.add_argument("bundle", type=Path)
    p_eval.add_argument("policy", type=Path)
    args = parser.parse_args(argv)

    if args.cmd == "evaluate":
        return _cli_evaluate(args.bundle, args.policy)
    return 2  # pragma: no cover


if __name__ == "__main__":
    sys.exit(_main(sys.argv[1:]))
