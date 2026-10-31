use anyhow::{Context, Result};

use crate::{
    aerospace::{Aerospace, AerospaceWorkspaceId},
    cli::CommandHandler,
    common::parse_workspace_list,
    config::Config,
    restore::restore::StageRestorer,
    stage::{WorkspaceRef, repository::StageRepository},
};

pub struct RestoreCommandHandler {
    pub stage: Option<String>,
    pub from_workspaces: Option<String>,
}

fn parse_workspace_refs(workspaces: &str) -> Vec<WorkspaceRef> {
    parse_workspace_list(workspaces)
        .into_iter()
        .map(WorkspaceRef::from)
        .collect()
}

fn resolve_workspace_refs(
    workspaces: &[WorkspaceRef],
    focused: Option<&str>,
) -> Result<Vec<AerospaceWorkspaceId>> {
    workspaces.iter().map(|ws| ws.resolve(focused)).collect()
}

impl CommandHandler for RestoreCommandHandler {
    fn run_command(&self, config: &Config) -> Result<()> {
        let aerospace = Aerospace::connect()?;

        let stage_name = config.resolve_stage_name(self.stage.as_deref());
        let stage_path = config.stage_directory.join(&stage_name);

        let stage_file = StageRepository::load_from_file(&stage_path)
            .with_context(|| "Failed to load stage from file.")?;

        let source_workspace_refs = self.from_workspaces.as_deref().map(parse_workspace_refs);

        // Query the focused workspace once, so the stage and the source
        // workspaces resolve @focused to the same workspace.
        let needs_focused = stage_file.stage.references_focused()
            || source_workspace_refs
                .iter()
                .flatten()
                .any(|ws| *ws == WorkspaceRef::Focused);
        let focused = if needs_focused {
            let focused = aerospace
                .list_workspace_focused()
                .context("Failed to query the focused workspace.")?;
            Some(focused.workspace)
        } else {
            None
        };

        let stage = stage_file
            .stage
            .resolve(focused.as_deref())
            .with_context(|| format!("Failed to resolve stage '{}'", &stage_name))?;

        let source_workspaces = source_workspace_refs
            .map(|refs| resolve_workspace_refs(&refs, focused.as_deref()))
            .transpose()?;

        let stage_restorer = StageRestorer::new(&aerospace, &stage);

        stage_restorer
            .restore_stage(source_workspaces.as_deref())
            .with_context(|| format!("Failed to restore stage '{}'", &stage_name))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_workspace_refs_recognizes_focused_token() {
        assert_eq!(
            parse_workspace_refs("@focused, 2"),
            vec![WorkspaceRef::Focused, WorkspaceRef::Named("2".into())]
        );
    }

    #[test]
    fn resolve_workspace_refs_replaces_focused() {
        let refs = parse_workspace_refs("@focused,2");
        assert_eq!(
            resolve_workspace_refs(&refs, Some("5")).unwrap(),
            vec!["5", "2"]
        );
    }

    #[test]
    fn resolve_workspace_refs_without_focused_keeps_named() {
        let refs = parse_workspace_refs("1,2");
        assert_eq!(resolve_workspace_refs(&refs, None).unwrap(), vec!["1", "2"]);
    }

    #[test]
    fn resolve_workspace_refs_fails_for_focused_without_focused() {
        let refs = parse_workspace_refs("@focused");
        assert!(resolve_workspace_refs(&refs, None).is_err());
    }
}
