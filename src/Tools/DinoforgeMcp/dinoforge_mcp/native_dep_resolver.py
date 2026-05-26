"""Unified resolver for native binary + game-path discovery (Python side).

Walks: env var -> installer-shipped paths.json -> hardcoded fallbacks -> raises.

Mirror of ``src/Tools/McpServer/NativeDepResolver.cs``. Same shape, same
``paths.json`` schema (logical key -> absolute path) so the C# and Python
sides agree on what the installer publishes.

Installer paths file location:
    Windows: %ProgramData%\\DINOForge\\paths.json
    Linux/macOS: /etc/dinoforge/paths.json
"""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Iterable, Optional


INSTALLER_PATHS_FILENAME = "paths.json"
INSTALLER_SUBDIR = "DINOForge"


class NativeDepNotFound(FileNotFoundError):
    """Raised when no candidate path resolves to an existing file/directory."""


def _installer_paths_file() -> Path:
    """Returns the absolute path to the installer's paths.json (regardless of existence)."""
    if os.name == "nt":
        program_data = os.environ.get("ProgramData", r"C:\ProgramData")
        return Path(program_data) / INSTALLER_SUBDIR / INSTALLER_PATHS_FILENAME
    return Path("/etc") / "dinoforge" / INSTALLER_PATHS_FILENAME


def _try_load_installer_paths() -> Optional[dict]:
    p = _installer_paths_file()
    if not p.exists():
        return None
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None


def _exists(path: str, require_file: bool) -> bool:
    p = Path(path)
    return p.is_file() if require_file else (p.is_file() or p.is_dir())


def resolve(
    key: str,
    env_var: str,
    hardcoded_fallbacks: Iterable[str],
    description: str,
    require_file: bool = True,
) -> str:
    """Resolve a native dependency. Returns absolute path. Raises NativeDepNotFound.

    Args:
        key: Logical key in installer paths.json (e.g. "playcua_native").
        env_var: Environment variable name (e.g. "PLAYCUA_NATIVE_EXE").
        hardcoded_fallbacks: Last-resort paths tried in order.
        description: Human-readable description used in the error message.
        require_file: If True (default) candidates must be files; if False, dirs are accepted.
    """
    fallbacks = list(hardcoded_fallbacks)

    # 1. Env var
    from_env = os.environ.get(env_var)
    if from_env and _exists(from_env, require_file):
        return from_env

    # 2. Installer paths.json
    installer_paths = _try_load_installer_paths()
    from_installer: Optional[str] = None
    if installer_paths and key in installer_paths:
        from_installer = installer_paths[key]
        if from_installer and _exists(from_installer, require_file):
            return from_installer

    # 3. Hardcoded fallbacks
    for fb in fallbacks:
        if fb and _exists(fb, require_file):
            return fb

    # 4. Loud error
    raise NativeDepNotFound(
        f"Native dependency {description!r} (key={key}) not found.\n"
        f"  Env ${env_var}: {from_env or '<unset>'}\n"
        f"  Installer paths.json key: {from_installer or '<not in paths.json>'}\n"
        f"  Hardcoded fallbacks: {fallbacks}\n"
        f"Set ${env_var} or run the DINOForge installer to populate paths.json."
    )


def try_resolve(
    key: str,
    env_var: str,
    hardcoded_fallbacks: Iterable[str],
    require_file: bool = True,
) -> Optional[str]:
    """Try-resolve variant. Returns None instead of raising when nothing matches."""
    try:
        return resolve(key, env_var, hardcoded_fallbacks, key, require_file)
    except NativeDepNotFound:
        return None
