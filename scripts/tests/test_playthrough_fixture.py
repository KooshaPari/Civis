#!/usr/bin/env python3
"""Bounded lifecycle regressions for the isolated live fixture runner."""

import argparse
import ctypes
import importlib.util
import json
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


SOURCE = Path(__file__).with_name("run_playthrough_fixture.py")
SPEC = importlib.util.spec_from_file_location("playthrough_fixture", SOURCE)
fixture_module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fixture_module)


@unittest.skipUnless(sys.platform.startswith("linux"), "Linux ownership probes")
class FixtureLifecycleTests(unittest.TestCase):
    def fixture(self, directory):
        args = argparse.Namespace(
            server_binary=Path("/tmp/unused/civ-server"),
            websocat=Path("/tmp/unused/websocat"),
            report=Path(directory) / "report.json",
        )
        return fixture_module.Fixture(args)

    def test_unexpected_schema_error_always_calls_cleanup(self):
        error = KeyError("unexpected wire schema")
        fixture = mock.Mock()
        fixture.run.side_effect = error
        fixture.finish.return_value = 1
        args = [
            str(SOURCE), "--server-binary", "/tmp/unused/civ-server",
            "--websocat", "/tmp/unused/websocat", "--report", "/tmp/unused/report",
        ]
        with mock.patch.object(sys, "argv", args):
            with mock.patch.object(fixture_module, "Fixture", return_value=fixture):
                self.assertEqual(fixture_module.main(), 1)
        fixture.finish.assert_called_once_with(error)

    def test_other_process_listener_is_never_owned(self):
        with tempfile.TemporaryDirectory(prefix="civis-fixture-test-") as directory:
            fixture = self.fixture(directory)
            with socket.socket() as listener:
                listener.bind(("127.0.0.1", 0))
                listener.listen()
                fixture.port = listener.getsockname()[1]
                process = subprocess.Popen(
                    [sys.executable, "-c", "import time; time.sleep(10)"],
                    start_new_session=True,
                )
                fixture.server = process
                fixture.sessions.append(process)
                try:
                    self.assertIsNone(fixture.owned_listener())
                finally:
                    fixture.stop(process)
                self.assertEqual(fixture_module.process_session(process.pid), [])

    def test_term_ignoring_child_is_killed_after_parent_exits_and_reported(self):
        self.assertEqual(ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0), 0)
        with tempfile.TemporaryDirectory(prefix="civis-fixture-test-") as directory:
            fixture = self.fixture(directory)
            marker = Path(directory) / "child.pid"
            child_code = (
                "import os, signal, time\n"
                "from pathlib import Path\n"
                "signal.signal(signal.SIGTERM, signal.SIG_IGN)\n"
                f"Path({str(marker)!r}).write_text(str(os.getpid()))\n"
                "time.sleep(10)\n"
            )
            parent_code = (
                "import subprocess, sys, time\n"
                "from pathlib import Path\n"
                f"subprocess.Popen([sys.executable, '-c', {child_code!r}])\n"
                f"marker = Path({str(marker)!r})\n"
                "while not marker.exists(): time.sleep(0.01)\n"
            )
            process = subprocess.Popen(
                [sys.executable, "-c", parent_code], start_new_session=True,
            )
            fixture.sessions.append(process)
            try:
                process.wait(timeout=2)
                child_pid = int(marker.read_text())
                self.assertIn(child_pid, fixture_module.process_session(process.pid))
                self.assertEqual(fixture.stop(process), [])
                self.assertIn(child_pid, fixture.reaped)
                with mock.patch("builtins.print"):
                    self.assertEqual(fixture.finish(None), 1)
                report = json.loads(fixture.report_path.read_text())
                self.assertFalse(report["passed"])
                self.assertFalse(report["cleanup"]["zero_new_orphans"])
            finally:
                fixture.stop(process)


if __name__ == "__main__":
    unittest.main()
