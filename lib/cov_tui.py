#!/usr/bin/env python3
"""Ghostty-friendly terminal interface for the Riley COV Toolkit."""

from __future__ import annotations

import subprocess
from pathlib import Path

from textual.app import App, ComposeResult
from textual.containers import Container, Horizontal, Vertical, VerticalScroll
from textual.widgets import Button, Footer, Header, Input, Label, RichLog, Select, Static


ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "bin"


class CovToolkitApp(App[None]):
    TITLE = "COV Artwork Toolkit"
    SUB_TITLE = "Search, save, and embed high-res album artwork"

    CSS = """
    Screen {
        background: #0f172a;
        color: #f8fafc;
        layout: vertical;
    }

    Header {
        background: #1e293b;
        color: #38bdf8;
        dock: top;
    }

    Footer {
        background: #1e293b;
        color: #94a3b8;
    }

    #main-container {
        padding: 1 2;
        height: 1fr;
    }

    #header-box {
        background: #1e293b;
        border: round #38bdf8;
        padding: 1 2;
        margin-bottom: 1;
        height: auto;
    }

    #header-title {
        color: #38bdf8;
        text-style: bold;
    }

    #header-desc {
        color: #94a3b8;
        margin-top: 0;
    }

    #content-grid {
        height: 1fr;
    }

    #form-panel {
        width: 55%;
        height: 100%;
        background: #1e293b;
        border: round #334155;
        padding: 1 2;
        margin-right: 1;
    }

    #log-panel {
        width: 45%;
        height: 100%;
        background: #1e293b;
        border: round #334155;
        padding: 1;
    }

    .section-title {
        color: #38bdf8;
        text-style: bold;
        margin-top: 1;
        margin-bottom: 0;
        border-bottom: solid #334155;
    }

    .field-label {
        color: #cbd5e1;
        margin-top: 1;
    }

    Input {
        background: #0f172a;
        border: tall #475569;
        color: #f8fafc;
        margin-bottom: 0;
    }

    Input:focus {
        border: tall #38bdf8;
    }

    Select {
        background: #0f172a;
        border: tall #475569;
        margin-bottom: 0;
    }

    Select:focus {
        border: tall #38bdf8;
    }

    #actions-row {
        height: auto;
        margin-top: 1;
        margin-bottom: 1;
    }

    Button {
        margin-right: 1;
        border: none;
        min-width: 14;
        height: 3;
    }

    #launch {
        background: #0284c7;
        color: #ffffff;
        text-style: bold;
    }

    #launch:hover {
        background: #0369a1;
    }

    #doctor {
        background: #334155;
        color: #f8fafc;
    }

    #doctor:hover {
        background: #475569;
    }

    #show-log {
        background: #334155;
        color: #f8fafc;
    }

    #show-log:hover {
        background: #475569;
    }

    #quit {
        background: #be123c;
        color: #ffffff;
    }

    #quit:hover {
        background: #9f1239;
    }

    #log {
        background: #0f172a;
        border: none;
        height: 100%;
        color: #e2e8f0;
    }
    """

    BINDINGS = [
        ("q", "quit", "Quit"),
        ("d", "doctor", "Doctor"),
        ("l", "launch", "Launch"),
        ("c", "clear_log", "Clear Console"),
    ]

    def compose(self) -> ComposeResult:
        yield Header()
        with Container(id="main-container"):
            with Container(id="header-box"):
                yield Label("COV INTEGRATION TOOLKIT", id="header-title")
                yield Label(
                    "Automatic metadata-aware cover search & high-res artwork tag embedder",
                    id="header-desc",
                )

            with Horizontal(id="content-grid"):
                with VerticalScroll(id="form-panel"):
                    yield Label("1. Workflow Target", classes="section-title")
                    yield Label("Source Provider", classes="field-label")
                    yield Select(
                        [
                            ("Automatic Context (Swinsian/Finder/Clipboard)", "context"),
                            ("Direct Path (Audio file or Album directory)", "path"),
                            ("Swinsian Selection / Playing Track", "swinsian"),
                            ("Finder Selection", "finder"),
                            ("Browse Folder (Native Chooser)...", "choose"),
                            ("Clipboard Path Text", "clipboard"),
                        ],
                        value="context",
                        id="source",
                    )
                    yield Label("Target Path (Direct Path only)", classes="field-label")
                    yield Input(placeholder="/Volumes/Audio/Artist/Album", id="path")

                    yield Label("2. Action Mode", classes="section-title")
                    yield Select(
                        [
                            ("Save Sidecar Cover Beside Album", "save"),
                            ("Save Cover & Embed into All Album Tracks", "embed"),
                        ],
                        value="save",
                        id="mode",
                    )

                    yield Label("3. Optional Search Overrides", classes="section-title")
                    yield Input(placeholder="Artist Name", id="artist")
                    yield Input(placeholder="Album Name", id="album")
                    yield Input(placeholder="Barcode / Catalogue Number", id="identifier")
                    yield Input(placeholder="Preferred Resolution (e.g. 1500)", id="resolution")
                    yield Input(placeholder="COV Source IDs (comma-separated)", id="sources")

                    with Horizontal(id="actions-row"):
                        yield Button("Launch COV", id="launch", variant="primary")
                        yield Button("Run Doctor", id="doctor")
                        yield Button("Show Log", id="show-log")
                        yield Button("Quit", id="quit", variant="error")

                with Vertical(id="log-panel"):
                    yield Label("Output & Execution Log", classes="section-title")
                    yield RichLog(id="log", markup=True, wrap=True)

        yield Footer()

    def write(self, message: str) -> None:
        self.query_one("#log", RichLog).write(message)

    def action_clear_log(self) -> None:
        self.query_one("#log", RichLog).clear()

    def command(self) -> list[str]:
        source = str(self.query_one("#source", Select).value)
        mode = str(self.query_one("#mode", Select).value)
        if source == "path":
            path = self.query_one("#path", Input).value.strip()
            if not path:
                raise ValueError("Please provide an audio file or album directory path.")
            command = [str(BIN / ("cov-open-embed" if mode == "embed" else "cov-open")), path]
        else:
            command = [str(BIN / f"cov-{source}"), mode]

        if source == "path":
            for widget_id, option in (
                ("artist", "--artist"),
                ("album", "--album"),
                ("identifier", "--identifier"),
                ("resolution", "--resolution"),
                ("sources", "--sources"),
            ):
                value = self.query_one(f"#{widget_id}", Input).value.strip()
                if value:
                    command.extend([option, value])
        return command

    def launch_cov(self) -> None:
        try:
            command = self.command()
        except ValueError as error:
            self.write(f"[bold red]Error:[/bold red] {error}")
            return
        self.write(f"[bold #38bdf8]Executing:[/bold #38bdf8] [dim]{' '.join(command)}[/dim]")
        completed = subprocess.run(command, capture_output=True, text=True, check=False)
        output = (completed.stdout + completed.stderr).strip()
        self.write(output or f"[green]Command executed cleanly (status code {completed.returncode}).[/green]")

    def run_doctor(self) -> None:
        self.write("[bold #38bdf8]Running Environment Doctor...[/bold #38bdf8]")
        completed = subprocess.run([str(BIN / "cov-doctor")], capture_output=True, text=True, check=False)
        self.write(completed.stdout.strip())

    def show_log(self) -> None:
        self.write("[bold #38bdf8]Fetching Live Integration Logs...[/bold #38bdf8]")
        completed = subprocess.run([str(BIN / "cov-log")], capture_output=True, text=True, check=False)
        self.write(completed.stdout.strip())

    def on_button_pressed(self, event: Button.Pressed) -> None:
        match event.button.id:
            case "launch":
                self.launch_cov()
            case "doctor":
                self.run_doctor()
            case "show-log":
                self.show_log()
            case "quit":
                self.exit()

    def action_launch(self) -> None:
        self.launch_cov()

    def action_doctor(self) -> None:
        self.run_doctor()


def main() -> None:
    CovToolkitApp().run()


if __name__ == "__main__":
    main()
