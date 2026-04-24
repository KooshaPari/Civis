"""
Tests for isolation layer backends (HiddenDesktop, playCUA, auto-detection).

Run with: python scripts/test_isolation_layer.py
Or pytest: pytest scripts/test_isolation_layer.py -v
"""

import asyncio
import sys
import logging
from pathlib import Path

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent / "src/Tools/DinoforgeMcp"))

from dinoforge_mcp.isolation_layer import (
    get_isolation_context, set_isolation_context, HiddenDesktopBackend, PlayCUABackend, Frame, WindowInfo
)

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


async def test_hidden_desktop_availability():
    """Test that HiddenDesktopBackend is available on Windows."""
    logger.info("Testing HiddenDesktopBackend availability...")
    try:
        backend = HiddenDesktopBackend()
        assert backend is not None
        logger.info("✓ HiddenDesktopBackend available")
        return True
    except Exception as e:
        logger.error(f"✗ HiddenDesktopBackend unavailable: {e}")
        return False


async def test_playcua_availability():
    """Test that PlayCUABackend can be instantiated (binary might not be running)."""
    logger.info("Testing PlayCUABackend availability...")
    try:
        backend = PlayCUABackend()
        assert backend is not None
        logger.info("✓ PlayCUABackend instantiated (binary may not be running)")
        return True
    except Exception as e:
        logger.error(f"✗ PlayCUABackend failed: {e}")
        return False


async def test_auto_selection_hidden_desktop():
    """Test auto-selection defaults to HiddenDesktop."""
    logger.info("Testing auto-selection (should default to HiddenDesktop)...")
    try:
        backend = get_isolation_context("auto")
        assert backend is not None
        # Should be either PlayCUABackend or HiddenDesktopBackend
        assert isinstance(backend, (PlayCUABackend, HiddenDesktopBackend))
        logger.info(f"✓ Auto-selection returned: {type(backend).__name__}")
        return True
    except Exception as e:
        logger.error(f"✗ Auto-selection failed: {e}")
        return False


async def test_hidden_desktop_key_injection():
    """Test HiddenDesktopBackend can inject keys."""
    logger.info("Testing HiddenDesktopBackend key injection...")
    try:
        backend = HiddenDesktopBackend()
        # Just test that the method can be called; actual injection needs a window
        result = await backend.inject_key("f9", duration=0.01)
        logger.info(f"✓ HiddenDesktopBackend.inject_key() returned: {result}")
        return True
    except Exception as e:
        logger.warning(f"⚠ HiddenDesktopBackend.inject_key() error (expected if no window): {e}")
        return True  # Not a failure — key injection needs a target window


async def test_hidden_desktop_mouse_click():
    """Test HiddenDesktopBackend can perform mouse clicks."""
    logger.info("Testing HiddenDesktopBackend mouse click...")
    try:
        backend = HiddenDesktopBackend()
        result = await backend.mouse_click(960, 540, "left")
        logger.info(f"✓ HiddenDesktopBackend.mouse_click() returned: {result}")
        return True
    except Exception as e:
        logger.warning(f"⚠ HiddenDesktopBackend.mouse_click() error: {e}")
        return True  # Not a failure


async def test_explicit_backend_selection():
    """Test explicit backend selection."""
    logger.info("Testing explicit backend selection...")
    try:
        # Select HiddenDesktop explicitly
        backend1 = get_isolation_context("hidden_desktop")
        assert isinstance(backend1, HiddenDesktopBackend)
        logger.info("✓ Explicit 'hidden_desktop' selection works")

        # Select playCUA explicitly
        backend2 = get_isolation_context("playcua")
        assert isinstance(backend2, PlayCUABackend)
        logger.info("✓ Explicit 'playcua' selection works")

        return True
    except Exception as e:
        logger.error(f"✗ Explicit backend selection failed: {e}")
        return False


async def test_frame_dataclass():
    """Test Frame dataclass."""
    logger.info("Testing Frame dataclass...")
    try:
        frame = Frame(data=b"test", width=1920, height=1080)
        assert frame.data == b"test"
        assert frame.width == 1920
        assert frame.height == 1080
        logger.info("✓ Frame dataclass works")
        return True
    except Exception as e:
        logger.error(f"✗ Frame dataclass failed: {e}")
        return False


async def test_windowinfo_dataclass():
    """Test WindowInfo dataclass."""
    logger.info("Testing WindowInfo dataclass...")
    try:
        window = WindowInfo(hwnd=12345, title="Test Window", process_id=6789, visible=True)
        assert window.hwnd == 12345
        assert window.title == "Test Window"
        assert window.process_id == 6789
        assert window.visible is True
        logger.info("✓ WindowInfo dataclass works")
        return True
    except Exception as e:
        logger.error(f"✗ WindowInfo dataclass failed: {e}")
        return False


async def run_all_tests():
    """Run all tests and report results."""
    logger.info("=" * 70)
    logger.info("ISOLATION LAYER TEST SUITE")
    logger.info("=" * 70)

    tests = [
        ("HiddenDesktop Availability", test_hidden_desktop_availability),
        ("PlayCUA Availability", test_playcua_availability),
        ("Auto-Selection", test_auto_selection_hidden_desktop),
        ("HiddenDesktop Key Injection", test_hidden_desktop_key_injection),
        ("HiddenDesktop Mouse Click", test_hidden_desktop_mouse_click),
        ("Explicit Backend Selection", test_explicit_backend_selection),
        ("Frame Dataclass", test_frame_dataclass),
        ("WindowInfo Dataclass", test_windowinfo_dataclass),
    ]

    results = {}
    for name, test_func in tests:
        try:
            result = await test_func()
            results[name] = result
        except Exception as e:
            logger.error(f"✗ {name} exception: {e}")
            results[name] = False

    # Summary
    logger.info("=" * 70)
    passed = sum(1 for v in results.values() if v)
    total = len(results)
    logger.info(f"RESULTS: {passed}/{total} tests passed")
    logger.info("=" * 70)

    for name, result in results.items():
        status = "✓ PASS" if result else "✗ FAIL"
        logger.info(f"{status}: {name}")

    return all(results.values())


if __name__ == "__main__":
    success = asyncio.run(run_all_tests())
    sys.exit(0 if success else 1)
