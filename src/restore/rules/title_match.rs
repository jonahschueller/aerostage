use crate::{
    aerospace::AerospaceWindow,
    restore::{
        rule::WindowResolverRule,
        types::{ResolveTarget, ResolvedWindowMatch},
    },
};

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
