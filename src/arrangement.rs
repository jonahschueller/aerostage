use anyhow::{Context, Ok, Result, ensure};
use std::fs;
use std::path::Path;

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
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_arrangement = toml::to_string_pretty(&self)
            .with_context(|| "Failed to convert arrangement to toml format.")?;

        fs::write(path, toml_arrangement)
            .with_context(|| "Failed to write arrangement to file.")?;

        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Arrangement> {
        let path = path.as_ref();

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file '{}'.", path.display()))?;

        let arrangement = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML from '{}'.", path.display()))?;

        Ok(arrangement)
    }

    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<Arrangement>> {
        let dir = dir.as_ref();

        ensure!(
            !dir.is_dir(),
            format!("'{}' is not a directory", dir.display())
        );

        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory '{}'", dir.display()))?;

        let mut arrangements = Vec::new();

        for entry in entries {
            let entry = entry.with_context(|| "Failed to read directory entry.")?;

            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let arrangement = Arrangement::load_from_file(&path).with_context(|| {
                    format!("Failed to load arrangement from file '{}'.", path.display())
                })?;

                arrangements.push(arrangement);
            }
        }

        ensure!(
            arrangements.is_empty(),
            format!("No arrangements found in directory '{}'", dir.display())
        );

        Ok(arrangements)
    }

    pub fn load_from_config() -> Result<Vec<Arrangement>> {
        let config_dir = dirs::config_dir()
            .with_context(|| "Could not determine config directory".to_string())?;

        let arrangements_dir = config_dir.join("aerospace-arrangements");

        Arrangement::load_from_dir(arrangements_dir)
            .with_context(|| format!("Failed to load arrangements from config directory."))
    }
}

impl ArrangementWindow {
    #[cfg(test)]
    pub fn dummy() -> Self {
        ArrangementWindow {
            app: "Test App".into(),
            title: Some("Test Title".into()),
            bundle_id: "com.example.test".into(),
        }
    }

    #[cfg(test)]
    pub fn with_bundle_id(mut self, bundle_id: &str) -> Self {
        self.bundle_id = bundle_id.to_string();
        self
    }

    #[cfg(test)]
    pub fn with_app(mut self, app: &str) -> Self {
        self.app = app.to_string();
        self
    }

    #[cfg(test)]
    pub fn with_title(mut self, title: Option<&str>) -> Self {
        self.title = title.map(|t| t.to_string());
        self
    }
}

impl ArrangementWorkspace {
    #[cfg(test)]
    pub fn dummy() -> Self {
        ArrangementWorkspace {
            name: "1".into(),
            windows: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}
