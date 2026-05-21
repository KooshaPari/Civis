"""AI stack preference parsing for routing decisions."""

from __future__ import annotations

import os
from dataclasses import dataclass
from typing import Any


PREF_ENV_VAR = "DINOFORGE_AI_STACK_PREFS"

PROVIDER_FASTMCP = "fastmcp"
PROVIDER_VERCEL_AI_SDK = "vercel_ai_sdk"
PROVIDER_BIFROST = "bifrost"
DEFAULT_PREFERENCE_ORDER = (
    PROVIDER_FASTMCP,
    PROVIDER_VERCEL_AI_SDK,
    PROVIDER_BIFROST,
)

_ALIAS_TO_CANONICAL: dict[str, str] = {
    PROVIDER_FASTMCP: PROVIDER_FASTMCP,
    "mcp": PROVIDER_FASTMCP,
    PROVIDER_VERCEL_AI_SDK: PROVIDER_VERCEL_AI_SDK,
    "vercel": PROVIDER_VERCEL_AI_SDK,
    "vercel_ai": PROVIDER_VERCEL_AI_SDK,
    "vercel-ai": PROVIDER_VERCEL_AI_SDK,
    PROVIDER_BIFROST: PROVIDER_BIFROST,
}


@dataclass(frozen=True)
class StackPreferences:
    """User preference list and source metadata."""

    order: tuple[str, ...]
    raw: str | None = None
    source: str = "default"

    def as_dict(self) -> dict[str, Any]:
        """Serialize preference state for API responses."""
        return {
            "order": list(self.order),
            "raw": self.raw,
            "source": self.source,
        }


def normalize_provider(value: str | None) -> str | None:
    """Normalize provider name aliases to canonical IDs."""
    if not value:
        return None

    key = value.strip().lower()
    if not key:
        return None
    key = key.replace(" ", "_")
    return _ALIAS_TO_CANONICAL.get(key)


def normalize_provider_list(values: list[str] | tuple[str, ...] | None) -> list[str]:
    """Normalize and dedupe an iterable of providers while preserving order."""
    normalized: list[str] = []
    if not values:
        return normalized

    for value in values:
        canon = normalize_provider(value)
        if not canon:
            continue
        if canon not in normalized:
            normalized.append(canon)
    return normalized


def _parse_env_value(raw: str | None) -> list[str]:
    if not raw:
        return []
    parts = [entry.strip() for entry in raw.split(",")]
    return normalize_provider_list(parts)


def get_preference_order(
    raw: str | None = None, *, fallback_default: bool = True
) -> StackPreferences:
    """
    Read preference order from provided input or `DINOFORGE_AI_STACK_PREFS`.

    Any invalid values are ignored and duplicates are removed while preserving
    order. If nothing valid is set, the default preference order is used.
    """
    raw_value = raw if raw is not None else os.getenv(PREF_ENV_VAR)
    ordered = _parse_env_value(raw_value)

    if ordered:
        source = "environment" if raw is None else "request"
        return StackPreferences(order=tuple(ordered), raw=raw_value, source=source)

    if not fallback_default:
        return StackPreferences(order=tuple(), raw=raw_value, source="disabled")

    return StackPreferences(
        order=DEFAULT_PREFERENCE_ORDER,
        raw=raw_value,
        source="default" if raw is None else "request",
    )
