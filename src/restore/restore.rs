use std::collections::HashMap;

use crate::{
    aerospace::{Aerospace, AerospaceWindow, AerospaceWindowId, AerospaceWorkspaceId},
    arrangement::{Arrangement, ArrangementWindow},
    restore::resolution::WindowResolution,
};

#[derive(Debug)]
enum RestoreAction {
    MoveToWorkspace {
        workspace: String,
        target_window: AerospaceWindowId,
    },
}

#[derive(Debug)]
struct RestorePlan {
    plan: Vec<RestoreAction>,
}

impl RestorePlan {
    fn resolve(
        resolution: &WindowResolution,
        live_windows: &[AerospaceWindow],
    ) -> Result<RestorePlan, String> {
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

        Ok(RestorePlan { plan: actions })
    }
}

pub fn restore_arrangement(aerospace: &Aerospace, arrangement: &Arrangement) -> Result<(), String> {
    let live_windows = dbg!(aerospace.list_windows()?);

    let resolution = dbg!(WindowResolution::resolve(arrangement, &live_windows));

    let restore_plan = dbg!(RestorePlan::resolve(&resolution, &live_windows));

    // todo!("Restore plan using aerospace")
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_arrangement_successfully() {}
}
