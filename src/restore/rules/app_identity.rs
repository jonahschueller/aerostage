use crate::{
    aerospace::AerospaceWindow,
    restore::{
        rule::WindowResolverRule,
        types::{ResolveTarget, ResolvedWindowMatch},
    },
};

pub struct UniqueBundleIdResolverRule {}

impl WindowResolverRule for UniqueBundleIdResolverRule {
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch> {
        let mut bundle_id_matches = windows.iter().filter(|window| {
            target
                .target_window
                .bundle_id
                .as_deref()
                .map_or(false, |bundle_id| window.app_bundle_id == bundle_id)
        });

        match (bundle_id_matches.next(), bundle_id_matches.next()) {
            (Some(first_match), None) => Some(ResolvedWindowMatch {
                target_workspace: target.target_workspace.name.clone(),
                window_id: first_match.window_id,
            }),
            _ => None,
        }
    }
}

pub struct UniqueAppNameResolverRule {}

impl WindowResolverRule for UniqueAppNameResolverRule {
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch> {
        let mut matches = windows.iter().filter(|window| {
            target
                .target_window
                .app
                .as_deref()
                .map_or(false, |app| window.app_name == app)
        });

        match (matches.next(), matches.next()) {
            (Some(first_match), None) => Some(ResolvedWindowMatch {
                target_workspace: target.target_workspace.name.clone(),
                window_id: first_match.window_id,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
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
}
