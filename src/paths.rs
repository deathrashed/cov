use anyhow::{Context, Result};
use directories::UserDirs;
use std::fs;
use std::path::{Path, PathBuf};

pub const LAUNCH_EXTS: &[&str] = &[
    "aiff", "aif", "ape", "dsf", "flac", "m4a", "mp3", "mp4", "ogg", "opus", "tak", "wav", "wv",
];

pub const EMBED_EXTS: &[&str] = &[
    "aiff", "aif", "ape", "dsf", "flac", "m4a", "mp3", "mp4", "ogg", "opus", "wav", "wv",
];

pub fn expand_tilde(path: &str) -> PathBuf {
    let p = Path::new(path);
    if let Ok(stripped) = p.strip_prefix("~")
        && let Some(user_dirs) = UserDirs::new()
    {
        return user_dirs.home_dir().join(stripped);
    }
    p.to_path_buf()
}

fn has_valid_extension(path: &Path, exts: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .map(|e| exts.contains(&e.as_str()))
        .unwrap_or(false)
}

fn is_hidden_mac_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with("._"))
        .unwrap_or(false)
}

pub fn resolve_audio_path(raw_path: &str) -> Result<PathBuf> {
    let expanded = expand_tilde(raw_path);
    let canonical =
        fs::canonicalize(&expanded).context("Failed to canonicalize path. Does it exist?")?;

    let metadata = fs::metadata(&canonical).context("Failed to read metadata")?;

    if metadata.is_file() {
        if !has_valid_extension(&canonical, LAUNCH_EXTS) {
            anyhow::bail!("Unsupported file extension");
        }
        if is_hidden_mac_file(&canonical) {
            anyhow::bail!("Hidden mac file not supported");
        }
        return Ok(canonical);
    }

    if metadata.is_dir() {
        let mut audio_files = Vec::new();
        for entry in fs::read_dir(&canonical).context("Failed to read directory")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && has_valid_extension(&path, LAUNCH_EXTS)
                && !is_hidden_mac_file(&path)
            {
                audio_files.push(path);
            }
        }
        if audio_files.is_empty() {
            anyhow::bail!("No matching audio files found in directory");
        }
        audio_files.sort();
        return Ok(audio_files.into_iter().next().unwrap());
    }

    anyhow::bail!("Path is neither a file nor a directory");
}

pub fn target_files(raw_target: &str) -> Vec<PathBuf> {
    let expanded = expand_tilde(raw_target);
    let canonical = match fs::canonicalize(&expanded) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let metadata = match fs::metadata(&canonical) {
        Ok(m) => m,
        Err(_) => return vec![],
    };

    if metadata.is_file() {
        if has_valid_extension(&canonical, EMBED_EXTS) && !is_hidden_mac_file(&canonical) {
            return vec![canonical];
        } else {
            return vec![];
        }
    }

    if metadata.is_dir() {
        let mut audio_files = Vec::new();
        if let Ok(entries) = fs::read_dir(&canonical) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && has_valid_extension(&path, EMBED_EXTS)
                    && !is_hidden_mac_file(&path)
                {
                    audio_files.push(path);
                }
            }
        }
        audio_files.sort();
        return audio_files;
    }

    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_valid_single_audio_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.mp3");
        File::create(&file_path).unwrap();

        let resolved = resolve_audio_path(file_path.to_str().unwrap()).unwrap();
        assert_eq!(resolved, fs::canonicalize(&file_path).unwrap());
    }

    #[test]
    fn test_directory_resolution() {
        let dir = tempdir().unwrap();
        let f1 = dir.path().join("b.flac");
        let f2 = dir.path().join("a.mp3");
        let hidden = dir.path().join("._c.wav");
        let unsupported = dir.path().join("d.txt");

        File::create(&f1).unwrap();
        File::create(&f2).unwrap();
        File::create(&hidden).unwrap();
        File::create(&unsupported).unwrap();

        let resolved = resolve_audio_path(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(resolved, fs::canonicalize(&f2).unwrap()); // a.mp3 comes first
    }

    #[test]
    fn test_non_existent_path() {
        let res = resolve_audio_path("/path/that/does/not/exist.mp3");
        assert!(res.is_err());
    }

    #[test]
    fn test_unsupported_extension() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let res = resolve_audio_path(file_path.to_str().unwrap());
        assert!(res.is_err());
    }

    #[test]
    fn test_target_files_single_and_dir() {
        let dir = tempdir().unwrap();
        let f1 = dir.path().join("b.flac");
        let f2 = dir.path().join("a.mp3");
        let hidden = dir.path().join("._c.wav");
        let tak = dir.path().join("c.tak"); // not in EMBED_EXTS

        File::create(&f1).unwrap();
        File::create(&f2).unwrap();
        File::create(&hidden).unwrap();
        File::create(&tak).unwrap();

        let targets = target_files(dir.path().to_str().unwrap());
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0], fs::canonicalize(&f2).unwrap());
        assert_eq!(targets[1], fs::canonicalize(&f1).unwrap());

        let single = target_files(f1.to_str().unwrap());
        assert_eq!(single.len(), 1);
        assert_eq!(single[0], fs::canonicalize(&f1).unwrap());
    }
}
