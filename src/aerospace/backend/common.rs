use std::fmt::Display;

pub enum AerospaceCommand {
    ListApps,
    ListWorkspaces,
    ListMonitors,
    ListWindows,
    MoveNodeToWorkspace,
    ChangeLayout,
    FlattenWorkspaceTree,
}

impl Display for AerospaceCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AerospaceCommand::ListApps => "list-apps",
            AerospaceCommand::ListWorkspaces => "list-workspaces",
            AerospaceCommand::ListMonitors => "list-monitors",
            AerospaceCommand::ListWindows => "list-windows",
            AerospaceCommand::MoveNodeToWorkspace => "move-node-to-workspace",
            AerospaceCommand::ChangeLayout => "layout",
            AerospaceCommand::FlattenWorkspaceTree => "flatten-workspace-tree",
        };
        write!(f, "{s}")
    }
}

pub fn format_aerospace(included_fields: &[&str]) -> String {
    included_fields
        .iter()
        .map(|field| format!("%{{{}}}", field))
        .collect::<Vec<_>>()
        .join(" ")
}
