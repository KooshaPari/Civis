#!/usr/bin/env python3
"""
DINOForge Complete Asset Downloader (v0.7.0 + v0.8.0)

Downloads all priority Star Wars Clone Wars models from Sketchfab.
Works without authentication (uses Sketchfab public download endpoints).

Usage:
  python scripts/download_all_priority_assets.py

Models downloaded (total 9):
  v0.7.0 (5 critical): General Grievous, Clone Trooper, AT-TE Walker, Jedi Temple, B2 Super Droid
  v0.8.0 (4 elite): Clone Trooper Alt, ARC Trooper, Droideka, AAT Tank

Output: packs/warfare-starwars/assets/raw/{asset_id}/model.{fbx|glb}
"""

import os
import json
import requests
import zipfile
import shutil
from pathlib import Path
from datetime import datetime
from urllib.parse import urljoin

# Configuration
SKETCHFAB_BASE_URL = "https://sketchfab.com"
SKETCHFAB_API_URL = "https://api.sketchfab.com/v3"
OUTPUT_BASE = Path("packs/warfare-starwars/assets/raw")
LOG_FILE = Path("scripts/asset_download_full.log")

# Complete priority list (v0.7.0 + v0.8.0)
ALL_PRIORITY_MODELS = [
    # v0.7.0 CRITICAL
    {
        "asset_id": "sw_general_grievous_sketchfab_001",
        "name": "General Grievous",
        "model_id": "5d162177a1df4f56abb615182007d5c4",
        "phase": "v0.7.0",
        "faction": "cis",
        "type": "hero"
    },
    {
        "asset_id": "sw_clone_trooper_phase2_sketchfab_001",
        "name": "Clone Trooper Phase II",
        "model_id": "ece97956b8134ca0b3fad3612573161d",
        "phase": "v0.7.0",
        "faction": "republic",
        "type": "infantry"
    },
    {
        "asset_id": "sw_at_te_walker_sketchfab_001",
        "name": "AT-TE Walker",
        "model_id": "81ef81cf6c554055b741b43a1a08d69f",
        "phase": "v0.7.0",
        "faction": "republic",
        "type": "vehicle"
    },
    {
        "asset_id": "sw_jedi_temple_sketchfab_001",
        "name": "Jedi Temple",
        "model_id": "317dedec15a845cbb1abc8c90804b840",
        "phase": "v0.7.0",
        "faction": "republic",
        "type": "building"
    },
    {
        "asset_id": "sw_b2_super_droid_sketchfab_001",
        "name": "B2 Super Battle Droid",
        "model_id": "927b3ec911cc45c3ad15c458b3de4d50",
        "phase": "v0.7.0",
        "faction": "cis",
        "type": "heavy_infantry"
    },
    # v0.8.0 ELITE
    {
        "asset_id": "sw_clone_trooper_phase2_alt_sketchfab_001",
        "name": "Phase 2 Clone Trooper (Alt)",
        "model_id": "0f022ffbbf2342f992f413f7935e221c",
        "phase": "v0.8.0",
        "faction": "republic",
        "type": "infantry"
    },
    {
        "asset_id": "sw_arc_trooper_sketchfab_001",
        "name": "Clone ARC Trooper",
        "model_id": "8ac7323517e04efb91abb5edcafc1871",
        "phase": "v0.8.0",
        "faction": "republic",
        "type": "elite"
    },
    {
        "asset_id": "sw_droideka_sketchfab_001",
        "name": "Droideka",
        "model_id": "42b2a42130f94d73a29eda6ecfdce98f",
        "phase": "v0.8.0",
        "faction": "cis",
        "type": "specialized"
    },
    {
        "asset_id": "sw_aat_walker_sketchfab_001",
        "name": "AAT Tank",
        "model_id": "7bdce26cbd3440fa8cbcfc135c698b1d",
        "phase": "v0.8.0",
        "faction": "cis",
        "type": "vehicle"
    }
]

def log_msg(msg: str, level: str = "INFO"):
    """Write to console and log file."""
    timestamp = datetime.now().isoformat()
    log_line = f"[{timestamp}] [{level}] {msg}"
    print(log_line)

    LOG_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(LOG_FILE, "a") as f:
        f.write(log_line + "\n")

def get_download_url(model_id: str) -> str:
    """
    Construct Sketchfab download URL using known pattern.
    Sketchfab URLs follow: https://sketchfab.com/3d-models/{name}-{model_id}
    Download endpoint: https://sketchfab.com/models/{model_id}/download
    """
    # Try direct API endpoint first
    return f"{SKETCHFAB_API_URL}/models/{model_id}/download"

def download_model_direct(model_id: str, asset_id: str, output_dir: Path) -> bool:
    """
    Download model using direct Sketchfab model page approach.
    Newer Sketchfab models can be accessed via JSON data embedded in page.
    """
    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        # Fetch model info from API (no auth required for public models)
        log_msg(f"Fetching model info for {model_id}...", "INFO")
        info_url = f"{SKETCHFAB_API_URL}/models/{model_id}"

        resp = requests.get(info_url, timeout=10)
        if resp.status_code != 200:
            log_msg(f"Model info fetch failed: {resp.status_code}", "ERROR")
            return False

        model_data = resp.json()

        # Try to get download URL from geometry file
        geometry = model_data.get("geometry", {})
        if not geometry:
            log_msg(f"No geometry data for {model_id}", "WARNING")
            return False

        # Sketchfab provides gltf/glb URLs in model data
        download_url = model_data.get("glbUrl") or model_data.get("gltfUrl")
        if not download_url:
            log_msg(f"No download URL found in model data", "ERROR")
            return False

        log_msg(f"Download URL: {download_url[:80]}...", "INFO")

        # Download file
        resp = requests.get(download_url, timeout=60)
        if resp.status_code != 200:
            log_msg(f"Download failed: {resp.status_code}", "ERROR")
            return False

        file_size_mb = len(resp.content) / (1024 * 1024)
        log_msg(f"Downloaded {file_size_mb:.1f}MB", "INFO")

        # Save and extract
        if download_url.endswith('.zip'):
            zip_path = output_dir / "model.zip"
            with open(zip_path, "wb") as f:
                f.write(resp.content)

            # Extract FBX/GLB
            try:
                with zipfile.ZipFile(zip_path, "r") as zf:
                    contents = zf.namelist()
                    model_file = None

                    for ext in [".fbx", ".glb", ".gltf", ".obj"]:
                        candidates = [f for f in contents if f.lower().endswith(ext.lower())]
                        if candidates:
                            model_file = candidates[0]
                            break

                    if not model_file:
                        log_msg(f"No model file found in ZIP", "ERROR")
                        return False

                    extracted = zf.extract(model_file, output_dir)
                    dest = output_dir / f"model{Path(extracted).suffix}"
                    shutil.move(extracted, dest)
                    log_msg(f"Extracted: {model_file} → model{Path(extracted).suffix}", "INFO")
            finally:
                zip_path.unlink(missing_ok=True)
        else:
            # Direct GLB/FBX file
            ext = ".glb" if "glb" in download_url.lower() else ".fbx"
            model_path = output_dir / f"model{ext}"
            with open(model_path, "wb") as f:
                f.write(resp.content)
            log_msg(f"Saved: model{ext}", "INFO")

        return True

    except Exception as e:
        log_msg(f"Download exception: {e}", "ERROR")
        return False

def create_manifest(model_info: dict, output_dir: Path, status: str):
    """Create asset_manifest.json."""
    manifest = {
        "asset_id": model_info["asset_id"],
        "name": model_info["name"],
        "model_id": model_info["model_id"],
        "phase": model_info["phase"],
        "faction": model_info["faction"],
        "type": model_info["type"],
        "status": status,
        "downloaded": datetime.now().isoformat(),
        "license": "CC-BY-4.0",
        "attribution": "See SKETCHFAB_MODELS.json for author info"
    }

    with open(output_dir / "asset_manifest.json", "w") as f:
        json.dump(manifest, f, indent=2)

def main():
    log_msg("=== DINOForge Asset Downloader (v0.7.0 + v0.8.0) ===", "INFO")
    log_msg(f"Total models to download: {len(ALL_PRIORITY_MODELS)}", "INFO")

    results = {
        "v0_7_0": {"total": 0, "success": 0, "failed": 0},
        "v0_8_0": {"total": 0, "success": 0, "failed": 0},
        "assets": []
    }

    for model in ALL_PRIORITY_MODELS:
        asset_id = model["asset_id"]
        model_id = model["model_id"]
        phase = model["phase"]
        output_dir = OUTPUT_BASE / asset_id

        # Track counts
        if phase == "v0.7.0":
            results["v0_7_0"]["total"] += 1
        else:
            results["v0_8_0"]["total"] += 1

        log_msg(f"\n--- {phase}: {model['name']} ({asset_id}) ---", "INFO")

        # Check if already exists
        if (output_dir / "model.fbx").exists() or (output_dir / "model.glb").exists():
            log_msg(f"Already downloaded", "INFO")
            results["assets"].append({
                "asset_id": asset_id,
                "phase": phase,
                "status": "already_downloaded"
            })
            if phase == "v0.7.0":
                results["v0_7_0"]["success"] += 1
            else:
                results["v0_8_0"]["success"] += 1
            continue

        # Download
        if download_model_direct(model_id, asset_id, output_dir):
            log_msg(f"✓ SUCCESS", "INFO")
            create_manifest(model, output_dir, "success")
            results["assets"].append({
                "asset_id": asset_id,
                "phase": phase,
                "status": "success"
            })
            if phase == "v0.7.0":
                results["v0_7_0"]["success"] += 1
            else:
                results["v0_8_0"]["success"] += 1
        else:
            log_msg(f"✗ FAILED", "ERROR")
            create_manifest(model, output_dir, "failed")
            results["assets"].append({
                "asset_id": asset_id,
                "phase": phase,
                "status": "failed"
            })
            if phase == "v0.7.0":
                results["v0_7_0"]["failed"] += 1
            else:
                results["v0_8_0"]["failed"] += 1

    # Summary
    log_msg(f"\n=== SUMMARY ===", "INFO")
    log_msg(f"v0.7.0: {results['v0_7_0']['success']}/{results['v0_7_0']['total']} success", "INFO")
    log_msg(f"v0.8.0: {results['v0_8_0']['success']}/{results['v0_8_0']['total']} success", "INFO")

    # Save results
    results_file = OUTPUT_BASE / "download_results_full.json"
    results_file.parent.mkdir(parents=True, exist_ok=True)
    with open(results_file, "w") as f:
        json.dump(results, f, indent=2)

    log_msg(f"Results saved to: {results_file}", "INFO")
    return (results["v0_7_0"]["failed"] + results["v0_8_0"]["failed"]) == 0

if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)
