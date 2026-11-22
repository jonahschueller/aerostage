//! A stage as a flat list of window entries. An entry keeps its id when it
//! moves to another workspace, unlike its position in the stage file.

use crate::stage::{
    StageWindow, StageWindowLocation, StageWorkspace, StageWorkspaceLayout, UnresolvedStage,
    UnresolvedStageWorkspace, WorkspaceRef,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntryId(usize);

#[derive(Debug, Clone, PartialEq)]
pub struct StageEntry {
    pub id: EntryId,
    pub workspace: WorkspaceRef,
    pub window: StageWindow,
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
}
