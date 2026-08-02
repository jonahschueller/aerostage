use crate::{
    aerospace::{AerospaceWindow, AerospaceWindowId, AerospaceWorkspaceId},
    arrangement::{Arrangement, ArrangementWindow, ArrangementWorkspace},
};
use std::vec;

struct ResolveTarget {
    target_workspace: ArrangementWorkspace,
    target_window: ArrangementWindow,
}

#[derive(Clone)]
pub struct ResolvedWindowMatch {
    pub target_workspace: AerospaceWorkspaceId,
    pub window_id: AerospaceWindowId,
}

pub struct UnresolvedWindow {
    pub window_id: AerospaceWindowId,
}

trait WindowResolverRule {
    fn match_window(
        &self,
        windows: &Vec<AerospaceWindow>,
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch>;
}

struct UniqueBundleIdResolverRule {}

impl WindowResolverRule for UniqueBundleIdResolverRule {
    fn match_window(
        &self,
        windows: &Vec<AerospaceWindow>,
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch> {
        let bundle_id_matches: Vec<&AerospaceWindow> = windows
            .iter()
            .filter(|window| window.app_bundle_id == target.target_window.app)
            .collect();

        if bundle_id_matches.len() == 1 {
            Some(ResolvedWindowMatch {
                target_workspace: target.target_workspace.name.clone(),
                window_id: bundle_id_matches.first().unwrap().window_id,
            })
        } else {
            None
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
    fn apply_rule(
        windows: &mut Vec<AerospaceWindow>,
        targets: &Vec<ResolveTarget>,
        rule: &Box<dyn WindowResolverRule>,
    ) -> Vec<ResolvedWindowMatch> {
        targets
            .iter()
            .map(|target| rule.match_window(windows, target))
            .filter(|matched| matched.is_some())
            .map(|matched| matched.unwrap())
            .collect()
    }

    fn resolve(
        &self,
        arrangement: &Arrangement,
        windows: &Vec<AerospaceWindow>,
    ) -> (Vec<ResolvedWindowMatch>, Vec<UnresolvedWindow>) {
        let mut resolve_targets: Vec<ResolveTarget> = arrangement
            .workspaces
            .iter()
            .flat_map(|workspace| {
                workspace.windows.iter().map(|window| ResolveTarget {
                    target_workspace: workspace.clone(),
                    target_window: window.clone(),
                })
            })
            .collect();
        let mut working_window_set = windows.clone();

        for rule in &self.rules {
            WindowResolver::apply_rule(&mut working_window_set, &resolve_targets, rule);
        }

        let mut resolved_window_matches = Vec::new();

        let unresolved_windows = Vec::new();
        (resolved_window_matches, unresolved_windows)
    }
}

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
