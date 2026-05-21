"""Utilities for stack-aware AI routing preferences.

This package intentionally contains stubs and preference plumbing only.
"""

from .preferences import (
    DEFAULT_PREFERENCE_ORDER,
    PROVIDER_BIFROST,
    PROVIDER_FASTMCP,
    PROVIDER_VERCEL_AI_SDK,
    PREF_ENV_VAR,
    StackPreferences,
    normalize_provider,
    normalize_provider_list,
)
from .routing import route_ai_request, get_provider_status

__all__ = [
    "DEFAULT_PREFERENCE_ORDER",
    "PROVIDER_BIFROST",
    "PROVIDER_FASTMCP",
    "PROVIDER_VERCEL_AI_SDK",
    "PREF_ENV_VAR",
    "StackPreferences",
    "get_provider_status",
    "normalize_provider",
    "normalize_provider_list",
    "route_ai_request",
]
