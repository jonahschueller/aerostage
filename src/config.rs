use std::{fs, path::PathBuf};

pub struct Config {
    pub stage_directory: PathBuf,
}

impl Config {
    pub fn new() -> Config {
        let home_dir = dirs::home_dir().expect("Failed to derive user home dir.");
        let stage_dir = home_dir.join(".aerospace-stage");

        Config {
            stage_directory: stage_dir,
        }
    }

    pub fn make_stage_dir_if_not_exists(&self) {
        _ = fs::create_dir_all(&self.stage_directory)
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
                .ends_with(".aerospace-stage")
        )
    }
}
