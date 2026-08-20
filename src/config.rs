use std::{fs, path::PathBuf};

pub struct Config {
    pub arrangement_directory: PathBuf,
}

impl Config {
    pub fn new() -> Config {
        let home_dir = dirs::home_dir().expect("Failed to derive user home dir.");
        let arrangement_dir = home_dir.join(".aerospace-arrangement");

        Config {
            arrangement_directory: arrangement_dir,
        }
    }

    pub fn make_arrangement_dir_if_not_exists(&self) {
        _ = fs::create_dir_all(&self.arrangement_directory)
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
                .arrangement_directory
                .to_str()
                .unwrap()
                .ends_with(".aerospace-arrangement")
        )
    }
}
