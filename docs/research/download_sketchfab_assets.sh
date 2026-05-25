#!/bin/bash

###############################################################################
# Sketchfab Star Wars Asset Downloader (FIXED)
# Downloads all Star Wars models from Sketchfab API v3
# Properly parses /v3/models/{id}/download endpoint for GLB URLs
# Usage: SKETCHFAB_API_TOKEN="your_token" bash download_sketchfab_assets.sh
###############################################################################

set -euo pipefail

# Configuration
API_BASE="https://api.sketchfab.com/v3"
PACK_ROOT="packs/warfare-starwars/assets"
RAW_DIR="${PACK_ROOT}/raw"
LOG_FILE="${PACK_ROOT}/DOWNLOAD_LOG.txt"
MANIFEST_FILE="${PACK_ROOT}/SKETCHFAB_DOWNLOADS_COMPLETE.json"

# Check for API token
if [[ -z "${SKETCHFAB_API_TOKEN:-}" ]]; then
    echo "ERROR: SKETCHFAB_API_TOKEN not set. Usage:"
    echo "  SKETCHFAB_API_TOKEN='your_token' bash download_sketchfab_assets.sh"
    exit 1
fi

# Create output directories
mkdir -p "$RAW_DIR"
mkdir -p "$(dirname "$LOG_FILE")"

echo "Starting Sketchfab asset downloads..." | tee -a "$LOG_FILE"
echo "Timestamp: $(date)" >> "$LOG_FILE"
echo "" >> "$LOG_FILE"

###############################################################################
# Download a single model by ID
###############################################################################
download_model() {
    local model_id="$1"
    local asset_id="$2"
    local asset_dir="${RAW_DIR}/${asset_id}"

    echo "Downloading: $asset_id ($model_id)..."

    mkdir -p "$asset_dir"

    # Get model metadata
    echo "  Getting model info..." >> "$LOG_FILE"
    local model_info=$(curl -s -f \
        -H "Authorization: Token ${SKETCHFAB_API_TOKEN}" \
        "${API_BASE}/models/${model_id}/" 2>/dev/null || echo "{}")

    if [[ "$model_info" == "{}" ]]; then
        echo "  ERROR: Failed to fetch model info for $model_id" | tee -a "$LOG_FILE"
        return 1
    fi

    local model_name=$(echo "$model_info" | python3 -c "import json,sys; print(json.load(sys.stdin).get('name','Unknown'))" 2>/dev/null || echo "Unknown")
    local vertex_count=$(echo "$model_info" | python3 -c "import json,sys; print(json.load(sys.stdin).get('vertexCount',0))" 2>/dev/null || echo "0")
    local face_count=$(echo "$model_info" | python3 -c "import json,sys; print(json.load(sys.stdin).get('faceCount',0))" 2>/dev/null || echo "0")

    # Get download URLs from /download endpoint
    echo "  Fetching download URLs..." >> "$LOG_FILE"
    local download_info=$(curl -s -f \
        -H "Authorization: Token ${SKETCHFAB_API_TOKEN}" \
        "${API_BASE}/models/${model_id}/download" 2>/dev/null || echo "{}")

    if [[ "$download_info" == "{}" ]]; then
        echo "  ERROR: Failed to fetch download info for $model_id" | tee -a "$LOG_FILE"
        return 1
    fi

    # Extract GLB download URL (preferred)
    local download_url=$(echo "$download_info" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('glb',{}).get('url',''))" 2>/dev/null || echo "")
    local file_format="glb"
    local file_size_remote=0

    if [[ -z "$download_url" ]]; then
        echo "  WARNING: No GLB found for $asset_id, trying gltf..." >> "$LOG_FILE"
        download_url=$(echo "$download_info" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('gltf',{}).get('url',''))" 2>/dev/null || echo "")
        file_format="gltf"
    fi

    if [[ -z "$download_url" ]]; then
        echo "  WARNING: No gltf found, trying source..." >> "$LOG_FILE"
        download_url=$(echo "$download_info" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('source',{}).get('url',''))" 2>/dev/null || echo "")
        file_format="source"
    fi

    if [[ -z "$download_url" ]]; then
        echo "  ERROR: No downloadable format found for $model_id" | tee -a "$LOG_FILE"
        return 1
    fi

    # Extract remote file size
    file_size_remote=$(echo "$download_info" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('${file_format}',{}).get('size',0))" 2>/dev/null || echo "0")

    # Download the file
    local output_file="${asset_dir}/${asset_id}.${file_format}"
    echo "  Downloading from S3 ($file_format, ~$((file_size_remote / 1024 / 1024))MB)..." >> "$LOG_FILE"

    if curl -L --progress-bar -f \
        -H "Authorization: Token ${SKETCHFAB_API_TOKEN}" \
        -o "$output_file" \
        "$download_url" 2>> "$LOG_FILE"; then

        local file_size_actual=$(stat -f%z "$output_file" 2>/dev/null || stat -c%s "$output_file" 2>/dev/null || echo "0")
        local file_size_human=$(numfmt --to=iec-i --suffix=B "$file_size_actual" 2>/dev/null || echo "~$((file_size_actual / 1024 / 1024))MB")
        echo "  ✓ Downloaded: $file_size_human" | tee -a "$LOG_FILE"

        # Create/update asset manifest
        cat > "${asset_dir}/asset_manifest.json" <<EOF
{
  "asset_id": "${asset_id}",
  "sketchfab_model_id": "${model_id}",
  "model_name": "${model_name}",
  "file_path": "${output_file}",
  "file_size": ${file_size_actual},
  "format": "${file_format}",
  "vertex_count": ${vertex_count},
  "face_count": ${face_count},
  "downloaded": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "status": "complete"
}
EOF
        echo "" >> "$LOG_FILE"
        return 0
    else
        echo "  ERROR: Failed to download $model_id" | tee -a "$LOG_FILE"
        echo "" >> "$LOG_FILE"
        return 1
    fi
}

###############################################################################
# Main: Download documented models
###############################################################################

echo "=== Phase 1: Downloading documented Star Wars models ===" | tee -a "$LOG_FILE"

# Array of model IDs to download - NOTE: Original IDs were invalid, using search results
declare -A MODELS=(
    ["sw_venator_prefab_001"]="8a1e1760391c4ac6a50373c2bf5efa2e"
)

success_count=0
fail_count=0

for asset_id in "${!MODELS[@]}"; do
    if download_model "${MODELS[$asset_id]}" "$asset_id"; then
        ((success_count++))
    else
        ((fail_count++))
    fi
done

echo ""

###############################################################################
# Search for additional Star Wars buildings/structures
###############################################################################

echo "=== Phase 2: Searching for additional Star Wars models ===" | tee -a "$LOG_FILE"

search_and_download() {
    local query="$1"
    local safe_query=$(echo "$query" | sed 's/ /_/g')

    echo "  Searching: $query" | tee -a "$LOG_FILE"

    # Search API
    local results=$(curl -s -f \
        -H "Authorization: Token ${SKETCHFAB_API_TOKEN}" \
        "${API_BASE}/search?q=$(echo -n "$query" | python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.stdin.read()))')&sort_by=-likeCount&count=20" 2>/dev/null || echo "{}")

    if [[ "$results" == "{}" ]]; then
        echo "    No results found" >> "$LOG_FILE"
        return
    fi

    # Extract model UIDs from results
    echo "$results" | python3 -c "
import json, sys
data = json.load(sys.stdin)
models = data.get('results', {}).get('models', [])
for i, model in enumerate(models[:5]):
    print(model.get('uid', ''))
" 2>/dev/null | while read -r model_id; do
        if [[ ! -z "$model_id" ]]; then
            local safe_id="sw_${safe_query}_${model_id:0:8}"
            if [[ ! -d "${RAW_DIR}/${safe_id}" ]]; then
                if download_model "$model_id" "$safe_id" 2>/dev/null; then
                    ((success_count++))
                else
                    ((fail_count++)) || true
                fi
            fi
        fi
    done
}

search_and_download "star wars building"
search_and_download "star wars structure"
search_and_download "star wars droid"

###############################################################################
# Generate manifest file and summary
###############################################################################

echo ""
echo "=== Generating manifest ===" | tee -a "$LOG_FILE"

# Create comprehensive manifest
python3 << 'MANIFEST_EOF'
import json
import os
from pathlib import Path

raw_dir = "packs/warfare-starwars/assets/raw"
manifest_file = "packs/warfare-starwars/assets/SKETCHFAB_DOWNLOADS_COMPLETE.json"

manifest = {
    "generated": os.popen("date -u +%Y-%m-%dT%H:%M:%SZ").read().strip(),
    "total_assets": 0,
    "successful": 0,
    "failed": 0,
    "assets": []
}

if os.path.exists(raw_dir):
    for asset_dir in sorted(Path(raw_dir).iterdir()):
        if asset_dir.is_dir():
            manifest_file_path = asset_dir / "asset_manifest.json"
            if manifest_file_path.exists():
                with open(manifest_file_path, 'r') as f:
                    asset_data = json.load(f)
                    manifest["assets"].append(asset_data)
                    manifest["total_assets"] += 1
                    if asset_data.get("status") == "complete":
                        manifest["successful"] += 1
                    else:
                        manifest["failed"] += 1

os.makedirs(os.path.dirname(manifest_file), exist_ok=True)
with open(manifest_file, 'w') as f:
    json.dump(manifest, f, indent=2)

print(f"Manifest created: {manifest_file}")
print(f"Total assets: {manifest['total_assets']}")
print(f"Successful: {manifest['successful']}")
print(f"Failed: {manifest['failed']}")
MANIFEST_EOF

echo ""
echo "=== Download Summary ===" | tee -a "$LOG_FILE"
echo "  Log file: $LOG_FILE" | tee -a "$LOG_FILE"
echo "  Manifest: $MANIFEST_FILE" | tee -a "$LOG_FILE"
echo "  Assets dir: $RAW_DIR" | tee -a "$LOG_FILE"

if [[ $fail_count -eq 0 ]]; then
    echo ""
    echo "✓ All downloads completed successfully"
    exit 0
else
    echo ""
    echo "⚠ Some downloads had issues. Check $LOG_FILE for details."
    exit 0  # Non-fatal, manifest still valid
fi
