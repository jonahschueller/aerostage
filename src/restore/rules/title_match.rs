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

        let title_regex = regex::RegexBuilder::new(&regex::escape(target_title))
            .case_insensitive(true)
            .build()
            .ok();

        let mut matches = windows
            .iter()
            .filter(|window| target.matches_window_app(window))
            .filter(|window| {
                target_title.to_lowercase() == window.window_title.to_lowercase()
                    || title_regex
                        .as_ref()
                        .map_or(false, |re| re.is_match(&window.window_title))
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

        let target_title_lowercase = target_title.to_lowercase();

        let mut ranked_matches: Vec<(&AerospaceWindow, f64)> = app_window_candidates
            .map(|window| {
                let score = strsim::normalized_levenshtein(
                    &target_title_lowercase,
                    &window.window_title.to_lowercase(),
                );

                return (window, score);
            })
            .filter(|(_, score)| *score >= self.threshold)
            .collect();

        ranked_matches.sort_by(|a, b| b.1.total_cmp(&a.1));

        let first_match = ranked_matches.first()?;

        return Some(ResolvedWindowMatch {
            target_workspace: target.target_workspace.name.clone(),
            window_id: first_match.0.window_id,
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::arrangement::{ArrangementWindow, ArrangementWorkspace};

    fn create_target(
        app_name: &str,
        bundle_id: &str,
        title: Option<&str>,
        workspace_name: &str,
    ) -> (ArrangementWindow, ArrangementWorkspace) {
        let window = ArrangementWindow::dummy()
            .with_app(app_name)
            .with_bundle_id(bundle_id)
            .with_title(title);
        let workspace = ArrangementWorkspace::dummy().with_name(workspace_name);
        (window, workspace)
    }

    // ==========================================
    // TitleMatchResolverRule Tests
    // ==========================================

    #[test]
    fn test_title_match_single_exact_match_returns_window() {
        let (target_window, target_workspace) = create_target(
            "Code",
            "com.microsoft.VSCode",
            Some("main.rs"),
            "workspace-1",
        );
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(1)
                .with_app_name("Code")
                .with_bundle_id("com.microsoft.VSCode")
                .with_window_title("main.rs"),
        ];

        let resolver = TitleMatchResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(
            result,
            Some(ResolvedWindowMatch {
                target_workspace: "workspace-1".to_string(),
                window_id: 1,
            })
        );
    }

    #[test]
    fn test_title_match_case_insensitive_match() {
        let (target_window, target_workspace) = create_target(
            "Code",
            "com.microsoft.VSCode",
            Some("MAIN.RS"),
            "workspace-1",
        );
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(1)
                .with_app_name("Code")
                .with_bundle_id("com.microsoft.VSCode")
                .with_window_title("main.rs"),
        ];

        let resolver = TitleMatchResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(
            result,
            Some(ResolvedWindowMatch {
                target_workspace: "workspace-1".to_string(),
                window_id: 1,
            })
        );
    }

    #[test]
    fn test_title_match_regex_substring_match() {
        let (target_window, target_workspace) =
            create_target("Browser", "com.browser.app", Some("GitHub"), "workspace-1");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        // RegEx "GitHub" should match "Dashboard - GitHub - Safari"
        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(42)
                .with_app_name("Browser")
                .with_bundle_id("com.browser.app")
                .with_window_title("Dashboard - GitHub - Safari"),
        ];

        let resolver = TitleMatchResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(
            result,
            Some(ResolvedWindowMatch {
                target_workspace: "workspace-1".to_string(),
                window_id: 42,
            })
        );
    }

    #[test]
    fn test_title_match_target_title_is_none_returns_none() {
        let (target_window, target_workspace) =
            create_target("Terminal", "com.apple.Terminal", None, "dev");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            AerospaceWindow::dummy()
                .with_app_name("Terminal")
                .with_bundle_id("com.apple.Terminal")
                .with_window_title("zsh"),
        ];

        let resolver = TitleMatchResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
    }

    #[test]
    fn test_title_match_multiple_matches_returns_none() {
        let (target_window, target_workspace) =
            create_target("Code", "com.microsoft.VSCode", Some("index.ts"), "dev");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(1)
                .with_app_name("Code")
                .with_bundle_id("com.microsoft.VSCode")
                .with_window_title("index.ts"),
            AerospaceWindow::dummy()
                .with_window_id(2)
                .with_app_name("Code")
                .with_bundle_id("com.microsoft.VSCode")
                .with_window_title("index.ts"),
        ];

        let resolver = TitleMatchResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
    }

    // ==========================================
    // TitleSimilarityResolverRule Tests
    // ==========================================

    #[test]
    fn test_similarity_selects_highest_scoring_match() {
        let (target_window, target_workspace) = create_target(
            "Notes",
            "com.apple.Notes",
            Some("Project Ideas 2026"),
            "work",
        );
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            // Low similarity
            AerospaceWindow::dummy()
                .with_window_id(10)
                .with_app_name("Notes")
                .with_bundle_id("com.apple.Notes")
                .with_window_title("Random Scratchpad"),
            // High similarity
            AerospaceWindow::dummy()
                .with_window_id(20)
                .with_app_name("Notes")
                .with_bundle_id("com.apple.Notes")
                .with_window_title("Project Ideas 2025"),
        ];

        let resolver = TitleSimilarityResolverRule { threshold: 0.6 };
        let result = resolver.match_window(&windows, &target);

        assert_eq!(
            result,
            Some(ResolvedWindowMatch {
                target_workspace: "work".to_string(),
                window_id: 20,
            })
        );
    }

    #[test]
    fn test_similarity_below_threshold_returns_none() {
        let (target_window, target_workspace) =
            create_target("Browser", "com.browser", Some("Rust Documentation"), "1");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            AerospaceWindow::dummy()
                .with_app_name("Browser")
                .with_bundle_id("com.browser")
                .with_window_title("Cooking Recipes"),
        ];

        // Set high threshold that low-similarity title won't hit
        let resolver = TitleSimilarityResolverRule { threshold: 0.8 };
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
    }

    #[test]
    fn test_similarity_ignores_different_app() {
        let (target_window, target_workspace) =
            create_target("TargetApp", "com.target.app", Some("Identical Title"), "1");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            // Exact title match, but wrong app
            AerospaceWindow::dummy()
                .with_window_id(99)
                .with_app_name("OtherApp")
                .with_bundle_id("com.other.app")
                .with_window_title("Identical Title"),
        ];

        let resolver = TitleSimilarityResolverRule { threshold: 0.5 };
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
    }

    #[test]
    fn test_similarity_target_title_is_none_returns_none() {
        let (target_window, target_workspace) =
            create_target("Notes", "com.apple.Notes", None, "1");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            AerospaceWindow::dummy()
                .with_app_name("Notes")
                .with_bundle_id("com.apple.Notes")
                .with_window_title("Some Title"),
        ];

        let resolver = TitleSimilarityResolverRule { threshold: 0.1 };
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
    }
}
