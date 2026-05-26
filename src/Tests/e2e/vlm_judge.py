"""
VLM Judge module for screenshot validation using Anthropic Claude API.

Provides async functions to validate screenshots against natural language assertions.
"""
import asyncio
import base64
import json
import re
from pathlib import Path
from typing import Any

import anthropic


async def judge_screenshot(
    screenshot_path: str | Path,
    assertion: str,
    model: str = "claude-haiku-4-5-20251001",
    max_tokens: int = 256,
) -> dict[str, Any]:
    """
    Use Claude as a VLM judge to validate a screenshot against an assertion.
    """
    try:
        screenshot_path = Path(screenshot_path)
        if not screenshot_path.exists():
            return {
                "pass": False,
                "confidence": 0.0,
                "reason": f"Screenshot file not found: {screenshot_path}",
                "error": "FileNotFoundError",
            }

        img_bytes = screenshot_path.read_bytes()
        img_base64 = base64.standard_b64encode(img_bytes).decode("utf-8")

        prompt = (
            f"Analyze this screenshot and answer: Does it show the following? "
            f"{assertion}\n\n"
            f"Respond with ONLY valid JSON (no markdown, no extra text):\n"
            f'{{"pass": true/false, "confidence": 0.0-1.0, "reason": "one sentence"}}'
        )

        client = anthropic.Anthropic()
        response = client.messages.create(
            model=model,
            max_tokens=max_tokens,
            messages=[
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": "image/png",
                                "data": img_base64,
                            },
                        },
                        {"type": "text", "text": prompt},
                    ],
                }
            ],
        )

        response_text = response.content[0].text.strip()
        json_match = re.search(r"\{.*\}", response_text, re.DOTALL)
        if not json_match:
            return {
                "pass": False,
                "confidence": 0.0,
                "reason": "VLM did not return JSON",
                "error": "ParseError",
            }

        result = json.loads(json_match.group())
        result.setdefault("error", None)
        return result

    except json.JSONDecodeError as e:
        return {"pass": False, "confidence": 0.0, "reason": f"Parse error: {str(e)}", "error": "JSONDecodeError"}
    except anthropic.APIError as e:
        return {"pass": False, "confidence": 0.0, "reason": f"API error: {str(e)}", "error": "APIError"}
    except Exception as e:
        return {"pass": False, "confidence": 0.0, "reason": f"Error: {str(e)}", "error": "UnexpectedError"}


async def judge_screenshot_batch(
    assertions: list[tuple[str | Path, str]],
    model: str = "claude-haiku-4-5-20251001",
) -> list[dict[str, Any]]:
    """Judge multiple screenshots in parallel."""
    tasks = [
        judge_screenshot(path, assertion, model=model)
        for path, assertion in assertions
    ]
    return await asyncio.gather(*tasks)


def judge_screenshot_sync(
    screenshot_path: str | Path,
    assertion: str,
    model: str = "claude-haiku-4-5-20251001",
) -> dict[str, Any]:
    """Synchronous wrapper for judge_screenshot."""
    try:
        loop = asyncio.get_event_loop()
        if loop.is_running():
            raise RuntimeError("Already in async context. Use judge_screenshot() instead.")
    except RuntimeError:
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)

    return loop.run_until_complete(
        judge_screenshot(screenshot_path, assertion, model=model)
    )
