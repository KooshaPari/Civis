"""Shared regex compile timeout for CI detect scripts (Sonar ReDoS)."""
from __future__ import annotations

import re
import sys

REGEX_TIMEOUT = 1


def compile(pattern: str, flags: int = 0) -> re.Pattern[str]:
    try:
        return re.compile(pattern, flags, timeout=REGEX_TIMEOUT)
    except TypeError:
        # re.compile(..., timeout=...) requires Python 3.13+
        return re.compile(pattern, flags)
