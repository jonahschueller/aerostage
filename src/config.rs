use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use figment::{
    Figment,
    providers::{Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

const AEROSTAGE_DIR: &str = ".aerostage";
const AEROSTAGE_CONFIG_FILE_NAME: &str = "config.toml";
const AEROSTAGE_DEFAULT_STAGES_DIR: &str = "stages";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub stage_directory: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let stage_dir = Self::default_stage_directory();

        Config {
            stage_directory: stage_dir,
        }
    }
}

impl Config {
    pub fn load(config: Option<PathBuf>) -> Result<Self> {
        let user_config = match config {
            Some(path) => {
                ensure_config_exists(&path)?;
                Some(path)
            }
            None => dirs::home_dir()
                .map(|home| home.join(AEROSTAGE_DIR).join(AEROSTAGE_CONFIG_FILE_NAME))
                .filter(|path| path.exists()),
        };

        let mut config_builder = Figment::new().merge(Serialized::defaults(Config::default()));

        if let Some(user_config) = user_config {
            config_builder = config_builder.merge(Toml::file(&user_config));
        }

        let config = config_builder
            .extract()
            .context("Failed to build aerostage config.")?;

        Ok(config)
    }

    fn default_stage_directory() -> PathBuf {
        if cfg!(debug_assertions) {
            return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }

        dirs::home_dir()
            .expect("Failed to derive user home dir.")
            .join(AEROSTAGE_DIR)
            .join(AEROSTAGE_DEFAULT_STAGES_DIR)
    }
}

fn ensure_config_exists(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    bail!("Config file '{}' does not exist.", path.display());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_errors_when_explicit_config_is_missing() {
        let missing = std::env::temp_dir().join(format!(
            "aerostage-missing-config-{}.toml",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&missing);

        let error = Config::load(Some(missing.clone())).unwrap_err();
        assert!(error.to_string().contains("does not exist"));
    }
}
