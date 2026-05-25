#!/bin/bash
set -e

TASK="${1:-build:all}"
LIST="${2:-}"

if [ "$LIST" = "--list" ] || [ "$LIST" = "-l" ]; then
    echo "📋 Available tasks:"
    ./bin/task --list
    exit 0
fi

echo "🚀 Running task: $TASK"
./bin/task "$TASK"

if [ $? -eq 0 ]; then
    echo "✅ Task complete: $TASK"
else
    echo "❌ Task failed: $TASK"
    exit 1
fi
