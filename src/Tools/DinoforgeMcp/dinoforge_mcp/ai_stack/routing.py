"""Routing helpers for AI stack preference selection."""

from __future__ import annotations

from typing import Any, Mapping, Sequence

from .preferences import (
    DEFAULT_PREFERENCE_ORDER,
    PROVIDER_BIFROST,
    PROVIDER_FASTMCP,
    PROVIDER_VERCEL_AI_SDK,
    StackPreferences,
    get_preference_order,
    normalize_provider,
    normalize_provider_list,
)
from . import bifrost_bridge, vercel_bridge


KNOWN_PROVIDER_IDS = (
    PROVIDER_FASTMCP,
    PROVIDER_VERCEL_AI_SDK,
    PROVIDER_BIFROST,
)


def get_provider_status() -> dict[str, bool]:
    """Return live availability state for each provider."""
    return {
        PROVIDER_FASTMCP: True,
        PROVIDER_VERCEL_AI_SDK: vercel_bridge.is_configured(),
        PROVIDER_BIFROST: bifrost_bridge.is_configured(),
    }


def _dispatch(provider: str, request_payload: Mapping[str, Any] | None, operation: str) -> dict[str, Any]:
    if provider == PROVIDER_FASTMCP:
        return {
            "provider": PROVIDER_FASTMCP,
            "status": "pass",
            "configured": True,
            "operation": operation,
            "message": (
                "FastMCP is the active existing protocol."
                " Route through standard MCP tool calls for real execution."
            ),
            "request": dict(request_payload or {}),
        }
    if provider == PROVIDER_VERCEL_AI_SDK:
        return vercel_bridge.route(request_payload, operation)
    if provider == PROVIDER_BIFROST:
        return bifrost_bridge.route(request_payload, operation)

    return {
        "provider": provider,
        "status": "unsupported",
        "configured": False,
        "operation": operation,
        "reason": f"Unsupported provider '{provider}'",
    }


def _select_candidates(
    preference: StackPreferences,
    requested_provider: str | None,
    explicit_order: Sequence[str] | None = None,
) -> list[str]:
    if explicit_order:
        candidates = normalize_provider_list(list(explicit_order))
        if requested_provider and requested_provider not in candidates:
            norm = normalize_provider(requested_provider)
            if norm and norm not in candidates:
                candidates.insert(0, norm)
    elif requested_provider:
        norm = normalize_provider(requested_provider)
        candidates = [norm] if norm else []
        if not candidates:
            return list(preference.order)
        candidates.extend(provider for provider in preference.order if provider != norm)
    else:
        candidates = list(preference.order)

    # If nothing usable was derived, use known defaults so routing always has a
    # fallback path instead of failing hard.
    if not candidates:
        return list(DEFAULT_PREFERENCE_ORDER)
    return candidates


def route_ai_request(
    request_payload: Mapping[str, Any] | None,
    *,
    operation: str = "chat",
    requested_provider: str | None = None,
    preference_raw: str | None = None,
    explicit_preference_order: Sequence[str] | None = None,
) -> dict[str, Any]:
    """
    Resolve provider order and execute a stub bridge dispatch.

    If a requested provider is unavailable, routing continues through remaining
    preferences. FastMCP remains always available as the terminal fallback.
    """
    preference = get_preference_order(preference_raw)
    requested_normalized = normalize_provider(requested_provider)
    candidates = _select_candidates(
        preference,
        requested_normalized,
        explicit_preference_order,
    )

    status = get_provider_status()
    resolved: list[str] = []
    for provider in candidates:
        if provider not in KNOWN_PROVIDER_IDS:
            continue
        if provider != PROVIDER_FASTMCP and not status[provider]:
            resolved.append(provider)
            continue

        dispatch = _dispatch(provider, request_payload, operation)
        return {
            "resolved_provider": provider,
            "status": dispatch.get("status", "stub"),
            "request": dict(request_payload or {}),
            "operation": operation,
            "requested_provider": requested_normalized,
            "preference_order": candidates,
            "provider_status": status,
            "dispatch": dispatch,
            "attempted_unavailable": resolved,
        }

    # No provider was able to handle the request. This should only happen in
    # strict external-bridge mode; return explicit diagnostics instead of silent
    # failure.
    return {
        "resolved_provider": None,
        "status": "unavailable",
        "request": dict(request_payload or {}),
        "operation": operation,
        "requested_provider": requested_normalized,
        "preference_order": candidates,
        "provider_status": status,
        "dispatch": {
            "status": "unavailable",
            "reason": "No provider in preference order is configured.",
            "attempted_unavailable": resolved,
        },
    }
