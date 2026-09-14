use std::collections::HashMap;

use anyhow::Result;

use crate::{
    aerospace::{Aerospace, AerospaceLayout, AerospaceWindow, AerospaceWindowId},
    restore::resolution::WindowResolution,
    stage::{Stage, StageWorkspace},
};

#[derive(Debug)]
enum RestoreAction {
    MoveToWorkspace {
        workspace: String,
        target_window: AerospaceWindowId,
    },
    ChangeLayout {
        workspace: String,
        target_layout: AerospaceLayout,
    },
    FlattenWorkspace {
        workspace: String,
    },
}

impl RestoreAction {
    fn execute(&self, aerospace: &mut Aerospace) -> Result<()> {
        match self {
            Self::MoveToWorkspace {
                workspace,
                target_window,
            } => aerospace.move_node_to_workspace(workspace, *target_window),
            Self::ChangeLayout {
                workspace,
                target_layout,
            } => aerospace.change_layout(workspace, target_layout),
            Self::FlattenWorkspace { workspace } => aerospace.flatten_workspace_tree(workspace),
        }
    }
}

#[derive(Debug)]
struct RestorePlan {
    plan: Vec<RestoreAction>,
}

impl RestorePlan {
    fn resolve_layout_actions(
        workspaces: &[StageWorkspace],
    ) -> impl Iterator<Item = RestoreAction> + '_ {
        workspaces.iter().filter_map(|ws| {
            ws.layout
                .as_ref()
                .map(|layout| RestoreAction::ChangeLayout {
                    workspace: ws.name.clone(),
                    target_layout: layout.clone().into(),
                })
        })
    }

    fn resolve_flatten_workspace_actions(
        workspaces: &[StageWorkspace],
    ) -> impl Iterator<Item = RestoreAction> + '_ {
        workspaces.iter().filter_map(|ws| {
            if ws.layout.is_some() {
                Some(RestoreAction::FlattenWorkspace {
                    workspace: ws.name.clone(),
                })
            } else {
                None
            }
        })
    }

    fn resolve_move_window_actions<'a>(
        resolution: &'a WindowResolution,
        live_windows: &'a [AerospaceWindow],
    ) -> impl Iterator<Item = RestoreAction> + 'a {
        let live_workspace_lookup: HashMap<_, _> = live_windows
            .iter()
            .map(|w| (&w.window_id, &w.workspace))
            .collect();

        resolution
            .resolved_windows
            .iter()
            .filter_map(move |mapping| {
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
    }

    fn resolve(
        workspaces: &[StageWorkspace],
        resolution: &WindowResolution,
        live_windows: &[AerospaceWindow],
    ) -> RestorePlan {
        let mut actions: Vec<RestoreAction> = Vec::with_capacity(
            workspaces.len() + // Change layout
            workspaces.len() + // Flatten workspace
            resolution.resolved_windows.len(), // Move To Workspace,
        );

        actions.extend(RestorePlan::resolve_move_window_actions(
            resolution,
            live_windows,
        ));
        actions.extend(RestorePlan::resolve_flatten_workspace_actions(workspaces));
        actions.extend(RestorePlan::resolve_layout_actions(workspaces));

        RestorePlan { plan: actions }
    }

    fn restore(&self, aerospace: &mut Aerospace) -> Result<()> {
        for action in &self.plan {
            action.execute(aerospace)?;
        }

        Ok(())
    }
}

pub fn restore_stage(aerospace: &mut Aerospace, stage: &Stage) -> Result<()> {
    let live_windows = aerospace.list_windows()?;

    let resolution = WindowResolution::resolve(stage, &live_windows);

    let restore_plan = RestorePlan::resolve(&stage.workspaces, &resolution, &live_windows);
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
    use crate::{
        restore::{resolution::WindowResolution, rules::workspace, types::ResolvedWindowMatch},
        stage::StageWorkspaceLayout,
    };

    #[test]
    fn skips_windows_already_on_the_target_workspace() {
        let workspaces = Vec::new();

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

        let plan = RestorePlan::resolve(&workspaces, &resolution, &live_windows);

        assert_eq!(plan.plan.len(), 1);
        match &plan.plan[0] {
            RestoreAction::MoveToWorkspace {
                workspace,
                target_window,
            } => {
                assert_eq!(workspace, "2");
                assert_eq!(*target_window, 2);
            }
            _ => panic!("Expected MoveToWorkspace action, but got ChangeLayout"),
        }
    }

    #[test]
    fn adds_layout_change_for_workspace() {
        let workspaces = vec![StageWorkspace {
            name: "1".to_string(),
            layout: Some(StageWorkspaceLayout::HAccordion),
            windows: Vec::new(),
        }];

        let live_windows = Vec::new();
        let resolution = WindowResolution {
            resolved_windows: Vec::new(),
            pending_targets: Vec::new(),
            unresolved_windows: Vec::new(),
        };

        let plan = RestorePlan::resolve(&workspaces, &resolution, &live_windows);

        assert_eq!(plan.plan.len(), 2);
        match &plan.plan[0] {
            RestoreAction::FlattenWorkspace { workspace } => {
                assert_eq!(workspace, "1");
            }
            _ => panic!("Expected MoveToWorkspace action, but got ChangeLayout"),
        }
        match &plan.plan[1] {
            RestoreAction::ChangeLayout {
                workspace,
                target_layout,
            } => {
                assert_eq!(workspace, "1");
                assert_eq!(*target_layout, AerospaceLayout::HAccordion);
            }
            _ => panic!("Expected MoveToWorkspace action, but got ChangeLayout"),
        }
    }
}
