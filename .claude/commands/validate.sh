#!/bin/bash
for pack in packs/*/; do
  echo "=== Validating $pack ==="
  dotnet run --project src/Tools/PackCompiler -- validate "$pack"
done
