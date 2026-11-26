use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use log::LevelFilter;
use serde::{Deserialize, Serialize};

const AEROSTAGE_DIR: &str = ".aerostage";
const AEROSTAGE_CONFIG_FILE_NAME: &str = "config.toml";
const AEROSTAGE_DEFAULT_STAGE_NAME: &str = "default.toml";
const AEROSTAGE_LOG_ENV: &str = "AEROSTAGE_LOG";
#[cfg(not(all(debug_assertions, feature = "cwd-stages")))]
const AEROSTAGE_DEFAULT_STAGES_DIR: &str = "stages";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub stage_directory: PathBuf,
    pub default_stage: String,
    pub log_file: PathBuf,
    pub log_level: LevelFilter,
}

impl Default for Config {
    fn default() -> Self {
        let stage_dir = Self::default_stage_directory();

        Config::with_stage_dir(stage_dir)
    }
}

impl Config {
    pub fn with_stage_dir(stage_dir: PathBuf) -> Self {
        Config {
            stage_directory: stage_dir,
            default_stage: AEROSTAGE_DEFAULT_STAGE_NAME.to_string(),
            log_file: Self::default_log_file(),
            log_level: LevelFilter::Info,
        }
    }

    pub fn resolve_stage_name<'a>(&'a self, requested: Option<&'a str>) -> &'a str {
        requested.unwrap_or(self.default_stage.as_str())
    }

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
        config_builder = config_builder.merge(
            Env::raw()
                .only(&[AEROSTAGE_LOG_ENV])
                .map(|_| "log_level".into()),
        );

        let config = config_builder
            .extract()
            .context("Failed to build aerostage config.")?;

        Ok(config)
    }

    fn default_log_file() -> PathBuf {
        dirs::home_dir()
            .expect("Failed to derive user home dir.")
            .join("Library/Logs/aerostage/aerostage.log")
    }

    fn default_stage_directory() -> PathBuf {
        #[cfg(all(debug_assertions, feature = "cwd-stages"))]
        {
            return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }

        #[cfg(not(all(debug_assertions, feature = "cwd-stages")))]
        {
            dirs::home_dir()
                .expect("Failed to derive user home dir.")
                .join(AEROSTAGE_DIR)
                .join(AEROSTAGE_DEFAULT_STAGES_DIR)
        }
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

    #[test]
    fn load_uses_default_stage_when_config_omits_it() {
        let path = std::env::temp_dir().join(format!(
            "aerostage-config-defaults-{}.toml",
            std::process::id()
        ));
        std::fs::write(&path, "stage_directory = \"/tmp/aerostage-stages\"\n").unwrap();

        let config = Config::load(Some(path.clone())).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(config.default_stage, "default.toml");
        assert_eq!(
            config.stage_directory,
            PathBuf::from("/tmp/aerostage-stages")
        );
    }

    #[test]
    fn load_reads_default_stage_from_config() {
        let path = std::env::temp_dir().join(format!(
            "aerostage-config-default-stage-{}.toml",
            std::process::id()
        ));
        std::fs::write(&path, "default_stage = \"focus.toml\"\n").unwrap();

        let config = Config::load(Some(path.clone())).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(config.default_stage, "focus.toml");
    }

    #[test]
    fn load_reads_log_settings_from_config() {
        let path =
            std::env::temp_dir().join(format!("aerostage-config-log-{}.toml", std::process::id()));
        std::fs::write(&path, "log_file = \"/tmp/a.log\"\nlog_level = \"debug\"\n").unwrap();

        let config = Config::load(Some(path.clone())).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(config.log_file, PathBuf::from("/tmp/a.log"));
        assert_eq!(config.log_level, LevelFilter::Debug);
    }

    #[test]
    fn resolve_stage_name_falls_back_to_default_stage() {
        let config = Config::with_stage_dir(PathBuf::from("/stages"));

        assert_eq!(config.resolve_stage_name(None), "default.toml");
        assert_eq!(config.resolve_stage_name(Some("work.toml")), "work.toml");
    }
}
