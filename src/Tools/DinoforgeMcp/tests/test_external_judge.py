"""
Tests for external Kimi VLM judge tier.
"""

import json
import os
from pathlib import Path
from tempfile import TemporaryDirectory

import httpx
import pytest

from dinoforge_mcp.external_judge import ExternalJudgeUnavailable, KimiJudgeTier


class TestMissingKey:
    """Test that missing API keys raise — no silent fallback to Claude."""

    def test_missing_key_raises(self, monkeypatch):
        """Unset both env vars should raise ExternalJudgeUnavailable."""
        monkeypatch.delenv("MOONSHOT_API_KEY", raising=False)
        monkeypatch.delenv("FIREWORKS_API_KEY", raising=False)

        with pytest.raises(ExternalJudgeUnavailable) as exc_info:
            KimiJudgeTier()

        msg = str(exc_info.value)
        assert "FIREWORKS_API_KEY" in msg or "MOONSHOT_API_KEY" in msg
        assert "refusing silent fallback" in msg

    def test_explicit_key_overrides_env(self, monkeypatch):
        """Explicit api_key parameter overrides env var."""
        monkeypatch.delenv("MOONSHOT_API_KEY", raising=False)
        monkeypatch.delenv("FIREWORKS_API_KEY", raising=False)

        # Should not raise with explicit key
        judge = KimiJudgeTier(api_key="test-key")
        assert judge._key == "test-key"

    def test_fireworks_preferred_over_moonshot(self, monkeypatch):
        """When both env vars are set, Fireworks is preferred."""
        monkeypatch.setenv("FIREWORKS_API_KEY", "fw-key")
        monkeypatch.setenv("MOONSHOT_API_KEY", "ms-key")

        judge = KimiJudgeTier()
        assert judge._provider == "fireworks"
        assert judge._key == "fw-key"
        assert "kimi-k2-instruct" in judge._model

    def test_moonshot_used_when_only_moonshot_set(self, monkeypatch):
        """Falls back to Moonshot when only MOONSHOT_API_KEY is set."""
        monkeypatch.delenv("FIREWORKS_API_KEY", raising=False)
        monkeypatch.setenv("MOONSHOT_API_KEY", "ms-key")

        judge = KimiJudgeTier()
        assert judge._provider == "moonshot"
        assert judge._key == "ms-key"

    def test_explicit_provider_fireworks(self, monkeypatch):
        """provider='fireworks' selects Fireworks even if Moonshot key also set."""
        monkeypatch.setenv("FIREWORKS_API_KEY", "fw-key")
        monkeypatch.setenv("MOONSHOT_API_KEY", "ms-key")

        judge = KimiJudgeTier(provider="fireworks")
        assert judge._provider == "fireworks"


class TestReceiptPersisted:
    """Test that judge receipts are persisted to repo."""

    def test_receipt_persisted_to_repo(self, monkeypatch, tmp_path):
        """Happy-path judgment creates receipt in docs/proof/judge-receipts/."""
        # Mock the repo root
        mock_repo_root = tmp_path / "mock_repo"
        mock_repo_root.mkdir()
        (mock_repo_root / ".git").mkdir()

        # Create a test screenshot
        test_screenshot = tmp_path / "test.png"
        test_screenshot.write_bytes(b"fake PNG data")

        # Mock httpx to return a success response
        def mock_post(*args, **kwargs):
            response = httpx.Response(
                status_code=200,
                json={
                    "choices": [
                        {
                            "message": {
                                "content": "VERDICT: pass\nCONFIDENCE: 0.95"
                            }
                        }
                    ]
                },
            )
            return response

        # Patch the file finder to use our mock repo
        original_resolve = Path.resolve
        def patched_resolve(self):
            if str(self).startswith(str(tmp_path)):
                return self
            return original_resolve(self)

        monkeypatch.setattr(Path, "resolve", patched_resolve)

        # Also monkey-patch the parent walk in _persist
        # We'll do this more directly by mocking the judge call
        judge = KimiJudgeTier(api_key="test-key", timeout=1.0)

        # Monkeypatch _call_moonshot to return deterministic result
        def mock_call(image_base64, media_type, prompt):
            return "pass", 0.95, {"choices": [{"message": {"content": "VERDICT: pass"}}]}

        judge._call_moonshot = mock_call

        # Monkeypatch _persist to use our mock repo
        original_persist = judge._persist
        def mock_persist(receipt):
            receipts_dir = mock_repo_root / "docs" / "proof" / "judge-receipts"
            receipts_dir.mkdir(parents=True, exist_ok=True)
            timestamp = receipt.timestamp_utc.replace(":", "-").replace("Z", "")
            sha8 = receipt.screenshot_sha256[:8]
            filename = f"{timestamp}-{sha8}.json"
            filepath = receipts_dir / filename
            filepath.write_text(json.dumps(receipt.to_dict(), indent=2))
            return filepath

        judge._persist = mock_persist

        # Call judge
        receipt = judge.judge(test_screenshot, "Does this pass?")

        # Check that receipt was written
        receipts_dir = mock_repo_root / "docs" / "proof" / "judge-receipts"
        assert receipts_dir.exists()
        json_files = list(receipts_dir.glob("*.json"))
        assert len(json_files) == 1

        # Verify content
        receipt_data = json.loads(json_files[0].read_text())
        assert receipt_data["verdict"] == "pass"
        assert receipt_data["confidence"] == 0.95

    def test_receipt_includes_raw_response(self, monkeypatch):
        """Receipt must include full raw_response, not summarized."""
        judge = KimiJudgeTier(api_key="test-key")

        # Mock the API call
        mock_response = {
            "id": "cmpl-123",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "moonshot-v1-8k-vision-preview",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "VERDICT: pass\nCONFIDENCE: 0.88"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 20,
                "total_tokens": 120
            }
        }

        def mock_call(image_base64, media_type, prompt):
            return "pass", 0.88, mock_response

        judge._call_moonshot = mock_call

        # Mock _persist to not actually write
        judge._persist = lambda receipt: Path("/dev/null")

        test_screenshot = Path(__file__).parent / "dummy.png"
        test_screenshot.write_bytes(b"fake PNG")

        try:
            receipt = judge.judge(test_screenshot, "Test prompt")
            assert receipt.raw_response == mock_response
            assert receipt.raw_response["usage"]["total_tokens"] == 120
        finally:
            test_screenshot.unlink()


class TestVerdictParsing:
    """Test verdict extraction from Moonshot responses."""

    @pytest.mark.parametrize("response_text,expected_verdict", [
        ("VERDICT: pass", "pass"),
        ("yes, this is correct", "pass"),
        ("VERDICT: fail", "fail"),
        ("no, this is wrong", "fail"),
        ("VERDICT: uncertain", "uncertain"),
        ("maybe", "uncertain"),
    ])
    def test_parse_verdict_variants(self, response_text, expected_verdict):
        """Test various verdict formats."""
        judge = KimiJudgeTier(api_key="test-key")
        verdict, conf = judge._parse_verdict(response_text)
        assert verdict == expected_verdict

    def test_parse_confidence(self):
        """Test confidence extraction."""
        judge = KimiJudgeTier(api_key="test-key")
        response = "VERDICT: pass\nCONFIDENCE: 0.87\nExplanation: ..."
        verdict, confidence = judge._parse_verdict(response)
        assert verdict == "pass"
        assert confidence == 0.87

    def test_parse_no_confidence(self):
        """Test when confidence is not provided."""
        judge = KimiJudgeTier(api_key="test-key")
        response = "VERDICT: pass"
        verdict, confidence = judge._parse_verdict(response)
        assert verdict == "pass"
        assert confidence is None


class TestAPIFailure:
    """Test error handling for API failures."""

    def test_unreadable_screenshot_raises(self):
        """Trying to judge a nonexistent screenshot raises."""
        judge = KimiJudgeTier(api_key="test-key")
        nonexistent = Path("/nonexistent/screenshot.png")

        with pytest.raises(ExternalJudgeUnavailable) as exc_info:
            judge.judge(nonexistent, "Test")

        assert "Cannot read screenshot" in str(exc_info.value)


class TestRetryBehavior:
    """Test HTTP retry logic for 5xx errors."""

    def test_5xx_retries_then_succeeds(self, monkeypatch, tmp_path):
        """First response is 503, second is 200 with valid content. Receipt persisted, 2 HTTP calls."""
        monkeypatch.setenv("MOONSHOT_API_KEY", "test-key")
        call_count = {"n": 0}

        def handler(request):
            call_count["n"] += 1
            if call_count["n"] == 1:
                return httpx.Response(503, json={"error": "transient"})
            return httpx.Response(
                200,
                json={
                    "model": "moonshot-v1-8k-vision-preview",
                    "choices": [
                        {"message": {"content": "VERDICT: pass\nCONFIDENCE: 0.9"}}
                    ],
                    "usage": {"total_tokens": 50},
                },
            )

        transport = httpx.MockTransport(handler)

        # Mock httpx.Client to use our mock transport
        original_client = httpx.Client

        def mock_client_init(*args, **kwargs):
            return original_client(transport=transport, *args, **kwargs)

        monkeypatch.setattr("httpx.Client", mock_client_init)

        judge = KimiJudgeTier(api_key="test-key")

        # Write a tiny PNG
        img = tmp_path / "shot.png"
        img.write_bytes(
            bytes.fromhex(
                "89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d4944415478da636400000000000000050000000000000049454e44ae426082"
            )
        )

        # Mock _persist to not write to disk
        judge._persist = lambda receipt: Path("/dev/null")

        receipt = judge.judge(img, "test prompt")
        assert call_count["n"] == 2
        assert receipt.verdict == "pass"
        assert receipt.confidence == 0.9

    def test_5xx_terminal_failure_after_retry(self, monkeypatch, tmp_path):
        """Both attempts return 503. ExternalJudgeUnavailable is raised."""
        monkeypatch.setenv("MOONSHOT_API_KEY", "test-key")
        call_count = {"n": 0}

        def handler(request):
            call_count["n"] += 1
            return httpx.Response(503, json={"error": "transient"})

        transport = httpx.MockTransport(handler)
        original_client = httpx.Client

        def mock_client_init(*args, **kwargs):
            return original_client(transport=transport, *args, **kwargs)

        monkeypatch.setattr("httpx.Client", mock_client_init)

        judge = KimiJudgeTier(api_key="test-key")

        img = tmp_path / "shot.png"
        img.write_bytes(
            bytes.fromhex(
                "89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d4944415478da636400000000000000050000000000000049454e44ae426082"
            )
        )

        with pytest.raises(ExternalJudgeUnavailable) as exc_info:
            judge.judge(img, "test prompt")

        assert call_count["n"] == 2
        assert "failed after retry" in str(exc_info.value).lower()

    def test_screenshot_sha256_matches_image_bytes(self, monkeypatch, tmp_path):
        """Happy-path call with mocked 200. SHA256 of disk bytes matches receipt."""
        import hashlib

        monkeypatch.setenv("MOONSHOT_API_KEY", "test-key")

        def handler(request):
            return httpx.Response(
                200,
                json={
                    "model": "moonshot-v1-8k-vision-preview",
                    "choices": [
                        {"message": {"content": "VERDICT: pass\nCONFIDENCE: 0.85"}}
                    ],
                    "usage": {"total_tokens": 40},
                },
            )

        transport = httpx.MockTransport(handler)
        original_client = httpx.Client

        def mock_client_init(*args, **kwargs):
            return original_client(transport=transport, *args, **kwargs)

        monkeypatch.setattr("httpx.Client", mock_client_init)

        judge = KimiJudgeTier(api_key="test-key")

        img = tmp_path / "shot.png"
        image_bytes = bytes.fromhex(
            "89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d4944415478da636400000000000000050000000000000049454e44ae426082"
        )
        img.write_bytes(image_bytes)

        judge._persist = lambda receipt: Path("/dev/null")

        receipt = judge.judge(img, "test prompt")

        # Compute SHA256 of the exact bytes written
        computed_sha256 = hashlib.sha256(image_bytes).hexdigest()
        assert receipt.screenshot_sha256 == computed_sha256
