use crate::config::CacheConfig;
use crate::paths::expand_tilde;
use crate::tui::scanner::Album;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const CACHE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedAlbum {
    pub dir: PathBuf,
    pub rel: String,
    pub display: String,
    pub tracks: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumCache {
    pub schema_version: u32,
    pub root: PathBuf,
    pub albums: Vec<CachedAlbum>,
}

pub fn cache_path(root: &Path, config: &CacheConfig) -> PathBuf {
    if let Some(path) = &config.path {
        return expand_tilde(&path.to_string_lossy());
    }

    let mut hasher = DefaultHasher::new();
    root.hash(&mut hasher);
    let file_name = format!("album-index-{:016x}.json", hasher.finish());

    if let Some(project_dirs) = directories::ProjectDirs::from("xyz", "musichoarders", "cov") {
        project_dirs.cache_dir().join("indexes").join(file_name)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".cache")
            .join("cov")
            .join("indexes")
            .join(file_name)
    } else {
        PathBuf::from("/tmp").join(file_name)
    }
}

pub fn load_cache(root: &Path, config: &CacheConfig) -> anyhow::Result<Option<Vec<Arc<Album>>>> {
    if !config.enabled {
        return Ok(None);
    }

    let cache_file = cache_path(root, config);
    if !cache_file.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&cache_file)?;
    let cache: AlbumCache = serde_json::from_str(&data)?;
    if cache.schema_version != CACHE_SCHEMA_VERSION || cache.root != root {
        return Ok(None);
    }

    let albums = cache
        .albums
        .into_iter()
        .map(|album| {
            Arc::new(Album {
                dir: album.dir,
                rel: album.rel,
                display: album.display,
                tracks: album.tracks,
            })
        })
        .collect();

    Ok(Some(albums))
}

pub fn save_cache(root: &Path, albums: &[Arc<Album>], config: &CacheConfig) -> anyhow::Result<()> {
    if !config.enabled {
        return Ok(());
    }

    let cache_file = cache_path(root, config);
    if let Some(parent) = cache_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let cache = AlbumCache {
        schema_version: CACHE_SCHEMA_VERSION,
        root: root.to_path_buf(),
        albums: albums
            .iter()
            .map(|album| CachedAlbum {
                dir: album.dir.clone(),
                rel: album.rel.clone(),
                display: album.display.clone(),
                tracks: album.tracks.clone(),
            })
            .collect(),
    };
    let data = serde_json::to_string(&cache)?;
    let temporary = cache_file.with_extension("json.tmp");
    fs::write(&temporary, data)?;
    fs::rename(temporary, cache_file)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn config(path: PathBuf) -> CacheConfig {
        CacheConfig {
            enabled: true,
            path: Some(path),
            ..CacheConfig::default()
        }
    }

    #[test]
    fn cache_roundtrip_uses_the_configured_path() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("music");
        fs::create_dir_all(&root).unwrap();
        let cache_config = config(temp.path().join("index.json"));
        let albums = vec![Arc::new(Album {
            dir: root.join("Artist/Album"),
            rel: "Artist/Album".to_string(),
            display: "Artist · Album".to_string(),
            tracks: vec![root.join("Artist/Album/01.flac")],
        })];

        save_cache(&root, &albums, &cache_config).unwrap();
        let loaded = load_cache(&root, &cache_config).unwrap().unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].display, "Artist · Album");
    }

    #[test]
    fn cache_is_scoped_to_the_library_root() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("music");
        let other_root = temp.path().join("other-music");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&other_root).unwrap();
        let cache_config = config(temp.path().join("index.json"));

        save_cache(&root, &[], &cache_config).unwrap();

        assert!(load_cache(&other_root, &cache_config).unwrap().is_none());
    }

    #[test]
    fn disabled_cache_does_not_write_an_index() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("music");
        fs::create_dir_all(&root).unwrap();
        let path = temp.path().join("index.json");
        let cache_config = CacheConfig {
            enabled: false,
            path: Some(path.clone()),
            ..CacheConfig::default()
        };

        save_cache(&root, &[], &cache_config).unwrap();

        assert!(!path.exists());
    }
}
