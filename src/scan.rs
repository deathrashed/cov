use crate::paths::LAUNCH_EXTS;
use anyhow::{Context, Result};
use lofty::file::TaggedFileExt;
use lofty::picture::PictureType;
use lofty::probe::Probe;
use std::collections::HashSet;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

const SIDECAR_NAMES: &[&str] = &["cover.jpg", "cover.jpeg", "cover.png"];

pub fn missing_sidecar(root: &Path, mut emit: impl FnMut(&Path) -> Result<()>) -> Result<()> {
    validate_root(root)?;
    let mut emitted = HashSet::new();

    for entry in visible_entries(root) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if !entry.file_type().is_file() || !is_audio(path) {
            continue;
        }
        let Some(parent) = path.parent() else {
            continue;
        };
        if emitted.contains(parent) || has_sidecar(parent) {
            continue;
        }
        emit(parent)?;
        emitted.insert(parent.to_path_buf());
    }

    Ok(())
}

pub fn missing_embedded(root: &Path, mut emit: impl FnMut(&Path) -> Result<()>) -> Result<()> {
    validate_root(root)?;

    for entry in visible_entries(root) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if entry.file_type().is_file() && is_audio(path) && !has_embedded_cover(path) {
            emit(path)?;
        }
    }

    Ok(())
}

fn validate_root(root: &Path) -> Result<()> {
    let metadata = std::fs::metadata(root)
        .with_context(|| format!("Failed to read library root: {}", root.display()))?;
    if !metadata.is_dir() {
        anyhow::bail!("Library root is not a directory: {}", root.display());
    }
    Ok(())
}

fn visible_entries(root: &Path) -> impl Iterator<Item = walkdir::Result<DirEntry>> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| entry.depth() == 0 || !is_hidden(entry.path()))
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            LAUNCH_EXTS
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
}

fn has_sidecar(dir: &Path) -> bool {
    SIDECAR_NAMES.iter().any(|name| dir.join(name).is_file())
}

fn has_embedded_cover(path: &Path) -> bool {
    let tagged_file = match Probe::open(path).and_then(|probe| probe.read()) {
        Ok(tagged_file) => tagged_file,
        Err(_) => return false,
    };

    tagged_file.tags().iter().any(|tag| {
        tag.pictures().iter().any(|picture| {
            matches!(
                picture.pic_type(),
                PictureType::CoverFront | PictureType::Other
            )
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn copy_fixture(name: &str, destination: &Path) {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name);
        fs::copy(source, destination).unwrap();
    }

    #[test]
    fn missing_sidecar_reports_only_visible_audio_directories_without_cover() {
        let library = tempdir().unwrap();
        let missing = library.path().join("Artist").join("Missing");
        let covered = library.path().join("Artist").join("Covered");
        let hidden = library.path().join(".Hidden").join("Album");
        fs::create_dir_all(&missing).unwrap();
        fs::create_dir_all(&covered).unwrap();
        fs::create_dir_all(&hidden).unwrap();
        fs::write(missing.join("01.mp3"), b"audio").unwrap();
        fs::write(missing.join("notes.txt"), b"text").unwrap();
        fs::write(covered.join("01.flac"), b"audio").unwrap();
        fs::write(covered.join("cover.jpeg"), b"image").unwrap();
        fs::write(hidden.join("01.mp3"), b"audio").unwrap();

        let mut found = Vec::new();
        missing_sidecar(library.path(), |path| {
            found.push(path.to_path_buf());
            Ok(())
        })
        .unwrap();

        assert_eq!(found, vec![missing]);
    }

    #[test]
    fn missing_embedded_reports_only_visible_tracks_without_front_or_other_cover() {
        let library = tempdir().unwrap();
        let album = library.path().join("Artist").join("Album");
        fs::create_dir_all(&album).unwrap();
        let missing = album.join("01 missing.mp3");
        let embedded = album.join("02 embedded.mp3");
        let hidden = album.join(".03 hidden.mp3");
        copy_fixture("fixture.mp3", &missing);
        copy_fixture("fixture.mp3", &embedded);
        copy_fixture("fixture.mp3", &hidden);
        let artwork = fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("fixtures")
                .join("artwork.jpg"),
        )
        .unwrap();
        crate::embed::embed_file(&embedded, &artwork, "image/jpeg").unwrap();

        let mut found = Vec::new();
        missing_embedded(library.path(), |path| {
            found.push(path.to_path_buf());
            Ok(())
        })
        .unwrap();

        assert_eq!(found, vec![missing]);
    }

    #[test]
    fn scans_reject_a_root_that_is_not_a_directory() {
        let library = tempdir().unwrap();
        let file = library.path().join("track.mp3");
        fs::write(&file, b"audio").unwrap();

        let error = missing_sidecar(&file, |_| Ok(())).unwrap_err();

        assert!(error.to_string().contains("not a directory"));
    }
}
