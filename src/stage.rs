use anyhow::{Context, Result, anyhow};
use std::fmt::Display;
use std::io::BufWriter;
use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::aerospace::AerospaceWorkspaceId;

pub mod commands;
pub mod repository;

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
pub struct StageWorkspace<W = AerospaceWorkspaceId> {
    pub name: W,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "W: Deserialize<'de>"))]
pub struct Stage<W = AerospaceWorkspaceId> {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "workspace", default)]
    pub workspaces: Vec<StageWorkspace<W>>,
    pub default_workspace: Option<W>,
}

/// A stage as stored on disk. Workspace names may reference `@focused` and
/// must be resolved before they can be restored.
pub type UnresolvedStage = Stage<WorkspaceRef>;
pub type UnresolvedStageWorkspace = StageWorkspace<WorkspaceRef>;

impl UnresolvedStage {
    /// Resolves every workspace reference to a concrete workspace name.
    pub fn resolve(self, focused: Option<&str>) -> Result<Stage> {
        let workspaces = self
            .workspaces
            .into_iter()
            .map(|ws| {
                Ok(StageWorkspace {
                    name: ws.name.resolve(focused)?,
                    windows: ws.windows,
                    layout: ws.layout,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let default_workspace = self
            .default_workspace
            .map(|ws| ws.resolve(focused))
            .transpose()?;

        Ok(Stage {
            name: self.name,
            description: self.description,
            workspaces,
            default_workspace,
        })
    }

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

    const FOCUSED_STAGE: &str = r#"
name = "teams"
default_workspace = "@focused"

[[workspace]]
name = "@focused"

[[workspace.window]]
app = "Microsoft Teams"

[[workspace]]
name = "2"
layout = "h_tiles"
"#;

    #[test]
    fn stage_parses_focused_workspace_references() {
        let stage: UnresolvedStage = toml::from_str(FOCUSED_STAGE).unwrap();

        assert_eq!(stage.name.as_deref(), Some("teams"));
        assert_eq!(stage.default_workspace, Some(WorkspaceRef::Focused));
        assert_eq!(stage.workspaces[0].name, WorkspaceRef::Focused);
        assert_eq!(stage.workspaces[1].name, WorkspaceRef::Named("2".into()));
    }

    #[test]
    fn stage_serializes_focused_workspace_as_token() {
        let stage: UnresolvedStage = toml::from_str(FOCUSED_STAGE).unwrap();
        let toml = toml::to_string(&stage).unwrap();

        assert!(toml.contains(r#"default_workspace = "@focused""#));
        assert!(toml.contains(r#"name = "@focused""#));
    }

    #[test]
    fn stage_resolves_workspace_references() {
        let stage: UnresolvedStage = toml::from_str(FOCUSED_STAGE).unwrap();
        let resolved = stage.resolve(Some("5")).unwrap();

        assert_eq!(resolved.name.as_deref(), Some("teams"));
        assert_eq!(resolved.default_workspace.as_deref(), Some("5"));
        assert_eq!(resolved.workspaces[0].name, "5");
        assert_eq!(resolved.workspaces[0].windows.len(), 1);
        assert_eq!(resolved.workspaces[1].name, "2");
        assert_eq!(
            resolved.workspaces[1].layout,
            Some(StageWorkspaceLayout::HTiles)
        );
    }

    #[test]
    fn stage_without_focused_resolves_without_focused_workspace() {
        let stage: UnresolvedStage = toml::from_str(
            r#"
default_workspace = "1"

[[workspace]]
name = "2"
"#,
        )
        .unwrap();
        let resolved = stage.resolve(None).unwrap();

        assert_eq!(resolved.default_workspace.as_deref(), Some("1"));
        assert_eq!(resolved.workspaces[0].name, "2");
    }

    #[test]
    fn stage_with_focused_fails_to_resolve_without_focused_workspace() {
        let stage: UnresolvedStage = toml::from_str(FOCUSED_STAGE).unwrap();
        assert!(stage.resolve(None).is_err());
    }
}
