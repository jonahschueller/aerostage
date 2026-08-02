use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Arrangement {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "workspace")]
    pub workspaces: Vec<ArrangementWorkspace>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArrangementWorkspace {
    pub name: String,
    // pub layout: String,
    // pub monitor: Vec<String>,
    // #[serde(default)]
    // pub focus: bool,
    #[serde(rename = "window")]
    pub windows: Vec<ArrangementWindow>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ArrangementWindow {
    pub app: String,
    pub title: Option<String>,
    pub bundle_id: String, // #[serde(rename = "launch-if-missing", default)]
                           // pub launch_if_missing: bool,
                           // #[serde(default)]
                           // pub float: bool,
}

impl Arrangement {
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        match toml::to_string_pretty(&self) {
            Ok(toml_arrangement) => {
                let write_res = fs::write(path, toml_arrangement);

                if write_res.is_err() {
                    return Err(String::from("Failed to write arrangement to file."));
                }

                Ok(())
            }
            Err(err) => Err(String::from(format!(
                "Failed to serialize arragement: {}",
                err
            ))),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Arrangement, String> {
        let path = path.as_ref();

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))?;

        let arrangement = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse TOML from '{}': {}", path.display(), e))?;

        Ok(arrangement)
    }

    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<Arrangement>, String> {
        let dir = dir.as_ref();

        if !dir.is_dir() {
            return Err(format!("'{}' is not a directory", dir.display()));
        }

        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory '{}': {}", dir.display(), e))?;

        let mut arrangements = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;

            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let arrangement = Arrangement::load_from_file(&path).map_err(|e| {
                    format!(
                        "Failed to load arrangement from file '{}': {}",
                        path.display(),
                        e
                    )
                })?;

                arrangements.push(arrangement);
            }
        }

        if arrangements.is_empty() {
            return Err(format!(
                "No arrangements found in directory '{}'",
                dir.display()
            ));
        }

        Ok(arrangements)
    }

    pub fn load_from_config() -> Result<Vec<Arrangement>, String> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| "Could not determine config directory".to_string())?;

        let arrangements_dir = config_dir.join("aerospace-arrangements");

        Arrangement::load_from_dir(arrangements_dir)
            .map_err(|e| format!("Failed to load arrangements from config directory: {}", e))
    }
}
