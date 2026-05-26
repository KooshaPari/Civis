#!/usr/bin/env python3
"""
Sketchfab Star Wars Asset Downloader
Properly downloads models from Sketchfab API v3
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
from typing import Dict, List, Tuple, Optional

class SketchfabDownloader:
    def __init__(self, token: str):
        self.token = token
        self.api_base = "https://api.sketchfab.com/v3"
        self.pack_root = Path("packs/warfare-starwars/assets")
        self.raw_dir = self.pack_root / "raw"
        self.log_file = self.pack_root / "DOWNLOAD_LOG.txt"
        self.manifest_file = self.pack_root / "SKETCHFAB_DOWNLOADS_COMPLETE.json"

        # Setup SSL context (disable verification for self-signed certs)
        self.ssl_context = ssl.create_default_context()
        self.ssl_context.check_hostname = False
        self.ssl_context.verify_mode = ssl.CERT_NONE

        # Create directories
        self.raw_dir.mkdir(parents=True, exist_ok=True)
        self.log_file.parent.mkdir(parents=True, exist_ok=True)

        self.success_count = 0
        self.fail_count = 0
        self.assets = []

    def log(self, msg: str, also_print: bool = True):
        """Log message to file and optionally print"""
        timestamp = ""
        with open(self.log_file, 'a') as f:
            f.write(msg + "\n")
        if also_print:
            print(msg)

    def api_request(self, endpoint: str, method: str = "GET") -> Optional[Dict]:
        """Make authenticated API request with rate limiting"""
        url = f"{self.api_base}{endpoint}"
        try:
            req = urllib.request.Request(
                url,
                headers={"Authorization": f"Token {self.token}"}
            )
            with urllib.request.urlopen(req, context=self.ssl_context, timeout=30) as response:
                result = json.loads(response.read().decode())
                time.sleep(0.5)  # Rate limiting
                return result
        except urllib.error.HTTPError as e:
            if e.code == 429:  # Rate limited
                self.log(f"    API Rate Limited: Waiting 5 seconds...", False)
                time.sleep(5)
                return None
            elif e.code == 403:  # Forbidden
                self.log(f"    API Forbidden (403): Model not available for download", False)
                return None
            else:
                self.log(f"    API Error ({e.code}): {e.reason}", False)
                return None
        except Exception as e:
            self.log(f"    API Error: {e}", False)
            return None

    def download_file(self, url: str, output_path: Path) -> bool:
        """Download file from S3 signed URL"""
        try:
            # S3 signed URLs don't need auth header, but include it anyway
            req = urllib.request.Request(url)
            # Add user agent to appear more like a browser
            req.add_header('User-Agent', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)')

            with urllib.request.urlopen(req, context=self.ssl_context, timeout=60) as response:
                with open(output_path, 'wb') as out:
                    # Read in chunks
                    chunk_size = 8192
                    total_size = int(response.headers.get('content-length', 0))
                    downloaded = 0

                    while True:
                        chunk = response.read(chunk_size)
                        if not chunk:
                            break
                        out.write(chunk)
                        downloaded += len(chunk)
                        if total_size > 0:
                            pct = int((downloaded / total_size) * 100)
                            print(f"\r  Downloading... {pct}%", end="", flush=True)

                    print()  # New line after progress
                    return True
        except Exception as e:
            self.log(f"    Download Error: {e}", False)
            return False

    def download_model(self, model_id: str, asset_id: str) -> bool:
        """Download a single model"""
        print(f"\nDownloading: {asset_id} ({model_id})")
        self.log(f"\n--- {asset_id} ({model_id}) ---", False)

        asset_dir = self.raw_dir / asset_id
        asset_dir.mkdir(parents=True, exist_ok=True)

        # Throttle between requests
        time.sleep(0.5)

        # Get model metadata
        self.log("  Getting model info...", False)
        model_info = self.api_request(f"/models/{model_id}/")
        if not model_info:
            self.log(f"  ERROR: Failed to fetch model info for {model_id}")
            self.fail_count += 1
            return False

        model_name = model_info.get("name", "Unknown")
        vertex_count = model_info.get("vertexCount", 0)
        face_count = model_info.get("faceCount", 0)

        self.log(f"  Model: {model_name}", False)

        # Get download URLs
        self.log("  Fetching download URLs...", False)
        download_info = self.api_request(f"/models/{model_id}/download")
        if not download_info:
            self.log(f"  WARNING: Download not available (restricted model)")
            self.fail_count += 1
            return False

        # Try formats in order: glb > gltf > source
        download_url = None
        file_format = None
        file_size_remote = 0

        for fmt in ["glb", "gltf", "source"]:
            if fmt in download_info and "url" in download_info[fmt]:
                download_url = download_info[fmt]["url"]
                file_format = fmt
                file_size_remote = download_info[fmt].get("size", 0)
                break

        if not download_url:
            self.log(f"  ERROR: No downloadable format found for {model_id}")
            self.fail_count += 1
            return False

        self.log(f"  Format: {file_format} (~{file_size_remote // (1024*1024)}MB)", False)

        # Download the file
        output_file = asset_dir / f"{asset_id}.{file_format}"
        self.log(f"  Downloading from S3...", False)

        if not self.download_file(download_url, output_file):
            self.log(f"  ERROR: Failed to download {model_id}")
            self.fail_count += 1
            return False

        file_size_actual = output_file.stat().st_size
        file_size_human = self._format_bytes(file_size_actual)
        self.log(f"  ✓ Downloaded: {file_size_human}")

        # Create asset manifest
        manifest = {
            "asset_id": asset_id,
            "sketchfab_model_id": model_id,
            "model_name": model_name,
            "file_path": str(output_file),
            "file_size": file_size_actual,
            "format": file_format,
            "vertex_count": vertex_count,
            "face_count": face_count,
            "downloaded": datetime.utcnow().isoformat() + "Z",
            "status": "complete"
        }

        with open(asset_dir / "asset_manifest.json", 'w') as f:
            json.dump(manifest, f, indent=2)

        self.assets.append(manifest)
        self.success_count += 1
        return True

    def search_and_download(self, query: str, max_results: int = 5) -> int:
        """Search for models and download top results"""
        print(f"\nSearching: {query}")
        self.log(f"\n--- Search: {query} ---", False)

        # URL encode query
        query_encoded = urllib.parse.quote(query)
        endpoint = f"/search?q={query_encoded}&sort_by=-likeCount&count={max_results}"

        results = self.api_request(endpoint)
        if not results or "results" not in results:
            self.log(f"  No results found", False)
            return 0

        models = results.get("results", {}).get("models", [])
        self.log(f"  Found {len(models)} models", False)

        downloaded = 0
        for i, model in enumerate(models[:max_results], 1):
            model_id = model.get("uid", "")
            model_name = model.get("name", "")

            if not model_id:
                continue

            # Create safe asset ID from query and model ID
            safe_query = query.replace(" ", "_").lower()
            asset_id = f"sw_{safe_query}_{model_id[:8]}"

            # Check if already downloaded
            if (self.raw_dir / asset_id).exists():
                self.log(f"    {i}. {model_name} - SKIPPED (already downloaded)", False)
                continue

            self.log(f"    {i}. {model_name}", False)

            if self.download_model(model_id, asset_id):
                downloaded += 1

        return downloaded

    def _format_bytes(self, bytes: int) -> str:
        """Format bytes to human readable"""
        for unit in ['B', 'KB', 'MB', 'GB']:
            if bytes < 1024:
                return f"{bytes:.1f}{unit}"
            bytes /= 1024
        return f"{bytes:.1f}TB"

    def create_manifest(self):
        """Create final manifest file"""
        manifest = {
            "generated": datetime.utcnow().isoformat() + "Z",
            "total_assets": len(self.assets),
            "successful": self.success_count,
            "failed": self.fail_count,
            "assets": self.assets
        }

        with open(self.manifest_file, 'w') as f:
            json.dump(manifest, f, indent=2)

        self.log(f"\nManifest created: {self.manifest_file}")
        self.log(f"Total assets: {len(self.assets)}")
        self.log(f"Successful: {self.success_count}")
        self.log(f"Failed: {self.fail_count}")

    def run(self):
        """Run the downloader"""
        self.log(f"\nStarting Sketchfab asset downloads...")
        self.log(f"Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

        print("\n" + "="*80)
        print("SKETCHFAB DOWNLOADER - STAR WARS ASSETS")
        print("="*80)

        # Phase 1: Download known models
        print("\n=== Phase 1: Downloading known Star Wars models ===")
        self.log("\n=== Phase 1: Known models ===", False)

        # Using actual valid models found via search
        known_models = {
            "sw_venator_prefab_001": "8a1e1760391c4ac6a50373c2bf5efa2e",
        }

        for asset_id, model_id in known_models.items():
            self.download_model(model_id, asset_id)

        # Phase 2: Search for additional models
        print("\n=== Phase 2: Searching for additional models ===")
        self.log("\n=== Phase 2: Search results ===", False)

        searches = [
            ("star wars building", 5),
            ("star wars structure", 5),
            ("star wars droid", 5),
        ]

        for query, max_results in searches:
            self.search_and_download(query, max_results)

        # Create final manifest
        print("\n=== Phase 3: Creating manifest ===")
        self.create_manifest()

        # Summary
        print("\n" + "="*80)
        print("DOWNLOAD SUMMARY")
        print("="*80)
        print(f"Successful: {self.success_count}")
        print(f"Failed: {self.fail_count}")
        print(f"Log file: {self.log_file}")
        print(f"Manifest: {self.manifest_file}")
        print(f"Assets: {self.raw_dir}")
        print("="*80)

        return 0 if self.fail_count == 0 else 1


if __name__ == "__main__":
    token = os.environ.get("SKETCHFAB_API_TOKEN")
    if not token:
        print("ERROR: SKETCHFAB_API_TOKEN environment variable not set")
        sys.exit(1)

    downloader = SketchfabDownloader(token)
    sys.exit(downloader.run())
