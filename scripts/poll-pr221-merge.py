#!/usr/bin/env python3
"""Poll PR 221 for CodeRabbit approval and merge when ready."""
import json
import subprocess
import sys
import time

REPO = "KooshaPari/Dino"
PR = "221"
INTERVAL = 90
MAX_WAIT = 15 * 60


def run(cmd):
    return subprocess.check_output(cmd, text=True, stderr=subprocess.STDOUT)


def pr_status():
    out = run(
        [
            "gh",
            "pr",
            "view",
            PR,
            "--repo",
            REPO,
            "--json",
            "reviewDecision,mergeStateStatus,headRefOid",
        ]
    )
    return json.loads(out)


def coderabbit_reviews():
    out = run(["gh", "api", f"repos/{REPO}/pulls/{PR}/reviews"])
    reviews = json.loads(out)
    return [
        {
            "state": r["state"],
            "submitted_at": r.get("submitted_at"),
            "id": r["id"],
        }
        for r in reviews
        if r.get("user", {}).get("login") == "coderabbitai[bot]"
    ]


def ready(status, cr):
    if status.get("reviewDecision") == "APPROVED":
        return True, "reviewDecision"
    for r in cr:
        if r.get("state") == "APPROVED":
            return True, "coderabbit_review"
    return False, None


def main():
    start = time.time()
    attempt = 0
    last_status = None
    last_cr = []

    while time.time() - start < MAX_WAIT:
        attempt += 1
        status = pr_status()
        cr = coderabbit_reviews()
        last_status = status
        last_cr = cr
        ok, reason = ready(status, cr)
        print(
            json.dumps(
                {
                    "attempt": attempt,
                    "elapsed_s": int(time.time() - start),
                    "status": status,
                    "coderabbit": cr,
                    "ready": ok,
                    "reason": reason,
                }
            ),
            flush=True,
        )
        if ok:
            try:
                merge_out = run(
                    [
                        "gh",
                        "pr",
                        "merge",
                        PR,
                        "--merge",
                        "--repo",
                        REPO,
                    ]
                )
                print(json.dumps({"merged": True, "merge_output": merge_out.strip()}))
                return 0
            except subprocess.CalledProcessError as e:
                print(json.dumps({"merged": False, "merge_error": e.output}))
                try:
                    wf = run(
                        [
                            "gh",
                            "workflow",
                            "run",
                            "agent-merge-on-bot-approve.yml",
                            "--repo",
                            REPO,
                            "--ref",
                            "followup/post-pr188-followups",
                            "-f",
                            "pr_number=221",
                        ]
                    )
                    print(json.dumps({"workflow_triggered": True, "output": wf.strip()}))
                except subprocess.CalledProcessError as e2:
                    print(json.dumps({"workflow_triggered": False, "error": e2.output}))
                return 1

        remaining = MAX_WAIT - (time.time() - start)
        if remaining <= 0:
            break
        time.sleep(min(INTERVAL, remaining))

    print(
        json.dumps(
            {
                "merged": False,
                "timeout": True,
                "final_status": last_status,
                "final_coderabbit": last_cr,
            }
        )
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
