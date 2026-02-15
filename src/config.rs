use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Top-level application configuration.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub shortcuts: Shortcuts,
    pub capture: CaptureConfig,
    pub history: HistoryConfig,
    pub storage: StorageConfig,
    pub ui: UiConfig,
    pub privacy: PrivacyConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Shortcuts {
    pub screenshot: String,
    pub history: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CaptureConfig {
    pub format: String,
    pub quality: u8,
    pub show_dimensions: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HistoryConfig {
    pub max_entries: usize,
    pub retention_days: i64,
    pub auto_cleanup: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    pub database_path: String,
    pub image_storage: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UiConfig {
    pub theme: String,
    pub thumbnail_size: u32,
    pub notification_duration: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PrivacyConfig {
    pub exclude_passwords: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            shortcuts: Shortcuts {
                screenshot: "Ctrl+Alt+S".to_string(),
                history: "Alt+H".to_string(),
            },
            capture: CaptureConfig {
                format: "png".to_string(),
                quality: 95,
                show_dimensions: true,
            },
            history: HistoryConfig {
                max_entries: 200,
                retention_days: 5,
                auto_cleanup: true,
            },
            storage: StorageConfig {
                database_path: "~/.config/clipboard-capture/history.db".to_string(),
                image_storage: "database".to_string(),
            },
            ui: UiConfig {
                theme: "auto".to_string(),
                thumbnail_size: 150,
                notification_duration: 2,
            },
            privacy: PrivacyConfig {
                exclude_passwords: true,
            },
        }
    }
}

impl Config {
    /// Return the path to the configuration directory.
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("clipboard-capture")
    }

    /// Return the path to the configuration file.
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Expand `~` in a path to the user's home directory.
    pub fn expand_path(path: &str) -> PathBuf {
        if path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(&path[2..]);
            }
        }
        PathBuf::from(path)
    }

    /// Resolve the database path with `~` expansion.
    pub fn resolved_db_path(&self) -> PathBuf {
        Self::expand_path(&self.storage.database_path)
    }

    /// Load config from the default path, or create a default config if missing.
    pub fn load_or_create_default() -> Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            Self::load(&config_path)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Load config from a specific file path.
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        let config: Config =
            toml::from_str(&content).with_context(|| "Failed to parse config TOML")?;
        Ok(config)
    }

    /// Save config to the default path.
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir();
        fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config dir: {:?}", config_dir))?;

        let config_path = Self::config_path();
        let content =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        log::info!("Config saved to {:?}", config_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.shortcuts.screenshot, "Ctrl+Alt+S");
        assert_eq!(config.shortcuts.history, "Alt+H");
        assert_eq!(config.history.max_entries, 200);
        assert_eq!(config.history.retention_days, 5);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.shortcuts.screenshot, config.shortcuts.screenshot);
        assert_eq!(parsed.history.max_entries, config.history.max_entries);
    }

    #[test]
    fn test_expand_path() {
        let expanded = Config::expand_path("~/.config/clipboard-capture/history.db");
        assert!(!expanded.to_str().unwrap().starts_with("~"));
    }
}
