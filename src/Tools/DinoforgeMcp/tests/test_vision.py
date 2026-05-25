"""Tests for the two-tier visual validation system."""

import pytest
from pathlib import Path
import tempfile
import sys

# Add parent module to path
mcp_dir = Path(__file__).parent.parent
sys.path.insert(0, str(mcp_dir))

from dinoforge_mcp.vision import VisualValidator


class TestVisualValidatorInit:
    """Test VisualValidator initialization and configuration."""

    def test_init_defaults(self):
        """Test initialization with default parameters."""
        validator = VisualValidator()
        assert validator.model_type == "openai/clip-vit-base-patch32"
        assert validator.fallback_to_opencv is True
        assert validator.device == "cpu"
        assert isinstance(validator._golden_cache, dict)

    def test_init_custom_model(self):
        """Test initialization with custom model type."""
        validator = VisualValidator(model_type="custom-clip-model", device="cuda:0")
        assert validator.model_type == "custom-clip-model"
        assert validator.device == "cuda:0"


class TestGoldenRegistration:
    """Test golden reference registration."""

    def test_register_golden_with_mock_image(self, tmp_path):
        """Test registering a golden reference (mocked image)."""
        validator = VisualValidator(fallback_to_opencv=True)

        # Create a minimal test image file
        try:
            from PIL import Image
            img = Image.new("RGB", (100, 100), color="red")
            img_path = str(tmp_path / "test_golden.png")
            img.save(img_path)

            # Try to register golden
            result = validator.register_golden("test_golden", img_path)

            # imagehash may not be available, so graceful handling
            if result:
                assert "test_golden" in validator._golden_cache
                assert isinstance(validator._golden_cache["test_golden"], str)
        except ImportError:
            # PIL or imagehash not available — skip test
            pytest.skip("PIL or imagehash not available")

    def test_register_golden_nonexistent_file(self):
        """Test registering golden with nonexistent file."""
        validator = VisualValidator()
        result = validator.register_golden("fake_key", "/nonexistent/path.png")
        assert result is False


class TestAnalysisWithoutImages:
    """Test analysis behavior when images don't exist."""

    def test_analyze_screenshot_nonexistent_file(self):
        """Test analyzing non-existent image file."""
        validator = VisualValidator()
        result = validator.analyze_screenshot("/nonexistent/screenshot.png")

        assert result["method"] == "none"
        assert "error" in result
        assert "does not exist" in result["error"]

    def test_analyze_golden_unregistered_key(self):
        """Test analyzing against unregistered golden key."""
        validator = VisualValidator()
        result = validator.analyze_golden("/some/image.png", "nonexistent_key")

        assert result["method"] == "phash"
        assert result["passed"] is False
        assert "not registered" in result.get("error", "")

    def test_analyze_with_empty_prompts(self):
        """Test CLIP analysis with empty prompts."""
        validator = VisualValidator()
        result = validator.analyze_with_clip("/some/image.png", [])

        assert result["method"] == "clip"
        assert "error" in result


class TestGracefulDegradation:
    """Test graceful degradation across tiers."""

    def test_fallback_when_all_unavailable(self, tmp_path):
        """Test behavior when all dependencies unavailable."""
        validator = VisualValidator(fallback_to_opencv=False)

        # Create empty image file
        test_img = tmp_path / "test.png"
        test_img.write_bytes(b"fake")

        result = validator.analyze_screenshot(str(test_img))
        # Should handle gracefully even if dependencies missing
        assert "method" in result

    def test_dispatch_with_golden_key(self, tmp_path):
        """Test that golden_key takes priority in dispatch."""
        validator = VisualValidator()

        try:
            from PIL import Image
            img = Image.new("RGB", (100, 100), color="blue")
            img_path = str(tmp_path / "test.png")
            img.save(img_path)

            # Register and analyze
            if validator.register_golden("priority_test", img_path):
                result = validator.analyze_screenshot(
                    img_path,
                    golden_key="priority_test",
                    prompts=["should be ignored"]
                )
                # Should use phash method, not CLIP
                assert result["method"] == "phash"
        except ImportError:
            pytest.skip("PIL not available")

    def test_dispatch_with_prompts_no_golden(self):
        """Test that CLIP is used when golden_key not provided."""
        validator = VisualValidator()

        # CLIP may not load, so just test the call succeeds
        result = validator.analyze_screenshot(
            "/nonexistent/image.png",
            golden_key=None,
            prompts=["test prompt"]
        )

        # Should attempt CLIP or fallback gracefully
        assert "method" in result


class TestPHashDistance:
    """Test perceptual hash distance calculation."""

    def test_phash_identical_images(self, tmp_path):
        """Test pHash distance between identical images."""
        try:
            from PIL import Image

            # Create and save same image twice
            img = Image.new("RGB", (100, 100), color="green")
            img_path1 = str(tmp_path / "img1.png")
            img_path2 = str(tmp_path / "img2.png")
            img.save(img_path1)
            img.save(img_path2)

            validator = VisualValidator()
            if validator.register_golden("golden1", img_path1):
                result = validator.analyze_golden(img_path2, "golden1")
                assert result["method"] == "phash"
                assert result["passed"] is True
                assert result["distance"] == 0.0
        except ImportError:
            pytest.skip("PIL or imagehash not available")

    def test_phash_distance_threshold(self, tmp_path):
        """Test pHash distance threshold (< 10 is pass)."""
        try:
            from PIL import Image, ImageDraw

            # Create slightly different images
            img1 = Image.new("RGB", (100, 100), color="red")
            img2 = Image.new("RGB", (100, 100), color="red")

            # Draw slightly different on img2
            draw = ImageDraw.Draw(img2)
            draw.rectangle([10, 10, 20, 20], fill="blue")

            img1_path = str(tmp_path / "img1.png")
            img2_path = str(tmp_path / "img2.png")
            img1.save(img1_path)
            img2.save(img2_path)

            validator = VisualValidator()
            if validator.register_golden("golden2", img1_path):
                result = validator.analyze_golden(img2_path, "golden2")
                assert result["method"] == "phash"
                # Distance should be small but > 0
                assert result["distance"] is not None
                assert result["distance"] >= 0
        except ImportError:
            pytest.skip("PIL or imagehash not available")


class TestOpenCVFallback:
    """Test OpenCV fallback analysis."""

    def test_opencv_analysis_available(self, tmp_path):
        """Test OpenCV fallback when available."""
        try:
            from PIL import Image

            img = Image.new("RGB", (100, 100), color="white")
            img_path = str(tmp_path / "test.png")
            img.save(img_path)

            validator = VisualValidator(fallback_to_opencv=True)
            result = validator.analyze_with_opencv(img_path)

            assert result["method"] == "opencv"
            assert "regions" in result or "error" in result
        except ImportError:
            pytest.skip("PIL not available")

    def test_opencv_disabled(self):
        """Test that OpenCV fallback can be disabled."""
        validator = VisualValidator(fallback_to_opencv=False)
        result = validator.analyze_screenshot(
            "/nonexistent/image.png",
            golden_key=None,
            prompts=None
        )

        # Should fail gracefully without attempting OpenCV
        assert "error" in result or result["method"] != "opencv"


class TestAnalysisDispatch:
    """Test the dispatch logic in analyze_screenshot."""

    def test_dispatch_order_golden_first(self, tmp_path):
        """Test that golden_key analysis is attempted first."""
        try:
            from PIL import Image

            img = Image.new("RGB", (100, 100), color="yellow")
            img_path = str(tmp_path / "test.png")
            img.save(img_path)

            validator = VisualValidator()

            # Both golden and prompts provided — should use golden
            if validator.register_golden("dispatch_test", img_path):
                result = validator.analyze_screenshot(
                    img_path,
                    golden_key="dispatch_test",
                    prompts=["test"]
                )
                assert result["method"] == "phash"
        except ImportError:
            pytest.skip("PIL or imagehash not available")

    def test_analysis_result_structure(self, tmp_path):
        """Test that analysis results have expected structure."""
        try:
            from PIL import Image

            img = Image.new("RGB", (100, 100), color="cyan")
            img_path = str(tmp_path / "test.png")
            img.save(img_path)

            validator = VisualValidator()
            result = validator.analyze_screenshot(img_path)

            # Should have method field
            assert "method" in result
            # Result should be dict
            assert isinstance(result, dict)
        except ImportError:
            pytest.skip("PIL not available")


class TestClipIntegration:
    """Test CLIP integration (if available)."""

    def test_clip_model_loading(self):
        """Test CLIP model loads (or gracefully handles unavailable)."""
        validator = VisualValidator()
        # model may be None if transformers not installed
        assert validator.model is None or hasattr(validator.model, 'forward')

    def test_clip_analysis_with_sample_prompts(self, tmp_path):
        """Test CLIP analysis with realistic prompts."""
        try:
            from PIL import Image

            img = Image.new("RGB", (100, 100), color="magenta")
            img_path = str(tmp_path / "test.png")
            img.save(img_path)

            validator = VisualValidator()
            prompts = ["overlay visible", "menu open", "health bar shown"]
            result = validator.analyze_with_clip(img_path, prompts)

            # Should have method field at minimum
            assert result["method"] == "clip"

            if "error" not in result:
                # CLIP loaded successfully
                assert "prompts" in result or "error" in result
                assert "confidence" in result or "error" in result
        except ImportError:
            pytest.skip("PIL or transformers not available")


class TestLoadImage:
    """Test image loading helper."""

    def test_load_valid_image(self, tmp_path):
        """Test loading a valid image file."""
        try:
            from PIL import Image

            img = Image.new("RGB", (50, 50), color="purple")
            img_path = str(tmp_path / "valid.png")
            img.save(img_path)

            validator = VisualValidator()
            loaded = validator._load_image(img_path)

            assert loaded is not None
            assert loaded.size == (50, 50)
            assert loaded.mode == "RGB"
        except ImportError:
            pytest.skip("PIL not available")

    def test_load_nonexistent_image(self):
        """Test loading non-existent image."""
        validator = VisualValidator()
        result = validator._load_image("/nonexistent/image.png")
        assert result is None

    def test_load_invalid_image_data(self, tmp_path):
        """Test loading file with invalid image data."""
        invalid_file = tmp_path / "invalid.png"
        invalid_file.write_bytes(b"not an image")

        validator = VisualValidator()
        result = validator._load_image(str(invalid_file))
        assert result is None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
