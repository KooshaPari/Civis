#!/usr/bin/env python3
"""Test with aggressive window focus management."""

import time
import pyautogui
import pygetwindow as gw
from PIL import ImageGrab
import os

SCREENSHOT_DIR = r"C:\Users\koosh\Dino\ui_test_screenshots"

def get_game_window():
    for window in gw.getAllWindows():
        if "Diplomacy" in window.title:
            return window
    return None

os.makedirs(SCREENSHOT_DIR, exist_ok=True)

print("Finding game window...")
game = get_game_window()

if not game:
    print("[ERROR] Game window not found")
    exit(1)

print(f"[FOUND] Game window: {game.title}")
print(f"[STATE] Is active: {game.isActive}")

# Activate and ensure focus
print("[ACTION] Activating window...")
game.activate()
time.sleep(0.5)

# Try clicking in the middle of the window to ensure focus
print("[ACTION] Clicking in window center to ensure focus...")
pyautogui.click(game.left + game.width//2, game.top + game.height//2)
time.sleep(1)

print(f"[STATE] Is active now: {game.isActive}")

# Baseline
print("[BASELINE] Taking screenshot before F10...")
screenshot = ImageGrab.grab()
screenshot.save(os.path.join(SCREENSHOT_DIR, "baseline.png"))

# F10 with extra wait and focus
print("[ACTION] Pressing F10 (with 3s wait)...")
pyautogui.press('f10')
time.sleep(3)

screenshot = ImageGrab.grab()
screenshot.save(os.path.join(SCREENSHOT_DIR, "after_f10.png"))
print("[CAPTURE] Screenshot saved")

# Check if menu opened by looking at window title or asking user
print("\n[OBSERVATION] Did the pack list menu appear on screen?")
print("  (Check the 'after_f10.png' screenshot)")

# Press F10 again to close
pyautogui.press('f10')
time.sleep(1)

# F9
print("\n[ACTION] Pressing F9 (with 3s wait)...")
pyautogui.press('f9')
time.sleep(3)

screenshot = ImageGrab.grab()
screenshot.save(os.path.join(SCREENSHOT_DIR, "after_f9.png"))
print("[CAPTURE] Screenshot saved")

print("\n[OBSERVATION] Did the debug panel appear on screen?")
print("  (Check the 'after_f9.png' screenshot)")

pyautogui.press('f9')
time.sleep(1)

print("\nDone. Check screenshots in:", SCREENSHOT_DIR)
