"""Vercel AI SDK bridge stubs."""

from __future__ import annotations

import os
from typing import Any, Mapping

from .preferences import PROVIDER_VERCEL_AI_SDK


def is_configured() -> bool:
    """Return True only when Vercel credentials are available."""
    return bool(
        os.getenv("VERCEL_AI_API_KEY")
        or os.getenv("VERCEL_API_KEY")
        or os.getenv("VERCEL_AI_GATEWAY_URL")
    )


def route(request: Mapping[str, Any] | None, operation: str) -> dict[str, Any]:
    """
    Execute a Vercel AI SDK route. Current implementation is a stub and returns
    an explicit status payload to make integration points visible.
    """
    if not is_configured():
        return {
            "provider": PROVIDER_VERCEL_AI_SDK,
            "status": "stub",
            "configured": False,
            "reason": "Missing VERCEL_AI_API_KEY (or VERCEL_API_KEY).",
            "operation": operation,
        }

    return {
        "provider": PROVIDER_VERCEL_AI_SDK,
        "status": "stub",
        "configured": True,
        "operation": operation,
        "message": (
            "Vercel AI SDK adapter is scaffolded in this stack."
            " Wire outbound traffic to production SDK client before enabling."
        ),
        "request": dict(request or {}),
    }
