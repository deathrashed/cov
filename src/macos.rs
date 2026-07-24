use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Run an AppleScript with positional arguments and return stdout.
pub fn osascript(script: &str, args: &[String]) -> Result<String> {
    let mut cmd = Command::new("/usr/bin/osascript");
    cmd.arg("-e").arg(script).arg("--");
    for a in args {
        cmd.arg(a);
    }
    let out = cmd.output().context("Failed to run osascript")?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        anyhow::bail!("osascript error: {}", stderr);
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Generic AppleScript runner with no arguments.
pub fn osascript_simple(script: &str) -> Result<String> {
    let args: Vec<String> = vec![];
    osascript(script, &args)
}

/// AppleScript to get the frontmost application name.
pub const FRONTMOST_SCRIPT: &str = r#"tell application "System Events"
    set frontApp to name of first application process whose frontmost is true
    return frontApp
end tell"#;

/// AppleScript to get the selected or currently playing Swinsian track path.
pub const SWINSIAN_TRACK_SCRIPT: &str = r#"tell application "Swinsian"
    if not running then error "Swinsian is not running."
    set chosenTrack to missing value
    try
        set selectedTracks to selection of front window
        if selectedTracks is not {} then set chosenTrack to item 1 of selectedTracks
    end try
    if chosenTrack is missing value then
        try
            set chosenTrack to current track
        end try
    end if
    if chosenTrack is missing value then error "Select or play a track in Swinsian first."
    return path of chosenTrack as text
end tell"#;

/// AppleScript to get the frontmost Finder selection.
pub const FINDER_SELECTION_SCRIPT: &str = r#"tell application "Finder"
    set selectedItems to selection
    if selectedItems is {} then error "Select an audio file or album folder in Finder first."
    return POSIX path of (item 1 of selectedItems as alias)
end tell"#;

/// AppleScript to prompt for a folder with a native dialog.
pub const CHOOSE_FOLDER_SCRIPT: &str = r#"set chosenFolder to choose folder with prompt "Choose an album folder for COV"
return POSIX path of chosenFolder"#;

/// AppleScript to rescan Swinsian for the given file paths.
pub const SWINSIAN_RESCAN_SCRIPT: &str = r#"on run argv
    tell application "Swinsian"
        if not running then return
        repeat with p in argv
            try
                rescan p
            end try
        end repeat
    end tell
end run"#;

/// Get the name of the frontmost application.
pub fn frontmost_app() -> Result<String> {
    osascript_simple(FRONTMOST_SCRIPT)
}

/// Get the path of the selected or currently playing Swinsian track.
pub fn swinsian_track_path() -> Result<String> {
    osascript_simple(SWINSIAN_TRACK_SCRIPT)
}

/// Get the POSIX path of the currently selected Finder item.
pub fn finder_selection() -> Result<String> {
    osascript_simple(FINDER_SELECTION_SCRIPT)
}

/// Prompt the user with a native macOS folder chooser and return the selected path.
pub fn choose_folder() -> Result<String> {
    osascript_simple(CHOOSE_FOLDER_SCRIPT)
}

/// Read the current clipboard text via `pbpaste`.
pub fn pbpaste() -> Result<String> {
    let out = Command::new("/usr/bin/pbpaste")
        .output()
        .context("Failed to run pbpaste")?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Send rescan commands to Swinsian for the given file paths.
pub fn rescan_swinsian(files: &[PathBuf]) -> Result<()> {
    let args: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    osascript(SWINSIAN_RESCAN_SCRIPT, &args)?;
    Ok(())
}
