"""
External VLM judge tier: Kimi via Moonshot API.

Provides deterministic verdicts on game screenshots without fallback.
If MOONSHOT_API_KEY is unset, raises ExternalJudgeUnavailable.
If API fails after retry, raises (no silent fallback to Claude).
"""

import base64
import hashlib
import json
import os
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path
from typing import Any

import httpx


class ExternalJudgeUnavailable(RuntimeError):
    """Raised when Moonshot API is unavailable or key is missing."""

    pass


@dataclass
class JudgeReceipt:
    """Immutable record of a judgment call to Moonshot."""

    model: str
    model_version: str
    timestamp_utc: str
    prompt: str
    screenshot_sha256: str
    raw_response: dict
    verdict: str
    confidence: float | None

    def to_dict(self) -> dict[str, Any]:
        """Convert to JSON-serializable dict."""
        return asdict(self)


class KimiJudgeTier:
    """Calls Moonshot (Kimi) vision API to judge game screenshots."""

    def __init__(
        self,
        api_key: str | None = None,
        model: str = "moonshot-v1-8k-vision-preview",
        timeout: float = 30.0,
    ):
        """
        Initialize Kimi judge tier.

        Args:
            api_key: Moonshot API key. If None, reads from MOONSHOT_API_KEY env var.
            model: Moonshot model ID (default v1-8k-vision-preview).
            timeout: HTTP request timeout in seconds.

        Raises:
            ExternalJudgeUnavailable: If MOONSHOT_API_KEY is not set and api_key is None.
        """
        key = api_key or os.environ.get("MOONSHOT_API_KEY")
        if not key:
            raise ExternalJudgeUnavailable(
                "MOONSHOT_API_KEY not set; refusing silent fallback to Claude"
            )
        self._key = key
        self._model = model
        self._timeout = timeout

    def judge(self, screenshot_path: Path, prompt: str) -> JudgeReceipt:
        """
        Judge a screenshot via Moonshot API.

        Args:
            screenshot_path: Path to screenshot file (PNG, JPEG, WebP, GIF).
            prompt: Judgment prompt (e.g., "Does this screenshot show a red squad bar?").

        Returns:
            JudgeReceipt with verdict, confidence, and full API response.

        Raises:
            ExternalJudgeUnavailable: If API fails after retry or file is unreadable.

        Side effects:
            - Persists the JudgeReceipt as JSON to docs/proof/judge-receipts/<utc>-<sha8>.json
              (relative to the repo root). Atomic write via .tmp file rename.
        """
        # Read and hash the image
        try:
            image_bytes = screenshot_path.read_bytes()
        except OSError as e:
            raise ExternalJudgeUnavailable(f"Cannot read screenshot {screenshot_path}: {e}")

        screenshot_sha256 = hashlib.sha256(image_bytes).hexdigest()
        image_base64 = base64.standard_b64encode(image_bytes).decode("utf-8")

        # Infer media type from extension
        ext = screenshot_path.suffix.lower()
        media_type_map = {
            ".png": "image/png",
            ".jpg": "image/jpeg",
            ".jpeg": "image/jpeg",
            ".webp": "image/webp",
            ".gif": "image/gif",
        }
        media_type = media_type_map.get(ext, "image/png")

        # Call Moonshot API (OpenAI-compatible format)
        timestamp_utc = datetime.utcnow().isoformat() + "Z"
        verdict, confidence, raw_response = self._call_moonshot(
            image_base64, media_type, prompt
        )

        receipt = JudgeReceipt(
            model="moonshot",
            model_version=self._model,
            timestamp_utc=timestamp_utc,
            prompt=prompt,
            screenshot_sha256=screenshot_sha256,
            raw_response=raw_response,
            verdict=verdict,
            confidence=confidence,
        )

        # Persist to disk
        self._persist(receipt)

        return receipt

    def _call_moonshot(self, image_base64: str, media_type: str, prompt: str) -> tuple:
        """
        Call Moonshot API with vision message.

        Returns:
            (verdict, confidence, raw_response)
            verdict: "pass" | "fail" | "uncertain"
            confidence: float or None
        """
        url = "https://api.moonshot.cn/v1/chat/completions"
        headers = {
            "Authorization": f"Bearer {self._key}",
            "Content-Type": "application/json",
        }

        payload = {
            "model": self._model,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": f"data:{media_type};base64,{image_base64}"
                            },
                        },
                        {"type": "text", "text": prompt},
                    ],
                }
            ],
            "temperature": 0.0,
        }

        # Try once, retry on 5xx
        last_error = None
        for attempt in range(2):
            try:
                with httpx.Client(timeout=self._timeout) as client:
                    response = client.post(url, headers=headers, json=payload)

                # Non-5xx errors are terminal
                if 400 <= response.status_code < 500:
                    raise ExternalJudgeUnavailable(
                        f"Moonshot API error {response.status_code}: {response.text}"
                    )

                response.raise_for_status()

                # Parse response
                data = response.json()
                raw_response = data

                # Extract verdict from response
                message_content = data.get("choices", [{}])[0].get("message", {}).get("content", "")
                verdict, confidence = self._parse_verdict(message_content)

                return verdict, confidence, raw_response

            except httpx.HTTPError as e:
                last_error = e
                if attempt < 1:
                    continue  # Retry once on network/5xx errors
                break

        raise ExternalJudgeUnavailable(
            f"Moonshot API failed after retry: {last_error}"
        )

    def _parse_verdict(self, response_text: str) -> tuple:
        """
        Parse Moonshot response to extract verdict.

        Looks for patterns like "VERDICT: pass" or "CONFIDENCE: 0.95".
        Returns: (verdict, confidence)
        """
        response_lower = response_text.lower()

        # Default to uncertain
        verdict = "uncertain"
        confidence = None

        if "pass" in response_lower or "yes" in response_lower or "correct" in response_lower:
            verdict = "pass"
        elif "fail" in response_lower or "no" in response_lower or "incorrect" in response_lower:
            verdict = "fail"

        # Try to extract numeric confidence
        for line in response_text.split("\n"):
            if "confidence" in line.lower():
                try:
                    # Extract float from "confidence: 0.95" or similar
                    parts = line.split(":")
                    if len(parts) > 1:
                        conf_str = parts[1].strip().split()[0]
                        confidence = float(conf_str)
                        if 0 <= confidence <= 1:
                            break
                except (ValueError, IndexError):
                    pass

        return verdict, confidence

    def _persist(self, receipt: JudgeReceipt) -> Path:
        """
        Persist receipt to docs/proof/judge-receipts/ in repo root.

        Writes atomically (tmp then rename).
        Returns the path where receipt was written.
        """
        # Find repo root: walk up from this file
        repo_root = Path(__file__).resolve()
        for _ in range(10):
            repo_root = repo_root.parent
            if (repo_root / ".git").exists():
                break
        else:
            raise ExternalJudgeUnavailable("Cannot find repo root (.git)")

        receipts_dir = repo_root / "docs" / "proof" / "judge-receipts"
        receipts_dir.mkdir(parents=True, exist_ok=True)

        # Filename: <timestamp>-<sha8>.json
        timestamp = receipt.timestamp_utc.replace(":", "-").replace("Z", "")
        sha8 = receipt.screenshot_sha256[:8]
        filename = f"{timestamp}-{sha8}.json"
        filepath = receipts_dir / filename

        # Write atomically
        tmp_path = filepath.with_suffix(".tmp")
        tmp_path.write_text(json.dumps(receipt.to_dict(), indent=2))
        tmp_path.rename(filepath)

        return filepath
