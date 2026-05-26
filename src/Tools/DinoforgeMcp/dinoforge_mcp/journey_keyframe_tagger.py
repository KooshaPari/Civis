"""Journey Keyframe Tagger — Extract semantic keyframes from BepInEx log streams.

This module reads a BepInEx log file (line-streamed or batch) and emits a
``keyframes.json`` describing each detected event token alongside its log
metadata. Keyframes are intended to be consumed by the journey-recording
pipeline (e.g. Remotion compositions, journey-doc viewers) so that a video
recording can be tagged at meaningful runtime moments instead of relying on
hand-curated timestamps.

Event tokens detected (configurable):
    - RuntimeDriver.OnDestroy
    - AssetSwapSystem.OnUpdate
    - PackUnitSpawner.Initialize
    - AerialSpawnSystem.Initialize
    - F9 pressed
    - F10 pressed
    - AssetBundleCache.Unload
    - EventSystem.current = null

Output schema (keyframes.json):
    {
        "source_log": "<path>",
        "scanned_at_utc": "<iso8601>",
        "frame_count_hint": <int>,  # heuristic: 1 frame per line at 60fps
        "keyframes": [
            {
                "frame_index": <int>,
                "timestamp_iso": "<iso8601 from BepInEx prefix or null>",
                "timestamp_raw": "<raw timestamp string from log or null>",
                "line_number": <int>,
                "event_token": "<token>",
                "log_line": "<full line>"
            },
            ...
        ]
    }

Standalone CLI:
    python -m dinoforge_mcp.journey_keyframe_tagger <log_path> [<out_json>]
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable, List, Optional, Sequence


# ---------------------------------------------------------------------------
# Event token table
# ---------------------------------------------------------------------------

# Each entry is (canonical_token, list-of-regex-patterns).  Patterns are matched
# case-sensitively against the log line; the FIRST match wins per line, and a
# single line may produce multiple keyframes if more than one distinct token
# appears (rare but possible for compound log messages).
DEFAULT_EVENT_TOKENS: Sequence[tuple[str, Sequence[str]]] = (
    ("RuntimeDriver.OnDestroy",      (r"RuntimeDriver\.OnDestroy",
                                       r"\[RuntimeDriver\].*OnDestroy")),
    ("AssetSwapSystem.OnUpdate",     (r"AssetSwapSystem\.OnUpdate",
                                       r"\[AssetSwapSystem\].*OnUpdate")),
    ("PackUnitSpawner.Initialize",   (r"PackUnitSpawner\.Initialize",
                                       r"\[PackUnitSpawner\].*Initialize")),
    ("AerialSpawnSystem.Initialize", (r"AerialSpawnSystem\.Initialize",
                                       r"\[AerialSpawnSystem\].*Initialize")),
    ("F9 pressed",                   (r"F9\s+pressed", r"F9 key pressed",
                                       r"OnF9Pressed")),
    ("F10 pressed",                  (r"F10\s+pressed", r"F10 key pressed",
                                       r"OnF10Pressed")),
    ("AssetBundleCache.Unload",      (r"AssetBundleCache\.Unload",
                                       r"\[AssetBundleCache\].*Unload")),
    ("EventSystem.current = null",   (r"EventSystem\.current\s*=\s*null",
                                       r"EventSystem\.current is null",
                                       r"No EventSystem")),
)


# BepInEx log line prefix: "[Info   :   DINOForge.Runtime] ..." or with timestamp
# Common: "[2026-05-19 14:32:01.123] [Info   : DINOForge.Runtime] message"
_TIMESTAMP_REGEX = re.compile(
    r"\[(?P<ts>\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:[.,]\d+)?Z?)\]"
)


# ---------------------------------------------------------------------------
# Data models
# ---------------------------------------------------------------------------

@dataclass
class Keyframe:
    frame_index: int
    line_number: int
    event_token: str
    log_line: str
    timestamp_iso: Optional[str] = None
    timestamp_raw: Optional[str] = None

    def to_dict(self) -> dict:
        return {
            "frame_index": self.frame_index,
            "timestamp_iso": self.timestamp_iso,
            "timestamp_raw": self.timestamp_raw,
            "line_number": self.line_number,
            "event_token": self.event_token,
            "log_line": self.log_line,
        }


@dataclass
class TaggerConfig:
    """Configuration for the keyframe tagger."""
    # Default to a 60fps frame index hint; callers can override.
    frames_per_log_line: float = 1.0
    event_tokens: Sequence[tuple[str, Sequence[str]]] = field(
        default_factory=lambda: DEFAULT_EVENT_TOKENS
    )


# ---------------------------------------------------------------------------
# Parsing helpers
# ---------------------------------------------------------------------------

def _extract_timestamp(line: str) -> tuple[Optional[str], Optional[str]]:
    """Try to extract a BepInEx-style timestamp from a log line.

    Returns (iso_string_or_none, raw_string_or_none).
    """
    m = _TIMESTAMP_REGEX.search(line)
    if not m:
        return None, None
    raw = m.group("ts")
    iso = raw
    try:
        # Normalize "yyyy-mm-dd HH:MM:SS.fff[Z]" → ISO 8601
        normalized = raw.replace(",", ".")
        if "T" not in normalized:
            normalized = normalized.replace(" ", "T", 1)
        # Python <3.11 fromisoformat rejects trailing 'Z' and >6-digit fractional secs.
        if normalized.endswith("Z"):
            normalized = normalized[:-1] + "+00:00"
        # Truncate fractional seconds to 6 digits (microsecond precision).
        m2 = re.match(r"(.*\.\d{6})\d+(.*)$", normalized)
        if m2:
            normalized = m2.group(1) + m2.group(2)
        parsed = datetime.fromisoformat(normalized)
        iso = parsed.isoformat()
    except ValueError:
        pass
    return iso, raw


def _compile_patterns(
    tokens: Sequence[tuple[str, Sequence[str]]],
) -> List[tuple[str, List[re.Pattern[str]]]]:
    return [
        (token, [re.compile(p) for p in patterns])
        for token, patterns in tokens
    ]


# ---------------------------------------------------------------------------
# Core tagger
# ---------------------------------------------------------------------------

def tag_lines(
    lines: Iterable[str],
    config: Optional[TaggerConfig] = None,
) -> List[Keyframe]:
    """Process an iterable of log lines and return a list of Keyframe objects.

    Pure-functional core — does no I/O. ``frame_index`` is assigned as
    ``round(line_number * frames_per_log_line)`` which is a deliberately coarse
    heuristic; for precise alignment with a video stream, callers should
    post-process using the actual recording fps.
    """
    cfg = config or TaggerConfig()
    compiled = _compile_patterns(cfg.event_tokens)
    keyframes: List[Keyframe] = []

    for idx, raw_line in enumerate(lines, start=1):
        line = raw_line.rstrip("\r\n")
        if not line:
            continue

        iso_ts, raw_ts = _extract_timestamp(line)
        frame_idx = int(round(idx * cfg.frames_per_log_line))

        for token, patterns in compiled:
            for pat in patterns:
                if pat.search(line):
                    keyframes.append(Keyframe(
                        frame_index=frame_idx,
                        line_number=idx,
                        event_token=token,
                        log_line=line,
                        timestamp_iso=iso_ts,
                        timestamp_raw=raw_ts,
                    ))
                    break  # Only emit one keyframe per (line, token).

    return keyframes


def tag_file(
    log_path: Path,
    config: Optional[TaggerConfig] = None,
) -> List[Keyframe]:
    """Read a BepInEx log file (UTF-8, fall back to latin-1) and return keyframes."""
    encodings = ("utf-8", "utf-8-sig", "latin-1")
    last_err: Optional[Exception] = None
    for enc in encodings:
        try:
            with log_path.open("r", encoding=enc, errors="strict") as f:
                return tag_lines(f, config=config)
        except UnicodeDecodeError as e:
            last_err = e
            continue
    # Fallback: read with errors='replace' to never crash.
    with log_path.open("r", encoding="utf-8", errors="replace") as f:
        return tag_lines(f, config=config)


def build_output(
    log_path: Path,
    keyframes: Sequence[Keyframe],
    config: Optional[TaggerConfig] = None,
) -> dict:
    """Build the final keyframes.json-ready dict for serialization."""
    cfg = config or TaggerConfig()
    line_count = (max((k.line_number for k in keyframes), default=0))
    return {
        "source_log": str(log_path),
        "scanned_at_utc": datetime.now(timezone.utc).isoformat(),
        "frame_count_hint": int(round(line_count * cfg.frames_per_log_line)),
        "keyframes": [k.to_dict() for k in keyframes],
    }


def write_keyframes_json(
    log_path: Path,
    out_path: Path,
    config: Optional[TaggerConfig] = None,
) -> dict:
    """Convenience helper: read log, tag keyframes, write keyframes.json, return dict."""
    keyframes = tag_file(log_path, config=config)
    payload = build_output(log_path, keyframes, config=config)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    return payload


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def _build_argparser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="journey_keyframe_tagger",
        description="Tag a BepInEx log with semantic journey keyframes.",
    )
    p.add_argument("log_path", type=Path, help="Path to BepInEx dinoforge_debug.log")
    p.add_argument(
        "out_path", type=Path, nargs="?", default=None,
        help="Output JSON path (default: <log_dir>/keyframes.json)",
    )
    p.add_argument(
        "--fps", type=float, default=1.0,
        help="Frames per log line (default: 1.0). Tune to match recording fps if known.",
    )
    return p


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = _build_argparser().parse_args(argv)
    log_path: Path = args.log_path
    if not log_path.exists():
        print(f"ERROR: log file not found: {log_path}", file=sys.stderr)
        return 2

    out_path: Path = args.out_path or (log_path.parent / "keyframes.json")
    cfg = TaggerConfig(frames_per_log_line=args.fps)
    payload = write_keyframes_json(log_path, out_path, config=cfg)
    kf_count = len(payload["keyframes"])
    print(f"Tagged {kf_count} keyframe(s) from {log_path} -> {out_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
