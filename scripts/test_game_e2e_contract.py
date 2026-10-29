import pathlib
import subprocess
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = (ROOT / "scripts/game-e2e.sh").read_text(encoding="utf-8")
WORKFLOW = (ROOT / ".github/workflows/game-e2e.yml").read_text(encoding="utf-8")
try:
    BASH_READY = (
        subprocess.run(["bash", "--version"], capture_output=True).returncode == 0
    )
except OSError:
    BASH_READY = False


class GameE2eContract(unittest.TestCase):
    def run_bash(self, code):
        return subprocess.run(["bash", "-c", code], text=True, capture_output=True)

    @unittest.skipUnless(BASH_READY, "Bash unavailable on this host")
    def test_capture_disabled_before_any_hook(self):
        function = SCRIPT.split("capture_screenshot() {", 1)[1].split("# Main menu", 1)[
            0
        ]
        code = "capture_screenshot() {" + function
        sentinels = """
sleep() { exit 88; }
try_bevy_capture() { exit 88; }
try_xwd_capture() { exit 88; }
try_macos_capture() { exit 88; }
"""
        for value in ("0", "", "invalid"):
            result = self.run_bash(
                sentinels
                + code
                + '\nCAPTURE_FRAMES="'
                + value
                + '"; capture_screenshot forbidden 0\n'
            )
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_capture_defaults_and_workflow_gates(self):
        self.assertIn('CAPTURE_FRAMES="${CIVIS_CAPTURE_FRAMES:-0}"', SCRIPT)
        self.assertIn('CIVIS_CAPTURE_FRAMES: "0"', WORKFLOW)
        self.assertIn(
            'if [[ "${CAPTURE_FRAMES}" == 1 ]]; then\nstep "Verifying captures..."',
            SCRIPT,
        )
        for block in ("visual-regression", "update-baselines"):
            self.assertRegex(
                WORKFLOW.split("  " + block + ":", 1)[1],
                r"(?s)^.*?if: \$\{\{ false \}\}",
            )
        # The "Upload captures" step must be immediately followed by a
        # disabled guard. Match it structurally instead of by a hard-coded
        # indent: the previous literal pinned exactly 8 leading spaces, which
        # only ever matched the pre-squash 2-space layout of this workflow.
        # The repo's .editorconfig mandates indent_size=4 for *.yml and CI
        # enforces it with prettier --check, so a canonical workflow file can
        # never satisfy that literal. The property under test is the guard,
        # not the indentation, so assert the guard and the step's key
        # nesting (the `if:` key is a sibling of `name:`, two columns in).
        self.assertRegex(
            WORKFLOW,
            r"(?m)^(?P<step>[ ]*)- name: Upload captures[ ]*\n"
            r"(?P=step)[ ]{2}if:[ ]+\$\{\{ false \}\}[ ]*(?:#.*)?$",
        )

    @unittest.skipUnless(BASH_READY, "Bash unavailable on this host")
    def test_occupied_port_never_kills_listener(self):
        block = SCRIPT.split("# Kill any stale processes on the port", 1)[1].split(
            "# ── Start Xvfb", 1
        )[0]
        result = self.run_bash(
            "PORT=3000\nlsof() { return 0; }\n"
            'kill() { exit 88; }\nfail() { echo "$*" >&2; }\n' + block
        )
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn("refusing to stop another process", result.stderr)
        self.assertIn("check_tool lsof ", SCRIPT)

    def test_cleanup_uses_only_current_children(self):
        cleanup = SCRIPT.split("cleanup() {", 1)[1].split("trap cleanup EXIT", 1)[0]
        self.assertNotIn("cat ", cleanup)
        self.assertIn('kill "${GAME_PID}"', cleanup)
        self.assertIn('kill "${SERVER_PID}"', cleanup)
        self.assertIn('SERVER_PID=""\nGAME_PID=""', SCRIPT)

    def test_both_build_commands_select_their_binary(self):
        for source in (SCRIPT, WORKFLOW):
            commands = source.replace("\\\n", " ")
            self.assertRegex(
                commands,
                r"cargo build --locked --release\s+-p civ-server\s+--bin civ-server",
            )
            self.assertRegex(
                commands,
                r"cargo build --locked --release\s+-p civ-bevy-ref\s+--bin civ-standalone",
            )

    @unittest.skipUnless(BASH_READY, "Bash unavailable on this host")
    def test_shell_syntax(self):
        result = subprocess.run(
            ["bash", "-n", str(ROOT / "scripts/game-e2e.sh")],
            text=True,
            capture_output=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
