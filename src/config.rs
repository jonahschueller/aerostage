use std::path::PathBuf;

use anyhow::{Context, Ok, Result};
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
        let user_config = config.or_else(|| {
            let default_user_config = dirs::home_dir()?
                .join(AEROSTAGE_DIR)
                .join(AEROSTAGE_CONFIG_FILE_NAME);

            if default_user_config.exists() {
                Some(default_user_config)
            } else {
                None
            }
        });

        let mut config_builder = Figment::new().merge(Serialized::defaults(Config::default()));

        if let Some(user_config) = user_config {
            config_builder = config_builder.merge(Toml::file(user_config));
        }

        let config = config_builder
            .extract()
            .context("Failed to build aerostage config.")?;

        Ok(config)
    }

    fn default_stage_directory() -> PathBuf {
        let home_dir = dirs::home_dir().expect("Failed to derive user home dir.");
        home_dir
            .join(AEROSTAGE_DIR)
            .join(AEROSTAGE_DEFAULT_STAGES_DIR)
    }
}
