#!/usr/bin/env python3
"""
Visual UI Testing - Directly observe game state with F10/F9 presses.
Uses keyboard input and screenshot capture to verify fixes.
"""

import time
import pyautogui
import pygetwindow as gw
from PIL import ImageGrab
import os

SCREENSHOT_DIR = r"C:\Users\koosh\Dino\ui_test_screenshots"

def get_game_window():
    """Find the game window."""
    for window in gw.getAllWindows():
        if "Diplomacy" in window.title or "DINO" in window.title:
            return window
    return None

def main():
    os.makedirs(SCREENSHOT_DIR, exist_ok=True)

    game_window = get_game_window()
    if not game_window:
        print("[ERROR] Game window not found. Is the game running?")
        return

    game_window.activate()
    time.sleep(1)

    print("\n" + "="*70)
    print("TEST 1: F10 PACK LIST VISUAL TEST")
    print("="*70)

    print("[ACTION] Pressing F10 to open pack list...")
    pyautogui.press('f10')
    time.sleep(2.5)

    screenshot = ImageGrab.grab()
    f10_path = os.path.join(SCREENSHOT_DIR, "TEST_1_F10_pack_list.png")
    screenshot.save(f10_path)
    print(f"[CAPTURE] Screenshot saved: {f10_path}")
    print("\n[OBSERVATION NEEDED]")
    print("  Q1: Can you see the LEFT SIDEBAR with pack names?")
    print("       (Economy Balanced, Example Balance Tweak, Scenario Tutorial, etc.)")
    print("  Q2: Are the pack names VISIBLE and READABLE?")
    print("  Q3: Can you click on different packs to select them?")

    input("\n[PAUSE] Look at the screenshot above. Press ENTER when ready to continue...")

    print("\n[ACTION] Pressing F10 again to close pack list...")
    pyautogui.press('f10')
    time.sleep(1)

    print("\n" + "="*70)
    print("TEST 2: F9 DEBUG PANEL VISUAL TEST")
    print("="*70)

    print("[ACTION] Pressing F9 to open debug panel...")
    pyautogui.press('f9')
    time.sleep(2.5)

    screenshot = ImageGrab.grab()
    f9_path = os.path.join(SCREENSHOT_DIR, "TEST_2_F9_debug_panel.png")
    screenshot.save(f9_path)
    print(f"[CAPTURE] Screenshot saved: {f9_path}")
    print("\n[OBSERVATION NEEDED]")
    print("  Q1: Is the DEBUG PANEL visible?")
    print("  Q2: Does it have CONTENT (sections, stats, diagnostic info)?")
    print("  Q3: Or is it completely empty/blank?")

    input("\n[PAUSE] Look at the screenshot. Press ENTER when ready...")

    print("\n[ACTION] Pressing F9 again to close debug panel...")
    pyautogui.press('f9')
    time.sleep(1)

    print("\n" + "="*70)
    print("TEST 3: NATIVE MODS BUTTON VISUAL TEST")
    print("="*70)

    print("[ACTION] Game is now at main menu. Looking for Mods button...")
    time.sleep(1)

    screenshot = ImageGrab.grab()
    mods_path = os.path.join(SCREENSHOT_DIR, "TEST_3_native_mods_button.png")
    screenshot.save(mods_path)
    print(f"[CAPTURE] Screenshot saved: {mods_path}")
    print("\n[OBSERVATION NEEDED]")
    print("  Q1: Can you see the 'Mods' button in the native menu?")
    print("       (Below the Settings/Options button)")
    print("  Q2: When you HOVER over it, does it highlight in red?")
    print("  Q3: Does it respond to clicks?")

    input("\n[PAUSE] Look at the screenshot. Press ENTER when done observing...")

    print("\n" + "="*70)
    print("TEST COMPLETE")
    print("="*70)
    print(f"\n[RESULTS] All screenshots saved to: {SCREENSHOT_DIR}")
    print("\nPlease describe what you observed for each test:")
    print("  - Are pack names visible in the F10 menu?")
    print("  - Is the F9 debug panel showing content?")
    print("  - Does the Mods button work and highlight?")

if __name__ == "__main__":
    main()
