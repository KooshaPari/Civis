#!/bin/bash
DUMP_DIR="G:/SteamLibrary/steamapps/common/Diplomacy is Not an Option/BepInEx/dinoforge_dumps"
LATEST=$(ls -td "$DUMP_DIR"/dump_* 2>/dev/null | head -1)
if [ -z "$LATEST" ]; then
  echo "No dumps found in $DUMP_DIR"
  exit 1
fi
echo "Analyzing: $LATEST"
dotnet run --project src/Tools/DumpTools -- analyze "$LATEST"
