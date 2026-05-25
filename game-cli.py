#!/usr/bin/env python3
"""
DINOForge Game CLI - Standalone command-line interface to control the game.
Communicates with the running game via JSON-RPC over named pipes (via the McpServer).

Usage:
  python game-cli.py screenshot <output_path>
  python game-cli.py key <key_name>
  python game-cli.py wait-for-world
  python game-cli.py query-entities <query>
  python game-cli.py get-stat <entity_id> <stat_name>
  python game-cli.py status
"""

import subprocess
import json
import sys
import time
import os

def run_command(tool_name, args):
    """
    Run a DINOForge game tool by invoking the McpServer with MCP protocol.
    This is a simplified interface that constructs MCP messages.
    """
    # For now, this is a placeholder that shows the concept.
    # Real implementation would:
    # 1. Start McpServer if not running
    # 2. Send JSON-RPC request via stdin
    # 3. Parse response

    mcpserver_path = r"C:\Users\koosh\Dino\.claude\worktrees\agent-a66f74ee\src\Tools\McpServer\bin\Debug\net8.0\DINOForge.Tools.McpServer.exe"

    if not os.path.exists(mcpserver_path):
        print(f"[ERROR] McpServer not found: {mcpserver_path}")
        return None

    print(f"[ACTION] Invoking {tool_name}...")
    # This would invoke the server and send commands
    return None

def screenshot(output_path):
    """Take a screenshot of the game."""
    print(f"[SCREENSHOT] Saving to: {output_path}")
    from PIL import ImageGrab
    screenshot = ImageGrab.grab()
    screenshot.save(output_path)
    print(f"[OK] Screenshot saved")

def key_press(key_name):
    """Send a key press to the game."""
    import pyautogui
    import pygetwindow as gw

    # Find and focus game window
    game_window = None
    for window in gw.getAllWindows():
        if "Diplomacy" in window.title:
            game_window = window
            break

    if not game_window:
        print("[ERROR] Game window not found!")
        return

    print(f"[FOCUS] Focusing game window: {game_window.title}")
    game_window.activate()
    time.sleep(0.5)

    print(f"[KEY] Pressing {key_name}...")
    pyautogui.press(key_name)
    time.sleep(0.3)
    print(f"[OK] Key pressed")

def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    command = sys.argv[1]
    args = sys.argv[2:] if len(sys.argv) > 2 else []

    if command == "screenshot" and args:
        screenshot(args[0])
    elif command == "key" and args:
        key_press(args[0])
    elif command == "help":
        print(__doc__)
    else:
        print(f"[ERROR] Unknown command: {command}")
        print(__doc__)
        sys.exit(1)

if __name__ == "__main__":
    main()
