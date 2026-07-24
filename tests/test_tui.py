"""Unit tests for lib/cov_tui.py."""

from __future__ import annotations

import unittest
from pathlib import Path

from textual.widgets import Select

from lib.cov_tui import CovToolkitApp


class TestCovTuiCommand(unittest.IsolatedAsyncioTestCase):
    async def test_command_generation_default_context(self) -> None:
        app = CovToolkitApp()
        async with app.run_test() as pilot:
            cmd = app.command()
            self.assertEqual(cmd[0], str(Path(__file__).resolve().parent.parent / "bin/cov-context"))
            self.assertEqual(cmd[1], "save")

    async def test_command_generation_blank_select_raises_value_error(self) -> None:
        app = CovToolkitApp()
        async with app.run_test() as pilot:
            app.query_one("#source", Select).clear()
            with self.assertRaises(ValueError) as ctx:
                app.command()
            self.assertIn("Please select a valid Workflow Source Target", str(ctx.exception))

    async def test_command_generation_blank_mode_raises_value_error(self) -> None:
        app = CovToolkitApp()
        async with app.run_test() as pilot:
            app.query_one("#mode", Select).clear()
            with self.assertRaises(ValueError) as ctx:
                app.command()
            self.assertIn("Please select a valid Action Mode", str(ctx.exception))


if __name__ == "__main__":
    unittest.main()
