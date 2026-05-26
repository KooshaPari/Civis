#!/usr/bin/env python3
"""Capture UI states without interaction."""

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

def capture(name, delay=2):
    time.sleep(delay)
    screenshot = ImageGrab.grab()
    path = os.path.join(SCREENSHOT_DIR, name)
    screenshot.save(path)
    print(f"[OK] {name}")
    return path

os.makedirs(SCREENSHOT_DIR, exist_ok=True)
game = get_game_window()

if not game:
    print("Game not found")
    exit(1)

game.activate()
time.sleep(1)

print("\n[CAPTURING UI STATES]")
print("-" * 50)

# Capture F10 pack list
print("Pressing F10...")
pyautogui.press('f10')
capture("1_F10_PACK_LIST_OPENED.png", 3)

print("Closing F10...")
pyautogui.press('f10')
time.sleep(1)

# Capture F9 debug panel
print("Pressing F9...")
pyautogui.press('f9')
capture("2_F9_DEBUG_PANEL_OPENED.png", 3)

print("Closing F9...")
pyautogui.press('f9')
time.sleep(1)

print("\n[DONE] Screenshots ready for visual inspection")
print(f"Location: {SCREENSHOT_DIR}")
