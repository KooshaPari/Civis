#!/usr/bin/env python3
"""Generate TTS audio files from a voice-over spec JSON.

Usage:
    python generate_tts.py --spec scripts/video/vo_spec.json --out $TEMP/DINOForge/tts

Each entry in the spec JSON must have: id, voice, text.
Outputs one MP3 per entry named <id>.mp3.
"""
import argparse
import asyncio
import io
import json
import os
import sys

import edge_tts

# Fix Windows console encoding issues
if sys.stdout.encoding != 'utf-8':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')


async def generate_one(entry: dict, out_dir: str, max_retries: int = 3) -> str:
    out_path = os.path.join(out_dir, f"{entry['id']}.mp3")

    for attempt in range(max_retries):
        try:
            communicate = edge_tts.Communicate(entry["text"], entry["voice"])
            await asyncio.wait_for(communicate.save(out_path), timeout=30.0)
            size = os.path.getsize(out_path)
            if size < 1024:
                raise RuntimeError(
                    f"{entry['id']}.mp3 is only {size} bytes — TTS likely failed silently"
                )
            print(f"  OK  {entry['id']}.mp3  ({size // 1024} KB)")
            return out_path
        except (asyncio.TimeoutError, Exception) as e:
            if attempt < max_retries - 1:
                wait_time = 2 ** attempt  # exponential backoff: 1s, 2s, 4s
                print(f"  RETRY {entry['id']}.mp3 (attempt {attempt + 1}/{max_retries}, backoff {wait_time}s): {type(e).__name__}", file=sys.stderr)
                await asyncio.sleep(wait_time)
            else:
                raise RuntimeError(f"Failed to generate {entry['id']}.mp3 after {max_retries} attempts: {e}")


async def main() -> None:
    parser = argparse.ArgumentParser(description="Generate TTS MP3s from vo_spec.json")
    parser.add_argument("--spec", required=True, help="Path to vo_spec.json")
    parser.add_argument("--out", required=True, help="Output directory for MP3 files")
    args = parser.parse_args()

    with open(args.spec, encoding="utf-8") as f:
        spec = json.load(f)

    os.makedirs(args.out, exist_ok=True)
    print(f"Generating {len(spec)} TTS file(s) -> {args.out}")

    tasks = [generate_one(entry, args.out) for entry in spec]
    results = await asyncio.gather(*tasks, return_exceptions=True)

    failures = [r for r in results if isinstance(r, Exception)]
    if failures:
        for err in failures:
            print(f"ERROR: {err}", file=sys.stderr)
        sys.exit(1)

    print(f"\nDone. {len(results)} file(s) written to {args.out}")


if __name__ == "__main__":
    asyncio.run(main())
