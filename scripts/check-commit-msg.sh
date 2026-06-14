#!/bin/sh
# Conventional-commit validator invoked by lefthook commit-msg hook.
# Lives in a script file (not inline in lefthook.yml) because lefthook on Windows
# runs hooks via cmd.exe -> sh, and cmd.exe pre-parses (, ), | in an inline command
# before sh sees them, mangling any grouped regex. A file path in the hook avoids that.
# Arg $1 = path to the commit-message file.
first_line=$(head -n1 "$1")

# Skip merge / revert commits — they don't follow conventional format.
case "$first_line" in
  "Merge "*) echo "Merge commit - skip conventional check"; exit 0 ;;
  "Revert "*) echo "Revert commit - skip conventional check"; exit 0 ;;
esac

if printf '%s\n' "$first_line" | grep -qE '^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?: .{1,100}'; then
  echo "Commit message OK"
  exit 0
fi

echo "Commit message must follow conventional commits format: <type>(<scope>): <subject>"
echo "  got: $first_line"
exit 1
