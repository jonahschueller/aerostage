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
use std::vec;
struct WindowResolver {
    fallback_workspace: Option<AerospaceWorkspaceId>,
    rules: Vec<Box<dyn WindowResolverRule>>,
}

impl WindowResolver {
    fn new(fallback_workspace: Option<String>) -> Self {
        WindowResolver {
            fallback_workspace: fallback_workspace,
            rules: vec![
                Box::new(TitleMatchResolverRule {}),
                Box::new(TitleSimilarityResolverRule { threshold: 0.75 }),
                Box::new(TargetWorkspaceResolverRule {}),
                Box::new(UniqueBundleIdResolverRule {}),
                Box::new(UniqueAppNameResolverRule {}),
            ],
        }
    }
}

impl WindowResolver {
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
    ) {
        let Some(fallback_workspace) = self.fallback_workspace.clone() else {
            return;
        };

        let mut remaining_windows: Vec<ResolvedWindowMatch> = available_windows
            .iter()
            .map(|win| ResolvedWindowMatch {
                target_workspace: fallback_workspace.clone(),
                window_id: win.window_id,
            })
            .collect();

        resolved_matches.append(&mut remaining_windows);
        available_windows.clear();
    }

    fn resolve(
        &self,
        stage: &Stage,
        windows: &[AerospaceWindow],
    ) -> (Vec<ResolvedWindowMatch>, Vec<UnresolvedWindow>) {
        let mut pending_targets: Vec<ResolveTarget> = stage
            .workspaces
            .iter()
            .flat_map(|workspace| {
                workspace.windows.iter().map(move |window| ResolveTarget {
                    target_workspace: &workspace,
                    target_window: &window,
                })
            })
            .collect();

        let mut available_windows = windows.to_vec();

        let mut resolved_matches =
            self.apply_resolver_rules(&mut pending_targets, &mut available_windows);

        self.apply_optional_fallback_resolver(&mut resolved_matches, &mut available_windows);

        let unresolved_windows = available_windows
            .iter()
            .map(|window| UnresolvedWindow {
                window_id: window.window_id,
            })
            .collect();
        (resolved_matches, unresolved_windows)
    }
}

#[derive(Debug)]
pub struct WindowResolution {
    pub resolved_windows: Vec<ResolvedWindowMatch>,
    pub unresolved_windows: Vec<UnresolvedWindow>,
}

impl WindowResolution {
    pub fn resolve(stage: &Stage, windows: &[AerospaceWindow]) -> Self {
        let resolver = WindowResolver::new(stage.default_workspace.clone());

        let (resolved_window_matches, unresolved_windows) = resolver.resolve(stage, &windows);

        WindowResolution {
            resolved_windows: resolved_window_matches,
            unresolved_windows: unresolved_windows,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::aerospace::tests;

    struct MockResolverRule {
        resolved_window: Option<ResolvedWindowMatch>,
    }

    impl WindowResolverRule for MockResolverRule {
        fn match_window(
            &self,
            windows: &[AerospaceWindow],
            target: &ResolveTarget,
        ) -> Option<ResolvedWindowMatch> {
            self.resolved_window.clone()
        }
    }
}
