use crate::{
    aerospace::{AerospaceWindow, AerospaceWindowId, AerospaceWorkspaceId},
    arrangement::{Arrangement, ArrangementWindow, ArrangementWorkspace},
};
use std::vec;
use strsim::normalized_levenshtein;

struct ResolveTarget<'a> {
    target_workspace: &'a ArrangementWorkspace,
    target_window: &'a ArrangementWindow,
}

impl<'a> ResolveTarget<'a> {
    fn matches_window_app(&self, window: &AerospaceWindow) -> bool {
        self.target_window.app == window.app_name
            || self.target_window.bundle_id == window.app_bundle_id
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedWindowMatch {
    pub target_workspace: AerospaceWorkspaceId,
    pub window_id: AerospaceWindowId,
}

#[derive(Debug)]
pub struct UnresolvedWindow {
    pub window_id: AerospaceWindowId,
}

trait WindowResolverRule {
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch>;
}

struct UniqueBundleIdResolverRule {}

impl WindowResolverRule for UniqueBundleIdResolverRule {
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch> {
        let mut bundle_id_matches = windows
            .iter()
            .filter(|window| window.app_bundle_id == target.target_window.bundle_id);

        match (bundle_id_matches.next(), bundle_id_matches.next()) {
            (Some(first_match), None) => Some(ResolvedWindowMatch {
                target_workspace: target.target_workspace.name.clone(),
                window_id: first_match.window_id,
            }),
            _ => None,
        }
    }
}

struct UniqueAppNameResolverRule {}

impl WindowResolverRule for UniqueAppNameResolverRule {
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch> {
        let mut bundle_id_matches = windows
            .iter()
            .filter(|window| window.app_name == target.target_window.app);

        match (bundle_id_matches.next(), bundle_id_matches.next()) {
            (Some(first_match), None) => Some(ResolvedWindowMatch {
                target_workspace: target.target_workspace.name.clone(),
                window_id: first_match.window_id,
            }),
            _ => None,
        }
    }
}

struct ExactTitleMatchResolverRule {}

impl WindowResolverRule for ExactTitleMatchResolverRule {
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch> {
        let target_title = target.target_window.title.as_ref()?;

        let mut bundle_id_matches = windows.iter().filter(|window| {
            if window.window_title != *target_title {
                return false;
            }

            target.matches_window_app(window)
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

/// Matches against windows which are already on the target workspace if they are
/// from the target application.
/// The rule only matches if there is exactly one window of this app on the target workspace
struct TargetWorkspaceResolverRule {}

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

struct TitleSimilarityResolverRule {
    threshold: f64,
}

impl WindowResolverRule for TitleSimilarityResolverRule {
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch> {
        let target_title = target.target_window.title.clone()?;

        let app_window_candidates = windows
            .iter()
            .filter(|window| target.matches_window_app(window));

        let mut ranked_matches: Vec<(&AerospaceWindow, f64)> = app_window_candidates
            .map(|window| {
                let score = normalized_levenshtein(&target_title, &window.window_title);

                return (window, score);
            })
            .filter(|(_, score)| *score >= self.threshold)
            .collect();

        ranked_matches.sort_by(|a, b| a.1.total_cmp(&b.1));
        ranked_matches.reverse();

        let first_match = ranked_matches.first()?;

        return Some(ResolvedWindowMatch {
            target_workspace: target.target_workspace.name.clone(),
            window_id: first_match.0.window_id,
        });
    }
}

struct WindowResolver {
    rules: Vec<Box<dyn WindowResolverRule>>,
}

impl Default for WindowResolver {
    fn default() -> Self {
        WindowResolver {
            rules: vec![
                Box::new(ExactTitleMatchResolverRule {}),
                Box::new(TitleSimilarityResolverRule { threshold: 0.5 }),
                Box::new(UniqueBundleIdResolverRule {}),
                Box::new(UniqueAppNameResolverRule {}),
                Box::new(TargetWorkspaceResolverRule {}),
            ],
        }
    }
}

impl WindowResolver {
    fn resolve(
        &self,
        arrangement: &Arrangement,
        windows: &[AerospaceWindow],
    ) -> (Vec<ResolvedWindowMatch>, Vec<UnresolvedWindow>) {
        let mut pending_targets: Vec<ResolveTarget> = arrangement
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
        let mut resolved_matches = Vec::new();

        loop {
            let initial_pending_count = pending_targets.len();

            for rule in &self.rules {
                let mut remaining_targets = Vec::new();
                for target in pending_targets {
                    let avail_slice: Vec<AerospaceWindow> =
                        available_windows.iter().map(|w| w.clone()).collect();

                    if let Some(matched) = rule.match_window(&avail_slice, &target) {
                        available_windows.retain(|windows| windows.window_id != matched.window_id);
                        resolved_matches.push(matched);
                    } else {
                        remaining_targets.push(target);
                    }
                }

                pending_targets = remaining_targets;

                if pending_targets.is_empty() {
                    break;
                }
            }

            if pending_targets.len() == initial_pending_count {
                break;
            }
        }

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
    pub fn resolve(arrangement: &Arrangement, windows: &[AerospaceWindow]) -> Self {
        let resolver = WindowResolver::default();

        let (resolved_window_matches, unresolved_windows) = resolver.resolve(arrangement, &windows);

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
