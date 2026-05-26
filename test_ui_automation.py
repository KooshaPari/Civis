#!/usr/bin/env python3
"""
UI Automation script to test DINOForge UI fixes.
Tests F10 pack list, F9 debug panel, and native Mods button.
"""

import time
import subprocess
import os
import pyautogui
import pygetwindow as gw
from PIL import ImageGrab

# Configuration
GAME_PATH = r"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
LOG_PATH = r"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\LogOutput.log"
SCREENSHOT_DIR = r"C:\Users\koosh\Dino\ui_test_screenshots"

# Create screenshot directory
os.makedirs(SCREENSHOT_DIR, exist_ok=True)

def take_screenshot(name):
    """Take a screenshot and save it."""
    path = os.path.join(SCREENSHOT_DIR, f"{name}_{int(time.time())}.png")
    screenshot = ImageGrab.grab()
    screenshot.save(path)
    print(f"[SCREENSHOT] Saved: {path}")
    return path

def find_game_window():
    """Find and return the game window."""
    print("[INIT] Looking for game window...")
    for window in gw.getAllWindows():
        if "Diplomacy" in window.title or "DINO" in window.title:
            print(f"[FOUND] Game window: {window.title}")
            return window
    print("[ERROR] Game window not found. Is the game running?")
    return None

def launch_game():
    """Launch the game if not already running."""
    try:
        window = find_game_window()
        if window:
            print("[SKIP] Game already running")
            window.activate()
            return window
    except:
        pass

    print("[LAUNCH] Starting game...")
    subprocess.Popen(GAME_PATH)
    time.sleep(30)  # Wait for game to load
    return find_game_window()

def get_latest_logs(n=50):
    """Read latest N lines from BepInEx log."""
    try:
        with open(LOG_PATH, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
            return ''.join(lines[-n:])
    except:
        return "[ERROR] Could not read log file"

def test_f10_pack_list(game_window):
    """Test F10 pack list sidebar."""
    print("\n[TEST 1] F10 Pack List Sidebar")
    print("=" * 50)

    # Focus game
    game_window.activate()
    time.sleep(0.5)

    # Take screenshot before F10
    take_screenshot("01_before_f10")

    # Press F10
    print("[ACTION] Pressing F10...")
    pyautogui.press('f10')
    time.sleep(1)

    # Take screenshot after F10
    take_screenshot("02_after_f10_menu")

    # Check logs for ModMenuPanel messages
    print("[LOG CHECK] Checking for ModMenuPanel logs...")
    logs = get_latest_logs(100)

    if "[ModMenuPanel]" in logs:
        print("[PASS] ModMenuPanel logs found")
        for line in logs.split('\n'):
            if "[ModMenuPanel]" in line:
                print(f"  {line}")
    else:
        print("[FAIL] No ModMenuPanel logs found")

    if "SetPacks" in logs or "RebuildPackList" in logs:
        print("[PASS] Pack loading logs found")
    else:
        print("[WARN] No pack loading logs found")

    # Close menu
    pyautogui.press('f10')
    time.sleep(0.5)

def test_f9_debug_panel(game_window):
    """Test F9 debug panel."""
    print("\n[TEST 2] F9 Debug Panel")
    print("=" * 50)

    game_window.activate()
    time.sleep(0.5)

    # Take screenshot before F9
    take_screenshot("03_before_f9")

    # Press F9
    print("[ACTION] Pressing F9...")
    pyautogui.press('f9')
    time.sleep(1)

    # Take screenshot after F9
    take_screenshot("04_after_f9_debug")

    # Check logs for DebugPanel messages
    print("[LOG CHECK] Checking for DebugPanel logs...")
    logs = get_latest_logs(100)

    if "[DebugPanel]" in logs:
        print("[PASS] DebugPanel logs found")
        for line in logs.split('\n'):
            if "[DebugPanel]" in line:
                print(f"  {line}")
    else:
        print("[FAIL] No DebugPanel logs found")

    # Close panel
    pyautogui.press('f9')
    time.sleep(0.5)

def test_native_mods_button(game_window):
    """Test native Mods button in main menu."""
    print("\n[TEST 3] Native Mods Button")
    print("=" * 50)

    game_window.activate()
    time.sleep(0.5)

    # Check logs for NativeMenuInjector messages
    print("[LOG CHECK] Checking for NativeMenuInjector logs...")
    logs = get_latest_logs(200)

    if "[NativeMenuInjector]" in logs:
        print("[PASS] NativeMenuInjector logs found")
        for line in logs.split('\n'):
            if "[NativeMenuInjector]" in line and ("SUCCESS" in line or "Found Settings" in line or "Clone successful" in line):
                print(f"  {line}")
    else:
        print("[FAIL] No NativeMenuInjector logs found")

    # Try to find Mods button on screen
    take_screenshot("05_main_menu_mods_button")
    print("[ACTION] Manual verification needed: Look at screenshot for Mods button visibility")

def main():
    """Main test execution."""
    print("[START] DINOForge UI Automation Test")
    print(f"Log file: {LOG_PATH}")
    print(f"Screenshot dir: {SCREENSHOT_DIR}")

    # Launch/find game
    game_window = launch_game()
    if not game_window:
        print("[FATAL] Game window not found. Exiting.")
        return

    # Run tests
    try:
        test_f10_pack_list(game_window)
        test_f9_debug_panel(game_window)
        test_native_mods_button(game_window)
    except Exception as e:
        print(f"[ERROR] Test failed with exception: {e}")
        import traceback
        traceback.print_exc()

    print("\n[COMPLETE] Tests finished")
    print("[ARTIFACTS] Check screenshots and log analysis above")

if __name__ == "__main__":
    main()
