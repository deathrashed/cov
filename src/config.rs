use anyhow::Result;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    pub library_root: Option<PathBuf>,
    pub default_mode: Mode,
    pub theme: String,
    pub covit_path: PathBuf,
    pub log_path: PathBuf,
    pub output_basename: String,
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
            default_mode: Mode::Save,
            theme: "default".to_string(),
            covit_path: expand_tilde("~/.local/bin/covit"),
            log_path: Config::default_log_path(),
            output_basename: "cover".to_string(),
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
        let path = override_path
            .map(|p| p.to_path_buf())
            .or_else(Self::config_path)
            .unwrap_or_else(|| PathBuf::from(".cov.toml"));

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            return Ok(config);
        }
        Ok(Config::default())
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path().unwrap_or_else(|| PathBuf::from(".cov.toml"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
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
        assert_eq!(config.default_mode, Mode::Save);
        assert_eq!(config.theme, "default");
        assert_eq!(config.covit_path, expand_tilde("~/.local/bin/covit"));
        assert_eq!(
            config.log_path,
            expand_tilde("~/Library/Logs/cov-toolkit.log")
        );
        assert_eq!(config.output_basename, "cover");
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
    fn test_config_save_roundtrip() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("config.toml");
        let mut cfg = Config::default();
        cfg.library_root = Some(PathBuf::from("/music/library"));

        let content = toml::to_string_pretty(&cfg).unwrap();
        fs::write(&path, content).unwrap();

        let loaded = Config::load_with_override(Some(&path)).unwrap();
        assert_eq!(loaded.library_root, Some(PathBuf::from("/music/library")));
    }
}
