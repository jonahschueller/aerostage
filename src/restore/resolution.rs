use crate::{
    aerospace::{AerospaceWindow, AerospaceWorkspaceId},
    restore::{
        rule::WindowResolverRule,
        rules::{
            TargetWorkspaceResolverRule, TitleMatchResolverRule, TitleSimilarityResolverRule,
            UniqueAppNameResolverRule, UniqueBundleIdResolverRule,
        },
        types::{ResolveTarget, ResolvedWindowMatch, UnresolvedWindow},
    },
    stage::Stage,
};

struct WindowResolver {
    fallback_workspace: Option<AerospaceWorkspaceId>,
    rules: Vec<Box<dyn WindowResolverRule>>,
}

impl WindowResolver {
    fn new(fallback_workspace: Option<String>) -> Self {
        WindowResolver {
            fallback_workspace,
            rules: vec![
                Box::new(TitleMatchResolverRule {}),
                Box::new(TitleSimilarityResolverRule { threshold: 0.75 }),
                Box::new(TargetWorkspaceResolverRule {}),
                Box::new(UniqueBundleIdResolverRule {}),
                Box::new(UniqueAppNameResolverRule {}),
            ],
        }
    }

    fn apply_resolver_rules(
        &self,
        pending_targets: &mut Vec<ResolveTarget>,
        available_windows: &mut Vec<AerospaceWindow>,
    ) -> Vec<ResolvedWindowMatch> {
        let mut resolved_matches = Vec::new();

        loop {
            let initial_pending_count = pending_targets.len();

            for rule in &self.rules {
                let mut remaining_targets = Vec::new();
                for target in pending_targets.drain(..) {
                    let Some(matched) = rule.match_window(available_windows, &target) else {
                        remaining_targets.push(target);
                        continue;
                    };

                    available_windows.retain(|windows| windows.window_id != matched.window_id);
                    resolved_matches.push(matched);
                }

                *pending_targets = remaining_targets;

                if pending_targets.is_empty() {
                    break;
                }
            }

            if pending_targets.len() == initial_pending_count {
                break;
            }
        }

        resolved_matches
    }

    fn apply_optional_fallback_resolver(
        &self,
        resolved_matches: &mut Vec<ResolvedWindowMatch>,
        available_windows: &mut Vec<AerospaceWindow>,
        pending_targets: &[ResolveTarget],
    ) {
        let Some(fallback_workspace) = self.fallback_workspace.as_ref() else {
            return;
        };

        let mut leftover_windows = Vec::new();
        available_windows.retain(|window| {
            if pending_targets
                .iter()
                .any(|target| target.matches_window_app(window))
            {
                return true;
            }

            leftover_windows.push(ResolvedWindowMatch {
                target_workspace: fallback_workspace.clone(),
                window_id: window.window_id,
            });
            false
        });

        resolved_matches.append(&mut leftover_windows);
    }

    fn resolve<'a>(
        &self,
        stage: &'a Stage,
        windows: &[AerospaceWindow],
    ) -> (
        Vec<ResolvedWindowMatch>,
        Vec<ResolveTarget<'a>>,
        Vec<UnresolvedWindow>,
    ) {
        let mut pending_targets: Vec<ResolveTarget> = stage
            .workspaces
            .iter()
            .flat_map(|workspace| {
                workspace.windows.iter().map(move |window| ResolveTarget {
                    target_workspace: workspace,
                    target_window: window,
                })
            })
            .collect();

        let mut available_windows = windows.to_vec();

        let mut resolved_matches =
            self.apply_resolver_rules(&mut pending_targets, &mut available_windows);

        self.apply_optional_fallback_resolver(
            &mut resolved_matches,
            &mut available_windows,
            &pending_targets,
        );

        let unresolved_windows = available_windows
            .iter()
            .map(|window| UnresolvedWindow {
                window_id: window.window_id,
            })
            .collect();

        (resolved_matches, pending_targets, unresolved_windows)
    }
}

pub struct WindowResolution<'a> {
    pub resolved_windows: Vec<ResolvedWindowMatch>,
    pub pending_targets: Vec<ResolveTarget<'a>>,

    // For now, the unresolved_windows are not used. Kept for completeness
    #[allow(unused)]
    pub unresolved_windows: Vec<UnresolvedWindow>,
}

impl<'a> WindowResolution<'a> {
    pub fn resolve(stage: &'a Stage, windows: &[AerospaceWindow]) -> Self {
        let resolver = WindowResolver::new(stage.default_workspace.clone());

        let (resolved_windows, pending_targets, unresolved_windows) =
            resolver.resolve(stage, windows);

        WindowResolution {
            resolved_windows,
            pending_targets,
            unresolved_windows,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::stage::{StageWindow, StageWorkspace};

    fn stage(windows: Vec<(StageWorkspace, Vec<StageWindow>)>, default: Option<&str>) -> Stage {
        Stage {
            name: None,
            description: None,
            workspaces: windows
                .into_iter()
                .map(|(mut workspace, stage_windows)| {
                    workspace.windows = stage_windows;
                    workspace
                })
                .collect(),
            default_workspace: default.map(str::to_string),
        }
    }

    #[test]
    fn fallback_moves_windows_the_stage_did_not_claim() {
        let safari = StageWindow::dummy()
            .with_app("Safari")
            .with_bundle_id("com.apple.Safari")
            .with_title(Some("Inbox"));
        let workspace = StageWorkspace::dummy().with_name("1");
        let stage = stage(vec![(workspace, vec![safari])], Some("9"));

        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(1)
                .with_app_name("Safari")
                .with_bundle_id("com.apple.Safari")
                .with_window_title("Inbox")
                .with_workspace("2"),
            AerospaceWindow::dummy()
                .with_window_id(2)
                .with_app_name("Slack")
                .with_bundle_id("com.tinyspeck.slackmacgap")
                .with_window_title("Team")
                .with_workspace("3"),
        ];

        let resolution = WindowResolution::resolve(&stage, &windows);
        let fallback = resolution
            .resolved_windows
            .iter()
            .find(|matched| matched.window_id == 2)
            .expect("Slack should move to the default workspace");

        assert_eq!(fallback.target_workspace, "9");
        assert!(resolution.pending_targets.is_empty());
    }

    #[test]
    fn fallback_leaves_windows_that_unmatched_stage_entries_may_own() {
        let safari = StageWindow::dummy()
            .with_app("Safari")
            .with_bundle_id("com.apple.Safari")
            .with_title(Some("Missing Tab"));
        let workspace = StageWorkspace::dummy().with_name("1");
        let stage = stage(vec![(workspace, vec![safari])], Some("9"));

        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(1)
                .with_app_name("Safari")
                .with_bundle_id("com.apple.Safari")
                .with_window_title("Tab A")
                .with_workspace("2"),
            AerospaceWindow::dummy()
                .with_window_id(2)
                .with_app_name("Safari")
                .with_bundle_id("com.apple.Safari")
                .with_window_title("Tab B")
                .with_workspace("3"),
            AerospaceWindow::dummy()
                .with_window_id(3)
                .with_app_name("Slack")
                .with_bundle_id("com.tinyspeck.slackmacgap")
                .with_window_title("Team")
                .with_workspace("4"),
        ];

        let resolution = WindowResolution::resolve(&stage, &windows);

        assert!(!resolution.pending_targets.is_empty());
        assert!(
            resolution
                .resolved_windows
                .iter()
                .any(|matched| matched.window_id == 3 && matched.target_workspace == "9")
        );
        assert!(
            !resolution
                .resolved_windows
                .iter()
                .any(|matched| matched.window_id == 1 || matched.window_id == 2)
        );
    }
}
