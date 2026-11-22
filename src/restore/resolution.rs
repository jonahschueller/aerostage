use std::collections::HashSet;

use crate::{
    aerospace::{AerospaceWindow, AerospaceWindowId, AerospaceWorkspaceId},
    restore::{
        assign::{ScoredPair, assign_unique_best},
        rule::WindowResolverRule,
        rules::{
            TargetWorkspaceResolverRule, TitleMatchResolverRule, TitleSimilarityResolverRule,
            UniqueAppNameResolverRule, UniqueBundleIdResolverRule,
        },
        types::{
            MatchOrigin, ResolveTarget, ResolvedWindowMatch, StageWindowIndex, UnresolvedWindow,
        },
    },
    stage::Stage,
};

struct PendingTarget<'a> {
    index: StageWindowIndex,
    target: ResolveTarget<'a>,
}

impl PendingTarget<'_> {
    fn resolve_to(&self, window_id: AerospaceWindowId) -> ResolvedWindowMatch {
        ResolvedWindowMatch {
            target_workspace: self.target.target_workspace.name.clone(),
            window_id,
            origin: MatchOrigin::StageWindow(self.index),
        }
    }
}

pub struct WindowResolver {
    title_rules: Vec<Box<dyn WindowResolverRule>>,
    leftover_rules: Vec<Box<dyn WindowResolverRule>>,
    count_by_app: bool,
    fallback_workspace: Option<AerospaceWorkspaceId>,
}

impl WindowResolver {
    pub fn new(fallback_workspace: Option<AerospaceWorkspaceId>) -> Self {
        Self::empty()
            .with_title_rules()
            .with_leftover_rules()
            .with_count_by_app()
            .with_fallback_workspace(fallback_workspace)
    }

    /// Resolves nothing until rule stages are added.
    pub fn empty() -> Self {
        WindowResolver {
            title_rules: Vec::new(),
            leftover_rules: Vec::new(),
            count_by_app: false,
            fallback_workspace: None,
        }
    }

    pub fn with_title_rules(mut self) -> Self {
        self.title_rules = vec![
            Box::new(TitleMatchResolverRule {}),
            Box::new(TitleSimilarityResolverRule { threshold: 0.75 }),
        ];
        self
    }

    /// Rules that commit only when exactly one window of the target's app is left.
    pub fn with_leftover_rules(mut self) -> Self {
        self.leftover_rules = vec![
            Box::new(TargetWorkspaceResolverRule {}),
            Box::new(UniqueBundleIdResolverRule {}),
            Box::new(UniqueAppNameResolverRule {}),
        ];
        self
    }

    pub fn with_count_by_app(mut self) -> Self {
        self.count_by_app = true;
        self
    }

    pub fn with_fallback_workspace(mut self, workspace: Option<AerospaceWorkspaceId>) -> Self {
        self.fallback_workspace = workspace;
        self
    }

    fn apply_title_assignment(
        &self,
        pending_targets: &mut Vec<PendingTarget<'_>>,
        available_windows: &mut Vec<AerospaceWindow>,
    ) -> Vec<ResolvedWindowMatch> {
        let mut pairs = Vec::new();
        for (target_index, pending) in pending_targets.iter().enumerate() {
            for rule in &self.title_rules {
                for candidate in rule.propose(available_windows, &pending.target) {
                    pairs.push(ScoredPair {
                        target_index,
                        window_id: candidate.window_id,
                        score: candidate.score,
                    });
                }
            }
        }

        let assignments = assign_unique_best(pairs);
        let assigned_targets: HashSet<usize> = assignments
            .iter()
            .map(|assigned| assigned.target_index)
            .collect();
        let assigned_windows: HashSet<_> = assignments
            .iter()
            .map(|assigned| assigned.window_id)
            .collect();

        let resolved = assignments
            .into_iter()
            .map(|assigned| pending_targets[assigned.target_index].resolve_to(assigned.window_id))
            .collect();

        let mut remaining_targets = Vec::new();
        for (index, target) in pending_targets.drain(..).enumerate() {
            if !assigned_targets.contains(&index) {
                remaining_targets.push(target);
            }
        }
        *pending_targets = remaining_targets;
        available_windows.retain(|window| !assigned_windows.contains(&window.window_id));

        resolved
    }

    fn apply_unique_leftover_rules(
        &self,
        pending_targets: &mut Vec<PendingTarget<'_>>,
        available_windows: &mut Vec<AerospaceWindow>,
    ) -> Vec<ResolvedWindowMatch> {
        let mut resolved_matches = Vec::new();

        loop {
            let initial_pending_count = pending_targets.len();

            for rule in &self.leftover_rules {
                let mut remaining_targets = Vec::new();
                for pending in pending_targets.drain(..) {
                    let candidates = rule.propose(available_windows, &pending.target);
                    if candidates.len() != 1 {
                        remaining_targets.push(pending);
                        continue;
                    }

                    let window_id = candidates[0].window_id;
                    available_windows.retain(|window| window.window_id != window_id);
                    resolved_matches.push(pending.resolve_to(window_id));
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

    fn apply_count_by_app(
        pending_targets: &mut Vec<PendingTarget<'_>>,
        available_windows: &mut Vec<AerospaceWindow>,
    ) -> Vec<ResolvedWindowMatch> {
        let mut resolved_matches = Vec::new();
        let mut remaining_targets = Vec::new();

        for pending in pending_targets.drain(..) {
            match available_windows
                .iter()
                .position(|window| pending.target.matches_window_app(window))
            {
                Some(index) => {
                    let window = available_windows.remove(index);
                    resolved_matches.push(pending.resolve_to(window.window_id));
                }
                None => remaining_targets.push(pending),
            }
        }

        *pending_targets = remaining_targets;
        resolved_matches
    }

    fn apply_optional_fallback_resolver(
        &self,
        resolved_matches: &mut Vec<ResolvedWindowMatch>,
        available_windows: &mut Vec<AerospaceWindow>,
        pending_targets: &[PendingTarget<'_>],
    ) {
        let Some(fallback_workspace) = self.fallback_workspace.as_ref() else {
            return;
        };

        let mut leftover_windows = Vec::new();
        available_windows.retain(|window| {
            if pending_targets
                .iter()
                .any(|pending| pending.target.matches_window_app(window))
            {
                return true;
            }

            leftover_windows.push(ResolvedWindowMatch {
                target_workspace: fallback_workspace.clone(),
                window_id: window.window_id,
                origin: MatchOrigin::DefaultWorkspace,
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
        let mut pending_targets: Vec<PendingTarget> = stage
            .workspaces
            .iter()
            .flat_map(|workspace| {
                workspace.windows.iter().map(move |window| ResolveTarget {
                    target_workspace: workspace,
                    target_window: window,
                })
            })
            .enumerate()
            .map(|(index, target)| PendingTarget { index, target })
            .collect();

        let mut available_windows = windows.to_vec();

        let mut resolved_matches =
            self.apply_title_assignment(&mut pending_targets, &mut available_windows);
        resolved_matches
            .extend(self.apply_unique_leftover_rules(&mut pending_targets, &mut available_windows));
        if self.count_by_app {
            resolved_matches.extend(Self::apply_count_by_app(
                &mut pending_targets,
                &mut available_windows,
            ));
        }

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

        let pending_targets = pending_targets
            .into_iter()
            .map(|pending| pending.target)
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
        Self::resolve_with(
            WindowResolver::new(stage.default_workspace.clone()),
            stage,
            windows,
        )
    }

    pub fn resolve_with(
        resolver: WindowResolver,
        stage: &'a Stage,
        windows: &[AerospaceWindow],
    ) -> Self {
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
mod tests {
    use super::*;
    use crate::stage::{StageWindow, StageWorkspace};

    fn stage_window(app: &str, title: &str) -> StageWindow {
        StageWindow {
            app: Some(app.into()),
            title: Some(title.into()),
            bundle_id: None,
        }
    }

    fn live_window(window_id: AerospaceWindowId, app: &str, title: &str) -> AerospaceWindow {
        AerospaceWindow::dummy()
            .with_window_id(window_id)
            .with_app_name(app)
            .with_window_title(title)
    }

    fn origin_of(resolution: &WindowResolution, window_id: AerospaceWindowId) -> MatchOrigin {
        resolution
            .resolved_windows
            .iter()
            .find(|matched| matched.window_id == window_id)
            .map(|matched| matched.origin)
            .expect("window should be resolved")
    }

    #[test]
    fn origin_counts_stage_windows_across_workspaces() {
        let stage = Stage {
            name: None,
            description: None,
            workspaces: vec![
                StageWorkspace {
                    name: "1".into(),
                    windows: vec![
                        stage_window("Safari", "Inbox"),
                        stage_window("Mail", "Today"),
                    ],
                    layout: None,
                },
                StageWorkspace {
                    name: "2".into(),
                    windows: vec![stage_window("Code", "aerostage")],
                    layout: None,
                },
            ],
            default_workspace: None,
        };
        let windows = vec![
            live_window(10, "Code", "aerostage"),
            live_window(11, "Mail", "Today"),
            live_window(12, "Safari", "Inbox"),
        ];

        let resolution = WindowResolution::resolve(&stage, &windows);

        assert_eq!(origin_of(&resolution, 12), MatchOrigin::StageWindow(0));
        assert_eq!(origin_of(&resolution, 11), MatchOrigin::StageWindow(1));
        assert_eq!(origin_of(&resolution, 10), MatchOrigin::StageWindow(2));
    }

    #[test]
    fn origin_marks_leftover_windows_as_default_workspace() {
        let stage = Stage {
            name: None,
            description: None,
            workspaces: vec![StageWorkspace {
                name: "1".into(),
                windows: vec![stage_window("Safari", "Inbox")],
                layout: None,
            }],
            default_workspace: Some("9".into()),
        };
        let windows = vec![
            live_window(10, "Safari", "Inbox"),
            live_window(11, "Finder", "Downloads"),
        ];

        let resolution = WindowResolution::resolve(&stage, &windows);

        assert_eq!(origin_of(&resolution, 10), MatchOrigin::StageWindow(0));
        assert_eq!(origin_of(&resolution, 11), MatchOrigin::DefaultWorkspace);
    }

    #[test]
    fn resolver_runs_only_the_rule_stages_it_was_built_with() {
        let stage = Stage {
            name: None,
            description: None,
            workspaces: vec![StageWorkspace {
                name: "1".into(),
                windows: vec![
                    stage_window("Safari", "Inbox"),
                    StageWindow {
                        app: Some("Slack".into()),
                        title: None,
                        bundle_id: None,
                    },
                ],
                layout: None,
            }],
            default_workspace: Some("9".into()),
        };
        let windows = vec![
            live_window(10, "Safari", "Inbox"),
            live_window(11, "Slack", "General"),
            live_window(12, "Finder", "Downloads"),
        ];

        let resolution = WindowResolution::resolve_with(
            WindowResolver::empty().with_title_rules(),
            &stage,
            &windows,
        );

        assert_eq!(resolution.resolved_windows.len(), 1);
        assert_eq!(origin_of(&resolution, 10), MatchOrigin::StageWindow(0));
        assert_eq!(resolution.pending_targets.len(), 1);
    }
}
