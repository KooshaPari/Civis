"""Shared regex compile timeout for CI detect scripts (Sonar ReDoS)."""
from __future__ import annotations

import re
import sys

REGEX_TIMEOUT = 1


def compile(pattern: str, flags: int = 0) -> re.Pattern[str]:
    if sys.version_info >= (3, 11):
        return re.compile(pattern, flags, timeout=REGEX_TIMEOUT)
    return re.compile(pattern, flags)
