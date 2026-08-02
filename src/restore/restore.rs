use crate::{
    aerospace::{Aerospace, AerospaceWindowId},
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
        aerospace: &Aerospace,
        resolution: &WindowResolution,
    ) -> Result<RestorePlan, String> {
        let windows = aerospace.list_windows()?;

        let actions = windows
            .iter()
            .filter_map(|window| {
                let Some(mapping) = resolution
                    .resolved_windows
                    .iter()
                    .find(|matched| matched.window_id == window.window_id)
                else {
                    return None;
                };

                Some((window, mapping))
            })
            .filter(|(window, mapping)| window.workspace != mapping.target_workspace)
            .map(|(window, mapping)| RestoreAction::MoveToWorkspace {
                workspace: mapping.target_workspace.clone(),
                target_window: window.window_id.clone(),
            })
            .collect();

        Ok(RestorePlan { plan: actions })
    }
}

pub fn restore_arrangement(aerospace: &Aerospace, arrangement: &Arrangement) -> Result<(), String> {
    let windows = dbg!(aerospace.list_windows()?);

    let resolution = dbg!(WindowResolution::resolve(arrangement, windows));

    let restore_plan = dbg!(RestorePlan::resolve(aerospace, &resolution));

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_arrangement_successfully() {}
}
