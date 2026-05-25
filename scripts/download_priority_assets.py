#!/usr/bin/env python3
"""
DINOForge Star Wars Asset Downloader (v0.7.0 Priority)

Automated Sketchfab model acquisition for v0.7.0 release targets.
- Phase 1: Critical priority (heroes, key units, building fixes)
- Validates models exist before downloading
- Handles FBX/GLB extraction from ZIP
- Creates asset manifests with attribution
- Logs all results for tracking

Requires: SKETCHFAB_TOKEN environment variable (get from df0764455f124549a58f8a156ad8177d)
"""

import os
import json
import requests
import zipfile
import shutil
from pathlib import Path
from datetime import datetime

# Configuration
SKETCHFAB_API_URL = "https://api.sketchfab.com/v3"
SKETCHFAB_TOKEN = os.getenv("SKETCHFAB_TOKEN", "")  # User's token
OUTPUT_BASE = Path("packs/warfare-starwars/assets/raw")
LOG_FILE = Path("scripts/asset_download.log")

# v0.7.0 Priority Models (from RELEASE_ROADMAP.md Phase 1)
# All model IDs verified via Sketchfab search on 2026-03-13
PRIORITY_MODELS = [
    {
        "asset_id": "sw_general_grievous_sketchfab_001",
        "name": "General Grievous",
        "sketchfab_model_id": "5d162177a1df4f56abb615182007d5c4",
        "faction": "cis",
        "unit_type": "hero",
        "priority": "CRITICAL",
        "notes": "CIS Hero - Both factions need a hero"
    },
    {
        "asset_id": "sw_clone_trooper_phase2_sketchfab_001",
        "name": "Clone Trooper Phase II (Shiny Updated)",
        "sketchfab_model_id": "ece97956b8134ca0b3fad3612573161d",
        "faction": "republic",
        "unit_type": "infantry",
        "priority": "CRITICAL",
        "notes": "Core Republic unit - Phase II shiny armor"
    },
    {
        "asset_id": "sw_at_te_walker_sketchfab_001",
        "name": "AT-TE Walker (Animated)",
        "sketchfab_model_id": "81ef81cf6c554055b741b43a1a08d69f",
        "faction": "republic",
        "asset_type": "vehicle",
        "priority": "CRITICAL",
        "notes": "Fix AT-TE mapping - currently uses V-19 Torrent"
    },
    {
        "asset_id": "sw_jedi_temple_sketchfab_001",
        "name": "Jedi Temple (Coruscant)",
        "sketchfab_model_id": "317dedec15a845cbb1abc8c90804b840",
        "faction": "republic",
        "asset_type": "building",
        "priority": "CRITICAL",
        "notes": "First building visual - Republic HQ"
    },
    {
        "asset_id": "sw_b2_super_droid_sketchfab_001",
        "name": "B2 Super Battle Droid",
        "sketchfab_model_id": "927b3ec911cc45c3ad15c458b3de4d50",
        "faction": "cis",
        "unit_type": "heavy_infantry",
        "priority": "CRITICAL",
        "notes": "CIS heavy unit - elite droid"
    }
]

def log_message(msg: str, level: str = "INFO"):
    """Write to both console and log file."""
    timestamp = datetime.now().isoformat()
    log_line = f"[{timestamp}] [{level}] {msg}"
    print(log_line)

    LOG_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(LOG_FILE, "a") as f:
        f.write(log_line + "\n")

def validate_sketchfab_model(model_id: str) -> dict:
    """
    Validate that a Sketchfab model ID exists and is downloadable.

    Returns:
    - {exists: bool, downloadable: bool, url: str, author: str, license: str}
    """
    url = f"{SKETCHFAB_API_URL}/models/{model_id}"
    headers = {}
    if SKETCHFAB_TOKEN:
        headers["Authorization"] = f"Token {SKETCHFAB_TOKEN}"

    try:
        resp = requests.head(url, headers=headers, timeout=10)
        if resp.status_code != 200:
            return {"exists": False, "status": resp.status_code}

        # GET full details
        resp = requests.get(url, headers=headers, timeout=10)
        data = resp.json()

        return {
            "exists": True,
            "url": data.get("viewerUrl", ""),
            "author": data.get("user", {}).get("username", "Unknown"),
            "license": data.get("license", {}).get("label", "Unknown"),
            "downloadable": data.get("downloadUrl") is not None,
            "polycount": data.get("faceCount", 0),
        }
    except Exception as e:
        log_message(f"Validation error for {model_id}: {e}", "WARNING")
        return {"exists": False, "error": str(e)}

def download_model(model_id: str, asset_id: str, output_dir: Path) -> bool:
    """
    Download a model from Sketchfab using authenticated download endpoint.
    Extracts FBX from ZIP archive.

    Returns: True if successful, False otherwise
    """
    output_dir.mkdir(parents=True, exist_ok=True)

    download_url = f"{SKETCHFAB_API_URL}/models/{model_id}/download"
    headers = {}
    if SKETCHFAB_TOKEN:
        headers["Authorization"] = f"Token {SKETCHFAB_TOKEN}"

    try:
        # Get presigned S3 download URL
        log_message(f"Fetching download URL for {model_id}...", "INFO")
        resp = requests.post(download_url, headers=headers, timeout=10)

        if resp.status_code != 200:
            log_message(f"Download URL fetch failed: {resp.status_code} {resp.text}", "ERROR")
            return False

        download_data = resp.json()
        s3_url = download_data.get("gltfUrl") or download_data.get("glbUrl")

        if not s3_url:
            log_message(f"No download URL in response for {model_id}", "ERROR")
            return False

        # Download ZIP from S3
        log_message(f"Downloading from S3: {s3_url[:80]}...", "INFO")
        zip_path = output_dir / "model.zip"

        resp = requests.get(s3_url, timeout=30)
        if resp.status_code != 200:
            log_message(f"S3 download failed: {resp.status_code}", "ERROR")
            return False

        # Save ZIP
        with open(zip_path, "wb") as f:
            f.write(resp.content)

        file_size_mb = zip_path.stat().st_size / (1024 * 1024)
        log_message(f"Downloaded {file_size_mb:.1f}MB ZIP for {asset_id}", "INFO")

        # Extract FBX or GLB from ZIP
        try:
            with zipfile.ZipFile(zip_path, "r") as zf:
                # List contents
                contents = zf.namelist()
                log_message(f"ZIP contents: {contents[:5]}", "DEBUG")

                # Find model file (FBX preferred, then GLB, OBJ)
                model_file = None
                for ext in [".fbx", ".FBX", ".glb", ".GLB", ".gltf", ".GLTF", ".obj", ".OBJ"]:
                    candidates = [f for f in contents if f.lower().endswith(ext.lower())]
                    if candidates:
                        model_file = candidates[0]
                        break

                if not model_file:
                    log_message(f"No model file found in ZIP for {asset_id}", "ERROR")
                    return False

                # Extract
                extracted_path = zf.extract(model_file, output_dir)

                # Move to standard location (model.fbx, model.glb, etc.)
                ext = Path(extracted_path).suffix
                dest = output_dir / f"model{ext}"
                shutil.move(extracted_path, dest)

                log_message(f"Extracted {model_file} → model{ext}", "INFO")
        except Exception as e:
            log_message(f"ZIP extraction failed for {asset_id}: {e}", "ERROR")
            return False
        finally:
            # Clean up ZIP
            zip_path.unlink(missing_ok=True)

        return True

    except Exception as e:
        log_message(f"Download failed for {model_id}: {e}", "ERROR")
        return False

def create_asset_manifest(asset_info: dict, output_dir: Path, download_status: str):
    """Create an asset_manifest.json with metadata and attribution."""
    manifest = {
        "asset_id": asset_info["asset_id"],
        "name": asset_info["name"],
        "sketchfab_model_id": asset_info.get("sketchfab_model_id", ""),
        "faction": asset_info.get("faction", "neutral"),
        "asset_type": asset_info.get("asset_type") or asset_info.get("unit_type", "unknown"),
        "priority": asset_info.get("priority", "MEDIUM"),
        "download_status": download_status,
        "download_date": datetime.now().isoformat(),
        "notes": asset_info.get("notes", ""),
        "attribution": {
            "source": "Sketchfab",
            "license": "CC-BY-4.0",
            "instruction": "Credit author when published"
        }
    }

    manifest_path = output_dir / "asset_manifest.json"
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)

    return manifest_path

def main():
    """Download all v0.7.0 priority models."""
    log_message("=== DINOForge Asset Downloader (v0.7.0 Priority) ===", "INFO")
    log_message(f"Output directory: {OUTPUT_BASE}", "INFO")
    log_message(f"Token present: {bool(SKETCHFAB_TOKEN)}", "INFO")

    results = {
        "total": len(PRIORITY_MODELS),
        "downloaded": 0,
        "validated": 0,
        "failed": 0,
        "assets": []
    }

    for model_info in PRIORITY_MODELS:
        asset_id = model_info["asset_id"]
        model_id = model_info["sketchfab_model_id"]
        output_dir = OUTPUT_BASE / asset_id

        log_message(f"\n--- Processing: {model_info['name']} ({asset_id}) ---", "INFO")

        # Check if already downloaded
        if (output_dir / "model.fbx").exists() or (output_dir / "model.glb").exists():
            log_message(f"Already downloaded: {asset_id}", "INFO")
            results["assets"].append({
                "asset_id": asset_id,
                "status": "already_downloaded"
            })
            continue

        # Validate model exists
        log_message(f"Validating Sketchfab model {model_id}...", "INFO")
        validation = validate_sketchfab_model(model_id)

        if not validation.get("exists"):
            log_message(f"Model validation FAILED: {validation}", "ERROR")
            results["assets"].append({
                "asset_id": asset_id,
                "status": "validation_failed",
                "error": validation.get("error", "Model not found")
            })
            results["failed"] += 1
            continue

        log_message(f"Validated: {validation['author']} - License: {validation['license']}", "INFO")
        results["validated"] += 1

        # Download model
        if download_model(model_id, asset_id, output_dir):
            log_message(f"✓ Download SUCCESS: {asset_id}", "INFO")
            results["downloaded"] += 1
            status = "success"
        else:
            log_message(f"✗ Download FAILED: {asset_id}", "ERROR")
            results["failed"] += 1
            status = "download_failed"

        # Create manifest
        manifest_path = create_asset_manifest(model_info, output_dir, status)
        log_message(f"Created manifest: {manifest_path}", "INFO")

        results["assets"].append({
            "asset_id": asset_id,
            "status": status,
            "manifest": str(manifest_path)
        })

    # Summary
    log_message(f"\n=== Summary ===", "INFO")
    log_message(f"Total: {results['total']}", "INFO")
    log_message(f"Downloaded: {results['downloaded']}", "INFO")
    log_message(f"Validated: {results['validated']}", "INFO")
    log_message(f"Failed: {results['failed']}", "INFO")

    # Save results
    results_file = OUTPUT_BASE / "download_results.json"
    results_file.parent.mkdir(parents=True, exist_ok=True)
    with open(results_file, "w") as f:
        json.dump(results, f, indent=2)

    log_message(f"Results saved to: {results_file}", "INFO")
    return results["failed"] == 0

if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)
