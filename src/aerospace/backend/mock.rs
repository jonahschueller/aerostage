use anyhow::Result;

use crate::aerospace::{
    backend::AerospaceBackend, AerospaceApp, AerospaceLayout, AerospaceWindow, AerospaceWindowId,
    AerospaceWorkspace, AerospaceWorkspaceId,
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
        _workspace: &AerospaceWorkspaceId,
        _window_id: AerospaceWindowId,
    ) -> Result<()> {
        Ok(())
    }

    fn layout(&self, _workspace: &AerospaceWorkspaceId, _layout: &AerospaceLayout) -> Result<()> {
        Ok(())
    }

    fn flatten_workspace_tree(&self, _workspace: &AerospaceWorkspaceId) -> Result<()> {
        Ok(())
    }
}
