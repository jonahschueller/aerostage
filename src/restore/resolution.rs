use crate::{
    aerospace::{AerospaceWindow, AerospaceWindowId, AerospaceWorkspaceId},
    arrangement::{Arrangement, ArrangementWindow, ArrangementWorkspace},
};
use std::vec;

struct ResolveTarget<'a> {
    target_workspace: &'a ArrangementWorkspace,
    target_window: &'a ArrangementWindow,
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

struct WindowResolver {
    rules: Vec<Box<dyn WindowResolverRule>>,
}

impl Default for WindowResolver {
    fn default() -> Self {
        WindowResolver {
            rules: vec![Box::new(UniqueBundleIdResolverRule {})],
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

        for rule in &self.rules {
            let mut remaining_targets = Vec::new();
            for target in pending_targets {
                if let Some(matched) = rule.match_window(&available_windows, &target) {
                    available_windows.retain(|windows| windows.window_id != matched.window_id);
                    resolved_matches.push(matched);
                } else {
                    remaining_targets.push(target);
                }
            }

            pending_targets = remaining_targets;
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
    resolved_windows: Vec<ResolvedWindowMatch>,
    unresolved_windows: Vec<UnresolvedWindow>,
}

impl WindowResolution {
    pub fn resolve(arrangement: &Arrangement, windows: Vec<AerospaceWindow>) -> Self {
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
