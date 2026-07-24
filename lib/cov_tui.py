#!/usr/bin/env python3
"""Ghostty-friendly terminal interface for the Riley COV Toolkit."""

from __future__ import annotations

import subprocess
from pathlib import Path

from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical, VerticalScroll
from textual.design import ColorSystem
from textual.widgets import Button, Footer, Header, Input, Label, RichLog, Select


ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "bin"


# Built-in themes using Textual's design tokens
THEMES: dict[str, dict[str, str]] = {
    "nord": {
        "primary": "#88c0d0",
        "secondary": "#81a1c1",
        "accent": "#b48ead",
        "background": "#2e3440",
        "surface": "#3b4252",
        "panel": "#434c5e",
        "warning": "#ebcb8b",
        "error": "#bf616a",
        "success": "#a3be8c",
        "dark": True,
    },
    "monokai": {
        "primary": "#f92672",
        "secondary": "#66d9ef",
        "accent": "#ae81ff",
        "background": "#272822",
        "surface": "#3e3d32",
        "panel": "#49483e",
        "warning": "#fd971f",
        "error": "#f92672",
        "success": "#a6e22e",
        "dark": True,
    },
    "cobalt": {
        "primary": "#38bdf8",
        "secondary": "#818cf8",
        "accent": "#f472b6",
        "background": "#0f172a",
        "surface": "#1e293b",
        "panel": "#334155",
        "warning": "#fbbf24",
        "error": "#f43f5e",
        "success": "#34d399",
        "dark": True,
    },
    "emerald": {
        "primary": "#10b981",
        "secondary": "#06b6d4",
        "accent": "#f59e0b",
        "background": "#064e3b",
        "surface": "#047857",
        "panel": "#065f46",
        "warning": "#f59e0b",
        "error": "#ef4444",
        "success": "#10b981",
        "dark": True,
    },
    "dracula": {
        "primary": "#bd93f9",
        "secondary": "#8be9fd",
        "accent": "#ff79c6",
        "background": "#282a36",
        "surface": "#44475a",
        "panel": "#6272a4",
        "warning": "#f1fa8c",
        "error": "#ff5555",
        "success": "#50fa7b",
        "dark": True,
    },
}


class CovToolkitApp(App[None]):
    TITLE = "COV Artwork Toolkit"
    SUB_TITLE = "Search, save, and embed high-res album artwork"

    CSS = """
    Screen {
        background: $background;
        color: $text;
        layout: vertical;
    }

    Header {
        background: $surface;
        color: $primary;
        dock: top;
    }

    Footer {
        background: $surface;
        color: $text-muted;
    }

    #main-grid {
        height: 1fr;
        padding: 0 1;
    }

    #form-column {
        width: 1fr;
        height: 100%;
        background: $surface;
        border: round $primary 50%;
        border-title-color: $primary;
        border-title-style: bold;
        padding: 0 1;
        margin-right: 1;
    }

    #log-column {
        width: 1fr;
        height: 100%;
        background: $surface;
        border: round $primary 50%;
        border-title-color: $accent;
        border-title-style: bold;
        padding: 0 1;
    }

    .group-box {
        background: $panel 40%;
        border: round $primary 30%;
        padding: 0 1;
        margin-top: 1;
        margin-bottom: 1;
        height: auto;
    }

    .group-title {
        color: $primary;
        text-style: bold;
        margin-top: 0;
        margin-bottom: 0;
    }

    .field-label {
        color: $text;
        margin-top: 0;
        margin-bottom: 0;
        text-style: bold;
    }

    Input {
        background: $background;
        border: tall $primary 40%;
        color: $text;
        height: 3;
        padding: 0 1;
        margin-bottom: 1;
    }

    Input:focus {
        border: tall $accent;
    }

    Select {
        background: $background;
        border: tall $primary 40%;
        height: 3;
        margin-bottom: 1;
    }

    Select:focus {
        border: tall $accent;
    }

    #actions-box {
        height: auto;
        padding: 1 0 0 0;
    }

    Button {
        margin-right: 1;
        min-width: 12;
        height: 3;
        border: none;
    }

    #launch {
        background: $primary;
        color: $background;
        text-style: bold;
    }

    #launch:hover {
        background: $primary-lighten-1;
    }

    #doctor {
        background: $panel;
        color: $text;
    }

    #doctor:hover {
        background: $panel-lighten-1;
    }

    #show-log {
        background: $panel;
        color: $text;
    }

    #show-log:hover {
        background: $panel-lighten-1;
    }

    #quit {
        background: $error;
        color: #ffffff;
    }

    #quit:hover {
        background: $error-darken-1;
    }

    #log {
        background: $background;
        border: none;
        height: 1fr;
        color: $text;
        padding: 1;
    }
    """

    BINDINGS = [
        ("q", "quit", "Quit"),
        ("d", "doctor", "Doctor"),
        ("l", "launch", "Launch"),
        ("c", "clear_log", "Clear Log"),
        ("t", "next_theme", "Switch Theme"),
    ]

    def __init__(self) -> None:
        super().__init__()
        self.current_theme_name = "cobalt"

    def on_mount(self) -> None:
        self.apply_theme(self.current_theme_name)

    def apply_theme(self, theme_name: str) -> None:
        if theme_name in THEMES:
            t = THEMES[theme_name]
            color_sys = ColorSystem(
                primary=t["primary"],
                secondary=t.get("secondary"),
                accent=t.get("accent"),
                background=t.get("background"),
                surface=t.get("surface"),
                panel=t.get("panel"),
                warning=t.get("warning"),
                error=t.get("error"),
                success=t.get("success"),
                dark=t.get("dark", True),
            )
            self.design = color_sys
            self.current_theme_name = theme_name

    def action_next_theme(self) -> None:
        theme_names = list(THEMES.keys())
        idx = (theme_names.index(self.current_theme_name) + 1) % len(theme_names)
        new_theme = theme_names[idx]
        self.apply_theme(new_theme)
        self.write(f"[bold $accent]Switched Theme:[/bold $accent] [italic]{new_theme.upper()}[/italic]")

    def compose(self) -> ComposeResult:
        yield Header()
        with Horizontal(id="main-grid"):
            with VerticalScroll(id="form-column"):
                with Vertical(classes="group-box"):
                    yield Label("Workflow Source Target", classes="group-title")
                    yield Select(
                        [
                            ("Automatic Context (Swinsian/Finder/Clipboard)", "context"),
                            ("Direct Path (Audio file or Album folder)", "path"),
                            ("Swinsian Selection / Playing Track", "swinsian"),
                            ("Finder Selection", "finder"),
                            ("Browse Folder (Native Chooser)...", "choose"),
                            ("Clipboard Path Text", "clipboard"),
                        ],
                        value="context",
                        id="source",
                    )
                    yield Label("Target Audio/Directory Path", classes="field-label")
                    yield Input(placeholder="/Volumes/Audio/Artist/Album", id="path")

                with Vertical(classes="group-box"):
                    yield Label("Action & Output Mode", classes="group-title")
                    yield Select(
                        [
                            ("Save Sidecar Cover Beside Album", "save"),
                            ("Save Cover & Embed into All Album Tracks", "embed"),
                        ],
                        value="save",
                        id="mode",
                    )

                with Vertical(classes="group-box"):
                    yield Label("Optional Search Overrides", classes="group-title")
                    yield Input(placeholder="Artist Name", id="artist")
                    yield Input(placeholder="Album Title", id="album")
                    yield Input(placeholder="Barcode / Catalogue Number", id="identifier")
                    yield Input(placeholder="Preferred Resolution (e.g. 1500)", id="resolution")
                    yield Input(placeholder="COV Source IDs (comma-separated)", id="sources")

                with Vertical(classes="group-box"):
                    yield Label("UI Color Theme", classes="group-title")
                    yield Select(
                        [
                            ("Cobalt (Default Dark Slate/Cyan)", "cobalt"),
                            ("Nord (Muted Arctic Blue)", "nord"),
                            ("Monokai (Vibrant Pink/Green)", "monokai"),
                            ("Emerald (Deep Ocean Green)", "emerald"),
                            ("Dracula (Purple/Pink Gothic)", "dracula"),
                        ],
                        value="cobalt",
                        id="theme-select",
                    )

                with Horizontal(id="actions-box"):
                    yield Button("Launch COV", id="launch", variant="primary")
                    yield Button("Doctor", id="doctor")
                    yield Button("Show Log", id="show-log")
                    yield Button("Quit", id="quit", variant="error")

            with Vertical(id="log-column"):
                yield Label("Output & Diagnostic Console", classes="group-title")
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
                raise ValueError("Please enter an audio file or album directory path.")
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
            self.write(f"[bold $error]Error:[/bold $error] {error}")
            return
        self.write(f"[bold $primary]Executing:[/bold $primary] [dim]{' '.join(command)}[/dim]")
        completed = subprocess.run(command, capture_output=True, text=True, check=False)
        output = (completed.stdout + completed.stderr).strip()
        self.write(output or f"[bold $success]Command executed cleanly (status code {completed.returncode}).[/bold $success]")

    def run_doctor(self) -> None:
        self.write("[bold $primary]Running Environment Doctor...[/bold $primary]")
        completed = subprocess.run([str(BIN / "cov-doctor")], capture_output=True, text=True, check=False)
        self.write(completed.stdout.strip())

    def show_log(self) -> None:
        self.write("[bold $primary]Fetching Live Integration Logs...[/bold $primary]")
        completed = subprocess.run([str(BIN / "cov-log")], capture_output=True, text=True, check=False)
        self.write(completed.stdout.strip())

    def on_select_changed(self, event: Select.Changed) -> None:
        if event.select.id == "theme-select" and event.value != Select.BLANK:
            self.apply_theme(str(event.value))

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
