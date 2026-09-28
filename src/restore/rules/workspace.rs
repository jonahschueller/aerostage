use crate::{
    aerospace::AerospaceWindow,
    restore::{
        rule::{Candidate, WindowResolverRule},
        types::ResolveTarget,
    },
};

/// Proposes live windows that are already on the target workspace and share
/// the target application. The resolver commits only when that set is unique.
pub struct TargetWorkspaceResolverRule {}

impl WindowResolverRule for TargetWorkspaceResolverRule {
    fn propose(&self, windows: &[AerospaceWindow], target: &ResolveTarget<'_>) -> Vec<Candidate> {
        windows
            .iter()
            .filter(|window| window.workspace == target.target_workspace.name)
            .filter(|window| target.matches_window_app(window))
            .map(|window| Candidate {
                window_id: window.window_id,
                score: 0.4,
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::restore::types::{ResolveTarget, ResolvedWindowMatch};
    use crate::stage::{StageWindow, StageWorkspace};

    // Helper to construct a standard ResolveTarget quickly
    fn create_target(
        app_name: &str,
        bundle_id: &str,
        workspace_name: &str,
    ) -> (StageWindow, StageWorkspace) {
        let window = StageWindow::dummy()
            .with_app(app_name)
            .with_bundle_id(bundle_id);
        let workspace = StageWorkspace::dummy().with_name(workspace_name);
        (window, workspace)
    }

    // ==========================================
    // TargetWorkspaceResolverRule Tests
    // ==========================================

    #[test]
    fn test_single_match_on_target_workspace_returns_resolved_window() {
        let (target_window, target_workspace) =
            create_target("Slack", "com.tinyspeck.slackmacgap", "work");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            // Matching app, matching workspace
            AerospaceWindow::dummy()
                .with_window_id(100)
                .with_app_name("Slack")
                .with_bundle_id("com.tinyspeck.slackmacgap")
                .with_workspace("work"),
            // Matching app, DIFFERENT workspace (ignored)
            AerospaceWindow::dummy()
                .with_window_id(101)
                .with_app_name("Slack")
                .with_bundle_id("com.tinyspeck.slackmacgap")
                .with_workspace("personal"),
        ];

        let resolver = TargetWorkspaceResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(
            result,
            Some(ResolvedWindowMatch {
                target_workspace: "work".to_string(),
                window_id: 100,
            })
        );
    }

    #[test]
    fn test_multiple_matches_on_target_workspace_returns_none() {
        let (target_window, target_workspace) =
            create_target("Terminal", "com.apple.Terminal", "dev");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        // Two instances of the target app on the target workspace
        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(1)
                .with_app_name("Terminal")
                .with_bundle_id("com.apple.Terminal")
                .with_workspace("dev"),
            AerospaceWindow::dummy()
                .with_window_id(2)
                .with_app_name("Terminal")
                .with_bundle_id("com.apple.Terminal")
                .with_workspace("dev"),
        ];

        let resolver = TargetWorkspaceResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
        assert_eq!(resolver.propose(&windows, &target).len(), 2);
    }

    #[test]
    fn test_app_matches_but_on_wrong_workspace_returns_none() {
        let (target_window, target_workspace) =
            create_target("Browser", "com.browser.app", "workspace-1");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(50)
                .with_app_name("Browser")
                .with_bundle_id("com.browser.app")
                .with_workspace("workspace-2"), // Not on target_workspace
        ];

        let resolver = TargetWorkspaceResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
    }

    #[test]
    fn test_correct_workspace_but_different_app_returns_none() {
        let (target_window, target_workspace) = create_target("Notes", "com.apple.Notes", "main");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(77)
                .with_app_name("Calculator")
                .with_bundle_id("com.apple.calculator")
                .with_workspace("main"), // Right workspace, wrong app
        ];

        let resolver = TargetWorkspaceResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
    }

    #[test]
    fn test_empty_windows_returns_none() {
        let (target_window, target_workspace) = create_target("App", "com.app", "1");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let resolver = TargetWorkspaceResolverRule {};
        let result = resolver.match_window(&[], &target);

        assert_eq!(result, None);
    }
}
