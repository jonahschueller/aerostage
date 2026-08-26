use crate::aerospace::Aerospace;
use crate::stage::{Stage, StageWindow, StageWorkspace};

pub fn capture_stage(aerospace: &Aerospace, name: Option<&str>) -> Result<Stage, String> {
    let workspaces = aerospace.list_workspaces().unwrap();
    let windows = aerospace.list_windows().unwrap();

    let workspaces: Vec<StageWorkspace> = workspaces
        .iter()
        .map(|workspace| {
            let name = workspace.workspace.to_string();
            let windows_of_workspace = windows
                .iter()
                .filter(|window| window.workspace == workspace.workspace)
                .map(|window| StageWindow {
                    app: Some(window.app_name.clone()),
                    title: Some(window.window_title.clone()),
                    bundle_id: Some(window.app_bundle_id.clone()),
                })
                .collect();

            StageWorkspace {
                name,
                windows: windows_of_workspace,
            }
        })
        .filter(|workspace| !workspace.windows.is_empty())
        .collect();

    let stage = Stage {
        name: name.map(|n| n.to_string()),
        description: None,
        workspaces: workspaces,
        default_workspace: None,
    };

    Ok(stage)
}
