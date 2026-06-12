use serde::de::DeserializeOwned;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Arrangement {
    pub name: String,
    pub description: String,
    #[serde(rename = "workspace")]
    pub workspaces: Vec<ArrangementWorkspace>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArrangementWorkspace {
    pub name: String,
    pub layout: String,
    pub monitor: Vec<String>,
    #[serde(default)]
    pub focus: bool,
    #[serde(rename = "window")]
    pub windows: Vec<ArrangementWindow>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArrangementWindow {
    pub app: String,
    pub title: Option<String>,
    #[serde(rename = "launch-if-missing", default)]
    pub launch_if_missing: bool,
    #[serde(default)]
    pub float: bool,
}

/// Load a single arrangement from a TOML file
///
/// # Arguments
///
/// * `path` - Path to the TOML file containing the arrangement
///
/// # Returns
///
/// * `Result<Arrangement, String>` - The parsed arrangement or an error message
pub fn load_arrangement<P: AsRef<Path>>(path: P) -> Result<Arrangement, String> {
    let path = path.as_ref();

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))?;

    toml::from_str(&content)
        .map_err(|e| format!("Failed to parse TOML from '{}': {}", path.display(), e))
}

/// Load all arrangements from a directory
///
/// # Arguments
///
/// * `dir` - Path to the directory containing TOML arrangement files
///
/// # Returns
///
/// * `Result<Vec<Arrangement>, String>` - Vector of parsed arrangements or an error message
pub fn load_arrangements_from_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<Arrangement>, String> {
    let dir = dir.as_ref();

    if !dir.is_dir() {
        return Err(format!("'{}' is not a directory", dir.display()));
    }

    let mut arrangements = Vec::new();

    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory '{}': {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;

        let path = entry.path();

        // Only process .toml files
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            match load_arrangement(&path) {
                Ok(arrangement) => arrangements.push(arrangement),
                Err(e) => eprintln!("Warning: {}", e),
            }
        }
    }

    Ok(arrangements)
}

/// Load all arrangements from the default configuration directory
///
/// This loads arrangements from `~/.config/aerospace-arrangements/`
///
/// # Returns
///
/// * `Result<Vec<Arrangement>, String>` - Vector of parsed arrangements or an error message
pub fn load_arrangements_from_config() -> Result<Vec<Arrangement>, String> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| "Could not determine config directory".to_string())?;

    let arrangements_dir = config_dir.join("aerospace-arrangements");

    if !arrangements_dir.exists() {
        return Err(format!(
            "Arrangements directory does not exist: '{}'",
            arrangements_dir.display()
        ));
    }

    load_arrangements_from_dir(arrangements_dir)
}
