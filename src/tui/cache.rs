use crate::tui::scanner::Album;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedAlbum {
    pub dir: PathBuf,
    pub rel: String,
    pub display: String,
    pub tracks: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumCache {
    pub root: PathBuf,
    pub albums: Vec<CachedAlbum>,
}

pub fn default_cache_path() -> PathBuf {
    if let Some(project_dirs) = directories::ProjectDirs::from("com", "COV", "cov") {
        project_dirs.cache_dir().join("albums_cache.json")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".cache")
            .join("cov")
            .join("albums_cache.json")
    } else {
        PathBuf::from("/tmp/cov_albums_cache.json")
    }
}

pub fn load_cache(root: &Path) -> Option<Vec<Arc<Album>>> {
    let cache_file = default_cache_path();
    if !cache_file.exists() {
        return None;
    }

    let data = fs::read_to_string(&cache_file).ok()?;
    let cache: AlbumCache = serde_json::from_str(&data).ok()?;

    if cache.root != root {
        return None; // Root changed
    }

    let albums = cache
        .albums
        .into_iter()
        .map(|ca| {
            Arc::new(Album {
                dir: ca.dir,
                rel: ca.rel,
                display: ca.display,
                tracks: ca.tracks,
            })
        })
        .collect();

    Some(albums)
}

pub fn save_cache(root: &Path, albums: &[Arc<Album>]) -> anyhow::Result<()> {
    let cache_file = default_cache_path();
    if let Some(parent) = cache_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let cached_albums: Vec<CachedAlbum> = albums
        .iter()
        .map(|a| CachedAlbum {
            dir: a.dir.clone(),
            rel: a.rel.clone(),
            display: a.display.clone(),
            tracks: a.tracks.clone(),
        })
        .collect();

    let cache = AlbumCache {
        root: root.to_path_buf(),
        albums: cached_albums,
    };

    let data = serde_json::to_string(&cache)?;
    fs::write(&cache_file, data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_roundtrip() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("music");
        fs::create_dir_all(&root).unwrap();

        let albums = vec![Arc::new(Album {
            dir: root.join("Artist/Album"),
            rel: "Artist/Album".to_string(),
            display: "Artist · Album".to_string(),
            tracks: vec![root.join("Artist/Album/01.flac")],
        })];

        let result = save_cache(&root, &albums);
        assert!(result.is_ok());

        let loaded = load_cache(&root);
        assert!(loaded.is_some());
        let loaded_albums = loaded.unwrap();
        assert_eq!(loaded_albums.len(), 1);
        assert_eq!(loaded_albums[0].display, "Artist · Album");
    }
}
