use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};

use crate::aerospace::Aerospace;
use crate::stage::{Stage, StageWindow, StageWorkspace};

pub fn capture_stage(
    aerospace: &Aerospace,
    name: Option<&str>,
    target_workspaces: Option<&[&str]>,
) -> Result<Stage> {
    let aerospace_workspaces = aerospace
        .list_workspaces()
        .context("Failed to query AeroSpace workspaces.")?;
    let windows = aerospace
        .list_windows()
        .context("Failed to query AeroSpace windows.")?;

    let workspace_filter_set: Option<HashSet<&str>> =
        target_workspaces.map(|ws| ws.iter().copied().collect());

    let mut windows_by_workspace: HashMap<&str, Vec<StageWindow>> = HashMap::new();
    for window in &windows {
        windows_by_workspace
            .entry(&window.workspace)
            .or_default()
            .push(StageWindow {
                app: Some(window.app_name.clone()),
                title: Some(window.window_title.clone()),
                bundle_id: Some(window.app_bundle_id.clone()),
            });
    }

    let workspaces: Vec<StageWorkspace> = aerospace_workspaces
        .iter()
        .filter(|ws| {
            workspace_filter_set.as_ref().map_or(true, |filter_set| {
                filter_set.contains(ws.workspace.as_str())
            })
        })
        .filter_map(|ws| {
            let windows = windows_by_workspace.remove(ws.workspace.as_str())?;
            Some(StageWorkspace {
                name: ws.workspace.clone(),
                windows,
            })
        })
        .collect();

    Ok(Stage {
        name: name.map(String::from),
        description: None,
        workspaces: workspaces,
        default_workspace: None,
    })
}
