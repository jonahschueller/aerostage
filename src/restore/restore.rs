use std::collections::HashMap;

use anyhow::Result;

use crate::{
    aerospace::{Aerospace, AerospaceWindow, AerospaceWindowId, AerospaceWorkspaceId},
    restore::resolution::WindowResolution,
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
            Self::MoveToWorkspace {
                workspace,
                target_window,
            } => aerospace.move_node_to_workspace(workspace, *target_window),
        }
    }
}

#[derive(Debug)]
struct RestorePlan {
    plan: Vec<RestoreAction>,
}

impl RestorePlan {
    fn resolve(resolution: &WindowResolution, live_windows: &[AerospaceWindow]) -> RestorePlan {
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
                        target_window: mapping.window_id,
                    })
                } else {
                    None
                }
            })
            .collect();

        RestorePlan { plan: actions }
    }

    fn restore(&self, aerospace: &Aerospace) -> Result<()> {
        for action in &self.plan {
            action.execute(aerospace)?;
        }

        Ok(())
    }
}

pub fn restore_stage(aerospace: &Aerospace, stage: &Stage) -> Result<()> {
    let live_windows = aerospace.list_windows()?;

    let resolution = WindowResolution::resolve(stage, &live_windows);

    let restore_plan = RestorePlan::resolve(&resolution, &live_windows);
    restore_plan.restore(aerospace)?;

    for target in resolution.pending_targets {
        let window = target.target_window;
        eprintln!(
            "Could not restore window {} | {} | {}",
            window.title.as_deref().unwrap_or("-"),
            window.app.as_deref().unwrap_or("-"),
            window.bundle_id.as_deref().unwrap_or("-"),
        )
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restore::{resolution::WindowResolution, types::ResolvedWindowMatch};

    #[test]
    fn skips_windows_already_on_the_target_workspace() {
        let live_windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(1)
                .with_workspace("1"),
            AerospaceWindow::dummy()
                .with_window_id(2)
                .with_workspace("1"),
        ];
        let resolution = WindowResolution {
            resolved_windows: vec![
                ResolvedWindowMatch {
                    target_workspace: "1".into(),
                    window_id: 1,
                },
                ResolvedWindowMatch {
                    target_workspace: "2".into(),
                    window_id: 2,
                },
            ],
            pending_targets: Vec::new(),
            unresolved_windows: Vec::new(),
        };

        let plan = RestorePlan::resolve(&resolution, &live_windows);

        assert_eq!(plan.plan.len(), 1);
        match &plan.plan[0] {
            RestoreAction::MoveToWorkspace {
                workspace,
                target_window,
            } => {
                assert_eq!(workspace, "2");
                assert_eq!(*target_window, 2);
            }
        }
    }
}
