#!/usr/bin/env python3
"""Ghostty-friendly terminal interface for the Riley COV Toolkit."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

from textual.app import App, ComposeResult
from textual.binding import Binding
from textual.command import DiscoveryHit, Hit, Hits, Provider
from textual.containers import Horizontal, Vertical, VerticalScroll
from textual.design import ColorSystem
from textual.screen import ModalScreen
from textual.widgets import Button, DirectoryTree, Footer, Header, Input, Label, RichLog, Select


ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "bin"


# Built-in themes matching ~/.config/_riley/theme/riley.theme.json
THEMES: dict[str, dict[str, str]] = {
    "riley": {
        "primary": "#B96CDB",
        "secondary": "#C74DED",
        "accent": "#00E8C6",
        "background": "#1E1E1E",
        "surface": "#222222",
        "panel": "#2A2A2A",
        "warning": "#F39C12",
        "error": "#EE5D43",
        "success": "#96E072",
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
}


class PathFuzzyFinderModal(ModalScreen[str]):
    """Fuzzy directory & audio file picker modal popup with live text input."""

    BINDINGS = [
        Binding("escape", "cancel", "Cancel Modal", key_display="esc"),
    ]

    CSS = """
    PathFuzzyFinderModal {
        background: black 65%;
        align: center middle;
    }

    #fuzzy-picker-dialog {
        background: $surface;
        border: round $primary;
        border-title-color: $accent;
        border-title-style: bold;
        width: 75%;
        height: 75%;
        padding: 1 2;
    }

    #fuzzy-title {
        color: $primary;
        text-style: bold;
        margin-bottom: 1;
    }

    #fuzzy-filter-input {
        background: $background;
        border: tall $primary 40%;
        color: $text;
        height: 3;
        padding: 0 1;
        margin-bottom: 1;
    }

    #fuzzy-filter-input:focus {
        border: tall $accent;
    }

    #fuzzy-tree {
        height: 1fr;
        background: $background;
        border: tall $primary 40%;
        margin-bottom: 1;
    }

    #fuzzy-actions {
        height: auto;
    }

    Button {
        min-width: 12;
        height: 1;
        border: none;
        margin-right: 1;
    }

    #fuzzy-select-btn {
        background: $primary;
        color: $background;
        text-style: bold;
    }

    #fuzzy-cancel-btn {
        background: $error;
        color: #ffffff;
    }
    """

    def __init__(self, initial_path: str = "~") -> None:
        super().__init__()
        resolved = Path(initial_path).expanduser().resolve()
        self.root_path = resolved if resolved.exists() else Path.home()

    def compose(self) -> ComposeResult:
        with Vertical(id="fuzzy-picker-dialog"):
            yield Label("Fuzzy Path Finder (Select Audio File or Album Directory)", id="fuzzy-title")
            yield Input(placeholder="Type path or filter directory...", value=str(self.root_path), id="fuzzy-filter-input")
            yield DirectoryTree(str(self.root_path), id="fuzzy-tree")
            with Horizontal(id="fuzzy-actions"):
                yield Button("Select Path", id="fuzzy-select-btn", variant="primary")
                yield Button("Cancel", id="fuzzy-cancel-btn", variant="error")

    def on_input_changed(self, event: Input.Changed) -> None:
        if event.input.id == "fuzzy-filter-input":
            val = event.value.strip()
            if val:
                p = Path(val).expanduser().resolve()
                if p.exists() and p.is_dir():
                    tree = self.query_one("#fuzzy-tree", DirectoryTree)
                    tree.path = p

    def on_directory_tree_file_selected(self, event: DirectoryTree.FileSelected) -> None:
        event.stop()
        self.dismiss(str(event.path))

    def on_directory_tree_directory_selected(self, event: DirectoryTree.DirectorySelected) -> None:
        event.stop()
        input_widget = self.query_one("#fuzzy-filter-input", Input)
        input_widget.value = str(event.path)

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "fuzzy-select-btn":
            val = self.query_one("#fuzzy-filter-input", Input).value.strip()
            if val:
                self.dismiss(str(Path(val).expanduser().resolve()))
            else:
                tree = self.query_one("#fuzzy-tree", DirectoryTree)
                if tree.cursor_node and tree.cursor_node.data:
                    self.dismiss(str(tree.cursor_node.data.path))
                else:
                    self.dismiss(str(self.root_path))
        elif event.button.id == "fuzzy-cancel-btn":
            self.dismiss("")

    def action_cancel(self) -> None:
        self.dismiss("")


class CovCommandProvider(Provider):
    """Custom command provider exposing all actions, options, themes, and settings."""

    async def discover(self) -> Hits:
        app = self.app
        assert isinstance(app, CovToolkitApp)

        # Core execution actions
        yield DiscoveryHit(
            "Launch COV Search",
            app.launch_cov,
            help="Start COVIT process and launch browser cover search",
        )
        yield DiscoveryHit(
            "Run Doctor Diagnostics",
            app.run_doctor,
            help="Verify dependencies, binaries, and local path tools",
        )
        yield DiscoveryHit(
            "Open Fuzzy Path Finder Modal",
            app.action_open_fuzzy_finder,
            help="Browse and select an audio file or album directory",
        )
        yield DiscoveryHit(
            "Show Live Log Output",
            app.show_log,
            help="Fetch live output from ~/Library/Logs/cov-toolkit.log",
        )
        yield DiscoveryHit(
            "Clear Console Log",
            app.action_clear_log,
            help="Clear the output log panel view",
        )

        # Workflow Target Options
        sources = [
            ("Automatic Context (Swinsian/Finder/Clipboard)", "context"),
            ("Direct Path (Audio file or Album folder)", "path"),
            ("Swinsian Selection / Playing Track", "swinsian"),
            ("Finder Selection", "finder"),
            ("Browse Folder (Native Chooser)...", "choose"),
            ("Clipboard Path Text", "clipboard"),
        ]
        for name, value in sources:
            yield DiscoveryHit(
                f"Source Target: {name}",
                lambda val=value: app.set_source_target(val),
                help=f"Set workflow source target to {name}",
            )

        # Action Modes
        modes = [
            ("Save Sidecar Cover Beside Album", "save"),
            ("Save Cover & Embed into All Album Tracks", "embed"),
        ]
        for name, value in modes:
            yield DiscoveryHit(
                f"Action Mode: {name}",
                lambda val=value: app.set_action_mode(val),
                help=f"Set artwork action mode to {name}",
            )

        # Color Themes
        for theme_name in THEMES.keys():
            yield DiscoveryHit(
                f"Theme: Switch to {theme_name.upper()}",
                lambda t=theme_name: app.apply_theme(t),
                help=f"Change UI color palette to {theme_name.upper()}",
            )

        # Application control
        yield DiscoveryHit(
            "Quit Application",
            app.action_quit,
            help="Close the COV Artwork Toolkit interface",
        )

    async def search(self, query: str) -> Hits:
        matcher = self.matcher(query)
        async for hit in self.discover():
            match_score = matcher.match(hit.text)
            if match_score > 0:
                hit.score = match_score
                yield hit


class CovToolkitApp(App[None]):
    TITLE = "COV Artwork Toolkit"
    SUB_TITLE = "Search, save, and embed high-res album artwork"
    COMMANDS = {CovCommandProvider}

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

    #right-column {
        width: 1fr;
        height: 100%;
    }

    #overrides-panel {
        height: auto;
        max-height: 50%;
        background: $surface;
        border: round $primary 50%;
        border-title-color: $primary;
        border-title-style: bold;
        padding: 0 1;
        margin-bottom: 1;
    }

    #log-panel {
        height: 1fr;
        background: $surface;
        border: round $primary 50%;
        border-title-color: $accent;
        border-title-style: bold;
        padding: 0 1;
    }

    .group-box {
        background: $panel 30%;
        border: round $primary 30%;
        padding: 0 1;
        margin-top: 0;
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
        padding: 0;
    }

    Button {
        margin-right: 1;
        min-width: 10;
        height: 1;
        border: none;
        padding: 0 1;
    }

    #launch {
        background: $primary;
        color: $background;
        text-style: bold;
    }

    #launch:hover {
        background: $primary-lighten-1;
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

    /* Centered Command Palette Popup Overlays */
    CommandPalette {
        background: black 65%;
        align: center middle;
    }

    CommandPalette > Vertical {
        background: $surface;
        border: round $primary;
        width: 65%;
        height: 65%;
    }
    """

    BINDINGS = [
        Binding("q", "quit", "Quit", show=True),
        Binding("d", "doctor", "Doctor Diagnostics", show=True),
        Binding("l", "launch", "Launch COV", show=True),
        Binding("p", "open_fuzzy_finder", "Fuzzy Finder", show=True),
        Binding("c", "clear_log", "Clear Log", show=True),
        Binding("t", "next_theme", "Theme", show=True),
        Binding("ctrl+p", "command_palette", "Command Palette", show=True),
    ]

    def __init__(self) -> None:
        super().__init__()
        self.current_theme_name = "riley"

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

    def action_open_fuzzy_finder(self) -> None:
        current_path = self.query_one("#path", Input).value.strip() or "~"
        def on_path_selected(selected_path: str) -> None:
            if selected_path:
                self.query_one("#source", Select).value = "path"
                self.query_one("#path", Input).value = selected_path
                self.write(f"[bold $accent]Selected Path via Fuzzy Finder:[/bold $accent] [italic]{selected_path}[/italic]")

        self.push_screen(PathFuzzyFinderModal(current_path), on_path_selected)

    def set_source_target(self, value: str) -> None:
        self.query_one("#source", Select).value = value
        self.write(f"[bold $primary]Source Target Set:[/bold $primary] {value}")

    def set_action_mode(self, value: str) -> None:
        self.query_one("#mode", Select).value = value
        self.write(f"[bold $primary]Action Mode Set:[/bold $primary] {value}")

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

                with Vertical(classes="group-box"):
                    yield Label("Fuzzy Path Finder & Target Path", classes="group-title")
                    yield Label("Direct Path Selection", classes="field-label")
                    yield Input(placeholder="/Volumes/Audio/Artist/Album (or press 'p' for Fuzzy Finder)", id="path")
                    yield Button("Open Fuzzy Path Finder", id="fuzzy-btn", variant="primary")

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

                with Horizontal(id="actions-box"):
                    yield Button("Launch COV", id="launch", variant="primary")
                    yield Button("Show Log", id="show-log")
                    yield Button("Quit", id="quit", variant="error")

            with Vertical(id="right-column"):
                with VerticalScroll(id="overrides-panel"):
                    yield Label("Optional Search Overrides", classes="group-title")
                    yield Input(placeholder="Artist Name", id="artist")
                    yield Input(placeholder="Album Title", id="album")
                    yield Input(placeholder="Barcode / Catalogue Number", id="identifier")
                    yield Input(placeholder="Preferred Resolution (e.g. 1500)", id="resolution")
                    yield Input(placeholder="COV Source IDs (comma-separated)", id="sources")

                with Vertical(id="log-panel"):
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

    def on_button_pressed(self, event: Button.Pressed) -> None:
        match event.button.id:
            case "launch":
                self.launch_cov()
            case "fuzzy-btn":
                self.action_open_fuzzy_finder()
            case "show-log":
                self.show_log()
            case "quit":
                self.action_quit()

    def action_launch(self) -> None:
        self.launch_cov()

    def action_doctor(self) -> None:
        self.run_doctor()

    def action_quit(self) -> None:
        self.exit()


def main() -> None:
    CovToolkitApp().run()


if __name__ == "__main__":
    main()
