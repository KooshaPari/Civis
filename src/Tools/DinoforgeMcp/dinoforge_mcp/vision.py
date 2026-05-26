"""
Two-tier visual validation system for screenshot regression testing.

Tier 1: pHash distance (imagehash library) — fast, ~1ms, good for golden reference matching
Tier 2: CLIP zero-shot classification (transformers) — moderate cost, ~200ms, flexible prompts
Tier 3: OpenCV contour analysis — fallback, ~100ms, basic UI region detection

The VisualValidator class implements graceful degradation: tries pHash first, falls back to CLIP
if golden_key unavailable, falls back to OpenCV if CLIP not installed, and returns None if all fail.
"""

from __future__ import annotations

import logging
from pathlib import Path
from typing import Any, Optional
import os

try:
    from PIL import Image
except ImportError:
    Image = None

try:
    import imagehash
except ImportError:
    imagehash = None

try:
    import torch
    from transformers import CLIPProcessor, CLIPModel
except ImportError:
    torch = None
    CLIPProcessor = None
    CLIPModel = None

try:
    import cv2
    import numpy as np
except ImportError:
    cv2 = None
    np = None


logger = logging.getLogger("dinoforge_mcp.vision")


class VisualValidator:
    """
    Two-tier visual validator for screenshot analysis and golden reference matching.

    Implements graceful degradation:
    - Tier 1: pHash (1ms, requires imagehash)
    - Tier 2: CLIP (200ms, requires transformers + torch)
    - Tier 3: OpenCV (100ms, requires cv2)
    """

    def __init__(
        self,
        model_type: str = "openai/clip-vit-base-patch32",
        fallback_to_opencv: bool = True,
        device: str = "cpu"
    ):
        """
        Initialize VisualValidator.

        Args:
            model_type: HuggingFace model ID for CLIP (e.g. "openai/clip-vit-base-patch32")
            fallback_to_opencv: If True, fall back to OpenCV when CLIP unavailable
            device: Torch device ("cpu" or "cuda:0")
        """
        self.model_type = model_type
        self.fallback_to_opencv = fallback_to_opencv
        self.device = device
        self.model = None
        self.processor = None
        self._golden_cache: dict[str, str] = {}  # golden_key -> pHash hex string

        # Try to load CLIP at startup
        self._load_clip()

    def _load_clip(self) -> bool:
        """Load CLIP model and processor. Returns True if successful."""
        if torch is None or CLIPModel is None:
            logger.debug("CLIP dependencies (transformers, torch) not available")
            return False

        try:
            logger.debug(f"Loading CLIP model: {self.model_type}")
            self.processor = CLIPProcessor.from_pretrained(self.model_type)
            self.model = CLIPModel.from_pretrained(self.model_type).to(self.device)
            self.model.eval()
            logger.info(f"CLIP model loaded: {self.model_type}")
            return True
        except Exception as e:
            logger.warning(f"Failed to load CLIP: {e}")
            return False

    def _load_image(self, image_path: str) -> Optional[Image.Image]:
        """Load image from file. Returns PIL Image or None."""
        if Image is None:
            logger.error("PIL not available")
            return None

        try:
            img = Image.open(image_path)
            if img.mode != "RGB":
                img = img.convert("RGB")
            return img
        except Exception as e:
            logger.error(f"Failed to load image {image_path}: {e}")
            return None

    def _compute_phash(self, image_path: str) -> Optional[str]:
        """Compute perceptual hash of image. Returns hex string or None."""
        if imagehash is None:
            logger.debug("imagehash not available")
            return None

        try:
            img = Image.open(image_path)
            hash_obj = imagehash.phash(img)
            return str(hash_obj)
        except Exception as e:
            logger.error(f"Failed to compute pHash for {image_path}: {e}")
            return None

    def register_golden(self, golden_key: str, image_path: str) -> bool:
        """
        Register a golden reference image by computing its pHash.

        Args:
            golden_key: Identifier for this golden (e.g., "cp2_f9_overlay")
            image_path: Path to the golden reference image

        Returns:
            True if hash computed successfully
        """
        hash_hex = self._compute_phash(image_path)
        if hash_hex is None:
            logger.warning(f"Failed to register golden: {golden_key}")
            return False

        self._golden_cache[golden_key] = hash_hex
        logger.info(f"Registered golden '{golden_key}': {hash_hex}")
        return True

    def analyze_golden(self, image_path: str, golden_key: str) -> dict[str, Any]:
        """
        Analyze screenshot against a golden reference using pHash distance.

        pHash distance < 10 is considered a match (allows for minor UI position/opacity changes).

        Args:
            image_path: Path to screenshot to analyze
            golden_key: Key of registered golden reference

        Returns:
            {
                "method": "phash",
                "golden_key": golden_key,
                "distance": float,
                "passed": bool,
                "error": str (optional)
            }
        """
        if golden_key not in self._golden_cache:
            return {
                "method": "phash",
                "golden_key": golden_key,
                "distance": None,
                "passed": False,
                "error": f"Golden key '{golden_key}' not registered"
            }

        current_hash_hex = self._compute_phash(image_path)
        if current_hash_hex is None:
            return {
                "method": "phash",
                "golden_key": golden_key,
                "distance": None,
                "passed": False,
                "error": "Failed to compute pHash of current image"
            }

        try:
            if imagehash is None:
                return {
                    "method": "phash",
                    "golden_key": golden_key,
                    "distance": None,
                    "passed": False,
                    "error": "imagehash not available"
                }

            # Reconstruct hash objects from hex strings
            golden_bits = bin(int(self._golden_cache[golden_key], 16))[2:].zfill(64)
            current_bits = bin(int(current_hash_hex, 16))[2:].zfill(64)
            distance = sum(g != c for g, c in zip(golden_bits, current_bits))
            passed = distance < 10

            return {
                "method": "phash",
                "golden_key": golden_key,
                "distance": float(distance),
                "passed": passed,
            }
        except Exception as e:
            logger.error(f"Error computing pHash distance: {e}")
            return {
                "method": "phash",
                "golden_key": golden_key,
                "distance": None,
                "passed": False,
                "error": str(e)
            }

    def analyze_with_clip(self, image_path: str, prompts: list[str]) -> dict[str, Any]:
        """
        Analyze screenshot using CLIP zero-shot classification.

        Args:
            image_path: Path to screenshot
            prompts: List of text prompts (e.g., ["overlay visible", "menu open", "health bar shown"])

        Returns:
            {
                "method": "clip",
                "prompts": {prompt: confidence_float},
                "top_prompt": str,
                "confidence": float,
                "error": str (optional)
            }
        """
        if not prompts:
            return {
                "method": "clip",
                "error": "No prompts provided"
            }

        if self.model is None or self.processor is None:
            return {
                "method": "clip",
                "error": "CLIP model not loaded"
            }

        try:
            img = self._load_image(image_path)
            if img is None:
                return {
                    "method": "clip",
                    "error": f"Failed to load image: {image_path}"
                }

            # Prepare text inputs
            text_inputs = self.processor(text=prompts, return_tensors="pt", padding=True).to(self.device)
            image_inputs = self.processor(images=img, return_tensors="pt").to(self.device)

            # Inference
            with torch.no_grad():
                text_features = self.model.get_text_features(**text_inputs)
                image_features = self.model.get_image_features(**image_inputs)

                # Normalize
                text_features = text_features / text_features.norm(p=2, dim=-1, keepdim=True)
                image_features = image_features / image_features.norm(p=2, dim=-1, keepdim=True)

                # Compute logits
                logits_per_image = (image_features @ text_features.t()) * self.model.logit_scale.exp()
                logits = logits_per_image[0].cpu().numpy()

            # Softmax to get probabilities
            exp_logits = np.exp(logits - np.max(logits))
            probs = exp_logits / np.sum(exp_logits)

            prompt_scores = {prompts[i]: float(probs[i]) for i in range(len(prompts))}
            top_idx = int(np.argmax(probs))

            return {
                "method": "clip",
                "prompts": prompt_scores,
                "top_prompt": prompts[top_idx],
                "confidence": float(probs[top_idx]),
            }
        except Exception as e:
            logger.error(f"CLIP analysis failed: {e}")
            return {
                "method": "clip",
                "error": str(e)
            }

    def analyze_with_opencv(self, image_path: str) -> dict[str, Any]:
        """
        Fallback: analyze using OpenCV contour detection.
        Returns basic UI region info (health bars, buttons, etc).

        Args:
            image_path: Path to screenshot

        Returns:
            {
                "method": "opencv",
                "regions": [...],
                "error": str (optional)
            }
        """
        if cv2 is None or np is None:
            return {
                "method": "opencv",
                "error": "OpenCV not available"
            }

        try:
            img = cv2.imread(image_path)
            if img is None:
                return {
                    "method": "opencv",
                    "error": f"Failed to read image: {image_path}"
                }

            # Convert to HSV for color detection
            hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)

            # Simple red/green region detection (common in game UI)
            lower_red = np.array([0, 100, 100])
            upper_red = np.array([10, 255, 255])
            mask_red = cv2.inRange(hsv, lower_red, upper_red)

            lower_green = np.array([35, 100, 100])
            upper_green = np.array([85, 255, 255])
            mask_green = cv2.inRange(hsv, lower_green, upper_green)

            combined_mask = cv2.bitwise_or(mask_red, mask_green)

            # Find contours
            contours, _ = cv2.findContours(combined_mask, cv2.RETR_TREE, cv2.CHAIN_APPROX_SIMPLE)

            regions = []
            for cnt in contours:
                area = cv2.contourArea(cnt)
                if area > 100:  # Filter small noise
                    x, y, w, h = cv2.boundingRect(cnt)
                    regions.append({
                        "x": int(x),
                        "y": int(y),
                        "width": int(w),
                        "height": int(h),
                        "area": int(area)
                    })

            return {
                "method": "opencv",
                "regions": regions,
                "region_count": len(regions)
            }
        except Exception as e:
            logger.error(f"OpenCV analysis failed: {e}")
            return {
                "method": "opencv",
                "error": str(e)
            }

    def analyze_screenshot(
        self,
        image_path: str,
        golden_key: Optional[str] = None,
        prompts: Optional[list[str]] = None
    ) -> dict[str, Any]:
        """
        Analyze screenshot with two-tier dispatch:
        1. If golden_key provided: use pHash (1ms) — PASS if distance < 10
        2. Elif prompts provided: use CLIP (200ms) — returns confidence scores
        3. Else: fallback to OpenCV (100ms) — returns UI regions

        Args:
            image_path: Path to screenshot
            golden_key: Optional key of golden reference
            prompts: Optional list of text prompts for CLIP

        Returns:
            {
                "method": "phash" | "clip" | "opencv",
                "passed": bool (for pHash),
                "confidence": float (for CLIP),
                "region_count": int (for OpenCV),
                ...other method-specific fields...
            }
        """
        if not Path(image_path).exists():
            return {
                "error": f"Image path does not exist: {image_path}",
                "method": "none"
            }

        # Tier 1: pHash golden reference matching
        if golden_key:
            result = self.analyze_golden(image_path, golden_key)
            logger.debug(f"pHash analysis: {golden_key} -> distance={result.get('distance')}, passed={result.get('passed')}")
            return result

        # Tier 2: CLIP zero-shot classification
        if prompts and self.model is not None:
            result = self.analyze_with_clip(image_path, prompts)
            if "error" not in result:
                logger.debug(f"CLIP analysis: top={result.get('top_prompt')}, conf={result.get('confidence'):.3f}")
                return result

        # Tier 3: OpenCV fallback
        if self.fallback_to_opencv:
            result = self.analyze_with_opencv(image_path)
            logger.debug(f"OpenCV fallback: {len(result.get('regions', []))} regions detected")
            return result

        return {
            "error": "No analysis method available",
            "method": "none"
        }
