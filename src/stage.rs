use anyhow::{Context, Result};
use std::io::BufWriter;
use std::io::Write;

use serde::{Deserialize, Serialize};

pub mod commands;
pub mod repository;

#[derive(Debug, Serialize, Deserialize)]
pub struct Stage {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "workspace", default)]
    pub workspaces: Vec<StageWorkspace>,
    pub default_workspace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StageWorkspaceLayout {
    HTiles,
    VTiles,
    HAccordion,
    VAccordion,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StageWorkspace {
    pub name: String,
    #[serde(rename = "window", default)]
    pub windows: Vec<StageWindow>,
    pub layout: Option<StageWorkspaceLayout>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct StageWindow {
    pub app: Option<String>,
    pub title: Option<String>,
    pub bundle_id: Option<String>,
}

impl Stage {
    pub fn write(&self, writer: Box<dyn Write>) -> Result<()> {
        let toml_stage = toml::to_string_pretty(&self)
            .with_context(|| "Failed to convert stage to toml format.")?;

        let mut buffered = BufWriter::new(writer);

        writeln!(buffered, "{}", toml_stage)
            .with_context(|| "Failed to write context to output.")?;

        Ok(())
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
            layout: None,
        }
    }

    #[cfg(test)]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}
