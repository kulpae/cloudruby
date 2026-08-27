use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    /// Directories, audio files, M3U/M3U8 playlists, or HTTP(S) stream URLs.
    pub sources: Vec<String>,
    pub no_shuffle: bool,
    pub ui: UiConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct UiConfig {
    pub colors: HashMap<String, Vec<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot determine the XDG configuration directory")]
    MissingConfigDirectory,
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse {path}: {message}")]
    Parse { path: PathBuf, message: String },
    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl Config {
    pub fn load(explicit: Option<&Path>) -> Result<(Self, ConfigSource), ConfigError> {
        if let Some(path) = explicit {
            return Self::load_path(path)
                .map(|config| (config, ConfigSource::Explicit(path.into())));
        }
        let xdg = Self::path()?;
        if xdg.is_file() {
            return Self::load_path(&xdg).map(|config| (config, ConfigSource::Xdg(xdg)));
        }
        Ok((Self::default(), ConfigSource::Defaults))
    }

    pub fn path() -> Result<PathBuf, ConfigError> {
        ProjectDirs::from("net", "cloudruby", "cloudruby")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .ok_or(ConfigError::MissingConfigDirectory)
    }

    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::Write {
                path: parent.into(),
                source,
            })?;
        }
        let encoded = toml::to_string_pretty(self).map_err(|error| ConfigError::Parse {
            path: path.into(),
            message: error.to_string(),
        })?;
        fs::write(path, encoded).map_err(|source| ConfigError::Write {
            path: path.into(),
            source,
        })
    }

    fn load_path(path: &Path) -> Result<Self, ConfigError> {
        let input = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.into(),
            source,
        })?;
        toml::from_str(&input).map_err(|error| ConfigError::Parse {
            path: path.into(),
            message: error.to_string(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigSource {
    Explicit(PathBuf),
    Xdg(PathBuf),
    Defaults,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_toml() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        let expected = Config {
            sources: vec!["~/Music".into(), "radio.m3u8".into()],
            no_shuffle: true,
            ..Config::default()
        };
        expected.save(&path).unwrap();
        let actual = Config::load_path(&path).unwrap();
        assert_eq!(actual.sources, expected.sources);
        assert!(actual.no_shuffle);
    }
}
