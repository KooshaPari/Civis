"""Tests for MCP bridge pipe-name resolution."""

from dinoforge_mcp import server


def test_select_pipe_name_prefers_configured_pipe_when_present(monkeypatch):
    monkeypatch.setenv("DINOFORGE_PIPE_NAME", "dinoforge_game")
    monkeypatch.setattr(server, "_pipe_exists", lambda pipe_name: pipe_name == "dinoforge_game")

    pipe_name, used_fallback = server._select_pipe_name()

    assert pipe_name == "dinoforge_game"
    assert used_fallback is False


def test_select_pipe_name_falls_back_to_default_when_configured_pipe_missing(monkeypatch):
    monkeypatch.setenv("DINOFORGE_PIPE_NAME", "dinoforge_game")
    monkeypatch.setattr(
        server,
        "_pipe_exists",
        lambda pipe_name: pipe_name == server.DEFAULT_GAME_PIPE_NAME,
    )

    pipe_name, used_fallback = server._select_pipe_name()

    assert pipe_name == server.DEFAULT_GAME_PIPE_NAME
    assert used_fallback is True


def test_select_pipe_name_keeps_explicit_pipe_even_if_default_exists(monkeypatch):
    monkeypatch.delenv("DINOFORGE_PIPE_NAME", raising=False)
    monkeypatch.setattr(server, "_pipe_exists", lambda pipe_name: pipe_name == server.DEFAULT_GAME_PIPE_NAME)

    pipe_name, used_fallback = server._select_pipe_name("custom-pipe")

    assert pipe_name == server.DEFAULT_GAME_PIPE_NAME
    assert used_fallback is True
