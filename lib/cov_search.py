#!/usr/bin/env python3
"""Interactive fzf and fuzzy search utility for COV Artwork Toolkit."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "bin"
AUDIO_EXTENSIONS = {".mp3", ".flac", ".m4a", ".wav", ".aiff", ".opus", ".ogg", ".dsf", ".ape", ".wv"}


def get_search_directories(user_dir: str | None = None) -> list[Path]:
    dirs = []
    if user_dir:
        p = Path(user_dir).expanduser().resolve()
        if p.exists():
            dirs.append(p)
    
    # Common default music locations if exists
    for candidate in [Path.home() / "Music", Path("/Volumes"), Path.cwd(), Path.home()]:
        if candidate.exists() and candidate not in dirs:
            dirs.append(candidate)
    return dirs


def scan_audio_items(search_dir: Path, max_items: int = 2000) -> list[str]:
    """Scan search_dir for audio files and directories using fd or os.walk."""
    fd_bin = shutil.which("fd")
    if fd_bin:
        cmd = [fd_bin, "--hidden", "--exclude", ".git", "--exclude", "Library"]
        for ext in AUDIO_EXTENSIONS:
            cmd.extend(["-e", ext.lstrip(".")])
        cmd.extend(["-t", "f", "-t", "d", ".", str(search_dir)])
        try:
            proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
            results = [line.strip() for line in proc.stdout.splitlines() if line.strip()]
            if results:
                return results[:max_items]
        except Exception:
            pass

    # Fast python fallback
    items = []
    try:
        for root, dirs, files in os.walk(search_dir):
            if ".git" in root or "Library" in root:
                continue
            for file in files:
                if Path(file).suffix.lower() in AUDIO_EXTENSIONS:
                    items.append(os.path.join(root, file))
                    if len(items) >= max_items:
                        return items
            for d in dirs:
                if not d.startswith("."):
                    items.append(os.path.join(root, d))
    except Exception:
        pass
    return items


def fuzzy_match_items(query: str, items: list[str], limit: int = 50) -> list[str]:
    """Fuzzily filter items matching all space-separated tokens in query."""
    if not query.strip():
        return items[:limit]

    tokens = [t.lower() for t in query.strip().split()]
    matches = []

    # Check direct path exact match
    p = Path(query).expanduser().resolve()
    if p.exists():
        matches.append(str(p))

    for item in items:
        item_lower = item.lower()
        if all(token in item_lower for token in tokens):
            if str(item) not in matches:
                matches.append(item)
                if len(matches) >= limit:
                    break
    return matches


def run_fzf(items: list[str], prompt: str = "Audio Library > ") -> tuple[str, str]:
    """Run fzf interactively, returning (action_key, selected_path)."""
    fzf_bin = shutil.which("fzf")
    if not fzf_bin:
        raise RuntimeError("fzf binary not found. Please install fzf via homebrew: brew install fzf")

    # Construct fzf command with custom keybindings and Riley theme colors
    fzf_cmd = [
        fzf_bin,
        "--prompt", prompt,
        "--header", "Press ENTER: cov-open | CTRL-E: cov-open-embed | ESC: Exit",
        "--expect", "enter,ctrl-e",
        "--color", "bg:#1E1E1E,bg+:#2A2A2A,fg:#D4D4D4,fg+:#FFFFFF,hl:#B96CDB,hl+:#00E8C6,prompt:#B96CDB,pointer:#00E8C6",
        "--height", "80%",
        "--layout", "reverse",
        "--border", "rounded",
    ]

    proc = subprocess.Popen(
        fzf_cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=None,
        text=True,
    )

    input_data = "\n".join(items)
    stdout, _ = proc.communicate(input=input_data)
    lines = stdout.splitlines()
    if not lines or len(lines) < 2:
        return "", ""

    key = lines[0]
    selection = lines[1]
    return key, selection


def main() -> None:
    parser = argparse.ArgumentParser(description="Fuzzy search audio files and albums for COV Toolkit.")
    parser.add_argument("directory", nargs="?", help="Directory to search (defaults to ~/Music or current dir)")
    parser.add_argument("--embed", action="store_true", help="Launch cov-open-embed directly on selection")
    args = parser.parse_args()

    search_dirs = get_search_directories(args.directory)
    if not search_dirs:
        print("Error: No search directory found.", file=sys.stderr)
        sys.exit(1)

    search_target = search_dirs[0]
    print(f"Scanning audio library in: {search_target} ...", file=sys.stderr)
    items = scan_audio_items(search_target)

    if not items:
        print(f"No audio files or directories found in {search_target}.", file=sys.stderr)
        sys.exit(1)

    key, selection = run_fzf(items)
    if not selection:
        print("Cancelled.", file=sys.stderr)
        sys.exit(0)

    print(f"Selected: {selection}")
    if args.embed or key == "ctrl-e":
        cmd = [str(BIN / "cov-open-embed"), selection]
    else:
        cmd = [str(BIN / "cov-open"), selection]

    print(f"Launching: {' '.join(cmd)}")
    subprocess.run(cmd)


if __name__ == "__main__":
    main()
