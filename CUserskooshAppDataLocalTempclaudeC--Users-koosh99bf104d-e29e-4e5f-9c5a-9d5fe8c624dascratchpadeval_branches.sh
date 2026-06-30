#!/bin/bash

BRANCHES=$(cat /tmp/branchbatch/nb_ac | tr -d ' ' | sort -u)
DECISIONS="/tmp/branchbatch/decisions.log"

# Initialize decisions log if needed
if [ ! -f "$DECISIONS" ]; then
  echo "# Branch evaluation log" > "$DECISIONS"
fi

cd C:/Users/koosh/Dev/Civis-cxA

for BRANCH in $BRANCHES; do
  # Check if branch exists on remote
  if ! git rev-parse origin/$BRANCH > /dev/null 2>&1; then
    echo "$BRANCH|NOTFOUND|branch does not exist on origin"
    echo "$BRANCH|NOTFOUND|branch does not exist on origin" >> "$DECISIONS"
    continue
  fi

  # Get unique commits (commits in branch not in main)
  UNIQUE_COMMITS=$(git log --oneline origin/main..origin/$BRANCH 2>/dev/null | wc -l)
  
  # Get merge base
  MERGE_BASE=$(git merge-base origin/main origin/$BRANCH 2>/dev/null)
  
  # Get NEW files added by this branch
  NEW_FILES=$(git diff $MERGE_BASE..origin/$BRANCH --diff-filter=A --name-only 2>/dev/null | wc -l)
  
  # Get a sample of what it adds (first 5 files)
  NEW_FILES_SAMPLE=$(git diff $MERGE_BASE..origin/$BRANCH --diff-filter=A --name-only 2>/dev/null | head -5 | tr '\n' '|')
  
  # Decision logic:
  # PRUNE if: no unique commits OR (no new files AND old/small branch)
  # KEEP if: has unique commits AND adds new files
  
  if [ "$UNIQUE_COMMITS" -eq 0 ]; then
    echo "$BRANCH|PRUNE|no unique commits past main" >> "$DECISIONS"
    echo "$BRANCH|PRUNE|no unique commits (0)"
  elif [ "$NEW_FILES" -eq 0 ]; then
    echo "$BRANCH|PRUNE|adds 0 new files, likely rebase/test noise" >> "$DECISIONS"
    echo "$BRANCH|PRUNE|no new files ($UNIQUE_COMMITS commits)"
  else
    echo "$BRANCH|KEEP|$UNIQUE_COMMITS commits, $NEW_FILES new files ($NEW_FILES_SAMPLE)" >> "$DECISIONS"
    echo "$BRANCH|KEEP|$UNIQUE_COMMITS commits, $NEW_FILES new files"
  fi
done
