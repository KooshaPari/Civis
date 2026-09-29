"""Report any changed line in a git diff that is not a comment.

The requirement-tag removals are supposed to be comment-only: a `// FR-...` line
deleted and a `// [unbound] FR-...: ...` rationale added. If a diff touches real
code, that is a bug in the audit tooling or an unauthorized edit, and it is the
thing most likely to explain a test failure. Run it before blaming or crediting
a test result.

Usage:  python docs/audits/_check_comment_only.py [ref]        (default: HEAD)
"""
import io
import re
import subprocess
import sys

REF = sys.argv[1] if len(sys.argv) > 1 else "HEAD"

# A changed line is "comment-only" if, after stripping the diff marker and
# leading whitespace, it starts with // or /* or * or */. Blank lines are fine.
CODE = re.compile(r"^(\}|\)|\]|pub |fn |let |impl |struct |enum |const |static |use |mod |#\[|//[^/])")


def main():
    out = subprocess.run(
        ["git", "diff", "-U0", REF, "--", "crates/"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    ).stdout

    current = None
    suspicious = []
    for line in out.splitlines():
        if line.startswith("+++ b/"):
            current = line[6:]
            continue
        if not line or line[0] not in "+-" or line.startswith(("+++", "---")):
            continue
        body = line[1:].strip()
        if not body:
            continue
        if body.startswith(("//", "/*", "*", "*/")):
            continue
        suspicious.append((current, line[0], body[:100]))

    if not suspicious:
        print("PASS: every changed line under crates/ is a comment or blank.")
        return 0

    print(f"FAIL: {len(suspicious)} changed non-comment line(s) under crates/ vs {REF}")
    for path, sign, body in suspicious[:60]:
        print(f"  {path} {sign} {body}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
