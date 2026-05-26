"""Tests for AI stack preference parsing and routing stubs."""

from dinoforge_mcp.ai_stack.preferences import (
    DEFAULT_PREFERENCE_ORDER,
    normalize_provider,
    get_preference_order,
)
from dinoforge_mcp.ai_stack.routing import route_ai_request


def test_normalize_provider_aliases():
    assert normalize_provider("MCP") == "fastmcp"
    assert normalize_provider("vercel ai sdk") == "vercel_ai_sdk"
    assert normalize_provider("bifrost") == "bifrost"
    assert normalize_provider("unknown") is None


def test_default_preference_order():
    assert get_preference_order(fallback_default=True).order == DEFAULT_PREFERENCE_ORDER


def test_route_prefers_explicit_fastmcp():
    result = route_ai_request({"prompt": "test"}, requested_provider="fastmcp")
    assert result["resolved_provider"] == "fastmcp"
    assert result["dispatch"]["status"] == "pass"


def test_route_falls_back_to_fastmcp_when_unavailable():
    # Requesting Vercel first without credentials should route to FastMCP fallback.
    result = route_ai_request(
        {"prompt": "test"},
        requested_provider="vercel_ai_sdk",
        preference_raw="vercel_ai_sdk,fastmcp,bifrost",
    )
    assert result["resolved_provider"] == "fastmcp"
    assert "attempted_unavailable" in result
    assert result["attempted_unavailable"]


def test_route_uses_vercel_stub_when_configured(monkeypatch):
    monkeypatch.setenv("VERCEL_AI_API_KEY", "x")
    result = route_ai_request(
        {"prompt": "test"},
        requested_provider="vercel_ai_sdk",
        preference_raw="vercel_ai_sdk,fastmcp",
    )
    assert result["resolved_provider"] == "vercel_ai_sdk"
    assert result["dispatch"]["status"] == "stub"
    assert result["dispatch"]["configured"] is True
