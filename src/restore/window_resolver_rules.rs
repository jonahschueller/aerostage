use crate::{
    aerospace::{AerospaceWindow, AerospaceWindowId, AerospaceWorkspaceId},
    arrangement::{ArrangementWindow, ArrangementWorkspace},
};

pub struct ResolveTarget<'a> {
    pub target_workspace: &'a ArrangementWorkspace,
    pub target_window: &'a ArrangementWindow,
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

pub trait WindowResolverRule {
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch>;
}

pub struct UniqueBundleIdResolverRule {}

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

pub struct UniqueAppNameResolverRule {}

impl WindowResolverRule for UniqueAppNameResolverRule {
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch> {
        let mut matches = windows
            .iter()
            .filter(|window| window.app_name == target.target_window.app);

        match (matches.next(), matches.next()) {
            (Some(first_match), None) => Some(ResolvedWindowMatch {
                target_workspace: target.target_workspace.name.clone(),
                window_id: first_match.window_id,
            }),
            _ => None,
        }
    }
}

pub struct TitleMatchResolverRule {}

impl WindowResolverRule for TitleMatchResolverRule {
    fn match_window(
        &self,
        windows: &[AerospaceWindow],
        target: &ResolveTarget,
    ) -> Option<ResolvedWindowMatch> {
        let target_title = target.target_window.title.as_ref()?;

        let title_regex = regex::RegexBuilder::new(target_title)
            .case_insensitive(true)
            .build()
            .ok()?;

        let mut matches = windows
            .iter()
            .filter(|window| target.matches_window_app(window))
            .filter(|window| {
                target_title.to_lowercase() == window.window_title.to_lowercase()
                    || title_regex.is_match(&window.window_title.to_lowercase())
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

pub struct TitleSimilarityResolverRule {
    pub threshold: f64,
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
                let score = strsim::normalized_levenshtein(
                    &target_title.to_lowercase(),
                    &window.window_title.to_lowercase(),
                );

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
