"""
External VLM judge tier: Kimi via Fireworks AI (preferred) or Moonshot API (fallback).

Provides deterministic verdicts on game screenshots without silent fallback to Claude.

Provider selection (in order of preference):
  1. Fireworks AI — if FIREWORKS_API_KEY is set. Uses Kimi K2 via OpenAI-compatible
     Chat Completions API at https://api.fireworks.ai/inference/v1/chat/completions.
     Default model: accounts/fireworks/models/kimi-k2-instruct.
  2. Moonshot — if MOONSHOT_API_KEY is set. Uses moonshot-v1-8k-vision-preview at
     https://api.moonshot.cn/v1/chat/completions.

If neither key is set, raises ExternalJudgeUnavailable.
If the chosen provider's API fails after retry, raises (no silent fallback).
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
    """Raised when no external judge API is available or key is missing."""

    pass


# Provider constants
PROVIDER_FIREWORKS = "fireworks"
PROVIDER_MOONSHOT = "moonshot"

FIREWORKS_URL = "https://api.fireworks.ai/inference/v1/chat/completions"
FIREWORKS_DEFAULT_MODEL = "accounts/fireworks/models/kimi-k2-instruct"

MOONSHOT_URL = "https://api.moonshot.cn/v1/chat/completions"
MOONSHOT_DEFAULT_MODEL = "moonshot-v1-8k-vision-preview"


@dataclass
class JudgeReceipt:
    """Immutable record of a judgment call to an external VLM judge."""

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

    def to_autograder_dict(self) -> dict[str, Any]:
        """Return the spec-shaped autograder payload.

        The autograder contract expects snake_case keys and a stable top-level shape:
        tier, score, pass, gaps, evidence.
        """
        return {
            "tier": self.model,
            "score": self.confidence if self.confidence is not None else 0.0,
            "pass": self.verdict == "pass",
            "gaps": [] if self.verdict == "pass" else [self.verdict],
            "evidence": {
                "model_version": self.model_version,
                "prompt": self.prompt,
                "screenshot_sha256": self.screenshot_sha256,
                "raw_response": self.raw_response,
            },
        }


class KimiJudgeTier:
    """
    Calls an external Kimi-class vision API to judge game screenshots.

    Selects provider based on environment variables:
      - FIREWORKS_API_KEY -> Fireworks (Kimi K2)
      - MOONSHOT_API_KEY  -> Moonshot (Kimi v1 vision preview)
    """

    def __init__(
        self,
        api_key: str | None = None,
        model: str | None = None,
        timeout: float = 30.0,
        provider: str | None = None,
    ):
        """
        Initialize Kimi judge tier.

        Args:
            api_key: Explicit API key. If None, reads from FIREWORKS_API_KEY then
                MOONSHOT_API_KEY env vars (in that preference order).
            model: Model ID override. If None, uses a provider-appropriate default.
            timeout: HTTP request timeout in seconds.
            provider: Explicit provider id ("fireworks" or "moonshot"). If None,
                auto-detected from env vars (fireworks preferred).

        Raises:
            ExternalJudgeUnavailable: If no API key can be located.
        """
        # Resolve provider + key
        resolved_provider, resolved_key = self._resolve_provider(api_key, provider)
        self._provider = resolved_provider
        self._key = resolved_key

        # Choose default model per provider
        if model is None:
            if self._provider == PROVIDER_FIREWORKS:
                model = FIREWORKS_DEFAULT_MODEL
            else:
                model = MOONSHOT_DEFAULT_MODEL
        self._model = model
        self._timeout = timeout

    @staticmethod
    def _resolve_provider(
        api_key: str | None, provider: str | None
    ) -> tuple[str, str]:
        """
        Resolve (provider, key) from explicit args + env vars.

        Preference order when both env vars are set: Fireworks > Moonshot.

        Raises:
            ExternalJudgeUnavailable: If neither an explicit key nor any env var key is set.
        """
        # Explicit provider requested
        if provider is not None:
            if provider == PROVIDER_FIREWORKS:
                key = api_key or os.environ.get("FIREWORKS_API_KEY")
                if not key:
                    raise ExternalJudgeUnavailable(
                        "FIREWORKS_API_KEY not set; refusing silent fallback to Claude"
                    )
                return PROVIDER_FIREWORKS, key
            if provider == PROVIDER_MOONSHOT:
                key = api_key or os.environ.get("MOONSHOT_API_KEY")
                if not key:
                    raise ExternalJudgeUnavailable(
                        "MOONSHOT_API_KEY not set; refusing silent fallback to Claude"
                    )
                return PROVIDER_MOONSHOT, key
            raise ExternalJudgeUnavailable(f"Unknown provider: {provider}")

        # Auto-detect
        if api_key:
            # Caller passed a bare key without specifying provider.
            # Default to Moonshot for backwards compatibility with prior signature
            # (api_key was previously a Moonshot key). Callers wanting Fireworks
            # should pass provider="fireworks" explicitly.
            return PROVIDER_MOONSHOT, api_key

        fireworks_key = os.environ.get("FIREWORKS_API_KEY")
        if fireworks_key:
            return PROVIDER_FIREWORKS, fireworks_key

        moonshot_key = os.environ.get("MOONSHOT_API_KEY")
        if moonshot_key:
            return PROVIDER_MOONSHOT, moonshot_key

        raise ExternalJudgeUnavailable(
            "No external VLM judge API key set "
            "(checked FIREWORKS_API_KEY, MOONSHOT_API_KEY); "
            "refusing silent fallback to Claude"
        )

    def judge(self, screenshot_path: Path, prompt: str) -> JudgeReceipt:
        """
        Judge a screenshot via the configured external VLM API.

        Args:
            screenshot_path: Path to screenshot file (PNG, JPEG, WebP, GIF).
            prompt: Judgment prompt (e.g., "Does this screenshot show a red squad bar?").

        Returns:
            JudgeReceipt with verdict, confidence, and full API response.

        Raises:
            ExternalJudgeUnavailable: If API fails after retry or file is unreadable.

        Side effects:
            Persists JudgeReceipt as JSON to docs/proof/judge-receipts/<utc>-<sha8>.json
            relative to the repo root. Atomic write via .tmp file rename.
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

        # Call the provider API
        timestamp_utc = datetime.utcnow().isoformat() + "Z"
        verdict, confidence, raw_response = self._call_api(
            image_base64, media_type, prompt
        )

        receipt = JudgeReceipt(
            model=self._provider,
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

    def judge_text(self, prompt: str) -> JudgeReceipt:
        """
        Text-only sanity check (no image). Useful for smoke testing the provider.

        Returns a JudgeReceipt with screenshot_sha256 set to sha256("<text-only>")
        and the parsed verdict. Receipt is NOT persisted to disk (no image hash).
        """
        timestamp_utc = datetime.utcnow().isoformat() + "Z"
        verdict, confidence, raw_response = self._call_api_text(prompt)

        return JudgeReceipt(
            model=self._provider,
            model_version=self._model,
            timestamp_utc=timestamp_utc,
            prompt=prompt,
            screenshot_sha256=hashlib.sha256(b"<text-only>").hexdigest(),
            raw_response=raw_response,
            verdict=verdict,
            confidence=confidence,
        )

    def _build_url_and_headers(self) -> tuple[str, dict]:
        """Return the (url, headers) for the configured provider."""
        if self._provider == PROVIDER_FIREWORKS:
            url = FIREWORKS_URL
        else:
            url = MOONSHOT_URL
        headers = {
            "Authorization": f"Bearer {self._key}",
            "Content-Type": "application/json",
            "Accept": "application/json",
        }
        return url, headers

    def _call_api(self, image_base64: str, media_type: str, prompt: str) -> tuple:
        """
        Call the provider's Chat Completions API with a vision message.

        Returns:
            (verdict, confidence, raw_response)
            verdict: "pass" | "fail" | "uncertain"
            confidence: float or None
        """
        url, headers = self._build_url_and_headers()

        # Both Fireworks and Moonshot use the OpenAI Chat Completions shape.
        # Fireworks Kimi K2 (kimi-k2-instruct) accepts the same image_url content part.
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
            "max_tokens": 512,
        }

        return self._post_with_retry(url, headers, payload)

    def _call_api_text(self, prompt: str) -> tuple:
        """Text-only Chat Completions call (smoke test path, no image)."""
        url, headers = self._build_url_and_headers()
        payload = {
            "model": self._model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.0,
            "max_tokens": 256,
        }
        return self._post_with_retry(url, headers, payload)

    def _post_with_retry(self, url: str, headers: dict, payload: dict) -> tuple:
        """POST with one retry on 5xx / network error. Returns parsed verdict tuple."""
        last_error: Exception | None = None
        for attempt in range(2):
            try:
                with httpx.Client(timeout=self._timeout) as client:
                    response = client.post(url, headers=headers, json=payload)

                # Non-5xx errors are terminal
                if 400 <= response.status_code < 500:
                    raise ExternalJudgeUnavailable(
                        f"{self._provider} API error {response.status_code}: {response.text}"
                    )

                response.raise_for_status()

                data = response.json()
                raw_response = data

                message_content = (
                    data.get("choices", [{}])[0].get("message", {}).get("content", "")
                )
                # Some providers may return content as a list of parts; flatten.
                if isinstance(message_content, list):
                    message_content = "".join(
                        part.get("text", "") if isinstance(part, dict) else str(part)
                        for part in message_content
                    )
                verdict, confidence = self._parse_verdict(message_content or "")

                return verdict, confidence, raw_response

            except httpx.HTTPError as e:
                last_error = e
                if attempt < 1:
                    continue
                break

        raise ExternalJudgeUnavailable(
            f"{self._provider} API failed after retry: {last_error}"
        )

    # Backwards-compat shim: legacy callers / tests reference `_call_moonshot`.
    # Route through the unified _call_api so existing monkeypatches still work
    # (tests stub _call_moonshot directly; we keep the name as the call hook).
    def _call_moonshot(self, image_base64: str, media_type: str, prompt: str) -> tuple:
        """Legacy alias retained for backwards compatibility. Prefer _call_api."""
        return self._call_api(image_base64, media_type, prompt)

    def _parse_verdict(self, response_text: str) -> tuple:
        """
        Parse provider response to extract verdict.

        Looks for patterns like "VERDICT: pass" or "CONFIDENCE: 0.95".
        Returns: (verdict, confidence)
        """
        response_lower = response_text.lower()

        verdict = "uncertain"
        confidence: float | None = None

        if "pass" in response_lower or "yes" in response_lower or "correct" in response_lower:
            verdict = "pass"
        elif "fail" in response_lower or "no" in response_lower or "incorrect" in response_lower:
            verdict = "fail"

        for line in response_text.split("\n"):
            if "confidence" in line.lower():
                try:
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
        repo_root = Path(__file__).resolve()
        for _ in range(10):
            repo_root = repo_root.parent
            if (repo_root / ".git").exists():
                break
        else:
            raise ExternalJudgeUnavailable("Cannot find repo root (.git)")

        receipts_dir = repo_root / "docs" / "proof" / "judge-receipts"
        receipts_dir.mkdir(parents=True, exist_ok=True)

        timestamp = receipt.timestamp_utc.replace(":", "-").replace("Z", "")
        sha8 = receipt.screenshot_sha256[:8]
        filename = f"{timestamp}-{sha8}.json"
        filepath = receipts_dir / filename

        tmp_path = filepath.with_suffix(".tmp")
        tmp_path.write_text(json.dumps(receipt.to_dict(), indent=2))
        tmp_path.rename(filepath)

        return filepath
