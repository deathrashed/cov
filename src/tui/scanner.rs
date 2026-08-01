use crate::paths::LAUNCH_EXTS;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::JoinHandle;

/// A single album entry discovered by the scanner.
#[derive(Debug, Clone)]
pub struct Album {
    pub dir: PathBuf,
    /// Path relative to library root — the match text for fuzzy search.
    pub rel: String,
    /// Display name: last two path components joined by " · ".
    pub display: String,
    /// Track files sorted by full path, filtered to LAUNCH_EXTS.
    pub tracks: Vec<PathBuf>,
}

/// Messages the scanner sends to the UI.
#[derive(Debug, Clone)]
pub enum ScanMsg {
    Batch { epoch: u64, albums: Vec<Album> },
    Done { epoch: u64, total: usize },
}

const BATCH_SIZE: usize = 32;

/// Spawn a background thread that recursively walks `root` and emits batches of albums.
pub fn spawn_scan(
    root: PathBuf,
    epoch: u64,
    cancel: Arc<AtomicU64>,
    tx: crossbeam::channel::Sender<ScanMsg>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut batch = Vec::with_capacity(BATCH_SIZE);
        let mut current_album: Option<Album> = None;
        let mut total = 0usize;

        for entry in walkdir::WalkDir::new(&root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Skip hidden directories
                e.depth() == 0
                    || e.file_name()
                        .to_str()
                        .map(|s| !s.starts_with('.') || s == ".")
                        .unwrap_or(false)
            })
            .filter_map(|e| e.ok())
        {
            // Check cancellation
            if cancel.load(Ordering::Relaxed) != epoch {
                return;
            }

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Check extension
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let ext = ext.to_lowercase();
            if !LAUNCH_EXTS.contains(&ext.as_str()) {
                continue;
            }

            // Skip AppleDouble
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("._"))
                .unwrap_or(false)
            {
                continue;
            }

            let parent = match path.parent() {
                Some(p) => p.to_path_buf(),
                None => continue,
            };

            match current_album.as_mut() {
                Some(album) if album.dir == parent => {
                    album.tracks.push(path.to_path_buf());
                }
                Some(_) => {
                    if let Some(mut album) = current_album.take() {
                        album.tracks.sort();
                        batch.push(album);
                        total += 1;
                        if batch.len() == BATCH_SIZE
                            && tx
                                .send(ScanMsg::Batch {
                                    epoch,
                                    albums: std::mem::take(&mut batch),
                                })
                                .is_err()
                        {
                            return;
                        }
                    }
                    current_album = Some(new_album(parent, &root, path.to_path_buf()));
                }
                None => {
                    current_album = Some(new_album(parent, &root, path.to_path_buf()));
                }
            }
        }

        if let Some(mut album) = current_album {
            album.tracks.sort();
            batch.push(album);
            total += 1;
        }

        if !batch.is_empty()
            && tx
                .send(ScanMsg::Batch {
                    epoch,
                    albums: batch,
                })
                .is_err()
        {
            return;
        }
        let _ = tx.send(ScanMsg::Done { epoch, total });
    })
}

fn new_album(dir: PathBuf, root: &std::path::Path, track: PathBuf) -> Album {
    let rel = dir
        .strip_prefix(root)
        .unwrap_or(&dir)
        .to_string_lossy()
        .to_string();
    let display = display_name(&dir, root);
    Album {
        dir,
        rel,
        display,
        tracks: vec![track],
    }
}

fn display_name(path: &std::path::Path, root: &std::path::Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let components: Vec<_> = rel.components().collect();
    let parts: Vec<String> = components
        .iter()
        .rev()
        .take(2)
        .rev()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();
    if parts.len() >= 2 {
        format!("{} · {}", parts[0], parts[1])
    } else {
        parts.join(" · ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_display_name_two_levels() {
        let root = tempdir().unwrap();
        let album = root.path().join("Artist").join("Album");
        std::fs::create_dir_all(&album).unwrap();
        let displayed = display_name(&album, root.path());
        assert!(displayed.contains("·"));
    }

    #[test]
    fn test_scanner_finds_albums() {
        let root = tempdir().unwrap();
        let a1 = root.path().join("Artist").join("Album1");
        let a2 = root.path().join("Artist").join("Album2").join("CD1");
        std::fs::create_dir_all(&a1).unwrap();
        std::fs::create_dir_all(&a2).unwrap();
        File::create(a1.join("01 track.mp3")).unwrap();
        File::create(a1.join("02 track.flac")).unwrap();
        File::create(a2.join("01 track.mp3")).unwrap();
        // hidden
        let hidden = root.path().join("._junk.mp3");
        File::create(hidden).unwrap();

        let epoch = 1;
        let cancel = Arc::new(AtomicU64::new(epoch));
        let (tx, rx) = crossbeam::channel::unbounded();
        let handle = spawn_scan(root.path().to_path_buf(), epoch, cancel, tx);

        let mut albums = Vec::new();
        while let Ok(msg) = rx.recv() {
            match msg {
                ScanMsg::Batch { albums: ref a, .. } => {
                    albums.extend(a.clone());
                }
                ScanMsg::Done { total, .. } => {
                    assert_eq!(total, 2, "should find 2 album dirs");
                    break;
                }
            }
        }
        handle.join().unwrap();
        assert_eq!(albums.len(), 2);
        // Check that ._junk.mp3 wasn't added
        assert!(albums.iter().all(|a| !a.tracks.is_empty()));
    }

    #[test]
    fn test_cancellation() {
        let root = tempdir().unwrap();
        let epoch = 1u64;
        let cancel = Arc::new(AtomicU64::new(epoch));
        let (tx, _rx) = crossbeam::channel::unbounded();

        // Bump epoch to cancel before scanning
        cancel.store(2, Ordering::Relaxed);

        let handle = spawn_scan(root.path().to_path_buf(), epoch, cancel, tx);
        handle.join().unwrap();
        // Should have exited without panicking
    }
}
