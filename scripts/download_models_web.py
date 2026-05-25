#!/usr/bin/env python3
"""
DINOForge Asset Downloader - Web Download Method

Downloads Star Wars models from Sketchfab using direct download links.
Works by visiting the Sketchfab model page and extracting the download URL.

Usage:
  python scripts/download_models_web.py
"""

import os
import json
import requests
import zipfile
import shutil
from pathlib import Path
from datetime import datetime
import re

# Configuration
OUTPUT_BASE = Path("packs/warfare-starwars/assets/raw")
LOG_FILE = Path("scripts/asset_download_web.log")

# Models with direct download URLs
MODELS_TO_DOWNLOAD = [
    # v0.7.0 CRITICAL
    {
        "asset_id": "sw_general_grievous_sketchfab_001",
        "name": "General Grievous",
        "model_id": "5d162177a1df4f56abb615182007d5c4",
        "sketchfab_url": "https://sketchfab.com/3d-models/general-grievous-5d162177a1df4f56abb615182007d5c4",
        "phase": "v0.7.0"
    },
    {
        "asset_id": "sw_clone_trooper_phase2_sketchfab_001",
        "name": "Clone Trooper Phase II",
        "model_id": "ece97956b8134ca0b3fad3612573161d",
        "sketchfab_url": "https://sketchfab.com/3d-models/clone-trooper-phase-2-shiny-updated-ece97956b8134ca0b3fad3612573161d",
        "phase": "v0.7.0"
    },
    {
        "asset_id": "sw_at_te_walker_sketchfab_001",
        "name": "AT-TE Walker",
        "model_id": "81ef81cf6c554055b741b43a1a08d69f",
        "sketchfab_url": "https://sketchfab.com/3d-models/at-te-walker-animated-81ef81cf6c554055b741b43a1a08d69f",
        "phase": "v0.7.0"
    },
    {
        "asset_id": "sw_jedi_temple_sketchfab_001",
        "name": "Jedi Temple",
        "model_id": "317dedec15a845cbb1abc8c90804b840",
        "sketchfab_url": "https://sketchfab.com/3d-models/jedi-temple-317dedec15a845cbb1abc8c90804b840",
        "phase": "v0.7.0"
    },
    {
        "asset_id": "sw_b2_super_droid_sketchfab_001",
        "name": "B2 Super Battle Droid",
        "model_id": "927b3ec911cc45c3ad15c458b3de4d50",
        "sketchfab_url": "https://sketchfab.com/3d-models/b2-super-battle-droid-927b3ec911cc45c3ad15c458b3de4d50",
        "phase": "v0.7.0"
    },
    # v0.8.0 ELITE
    {
        "asset_id": "sw_clone_trooper_phase2_alt_sketchfab_001",
        "name": "Phase 2 Clone Trooper (Alt)",
        "model_id": "0f022ffbbf2342f992f413f7935e221c",
        "sketchfab_url": "https://sketchfab.com/3d-models/phase-2-clone-trooper-0f022ffbbf2342f992f413f7935e221c",
        "phase": "v0.8.0"
    },
    {
        "asset_id": "sw_arc_trooper_sketchfab_001",
        "name": "Clone ARC Trooper",
        "model_id": "8ac7323517e04efb91abb5edcafc1871",
        "sketchfab_url": "https://sketchfab.com/3d-models/clone-arc-trooper-8ac7323517e04efb91abb5edcafc1871",
        "phase": "v0.8.0"
    },
    {
        "asset_id": "sw_droideka_sketchfab_001",
        "name": "Droideka",
        "model_id": "42b2a42130f94d73a29eda6ecfdce98f",
        "sketchfab_url": "https://sketchfab.com/3d-models/droideka-42b2a42130f94d73a29eda6ecfdce98f",
        "phase": "v0.8.0"
    },
    {
        "asset_id": "sw_aat_walker_sketchfab_001",
        "name": "AAT Tank",
        "model_id": "7bdce26cbd3440fa8cbcfc135c698b1d",
        "sketchfab_url": "https://sketchfab.com/3d-models/aat-armored-assault-tank-7bdce26cbd3440fa8cbcfc135c698b1d",
        "phase": "v0.8.0"
    }
]

def log_msg(msg: str, level: str = "INFO"):
    """Write to console and log file."""
    timestamp = datetime.now().isoformat()
    log_line = f"[{timestamp}] [{level}] {msg}"
    try:
        print(log_line)
    except UnicodeEncodeError:
        # Fallback for Windows cmd encoding issues
        print(log_line.encode('utf-8', 'replace').decode('utf-8', 'replace'))

    LOG_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(LOG_FILE, "a", encoding="utf-8") as f:
        f.write(log_line + "\n")

def extract_model_from_zip(zip_path: Path, output_dir: Path) -> bool:
    """Extract FBX/GLB from ZIP."""
    try:
        with zipfile.ZipFile(zip_path, "r") as zf:
            contents = zf.namelist()
            model_file = None

            # Find model file
            for ext in [".fbx", ".glb", ".gltf", ".obj"]:
                candidates = [f for f in contents if f.lower().endswith(ext.lower())]
                if candidates:
                    model_file = candidates[0]
                    break

            if not model_file:
                log_msg(f"No model file found in ZIP", "ERROR")
                return False

            # Extract
            extracted = zf.extract(model_file, output_dir)
            dest = output_dir / f"model{Path(extracted).suffix}"
            shutil.move(extracted, dest)
            log_msg(f"Extracted: {Path(extracted).name} -> model{Path(extracted).suffix}", "INFO")
            return True
    except Exception as e:
        log_msg(f"ZIP extraction error: {e}", "ERROR")
        return False
    finally:
        zip_path.unlink(missing_ok=True)

def download_model_file(model: dict, output_dir: Path) -> bool:
    """Download a single model file from Sketchfab."""
    output_dir.mkdir(parents=True, exist_ok=True)
    model_url = model["sketchfab_url"]

    try:
        log_msg(f"Fetching Sketchfab page: {model_url}", "INFO")

        # Visit the Sketchfab page
        session = requests.Session()
        session.headers.update({
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
        })

        resp = session.get(model_url, timeout=15)
        if resp.status_code != 200:
            log_msg(f"Page fetch failed: {resp.status_code}", "ERROR")
            return False

        # Extract download URL from page (usually in JSON or links)
        # Look for download endpoint in common patterns
        download_patterns = [
            r'href=["\']([^"\']*download[^"\']*)["\']',
            r'"downloadUrl":\s*"([^"]+)"',
            r'data-download-url=["\']([^"\']+)["\']',
        ]

        download_url = None
        for pattern in download_patterns:
            matches = re.findall(pattern, resp.text, re.IGNORECASE)
            if matches:
                download_url = matches[0]
                break

        if not download_url:
            # Try common Sketchfab download endpoint pattern
            download_url = f"https://sketchfab.com/models/{model['model_id']}/download"
            log_msg(f"Using standard download endpoint", "INFO")

        # Make download URL absolute if relative
        if download_url.startswith("/"):
            download_url = "https://sketchfab.com" + download_url
        elif not download_url.startswith("http"):
            download_url = "https://sketchfab.com/" + download_url

        log_msg(f"Attempting download from: {download_url[:80]}...", "INFO")

        # Download the model file
        resp = session.get(download_url, timeout=60, allow_redirects=True)
        if resp.status_code != 200:
            log_msg(f"Download failed: {resp.status_code}", "ERROR")
            return False

        file_size_mb = len(resp.content) / (1024 * 1024)
        log_msg(f"Downloaded {file_size_mb:.1f}MB", "INFO")

        # Determine file type and save
        content_type = resp.headers.get("content-type", "").lower()

        if "zip" in content_type or resp.content[:2] == b'PK':
            # ZIP file - extract model
            zip_path = output_dir / "model.zip"
            with open(zip_path, "wb") as f:
                f.write(resp.content)
            return extract_model_from_zip(zip_path, output_dir)
        elif "octet-stream" in content_type or content_type == "application/x-gltf-binary":
            # Binary GLB file
            model_path = output_dir / "model.glb"
            with open(model_path, "wb") as f:
                f.write(resp.content)
            log_msg(f"Saved as GLB", "INFO")
            return True
        elif "fbx" in content_type or "model" in content_type:
            # FBX file
            model_path = output_dir / "model.fbx"
            with open(model_path, "wb") as f:
                f.write(resp.content)
            log_msg(f"Saved as FBX", "INFO")
            return True
        else:
            # Try to infer from magic bytes
            if resp.content[:2] == b'PK':  # ZIP signature
                zip_path = output_dir / "model.zip"
                with open(zip_path, "wb") as f:
                    f.write(resp.content)
                return extract_model_from_zip(zip_path, output_dir)
            else:
                # Default to GLB
                model_path = output_dir / "model.glb"
                with open(model_path, "wb") as f:
                    f.write(resp.content)
                log_msg(f"Saved as GLB (inferred)", "INFO")
                return True

    except Exception as e:
        log_msg(f"Download exception: {e}", "ERROR")
        return False

def create_manifest(model: dict, output_dir: Path, status: str):
    """Create asset_manifest.json."""
    manifest = {
        "asset_id": model["asset_id"],
        "name": model["name"],
        "model_id": model["model_id"],
        "phase": model["phase"],
        "status": status,
        "downloaded": datetime.now().isoformat(),
        "license": "CC-BY-4.0",
        "attribution": "See SKETCHFAB_MODELS.json for author info",
        "source": model["sketchfab_url"]
    }

    with open(output_dir / "asset_manifest.json", "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)

def main():
    log_msg("=" * 60, "INFO")
    log_msg("DINOForge Asset Downloader (v0.7.0 + v0.8.0 Web Method)", "INFO")
    log_msg("=" * 60, "INFO")
    log_msg(f"Total models: {len(MODELS_TO_DOWNLOAD)}", "INFO")

    results = {
        "v0_7_0": {"total": 0, "success": 0, "failed": 0, "already_downloaded": 0},
        "v0_8_0": {"total": 0, "success": 0, "failed": 0, "already_downloaded": 0},
        "assets": []
    }

    for model in MODELS_TO_DOWNLOAD:
        asset_id = model["asset_id"]
        phase = model["phase"]
        output_dir = OUTPUT_BASE / asset_id

        # Track counts
        phase_key = "v0_7_0" if phase == "v0.7.0" else "v0_8_0"
        results[phase_key]["total"] += 1

        log_msg(f"\n[{phase}] {model['name']} ({asset_id})", "INFO")

        # Check if already exists
        if (output_dir / "model.fbx").exists() or (output_dir / "model.glb").exists():
            log_msg("Already downloaded", "INFO")
            results[phase_key]["already_downloaded"] += 1
            results["assets"].append({
                "asset_id": asset_id,
                "phase": phase,
                "status": "already_downloaded"
            })
            continue

        # Download
        if download_model_file(model, output_dir):
            log_msg("[SUCCESS]", "INFO")
            create_manifest(model, output_dir, "success")
            results[phase_key]["success"] += 1
            results["assets"].append({
                "asset_id": asset_id,
                "phase": phase,
                "status": "success"
            })
        else:
            log_msg("[FAILED]", "ERROR")
            create_manifest(model, output_dir, "failed")
            results[phase_key]["failed"] += 1
            results["assets"].append({
                "asset_id": asset_id,
                "phase": phase,
                "status": "failed"
            })

    # Summary
    log_msg("\n" + "=" * 60, "INFO")
    log_msg("SUMMARY", "INFO")
    log_msg("=" * 60, "INFO")
    log_msg(f"v0.7.0: {results['v0_7_0']['success']} success, {results['v0_7_0']['failed']} failed, {results['v0_7_0']['already_downloaded']} already", "INFO")
    log_msg(f"v0.8.0: {results['v0_8_0']['success']} success, {results['v0_8_0']['failed']} failed, {results['v0_8_0']['already_downloaded']} already", "INFO")

    # Save results
    results_file = OUTPUT_BASE / "download_results.json"
    results_file.parent.mkdir(parents=True, exist_ok=True)
    with open(results_file, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)

    log_msg(f"Results: {results_file}", "INFO")
    log_msg("=" * 60, "INFO")

    total_failed = results["v0_7_0"]["failed"] + results["v0_8_0"]["failed"]
    return total_failed == 0

if __name__ == "__main__":
    try:
        success = main()
        exit(0 if success else 1)
    except Exception as e:
        log_msg(f"Fatal error: {e}", "ERROR")
        exit(1)
