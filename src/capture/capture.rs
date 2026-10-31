use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};

use crate::aerospace::Aerospace;
use crate::stage::{StageWindow, UnresolvedStage, UnresolvedStageWorkspace, WorkspaceRef};

pub struct StageCapturer<'a> {
    pub aerospace: &'a Aerospace,
}

impl<'a> StageCapturer<'a> {
    pub fn new(aerospace: &'a Aerospace) -> Self {
        StageCapturer { aerospace }
    }

    fn captured_windows_by_workspace(&self) -> Result<HashMap<String, Vec<StageWindow>>> {
        let windows = self
            .aerospace
            .list_windows()
            .context("Failed to query AeroSpace windows.")?;

        let mut windows_by_workspace: HashMap<String, Vec<StageWindow>> = HashMap::new();
        for window in windows {
            windows_by_workspace
                .entry(window.workspace)
                .or_default()
                .push(StageWindow {
                    app: Some(window.app_name),
                    title: Some(window.window_title),
                    bundle_id: Some(window.app_bundle_id),
                });
        }

        Ok(windows_by_workspace)
    }

    pub fn capture(
        &self,
        name: Option<&str>,
        target_workspaces: Option<&[&str]>,
        default_workspace: Option<&str>,
    ) -> Result<UnresolvedStage> {
        let aerospace_workspaces = self
            .aerospace
            .list_workspaces()
            .context("Failed to query AeroSpace workspaces.")?;

        let mut windows_by_workspace = self.captured_windows_by_workspace()?;

        let workspace_filter_set: Option<HashSet<&str>> =
            target_workspaces.map(|ws| ws.iter().copied().collect());

        let workspaces: Vec<UnresolvedStageWorkspace> = aerospace_workspaces
            .into_iter()
            .filter(|ws| {
                workspace_filter_set
                    .as_ref()
                    .is_none_or(|filter_set| filter_set.contains(ws.workspace.as_str()))
            })
            .filter_map(|ws| {
                let windows = windows_by_workspace.remove(ws.workspace.as_str())?;
                Some(UnresolvedStageWorkspace {
                    name: WorkspaceRef::Named(ws.workspace),
                    windows,
                    layout: Some(ws.layout.into()),
                })
            })
            .collect();

        Ok(UnresolvedStage {
            name: name.map(String::from),
            description: None,
            workspaces,
            default_workspace: default_workspace.map(WorkspaceRef::from),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_keeps_focused_default_workspace_as_token() {
        let aerospace = Aerospace::mock();
        let stage = StageCapturer::new(&aerospace)
            .capture(Some("work"), None, Some("@focused"))
            .unwrap();

        assert_eq!(stage.default_workspace, Some(WorkspaceRef::Focused));

        let toml = toml::to_string(&stage).unwrap();
        assert!(toml.contains(r#"default_workspace = "@focused""#), "{toml}");
    }

    #[test]
    fn capture_keeps_named_default_workspace() {
        let aerospace = Aerospace::mock();
        let stage = StageCapturer::new(&aerospace)
            .capture(Some("work"), None, Some("1"))
            .unwrap();

        assert_eq!(
            stage.default_workspace,
            Some(WorkspaceRef::Named("1".into()))
        );
    }
}
