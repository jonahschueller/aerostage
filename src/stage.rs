use anyhow::{Context, Ok, Result, ensure};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Stage {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "workspace")]
    pub workspaces: Vec<StageWorkspace>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StageWorkspace {
    pub name: String,
    // pub layout: String,
    // pub monitor: Vec<String>,
    // #[serde(default)]
    // pub focus: bool,
    #[serde(rename = "window")]
    pub windows: Vec<StageWindow>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct StageWindow {
    pub app: Option<String>,
    pub title: Option<String>,
    pub bundle_id: Option<String>, // #[serde(rename = "launch-if-missing", default)]
                                   // pub launch_if_missing: bool,
                                   // #[serde(default)]
                                   // pub float: bool,
}

impl Stage {
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_stage = toml::to_string_pretty(&self)
            .with_context(|| "Failed to convert stage to toml format.")?;

        fs::write(&path, toml_stage).with_context(|| {
            format!(
                "Failed to write stage to file '{}'.",
                path.as_ref().display()
            )
        })?;

        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Stage> {
        let path = path.as_ref();

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file '{}'.", path.display()))?;

        let stage = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML from '{}'.", path.display()))?;

        Ok(stage)
    }

    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<Stage>> {
        let dir = dir.as_ref();

        ensure!(
            !dir.is_dir(),
            format!("'{}' is not a directory", dir.display())
        );

        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory '{}'", dir.display()))?;

        let mut stages = Vec::new();

        for entry in entries {
            let entry = entry.with_context(|| "Failed to read directory entry.")?;

            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let stage = Stage::load_from_file(&path).with_context(|| {
                    format!("Failed to load stage from file '{}'.", path.display())
                })?;

                stages.push(stage);
            }
        }

        ensure!(
            stages.is_empty(),
            format!("No stages found in directory '{}'", dir.display())
        );

        Ok(stages)
    }

    pub fn load_from_config() -> Result<Vec<Stage>> {
        let config_dir = dirs::config_dir()
            .with_context(|| "Could not determine config directory".to_string())?;

        let stages_dir = config_dir.join("aerospace-stages");

        Stage::load_from_dir(stages_dir)
            .with_context(|| format!("Failed to load stages from config directory."))
    }
}

impl StageWindow {
    #[cfg(test)]
    pub fn dummy() -> Self {
        StageWindow {
            app: Some("Test App".into()),
            title: Some("Test Title".into()),
            bundle_id: Some("com.example.test".into()),
        }
    }

    #[cfg(test)]
    pub fn with_bundle_id(mut self, bundle_id: &str) -> Self {
        self.bundle_id = Some(bundle_id.to_string());
        self
    }

    #[cfg(test)]
    pub fn with_app(mut self, app: &str) -> Self {
        self.app = Some(app.to_string());
        self
    }

    #[cfg(test)]
    pub fn with_title(mut self, title: Option<&str>) -> Self {
        self.title = title.map(|t| t.to_string());
        self
    }
}

impl StageWorkspace {
    #[cfg(test)]
    pub fn dummy() -> Self {
        StageWorkspace {
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
