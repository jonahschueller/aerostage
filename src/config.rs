use std::{fs, path::PathBuf};

use anyhow::{Context, Result};

pub struct Config {
    pub stage_directory: PathBuf,
}

impl Config {
    pub fn new() -> Config {
        let stage_dir = Self::default_stage_directory();

        Config {
            stage_directory: stage_dir,
        }
    }

    #[cfg(debug_assertions)]
    fn default_stage_directory() -> PathBuf {
        std::env::current_dir().expect("Failed to derive current working directory.")
    }

    #[cfg(not(debug_assertions))]
    fn default_stage_directory() -> PathBuf {
        let home_dir = dirs::home_dir().expect("Failed to derive user home dir.");
        home_dir.join(".aerostage")
    }

    pub fn make_stage_dir_if_not_exists(&self) -> Result<()> {
        fs::create_dir_all(&self.stage_directory)
            .with_context(|| "Failed to create stage directory.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_config() {
        let config = Config::new();

        assert!(
            config
                .stage_directory
                .to_str()
                .unwrap()
                .ends_with(".aerostage")
        )
    }
}
