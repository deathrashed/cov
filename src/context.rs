use crate::macos;
use crate::paths::expand_tilde;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// The source of a detected audio path.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextSource {
    Swinsian,
    Finder,
    FolderPicker,
    Clipboard(PathBuf),
}

/// Route from a frontmost app name and clipboard content to the best context source.
pub fn route(frontmost: &str, clipboard: &str) -> Option<ContextSource> {
    match frontmost {
        "Swinsian" => return Some(ContextSource::Swinsian),
        "Finder" => return Some(ContextSource::Finder),
        _ => {}
    }
    let trimmed = clipboard.trim();
    if !trimmed.is_empty() {
        let expanded = expand_tilde(trimmed);
        if expanded.exists() {
            return Some(ContextSource::Clipboard(expanded));
        }
    }
    None
}

/// Auto-detect a source path using frontmost app, then Finder, then clipboard fallback.
pub fn detect() -> Result<(ContextSource, PathBuf)> {
    // Try frontmost app first
    if let Ok(app) = macos::frontmost_app() {
        match app.as_str() {
            "Swinsian" => {
                let p =
                    macos::swinsian_track_path().context("Failed to get Swinsian track path")?;
                return Ok((ContextSource::Swinsian, PathBuf::from(p)));
            }
            "Finder" => {
                let p = macos::finder_selection().context("Failed to get Finder selection")?;
                return Ok((ContextSource::Finder, PathBuf::from(p)));
            }
            _ => {}
        }
    }

    // Try clipboard
    if let Ok(clip) = macos::pbpaste() {
        let trimmed = clip.trim();
        if !trimmed.is_empty() {
            let expanded = expand_tilde(trimmed);
            if expanded.exists() {
                let path = expanded.clone();
                return Ok((ContextSource::Clipboard(expanded), path));
            }
        }
    }

    anyhow::bail!("No usable context found. Select a track in Swinsian or Finder, or copy a path.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn with_file() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.mp3");
        std::fs::write(&file, b"dummy").unwrap();
        (dir, file)
    }

    #[test]
    fn test_route_swinsian() {
        assert_eq!(route("Swinsian", ""), Some(ContextSource::Swinsian));
    }

    #[test]
    fn test_route_finder() {
        assert_eq!(route("Finder", ""), Some(ContextSource::Finder));
    }

    #[test]
    fn test_route_clipboard() {
        let (_dir, file) = with_file();
        let clip = file.to_string_lossy().to_string();
        let result = route("Terminal", &clip);
        assert_eq!(result, Some(ContextSource::Clipboard(file)));
    }

    #[test]
    fn test_route_empty_clipboard_with_unknown_app() {
        assert_eq!(route("Terminal", ""), None);
    }

    #[test]
    fn test_route_nonexistent_clipboard_path() {
        assert_eq!(route("Terminal", "/nonexistent/path"), None);
    }
}
