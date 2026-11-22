//! A stage as a flat list of window entries. An entry keeps its id when it
//! moves to another workspace, unlike its position in the stage file.

use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::{
    aerospace::{
        Aerospace, AerospaceWindow, AerospaceWindowId, AerospaceWorkspace, AerospaceWorkspaceId,
    },
    restore::{MatchOrigin, StageRestorer},
    stage::{
        StageWindow, StageWindowLocation, StageWorkspace, StageWorkspaceLayout, UnresolvedStage,
        UnresolvedStageWorkspace, WorkspaceRef,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntryId(usize);

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub window_id: AerospaceWindowId,
    /// The workspace the entry's workspace reference resolved to when it was bound.
    pub workspace: AerospaceWorkspaceId,
    /// Whether the entry's title was an exact copy of the window title.
    pub title_tracked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StageEntry {
    pub id: EntryId,
    pub workspace: WorkspaceRef,
    pub window: StageWindow,
    pub binding: Option<Binding>,
}

impl StageEntry {
    /// A `@focused` entry stays `@focused` until its window leaves the
    /// workspace it was bound on.
    fn follow_workspace(&mut self, window: &AerospaceWindow) -> bool {
        let Some(binding) = &mut self.binding else {
            return false;
        };
        if binding.workspace == window.workspace {
            return false;
        }

        binding.workspace = window.workspace.clone();
        self.workspace = WorkspaceRef::Named(window.workspace.clone());
        true
    }

    fn follow_title(&mut self, window: &AerospaceWindow) -> bool {
        let tracked = self
            .binding
            .as_ref()
            .is_some_and(|binding| binding.title_tracked);
        if !tracked || self.window.title.as_deref() == Some(window.window_title.as_str()) {
            return false;
        }

        self.window.title = Some(window.window_title.clone());
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
struct WorkspaceTable {
    name: WorkspaceRef,
    layout: Option<StageWorkspaceLayout>,
}

/// Entry ids grouped like the workspaces and windows of a built stage.
#[derive(Debug)]
pub struct EntryIds(Vec<Vec<EntryId>>);

impl EntryIds {
    pub fn at(&self, location: StageWindowLocation) -> EntryId {
        self.0[location.workspace_index][location.window_index]
    }
}

#[derive(Debug)]
pub struct WatchedStage {
    name: Option<String>,
    description: Option<String>,
    default_workspace: Option<WorkspaceRef>,
    workspaces: Vec<WorkspaceTable>,
    entries: Vec<StageEntry>,
}

impl From<UnresolvedStage> for WatchedStage {
    fn from(stage: UnresolvedStage) -> Self {
        let mut workspaces = Vec::with_capacity(stage.workspaces.len());
        let mut entries = Vec::new();

        for workspace in stage.workspaces {
            for window in workspace.windows {
                entries.push(StageEntry {
                    id: EntryId(entries.len()),
                    workspace: workspace.name.clone(),
                    window,
                    binding: None,
                });
            }
            workspaces.push(WorkspaceTable {
                name: workspace.name,
                layout: workspace.layout,
            });
        }

        WatchedStage {
            name: stage.name,
            description: stage.description,
            default_workspace: stage.default_workspace,
            workspaces,
            entries,
        }
    }
}

impl WatchedStage {
    pub fn entries(&self) -> &[StageEntry] {
        &self.entries
    }

    /// Restores the stage and binds every entry whose window was matched.
    pub fn restore(&mut self, aerospace: &Aerospace, focused: Option<&str>) -> Result<()> {
        let (stage, entry_ids) = self.to_stage_with_entry_ids();
        let resolved = stage.resolve_with_locations(focused)?;
        let report = StageRestorer::new(aerospace, &resolved.stage).restore_stage(None)?;

        for matched in report.matches {
            let MatchOrigin::StageWindow(index) = matched.origin else {
                continue;
            };
            let Some(window) = report
                .live_windows
                .iter()
                .find(|window| window.window_id == matched.window_id)
            else {
                continue;
            };

            let id = entry_ids.at(resolved.window_locations[index]);
            self.bind(id, window, matched.target_workspace);
        }

        Ok(())
    }

    /// Returns whether the stage changed. An entry whose window closed is
    /// unbound but stays in the stage.
    pub fn reconcile(
        &mut self,
        live_windows: &[AerospaceWindow],
        live_workspaces: &[AerospaceWorkspace],
    ) -> bool {
        let live_by_id: HashMap<AerospaceWindowId, &AerospaceWindow> = live_windows
            .iter()
            .map(|window| (window.window_id, window))
            .collect();
        let mut changed = false;

        for entry in &mut self.entries {
            let Some(window_id) = entry.binding.as_ref().map(|binding| binding.window_id) else {
                continue;
            };

            match live_by_id.get(&window_id) {
                None => entry.binding = None,
                Some(window) => {
                    changed |= entry.follow_workspace(window);
                    changed |= entry.follow_title(window);
                }
            }
        }

        changed |= self.follow_layouts(live_workspaces);
        changed
    }

    /// Only workspaces that hold a bound window are followed, so an empty
    /// workspace after a restart does not overwrite the saved layout.
    fn follow_layouts(&mut self, live_workspaces: &[AerospaceWorkspace]) -> bool {
        let bound_workspaces: HashSet<&AerospaceWorkspaceId> = self
            .entries
            .iter()
            .filter_map(|entry| entry.binding.as_ref())
            .map(|binding| &binding.workspace)
            .collect();
        let mut changed = false;

        for table in &mut self.workspaces {
            let WorkspaceRef::Named(name) = &table.name else {
                continue;
            };
            let Some(live) = live_workspaces.iter().find(|ws| &ws.workspace == name) else {
                continue;
            };
            if table.layout.is_none() || !bound_workspaces.contains(name) {
                continue;
            }

            let layout = Some(StageWorkspaceLayout::from(live.layout.clone()));
            if table.layout != layout {
                table.layout = layout;
                changed = true;
            }
        }

        changed
    }

    fn bind(&mut self, id: EntryId, window: &AerospaceWindow, workspace: AerospaceWorkspaceId) {
        let entry = &mut self.entries[id.0];
        entry.binding = Some(Binding {
            window_id: window.window_id,
            workspace,
            title_tracked: entry.window.title.as_deref() == Some(window.window_title.as_str()),
        });
    }

    pub fn to_stage(&self) -> UnresolvedStage {
        self.to_stage_with_entry_ids().0
    }

    /// Entries whose workspace has no table get a new table at the end.
    pub fn to_stage_with_entry_ids(&self) -> (UnresolvedStage, EntryIds) {
        let mut workspaces: Vec<UnresolvedStageWorkspace> = self
            .workspaces
            .iter()
            .map(|table| StageWorkspace {
                name: table.name.clone(),
                windows: Vec::new(),
                layout: table.layout.clone(),
            })
            .collect();
        let mut entry_ids: Vec<Vec<EntryId>> = vec![Vec::new(); workspaces.len()];

        for entry in &self.entries {
            let index = match workspaces.iter().position(|ws| ws.name == entry.workspace) {
                Some(index) => index,
                None => {
                    workspaces.push(StageWorkspace {
                        name: entry.workspace.clone(),
                        windows: Vec::new(),
                        layout: None,
                    });
                    entry_ids.push(Vec::new());
                    workspaces.len() - 1
                }
            };
            workspaces[index].windows.push(entry.window.clone());
            entry_ids[index].push(entry.id);
        }

        let stage = UnresolvedStage {
            name: self.name.clone(),
            description: self.description.clone(),
            workspaces,
            default_workspace: self.default_workspace.clone(),
        };

        (stage, EntryIds(entry_ids))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aerospace::AerospaceLayout;

    const STAGE: &str = r#"
name = "work"
description = "Daily setup"
default_workspace = "9"

[[workspace]]
name = "1"
layout = "h_tiles"

[[workspace.window]]
app = "Safari"
title = "Docs"

[[workspace.window]]
app = "Ghostty"

[[workspace]]
name = "@focused"

[[workspace.window]]
app = "Slack"
bundle_id = "com.tinyspeck.slackmacgap"

[[workspace]]
name = "3"
layout = "v_accordion"
"#;

    fn load(toml: &str) -> WatchedStage {
        WatchedStage::from(toml::from_str::<UnresolvedStage>(toml).unwrap())
    }

    fn to_toml(stage: &UnresolvedStage) -> String {
        toml::to_string(stage).unwrap()
    }

    #[test]
    fn loads_one_entry_per_window_in_stage_order() {
        let watched = load(STAGE);

        let entries: Vec<_> = watched
            .entries()
            .iter()
            .map(|entry| (entry.workspace.to_string(), entry.window.app.as_deref()))
            .collect();
        assert_eq!(
            entries,
            [
                ("1".to_string(), Some("Safari")),
                ("1".to_string(), Some("Ghostty")),
                ("@focused".to_string(), Some("Slack")),
            ]
        );
    }

    #[test]
    fn assigns_distinct_entry_ids() {
        let watched = load(STAGE);

        let ids: Vec<EntryId> = watched.entries().iter().map(|entry| entry.id).collect();
        assert_eq!(ids, [EntryId(0), EntryId(1), EntryId(2)]);
    }

    #[test]
    fn round_trip_keeps_the_stage_as_written() {
        let original: UnresolvedStage = toml::from_str(STAGE).unwrap();
        let expected = to_toml(&original);

        let rebuilt = WatchedStage::from(original).to_stage();

        assert_eq!(to_toml(&rebuilt), expected);
    }

    #[test]
    fn round_trip_keeps_empty_workspaces_and_their_layout() {
        let rebuilt = load(STAGE).to_stage();

        let last = rebuilt.workspaces.last().unwrap();
        assert_eq!(last.name, WorkspaceRef::Named("3".into()));
        assert!(last.windows.is_empty());
        assert_eq!(last.layout, Some(StageWorkspaceLayout::VAccordion));
    }

    #[test]
    fn entry_moved_to_an_existing_workspace_joins_its_table() {
        let mut watched = load(STAGE);
        watched.entries[0].workspace = WorkspaceRef::Named("3".into());

        let rebuilt = watched.to_stage();

        let apps = |index: usize| -> Vec<Option<String>> {
            rebuilt.workspaces[index]
                .windows
                .iter()
                .map(|window| window.app.clone())
                .collect()
        };
        assert_eq!(apps(0), [Some("Ghostty".to_string())]);
        assert_eq!(apps(2), [Some("Safari".to_string())]);
        assert_eq!(
            rebuilt.workspaces[2].layout,
            Some(StageWorkspaceLayout::VAccordion)
        );
    }

    #[test]
    fn entry_moved_to_an_unknown_workspace_gets_a_new_table_at_the_end() {
        let mut watched = load(STAGE);
        watched.entries[1].workspace = WorkspaceRef::Named("7".into());

        let rebuilt = watched.to_stage();

        assert_eq!(rebuilt.workspaces.len(), 4);
        let added = &rebuilt.workspaces[3];
        assert_eq!(added.name, WorkspaceRef::Named("7".into()));
        assert_eq!(added.layout, None);
        assert_eq!(added.windows.len(), 1);
        assert_eq!(added.windows[0].app.as_deref(), Some("Ghostty"));
    }

    #[test]
    fn windows_of_repeated_workspace_tables_join_the_first_table() {
        let rebuilt = load(
            r#"
[[workspace]]
name = "1"

[[workspace.window]]
app = "Safari"

[[workspace]]
name = "1"
layout = "h_tiles"

[[workspace.window]]
app = "Ghostty"
"#,
        )
        .to_stage();

        assert_eq!(rebuilt.workspaces.len(), 2);
        assert_eq!(rebuilt.workspaces[0].windows.len(), 2);
        assert!(rebuilt.workspaces[1].windows.is_empty());
        assert_eq!(
            rebuilt.workspaces[1].layout,
            Some(StageWorkspaceLayout::HTiles)
        );
    }

    #[test]
    fn entry_ids_follow_entries_to_their_new_table() {
        let mut watched = load(STAGE);
        watched.entries[0].workspace = WorkspaceRef::Named("3".into());

        let (_, entry_ids) = watched.to_stage_with_entry_ids();

        let location = |workspace_index, window_index| StageWindowLocation {
            workspace_index,
            window_index,
        };
        assert_eq!(entry_ids.at(location(0, 0)), EntryId(1));
        assert_eq!(entry_ids.at(location(1, 0)), EntryId(2));
        assert_eq!(entry_ids.at(location(2, 0)), EntryId(0));
    }

    fn live(
        window_id: AerospaceWindowId,
        app: &str,
        title: &str,
        workspace: &str,
    ) -> AerospaceWindow {
        AerospaceWindow::dummy()
            .with_window_id(window_id)
            .with_app_name(app)
            .with_bundle_id(&format!("com.example.{app}"))
            .with_window_title(title)
            .with_workspace(workspace)
    }

    fn binding(watched: &WatchedStage, id: usize) -> Option<&Binding> {
        watched.entries[id].binding.as_ref()
    }

    #[test]
    fn restore_binds_matched_entries_to_their_windows() {
        let mut watched = load(STAGE);
        let aerospace = Aerospace::mock_with_windows(vec![
            live(10, "Safari", "Docs", "3"),
            live(12, "Slack", "General", "1").with_bundle_id("com.tinyspeck.slackmacgap"),
        ]);

        watched.restore(&aerospace, Some("5")).unwrap();

        assert_eq!(
            binding(&watched, 0),
            Some(&Binding {
                window_id: 10,
                workspace: "1".into(),
                title_tracked: true,
            })
        );
        assert_eq!(binding(&watched, 1), None);
        assert_eq!(
            binding(&watched, 2),
            Some(&Binding {
                window_id: 12,
                workspace: "5".into(),
                title_tracked: false,
            })
        );
    }

    #[test]
    fn restore_tracks_only_exact_title_copies() {
        let mut watched = load(STAGE);
        let aerospace =
            Aerospace::mock_with_windows(vec![live(10, "Safari", "Docs — Safari", "1")]);

        watched.restore(&aerospace, Some("5")).unwrap();

        let bound = binding(&watched, 0).unwrap();
        assert_eq!(bound.window_id, 10);
        assert!(!bound.title_tracked);
    }

    #[test]
    fn restore_does_not_bind_windows_sent_to_the_default_workspace() {
        let mut watched = load(STAGE);
        let aerospace = Aerospace::mock_with_windows(vec![live(13, "Finder", "Downloads", "1")]);

        watched.restore(&aerospace, Some("5")).unwrap();

        assert!(
            watched
                .entries()
                .iter()
                .all(|entry| entry.binding.is_none())
        );
    }

    #[test]
    fn reconcile_unbinds_entries_whose_window_closed() {
        let mut watched = load(STAGE);
        let safari = live(10, "Safari", "Docs", "1");
        let slack = live(12, "Slack", "General", "5").with_bundle_id("com.tinyspeck.slackmacgap");
        let aerospace = Aerospace::mock_with_windows(vec![safari.clone(), slack]);
        watched.restore(&aerospace, Some("5")).unwrap();
        let stage_before = toml::to_string(&watched.to_stage()).unwrap();

        assert!(!watched.reconcile(&[safari], &[]));

        assert_eq!(binding(&watched, 0).map(|b| b.window_id), Some(10));
        assert_eq!(binding(&watched, 2), None);
        assert_eq!(watched.entries().len(), 3);
        assert_eq!(toml::to_string(&watched.to_stage()).unwrap(), stage_before);
    }

    fn restored(windows: Vec<AerospaceWindow>) -> WatchedStage {
        let mut watched = load(STAGE);
        watched
            .restore(&Aerospace::mock_with_windows(windows), Some("5"))
            .unwrap();
        watched
    }

    fn slack(workspace: &str) -> AerospaceWindow {
        live(12, "Slack", "General", workspace).with_bundle_id("com.tinyspeck.slackmacgap")
    }

    #[test]
    fn reconcile_moves_entries_with_their_window() {
        let mut watched = restored(vec![live(10, "Safari", "Docs", "1")]);

        assert!(watched.reconcile(&[live(10, "Safari", "Docs", "3")], &[]));

        assert_eq!(
            watched.entries[0].workspace,
            WorkspaceRef::Named("3".into())
        );
        assert_eq!(binding(&watched, 0).unwrap().workspace, "3");
        let rebuilt = watched.to_stage();
        assert_eq!(
            rebuilt.workspaces[2].windows[0].title.as_deref(),
            Some("Docs")
        );
    }

    #[test]
    fn reconcile_reports_no_change_when_nothing_moved() {
        let mut watched = restored(vec![live(10, "Safari", "Docs", "1")]);

        assert!(!watched.reconcile(&[live(10, "Safari", "Docs", "1")], &[]));
    }

    #[test]
    fn reconcile_keeps_focused_while_the_window_stays_on_its_workspace() {
        let mut watched = restored(vec![slack("1")]);

        assert!(!watched.reconcile(&[slack("5")], &[]));
        assert_eq!(watched.entries[2].workspace, WorkspaceRef::Focused);

        assert!(watched.reconcile(&[slack("2")], &[]));
        assert_eq!(
            watched.entries[2].workspace,
            WorkspaceRef::Named("2".into())
        );
    }

    #[test]
    fn reconcile_follows_tracked_titles() {
        let mut watched = restored(vec![live(10, "Safari", "Docs", "1")]);

        assert!(watched.reconcile(&[live(10, "Safari", "Inbox", "1")], &[]));

        assert_eq!(watched.entries[0].window.title.as_deref(), Some("Inbox"));
    }

    #[test]
    fn reconcile_keeps_pinned_titles() {
        let mut watched = restored(vec![live(10, "Safari", "Docs — Safari", "1")]);

        assert!(!watched.reconcile(&[live(10, "Safari", "Inbox", "1")], &[]));

        assert_eq!(watched.entries[0].window.title.as_deref(), Some("Docs"));
    }

    #[test]
    fn reconcile_leaves_entries_without_a_title_untitled() {
        let mut watched = restored(vec![slack("5")]);

        assert!(!watched.reconcile(&[slack("5")], &[]));

        assert_eq!(watched.entries[2].window.title, None);
    }

    fn workspace(name: &str, layout: AerospaceLayout) -> AerospaceWorkspace {
        AerospaceWorkspace {
            workspace: name.into(),
            layout,
        }
    }

    #[test]
    fn reconcile_follows_the_layout_of_workspaces_with_bound_windows() {
        let mut watched = restored(vec![live(10, "Safari", "Docs", "1")]);

        assert!(watched.reconcile(
            &[live(10, "Safari", "Docs", "1")],
            &[workspace("1", AerospaceLayout::VAccordion)],
        ));

        assert_eq!(
            watched.to_stage().workspaces[0].layout,
            Some(StageWorkspaceLayout::VAccordion)
        );
    }

    #[test]
    fn reconcile_keeps_the_layout_of_workspaces_without_bound_windows() {
        let mut watched = restored(vec![live(10, "Safari", "Docs", "1")]);

        assert!(!watched.reconcile(
            &[live(10, "Safari", "Docs", "1")],
            &[workspace("3", AerospaceLayout::HTiles)],
        ));

        assert_eq!(
            watched.to_stage().workspaces[2].layout,
            Some(StageWorkspaceLayout::VAccordion)
        );
    }

    #[test]
    fn reconcile_does_not_add_a_layout_the_stage_does_not_declare() {
        let mut watched = load(
            r#"
[[workspace]]
name = "1"

[[workspace.window]]
app = "Safari"
title = "Docs"
"#,
        );
        watched
            .restore(
                &Aerospace::mock_with_windows(vec![live(10, "Safari", "Docs", "1")]),
                None,
            )
            .unwrap();

        assert!(!watched.reconcile(
            &[live(10, "Safari", "Docs", "1")],
            &[workspace("1", AerospaceLayout::VTiles)],
        ));

        assert_eq!(watched.to_stage().workspaces[0].layout, None);
    }
}
