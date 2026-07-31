use crate::aerospace::Aerospace;
use crate::arrangement::{Arrangement, ArrangementWindow, ArrangementWorkspace};

pub fn capture_arrangement(aerospace: &Aerospace, name: &str) -> Result<Arrangement, String> {
    let workspaces = aerospace.list_workspaces().unwrap();
    let windows = aerospace.list_windows().unwrap();

    let workspaces: Vec<ArrangementWorkspace> = workspaces
        .iter()
        .map(|workspace| {
            let name = workspace.workspace.to_string();
            let windows_of_workspace = windows
                .iter()
                .filter(|window| window.workspace == workspace.workspace)
                .map(|window| ArrangementWindow {
                    app: window.app_name.clone(),
                    title: Some(window.window_title.clone()),
                })
                .collect();

            ArrangementWorkspace {
                name,
                windows: windows_of_workspace,
            }
        })
        .collect();

    let arrangement = Arrangement {
        name: name.to_string(),
        description: None,
        workspaces: workspaces,
    };

    Ok(arrangement)
}
