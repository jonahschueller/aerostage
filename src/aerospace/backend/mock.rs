use anyhow::Result;

use crate::aerospace::{
    AerospaceApp, AerospaceLayout, AerospaceWindow, AerospaceWindowId, AerospaceWorkspace,
    AerospaceWorkspaceId, backend::AerospaceBackend,
};

pub struct MockAerospaceBackend {}

impl MockAerospaceBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl AerospaceBackend for MockAerospaceBackend {
    fn list_apps(&self) -> Result<Vec<AerospaceApp>> {
        Ok(vec![])
    }
    fn list_workspaces(&self) -> Result<Vec<AerospaceWorkspace>> {
        Ok(vec![])
    }

    fn list_windows(&self) -> Result<Vec<AerospaceWindow>> {
        Ok(vec![])
    }

    fn move_node_to_workspace(
        &self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> Result<()> {
        Ok(())
    }

    fn layout(&self, workspace: &AerospaceWorkspaceId, layout: &AerospaceLayout) -> Result<()> {
        Ok(())
    }

    fn flatten_workspace_tree(&self, workspace: &AerospaceWorkspaceId) -> Result<()> {
        Ok(())
    }
}
