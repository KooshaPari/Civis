#!/usr/bin/env python3
"""
Simplified Sketchfab downloader with better error handling
"""

import os
import sys
import json
import time
import urllib.request
import urllib.parse
import ssl
from pathlib import Path
from datetime import datetime

TOKEN = os.environ.get("SKETCHFAB_API_TOKEN")
if not TOKEN:
    print("ERROR: SKETCHFAB_API_TOKEN not set")
    sys.exit(1)

API_BASE = "https://api.sketchfab.com/v3"
RAW_DIR = Path("packs/warfare-starwars/assets/raw")
MANIFEST_FILE = Path("packs/warfare-starwars/assets/SKETCHFAB_DOWNLOADS_COMPLETE.json")

RAW_DIR.mkdir(parents=True, exist_ok=True)
MANIFEST_FILE.parent.mkdir(parents=True, exist_ok=True)

ssl_context = ssl.create_default_context()
ssl_context.check_hostname = False
ssl_context.verify_mode = ssl.CERT_NONE

def api_request(endpoint):
    """Make authenticated API request"""
    url = f"{API_BASE}{endpoint}"
    try:
        req = urllib.request.Request(
            url,
            headers={"Authorization": f"Token {TOKEN}"}
        )
        with urllib.request.urlopen(req, context=ssl_context, timeout=30) as response:
            return json.loads(response.read().decode())
    except Exception as e:
        print(f"    API Error: {e}")
        return None

def download_file(url, output_path):
    """Download file from S3"""
    try:
        req = urllib.request.Request(url)
        req.add_header('User-Agent', 'Mozilla/5.0')

        with urllib.request.urlopen(req, context=ssl_context, timeout=60) as response:
            file_size = int(response.headers.get('content-length', 0))
            with open(output_path, 'wb') as out:
                chunk_size = 8192
                downloaded = 0

                while True:
                    chunk = response.read(chunk_size)
                    if not chunk:
                        break
                    out.write(chunk)
                    downloaded += len(chunk)
                    if file_size > 0:
                        pct = int((downloaded / file_size) * 100)
                        print(f"\r  Downloading... {pct}%", end="", flush=True)

                print()
                return True
    except Exception as e:
        print(f"    Download Error: {e}")
        return False

def download_model(model_id, asset_id):
    """Download single model"""
    print(f"\n{asset_id}")
    asset_dir = RAW_DIR / asset_id
    asset_dir.mkdir(parents=True, exist_ok=True)

    # Get model info
    model_info = api_request(f"/models/{model_id}/")
    if not model_info:
        print("  ERROR: Could not fetch model info")
        return None

    model_name = model_info.get("name", "Unknown")
    vertex_count = model_info.get("vertexCount", 0)
    face_count = model_info.get("faceCount", 0)
    print(f"  {model_name}")

    time.sleep(1)  # Rate limit

    # Get download URLs
    download_info = api_request(f"/models/{model_id}/download")
    if not download_info:
        print("  WARNING: Not available for download (restricted)")
        return None

    # Find best format
    url = None
    fmt = None
    size_remote = 0

    for format_name in ["glb", "gltf", "source"]:
        if format_name in download_info and "url" in download_info[format_name]:
            url = download_info[format_name]["url"]
            fmt = format_name
            size_remote = download_info[format_name].get("size", 0)
            break

    if not url:
        print("  ERROR: No download URL found")
        return None

    print(f"  Format: {fmt} (~{size_remote // (1024*1024)}MB)")

    # Download
    output_file = asset_dir / f"{asset_id}.{fmt}"
    if not download_file(url, output_file):
        print(f"  ERROR: Download failed")
        return None

    file_size = output_file.stat().st_size
    print(f"  OK: {file_size / (1024*1024):.1f}MB")

    # Create manifest
    manifest = {
        "asset_id": asset_id,
        "sketchfab_model_id": model_id,
        "model_name": model_name,
        "file_path": str(output_file.relative_to(Path.cwd())),
        "file_size": file_size,
        "format": fmt,
        "vertex_count": vertex_count,
        "face_count": face_count,
        "downloaded": datetime.utcnow().isoformat() + "Z",
        "status": "complete"
    }

    with open(asset_dir / "asset_manifest.json", 'w') as f:
        json.dump(manifest, f, indent=2)

    return manifest

# List of models to try downloading
models_to_download = [
    ("cf2f3b9d265e45f7ab36b440c8bf690d", "sw_star_wars_building_cf2f3b9d"),  # Already downloaded
    ("db883b9fd31f438e94abd3cb8cb3ead5", "sw_star_wars_building_db883b9f"),
    ("b5a2140fa2264bac8a4b6de3693272b6", "sw_star_wars_building_b5a2140f"),
    ("f7752038addb4ba1bbd5783ac44bd989", "sw_star_wars_structure_f7752038"),
    ("64b34e669f7740e3a9060e4deb2658de", "sw_star_wars_structure_64b34e66"),
    ("f4a202a3dfdf4abba7715239c6230935", "sw_star_wars_structure_f4a202a3"),
    ("88cdaa95723449d4ab8ee64632a2c91a", "sw_star_wars_structure_88cdaa95"),
    ("5e098a79422b476388f59b189c5bbd4e", "sw_star_wars_droid_5e098a79"),
]

print("="*80)
print("SKETCHFAB DOWNLOADER - STAR WARS ASSETS")
print("="*80)

all_assets = []
success = 0
failed = 0

for model_id, asset_id in models_to_download:
    result = download_model(model_id, asset_id)
    if result:
        all_assets.append(result)
        success += 1
    else:
        failed += 1

    time.sleep(1)  # Delay between downloads

# Create final manifest
manifest = {
    "generated": datetime.utcnow().isoformat() + "Z",
    "total_assets": len(all_assets),
    "successful": success,
    "failed": failed,
    "assets": all_assets
}

with open(MANIFEST_FILE, 'w') as f:
    json.dump(manifest, f, indent=2)

print("\n" + "="*80)
print("SUMMARY")
print("="*80)
print(f"Downloaded: {success}")
print(f"Failed/Skipped: {failed}")
print(f"Manifest: {MANIFEST_FILE}")
print("="*80)
