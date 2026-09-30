use crate::{
    aerospace::AerospaceWindow,
    restore::{
        rule::{Candidate, WindowResolverRule},
        title::normalize_title,
        types::{ResolveTarget, present_text},
    },
};

fn comparable_title(title: &str, app_name: &str) -> String {
    normalize_title(title, app_name).to_lowercase()
}

fn target_comparable_title(target: &ResolveTarget<'_>) -> Option<String> {
    let title = present_text(target.target_window.title.as_deref())?;
    let app = target.target_window.app.as_deref().unwrap_or("");
    let comparable = comparable_title(title, app);
    present_text(Some(&comparable)).map(str::to_string)
}

pub struct TitleMatchResolverRule {}

impl WindowResolverRule for TitleMatchResolverRule {
    fn propose(&self, windows: &[AerospaceWindow], target: &ResolveTarget<'_>) -> Vec<Candidate> {
        let Some(target_title) = target_comparable_title(target) else {
            return Vec::new();
        };

        windows
            .iter()
            .filter(|window| target.matches_window_app(window))
            .filter(|window| {
                comparable_title(&window.window_title, &window.app_name) == target_title
            })
            .map(|window| Candidate {
                window_id: window.window_id,
                score: 1.0,
            })
            .collect()
    }
}

pub struct TitleSimilarityResolverRule {
    pub threshold: f64,
}

impl WindowResolverRule for TitleSimilarityResolverRule {
    fn propose(&self, windows: &[AerospaceWindow], target: &ResolveTarget<'_>) -> Vec<Candidate> {
        let Some(target_title) = target_comparable_title(target) else {
            return Vec::new();
        };

        windows
            .iter()
            .filter(|window| target.matches_window_app(window))
            .filter_map(|window| {
                let score = strsim::normalized_levenshtein(
                    &target_title,
                    &comparable_title(&window.window_title, &window.app_name),
                );
                (score >= self.threshold).then_some(Candidate {
                    window_id: window.window_id,
                    score,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::restore::types::{ResolveTarget, ResolvedWindowMatch};
    use crate::stage::{StageWindow, StageWorkspace};

    fn create_target(
        app_name: &str,
        bundle_id: &str,
        title: Option<&str>,
        workspace_name: &str,
    ) -> (StageWindow, StageWorkspace) {
        let window = StageWindow::dummy()
            .with_app(app_name)
            .with_bundle_id(bundle_id)
            .with_title(title);
        let workspace = StageWorkspace::dummy().with_name(workspace_name);
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
    fn test_title_match_does_not_use_substring() {
        let (target_window, target_workspace) =
            create_target("Browser", "com.browser.app", Some("GitHub"), "workspace-1");
        let target = ResolveTarget {
            target_window: &target_window,
            target_workspace: &target_workspace,
        };

        let windows = vec![
            AerospaceWindow::dummy()
                .with_window_id(42)
                .with_app_name("Browser")
                .with_bundle_id("com.browser.app")
                .with_window_title("Dashboard - GitHub - Safari"),
        ];

        let resolver = TitleMatchResolverRule {};
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
    }

    #[test]
    fn test_title_match_empty_title_returns_none() {
        let (target_window, target_workspace) =
            create_target("Terminal", "com.apple.Terminal", Some(""), "dev");
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
        assert_eq!(resolver.propose(&windows, &target).len(), 2);
    }

    #[test]
    fn test_title_match_ignores_app_name_suffix() {
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
                .with_window_title("main.rs — Code"),
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

    #[test]
    fn test_similarity_multiple_matches_returns_none() {
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
            AerospaceWindow::dummy()
                .with_window_id(10)
                .with_app_name("Notes")
                .with_bundle_id("com.apple.Notes")
                .with_window_title("Project Ideas 2025"),
            AerospaceWindow::dummy()
                .with_window_id(20)
                .with_app_name("Notes")
                .with_bundle_id("com.apple.Notes")
                .with_window_title("Project Ideas 2024"),
        ];

        let resolver = TitleSimilarityResolverRule { threshold: 0.6 };
        let result = resolver.match_window(&windows, &target);

        assert_eq!(result, None);
        assert_eq!(resolver.propose(&windows, &target).len(), 2);
    }
}
