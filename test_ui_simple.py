#!/usr/bin/env python3
"""
Simple UI test - focus on visual observation only
"""

import time
import subprocess
import os
import pyautogui
import pygetwindow as gw

GAME_PATH = r"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"

def find_game_window():
    """Find and return the game window."""
    print("[INIT] Looking for game window...")
    for window in gw.getAllWindows():
        if "Diplomacy" in window.title or "DINO" in window.title:
            print(f"[FOUND] Game window: {window.title}")
            return window
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
    time.sleep(35)  # Wait for game to load
    return find_game_window()

def main():
    print("[START] DINOForge UI Visual Test")

    game_window = launch_game()
    if not game_window:
        print("[FATAL] Game window not found")
        return

    game_window.activate()
    time.sleep(1)

    print("\n[TEST 1] F10 Pack List Sidebar")
    print("=" * 50)
    print("[ACTION] Pressing F10 to open pack list menu...")
    pyautogui.press('f10')
    time.sleep(2)
    print("[USER ACTION REQUIRED] Look at the game window.")
    print("  Q: Can you see the pack list sidebar on the left?")
    print("  Q: Are pack names visible? (Economy Balanced, etc.)")
    print("  Q: Is there ANY text in the left panel or is it completely empty?")
    input("Press ENTER when you've observed and ready to continue...")
    pyautogui.press('f10')
    time.sleep(0.5)

    print("\n[TEST 2] F9 Debug Panel")
    print("=" * 50)
    print("[ACTION] Pressing F9 to open debug panel...")
    pyautogui.press('f9')
    time.sleep(2)
    print("[USER ACTION REQUIRED] Look at the game window.")
    print("  Q: Is the debug panel visible?")
    print("  Q: Does it have any content or is it empty?")
    print("  Q: Can you see section headers (Diagnostics, Gameplay, etc)?")
    input("Press ENTER when you've observed and ready to continue...")
    pyautogui.press('f9')
    time.sleep(0.5)

    print("\n[TEST 3] Native Mods Button")
    print("=" * 50)
    print("[ACTION] Returning to main menu to check Mods button...")
    time.sleep(1)
    print("[USER ACTION REQUIRED] Look at main menu buttons.")
    print("  Q: Can you see the 'Mods' button below Settings/Options?")
    print("  Q: When you hover over it, does it highlight in red like other buttons?")
    print("  Q: Can you click it?")
    input("Press ENTER when done observing...")

    print("\n[COMPLETE] Visual test finished")
    print("[NEXT] Share your observations above so I can diagnose the root causes")

if __name__ == "__main__":
    main()
