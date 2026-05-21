"""Bifrost bridge stubs."""

from __future__ import annotations

import os
from typing import Any, Mapping

from .preferences import PROVIDER_BIFROST


def is_configured() -> bool:
    """Return True when a Bifrost endpoint is configured."""
    return bool(
        os.getenv("BIFROST_ENDPOINT")
        or os.getenv("BIFROST_URL")
        or os.getenv("BIFROST_TOKEN")
    )


def route(request: Mapping[str, Any] | None, operation: str) -> dict[str, Any]:
    """
    Execute a Bifrost route. Current implementation is a stub and returns a
    diagnostic payload so callers can safely switch when backend is available.
    """
    if not is_configured():
        return {
            "provider": PROVIDER_BIFROST,
            "status": "stub",
            "configured": False,
            "reason": "Missing BIFROST_ENDPOINT (or BIFROST_URL).",
            "operation": operation,
        }

    return {
        "provider": PROVIDER_BIFROST,
        "status": "stub",
        "configured": True,
        "operation": operation,
        "message": (
            "Bifrost adapter is scaffolded in this stack."
            " Wire transport + auth path before enabling."
        ),
        "request": dict(request or {}),
    }
