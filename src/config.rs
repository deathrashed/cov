use anyhow::Result;
use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    pub library_root: Option<PathBuf>,
    pub cache: CacheConfig,
    pub default_mode: Mode,
    pub theme: String,
    pub covit_path: PathBuf,
    pub log_path: PathBuf,
    pub output_basename: String,
    pub default_resolution: Option<u32>,
    pub default_sources: Option<String>,
    #[serde(skip)]
    config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    pub enabled: bool,
    pub path: Option<PathBuf>,
    pub refresh: CacheRefresh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheRefresh {
    Manual,
    Startup,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: None,
            refresh: CacheRefresh::Manual,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Save,
    Embed,
}

use crate::paths::expand_tilde;

impl Default for Config {
    fn default() -> Self {
        Self {
            library_root: None,
            cache: CacheConfig::default(),
            default_mode: Mode::Save,
            theme: "default".to_string(),
            covit_path: expand_tilde("~/.local/bin/covit"),
            log_path: Config::default_log_path(),
            output_basename: "cover".to_string(),
            default_resolution: None,
            default_sources: None,
            config_path: None,
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("xyz", "musichoarders", "cov")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }

    pub fn default_log_path() -> PathBuf {
        expand_tilde("~/Library/Logs/cov-toolkit.log")
    }

    pub fn load() -> Result<Self> {
        Self::load_with_override(None)
    }

    pub fn load_with_override(override_path: Option<&std::path::Path>) -> Result<Self> {
        Self::load_with_override_and_music_dir(override_path, Self::music_dir_from_env())
    }

    fn load_with_override_and_music_dir(
        override_path: Option<&Path>,
        music_dir: Option<PathBuf>,
    ) -> Result<Self> {
        let path = override_path
            .map(|p| p.to_path_buf())
            .or_else(Self::config_path)
            .unwrap_or_else(|| PathBuf::from(".cov.toml"));

        let mut config = if path.exists() {
            let content = fs::read_to_string(&path)?;
            toml::from_str(&content)?
        } else {
            Config::default()
        };
        config.library_root = Self::usable_library_root(config.library_root, music_dir);
        config.config_path = Some(path);
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = self
            .config_path
            .clone()
            .or_else(Self::config_path)
            .unwrap_or_else(|| PathBuf::from(".cov.toml"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    fn music_dir_from_env() -> Option<PathBuf> {
        std::env::var_os("MUSIC_DIR")
            .map(PathBuf::from)
            .filter(|path| path.is_dir())
    }

    fn usable_library_root(
        configured: Option<PathBuf>,
        music_dir: Option<PathBuf>,
    ) -> Option<PathBuf> {
        configured
            .filter(|path| path.is_dir())
            .or_else(|| music_dir.filter(|path| path.is_dir()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_values() {
        let config = Config::default();
        assert_eq!(config.library_root, None);
        assert!(config.cache.enabled);
        assert_eq!(config.cache.path, None);
        assert_eq!(config.cache.refresh, CacheRefresh::Manual);
        assert_eq!(config.default_mode, Mode::Save);
        assert_eq!(config.theme, "default");
        assert_eq!(config.covit_path, expand_tilde("~/.local/bin/covit"));
        assert_eq!(
            config.log_path,
            expand_tilde("~/Library/Logs/cov-toolkit.log")
        );
        assert_eq!(config.output_basename, "cover");
        assert_eq!(config.default_resolution, None);
        assert_eq!(config.default_sources, None);
    }

    #[test]
    fn test_toml_serialization() {
        let config = Config {
            library_root: Some(PathBuf::from("/test/root")),
            default_mode: Mode::Embed,
            theme: "dark".to_string(),
            ..Config::default()
        };

        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.library_root, config.library_root);
        assert_eq!(deserialized.default_mode, config.default_mode);
        assert_eq!(deserialized.theme, config.theme);
    }

    #[test]
    fn test_partial_toml_deserialization() {
        let toml_str = r#"
            theme = "dracula"
            default_mode = "embed"
        "#;

        let deserialized: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(deserialized.theme, "dracula");
        assert_eq!(deserialized.default_mode, Mode::Embed);
        assert_eq!(deserialized.library_root, None);
        assert_eq!(deserialized.covit_path, expand_tilde("~/.local/bin/covit"));
    }

    #[test]
    fn test_cache_settings_deserialize() {
        let toml_str = r#"
            [cache]
            enabled = false
            path = "/tmp/cov-index.json"
            refresh = "startup"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();

        assert!(!config.cache.enabled);
        assert_eq!(
            config.cache.path,
            Some(PathBuf::from("/tmp/cov-index.json"))
        );
        assert_eq!(config.cache.refresh, CacheRefresh::Startup);
    }

    #[test]
    fn test_cover_defaults_deserialize() {
        let toml_str = r#"
            output_basename = "folder"
            default_resolution = 1500
            default_sources = "bandcamp,deezer"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();

        assert_eq!(config.output_basename, "folder");
        assert_eq!(config.default_resolution, Some(1500));
        assert_eq!(config.default_sources.as_deref(), Some("bandcamp,deezer"));
    }

    #[test]
    fn test_config_save_roundtrip() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("config.toml");
        let library_root = temp.path().join("library");
        fs::create_dir(&library_root).unwrap();
        let cfg = Config {
            library_root: Some(library_root.clone()),
            ..Config::default()
        };

        let content = toml::to_string_pretty(&cfg).unwrap();
        fs::write(&path, content).unwrap();

        let loaded = Config::load_with_override(Some(&path)).unwrap();
        assert_eq!(loaded.library_root, Some(library_root));
    }

    #[test]
    fn test_invalid_configured_library_uses_music_dir() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        let music_dir = temp.path().join("music");
        fs::create_dir(&music_dir).unwrap();

        let config = Config {
            library_root: Some(temp.path().join("missing-library")),
            ..Config::default()
        };
        fs::write(&config_path, toml::to_string_pretty(&config).unwrap()).unwrap();

        let loaded =
            Config::load_with_override_and_music_dir(Some(&config_path), Some(music_dir.clone()))
                .unwrap();

        assert_eq!(loaded.library_root, Some(music_dir));
    }
}
