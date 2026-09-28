use crate::{
    aerospace::AerospaceWindow,
    restore::{
        rule::{Candidate, WindowResolverRule},
        types::{ResolveTarget, present_text},
    },
};

pub struct UniqueBundleIdResolverRule {}

impl WindowResolverRule for UniqueBundleIdResolverRule {
    fn propose(&self, windows: &[AerospaceWindow], target: &ResolveTarget<'_>) -> Vec<Candidate> {
        let Some(bundle_id) = present_text(target.target_window.bundle_id.as_deref()) else {
            return Vec::new();
        };

        windows
            .iter()
            .filter(|window| window.app_bundle_id == bundle_id)
            .map(|window| Candidate {
                window_id: window.window_id,
                score: 0.2,
            })
            .collect()
    }
}

pub struct UniqueAppNameResolverRule {}

impl WindowResolverRule for UniqueAppNameResolverRule {
    fn propose(&self, windows: &[AerospaceWindow], target: &ResolveTarget<'_>) -> Vec<Candidate> {
        let Some(app) = present_text(target.target_window.app.as_deref()) else {
            return Vec::new();
        };

        windows
            .iter()
            .filter(|window| window.app_name == app)
            .map(|window| Candidate {
                window_id: window.window_id,
                score: 0.1,
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
    // UniqueBundleIdResolverRule Tests
    // ==========================================

    #[test]
    fn test_bundle_id_single_match_returns_resolved_window() {
        let (target_window, target_workspace) =
            create_target("Test App", "com.example.app", "workspace-1");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(10)
                .with_bundle_id("com.example.app"),
            AerospaceWindow::dummy()
                .with_window_id(20)
                .with_bundle_id("com.other.app"),
        ];

        let resolver = UniqueBundleIdResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(
            result,
            Some(ResolvedWindowMatch {
                target_workspace: "workspace-1".to_string(),
                window_id: 10,
            })
        );
    }

    #[test]
    fn test_bundle_id_no_match_returns_none() {
        let (target_window, target_workspace) =
            create_target("Test App", "com.example.app", "workspace-1");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![AerospaceWindow::dummy().with_bundle_id("com.different.app")];

        let resolver = UniqueBundleIdResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
    }

    #[test]
    fn test_bundle_id_multiple_matches_returns_none() {
        let (target_window, target_workspace) =
            create_target("Test App", "com.example.app", "workspace-1");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        // Multiple windows matching the same bundle ID violate uniqueness
        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(10)
                .with_bundle_id("com.example.app"),
            AerospaceWindow::dummy()
                .with_window_id(20)
                .with_bundle_id("com.example.app"),
        ];

        let resolver = UniqueBundleIdResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
        assert_eq!(resolver.propose(&windows, &target).len(), 2);
    }

    #[test]
    fn test_bundle_id_empty_windows_returns_none() {
        let (target_window, target_workspace) =
            create_target("Test App", "com.example.app", "workspace-1");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let resolver = UniqueBundleIdResolverRule {};
        let result = resolver.match_window(&[], &target);

        assert_eq!(result, None);
    }

    // ==========================================
    // UniqueAppNameResolverRule Tests
    // ==========================================

    #[test]
    fn test_app_name_single_match_returns_resolved_window() {
        let (target_window, target_workspace) =
            create_target("Safari", "com.apple.Safari", "main-space");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(42)
                .with_app_name("Safari"),
            AerospaceWindow::dummy()
                .with_window_id(43)
                .with_app_name("Firefox"),
        ];

        let resolver = UniqueAppNameResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(
            result,
            Some(ResolvedWindowMatch {
                target_workspace: "main-space".to_string(),
                window_id: 42,
            })
        );
    }

    #[test]
    fn test_app_name_no_match_returns_none() {
        let (target_window, target_workspace) =
            create_target("Safari", "com.apple.Safari", "main-space");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![AerospaceWindow::dummy().with_app_name("Chrome")];

        let resolver = UniqueAppNameResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
    }

    #[test]
    fn test_app_name_multiple_matches_returns_none() {
        let (target_window, target_workspace) =
            create_target("Terminal", "com.apple.Terminal", "dev");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        // Multiple terminal instances running
        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(101)
                .with_app_name("Terminal"),
            AerospaceWindow::dummy()
                .with_window_id(102)
                .with_app_name("Terminal"),
        ];

        let resolver = UniqueAppNameResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
        assert_eq!(resolver.propose(&windows, &target).len(), 2);
    }

    #[test]
    fn test_app_name_case_sensitivity() {
        let (target_window, target_workspace) =
            create_target("safari", "com.apple.Safari", "main-space");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        // Standard string comparison should be case sensitive ("safari" != "Safari")
        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(1)
                .with_app_name("Safari"),
        ];

        let resolver = UniqueAppNameResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
    }

    #[test]
    fn test_bundle_id_empty_string_returns_none() {
        let window = StageWindow::dummy().with_bundle_id("");
        let workspace = StageWorkspace::dummy().with_name("workspace-1");
        let target = ResolveTarget {
            target_window: &window,
            target_workspace: &workspace,
        };

        let windows = vec![AerospaceWindow::dummy().with_bundle_id("")];

        let resolver = UniqueBundleIdResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
    }
}
