#!/bin/bash
# Deploy debug version of DINOForge.Runtime.dll

SRC="C:/Users/koosh/Dino/src/Runtime/bin/Debug/netstandard2.0/DINOForge.Runtime.dll"
DEST="G:/SteamLibrary/steamapps/common/Diplomacy is Not an Option/BepInEx/plugins/DINOForge.Runtime.dll"
DEST_PDB="${DEST%.dll}.pdb"
SRC_PDB="${SRC%.dll}.pdb"

echo "Checking source file..."
if [ ! -f "$SRC" ]; then
    echo "ERROR: Source DLL not found: $SRC"
    exit 1
fi

echo "Source DLL timestamp:"
ls -lh "$SRC"

echo ""
echo "Current deployed DLL timestamp:"
ls -lh "$DEST"

echo ""
echo "CLOSE THE GAME NOW before proceeding"
echo "Press Enter when ready to deploy..."
read

echo ""
echo "Deploying new DLL..."
cp "$SRC" "$DEST" && cp "$SRC_PDB" "$DEST_PDB"

echo ""
echo "New deployed DLL timestamp:"
ls -lh "$DEST"

echo ""
echo "Deployment complete. You can now start the game."
