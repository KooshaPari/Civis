"""CLI shim for proof_policy — invoked by prove-features-gate.ps1 (Phase 2).

Usage:
    python -m dinoforge_mcp.proof_policy_cli load <yaml_path>
    python -m dinoforge_mcp.proof_policy_cli check_judge <model> <yaml_path> [<feature>]
    python -m dinoforge_mcp.proof_policy_cli list_features <yaml_path>

Exit codes:
    0 = ok / allowed
    1 = forbidden / policy violation
    2 = usage / IO error

All output is JSON to stdout. Errors go to stderr.
"""
from __future__ import annotations
import argparse
import json
import sys
from dataclasses import asdict
from pathlib import Path

from dinoforge_mcp import proof_policy


def _policy_to_dict(p: proof_policy.ProofPolicy) -> dict:
    d = {
        "version": p.version,
        "policy_id": p.policy_id,
        "description": p.description,
        "forbidden_judges_global": list(p.forbidden_judges_global),
        "features": {name: asdict(fp) for name, fp in p.features.items()},
    }
    return d


def _cli(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="dinoforge_mcp.proof_policy_cli")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_load = sub.add_parser("load", help="load + dump policy as JSON")
    p_load.add_argument("yaml_path", type=Path)

    p_chk = sub.add_parser("check_judge", help="check whether a judge model is forbidden")
    p_chk.add_argument("model")
    p_chk.add_argument("yaml_path", type=Path)
    p_chk.add_argument("feature", nargs="?", default=None)

    p_list = sub.add_parser("list_features", help="list policy feature names")
    p_list.add_argument("yaml_path", type=Path)

    args = parser.parse_args(argv)

    try:
        policy = proof_policy.load_policy(args.yaml_path)
    except (FileNotFoundError, ValueError) as e:
        print(json.dumps({"ok": False, "error": f"policy load failed: {e}"}), file=sys.stderr)
        return 2

    if args.cmd == "load":
        print(json.dumps({"ok": True, "policy": _policy_to_dict(policy)}))
        return 0

    if args.cmd == "check_judge":
        forbidden = proof_policy.is_judge_forbidden(args.model, policy, feature_name=args.feature)
        out = {
            "ok": not forbidden,
            "model": args.model,
            "feature": args.feature,
            "forbidden": forbidden,
        }
        if forbidden:
            out["reason"] = f"judge {args.model!r} matches forbidden pattern"
        print(json.dumps(out))
        return 1 if forbidden else 0

    if args.cmd == "list_features":
        print(json.dumps({"ok": True, "features": list(policy.features.keys())}))
        return 0

    return 2  # pragma: no cover


if __name__ == "__main__":
    sys.exit(_cli(sys.argv[1:]))
