use crate::{
    aerospace::AerospaceWindow,
    restore::{
        rule::WindowResolverRule,
        types::{ResolveTarget, ResolvedWindowMatch},
    },
};

/// Matches against windows which are already on the target workspace if they are
/// from the target application.
/// The rule only matches if there is exactly one window of this app on the target workspace
pub struct TargetWorkspaceResolverRule {}

impl WindowResolverRule for TargetWorkspaceResolverRule {
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch> {
        let target_workspace = target.target_workspace;

        let mut workspace_matches = windows
            .iter()
            .filter(|window| window.workspace == target_workspace.name)
            .filter(|window| target.matches_window_app(window));

        match (workspace_matches.next(), workspace_matches.next()) {
            (Some(first_match), None) => Some(ResolvedWindowMatch {
                target_workspace: target.target_workspace.name.clone(),
                window_id: first_match.window_id,
            }),
            _ => None,
        }
    }
}
