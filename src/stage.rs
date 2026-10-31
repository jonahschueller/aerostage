use anyhow::{Context, Result, anyhow};
use std::fmt::Display;
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

pub const FOCUSED_WORKSPACE_TOKEN: &str = "@focused";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum WorkspaceRef {
    Named(String),
    Focused,
}

impl From<String> for WorkspaceRef {
    fn from(value: String) -> Self {
        if value == FOCUSED_WORKSPACE_TOKEN {
            WorkspaceRef::Focused
        } else {
            WorkspaceRef::Named(value)
        }
    }
}

impl From<&str> for WorkspaceRef {
    fn from(value: &str) -> Self {
        WorkspaceRef::from(value.to_string())
    }
}

impl From<WorkspaceRef> for String {
    fn from(value: WorkspaceRef) -> Self {
        match value {
            WorkspaceRef::Named(name) => name,
            WorkspaceRef::Focused => FOCUSED_WORKSPACE_TOKEN.to_string(),
        }
    }
}

impl Display for WorkspaceRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceRef::Named(name) => write!(f, "{}", name),
            WorkspaceRef::Focused => write!(f, "{}", FOCUSED_WORKSPACE_TOKEN),
        }
    }
}

impl WorkspaceRef {
    /// Resolves the reference to a concrete workspace name. `focused` is the
    /// currently focused workspace, if it has been queried.
    pub fn resolve(&self, focused: Option<&str>) -> Result<String> {
        match self {
            WorkspaceRef::Named(name) => Ok(name.clone()),
            WorkspaceRef::Focused => focused.map(str::to_string).ok_or_else(|| {
                anyhow!(
                    "Cannot resolve '{}' without the focused workspace.",
                    FOCUSED_WORKSPACE_TOKEN
                )
            }),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct WorkspaceRefHolder {
        workspace: WorkspaceRef,
    }

    #[test]
    fn workspace_ref_parses_focused_token() {
        assert_eq!(WorkspaceRef::from("@focused"), WorkspaceRef::Focused);
    }

    #[test]
    fn workspace_ref_parses_other_values_as_named() {
        assert_eq!(WorkspaceRef::from("3"), WorkspaceRef::Named("3".into()));
        assert_eq!(
            WorkspaceRef::from("@Focused"),
            WorkspaceRef::Named("@Focused".into())
        );
        assert_eq!(
            WorkspaceRef::from(" @focused"),
            WorkspaceRef::Named(" @focused".into())
        );
    }

    #[test]
    fn workspace_ref_converts_back_to_string() {
        assert_eq!(String::from(WorkspaceRef::Focused), "@focused");
        assert_eq!(String::from(WorkspaceRef::Named("3".into())), "3");
        assert_eq!(WorkspaceRef::Focused.to_string(), "@focused");
        assert_eq!(WorkspaceRef::Named("3".into()).to_string(), "3");
    }

    #[test]
    fn workspace_ref_round_trips_through_toml() {
        for workspace in [WorkspaceRef::Focused, WorkspaceRef::Named("3".into())] {
            let holder = WorkspaceRefHolder { workspace };
            let toml = toml::to_string(&holder).unwrap();
            assert_eq!(toml::from_str::<WorkspaceRefHolder>(&toml).unwrap(), holder);
        }

        let parsed: WorkspaceRefHolder = toml::from_str(r#"workspace = "@focused""#).unwrap();
        assert_eq!(parsed.workspace, WorkspaceRef::Focused);
    }

    #[test]
    fn workspace_ref_resolves_named_without_focused() {
        let workspace = WorkspaceRef::Named("3".into());
        assert_eq!(workspace.resolve(None).unwrap(), "3");
        assert_eq!(workspace.resolve(Some("1")).unwrap(), "3");
    }

    #[test]
    fn workspace_ref_resolves_focused_to_given_workspace() {
        assert_eq!(WorkspaceRef::Focused.resolve(Some("1")).unwrap(), "1");
    }

    #[test]
    fn workspace_ref_fails_to_resolve_focused_without_focused() {
        assert!(WorkspaceRef::Focused.resolve(None).is_err());
    }
}
