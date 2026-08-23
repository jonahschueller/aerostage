use std::collections::HashMap;

use anyhow::Result;

use crate::{
    aerospace::{Aerospace, AerospaceWindow, AerospaceWindowId, AerospaceWorkspaceId},
    restore::{resolution::WindowResolution, restore::RestoreAction::MoveToWorkspace},
    stage::Stage,
};

#[derive(Debug)]
enum RestoreAction {
    MoveToWorkspace {
        workspace: String,
        target_window: AerospaceWindowId,
    },
}

impl RestoreAction {
    fn execute(&self, aerospace: &Aerospace) -> Result<()> {
        match self {
            MoveToWorkspace {
                workspace,
                target_window,
            } => aerospace.move_node_to_workspace(workspace, target_window.clone()),
        }
    }
}

#[derive(Debug)]
struct RestorePlan {
    plan: Vec<RestoreAction>,
}

impl RestorePlan {
    fn resolve(
        resolution: &WindowResolution,
        live_windows: &[AerospaceWindow],
    ) -> Result<RestorePlan> {
        let live_workspace_lookup: HashMap<&AerospaceWindowId, &AerospaceWorkspaceId> =
            live_windows
                .iter()
                .map(|w| (&w.window_id, &w.workspace))
                .collect();

        let actions = resolution
            .resolved_windows
            .iter()
            .filter_map(|mapping| {
                let current_workspace = live_workspace_lookup.get(&mapping.window_id)?;

                if *current_workspace != &mapping.target_workspace {
                    Some(RestoreAction::MoveToWorkspace {
                        workspace: mapping.target_workspace.clone(),
                        target_window: mapping.window_id.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        resolution
            .unresolved_windows
            .iter()
            .filter_map(|window| {
                live_windows
                    .iter()
                    .find(|live| live.window_id == window.window_id)
            })
            .for_each(|window| {
                println!(
                    "Could not restore window {} | {} | {}",
                    window.window_title, window.app_name, window.app_bundle_id
                )
            });

        Ok(RestorePlan { plan: actions })
    }

    fn restore(&self, aerospace: &Aerospace) {
        for action in &self.plan {
            action.execute(aerospace).expect("Failed to restore stage.")
        }
    }
}

pub fn restore_stage(aerospace: &Aerospace, stage: &Stage) -> Result<()> {
    let live_windows = aerospace.list_windows()?;

    let resolution = WindowResolution::resolve(stage, &live_windows);

    let restore_plan = RestorePlan::resolve(&resolution, &live_windows)?;

    restore_plan.restore(aerospace);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_stage_successfully() {}
}
