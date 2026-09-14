use anyhow::{anyhow, Result};

use crate::aerospace::{
    backend::{AerospaceBackend, AerospaceCliCBackend},
    AerospaceApp, AerospaceLayout, AerospaceWindow, AerospaceWindowId, AerospaceWorkspace,
    AerospaceWorkspaceId,
};

pub struct Aerospace<B: AerospaceBackend = AerospaceCliCBackend> {
    backend: B,
}

impl Default for Aerospace {
    fn default() -> Self {
        Self {
            backend: AerospaceCliCBackend::default(),
        }
    }
}

impl Aerospace {
    pub fn ensure_aerospace_installed() -> Result<()> {
        which::which("aerospace").map(|_| ()).map_err(|_| {
            anyhow!(
                "'aerospace' command not found. Please install AeroSpace and put it on your PATH."
            )
        })
    }

    pub fn list_apps(&mut self) -> Result<Vec<AerospaceApp>> {
        self.backend.list_apps()
    }

    pub fn list_workspaces(&mut self) -> Result<Vec<AerospaceWorkspace>> {
        self.backend.list_workspaces()
    }

    pub fn list_windows(&mut self) -> Result<Vec<AerospaceWindow>> {
        self.backend.list_windows()
    }

    pub fn move_node_to_workspace(
        &mut self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> Result<()> {
        self.backend.move_node_to_workspace(workspace, window_id)
    }

    pub fn change_layout(
        &mut self,
        workspace: &AerospaceWorkspaceId,
        layout: &AerospaceLayout,
    ) -> Result<()> {
        self.backend.layout(workspace, layout)
    }

    pub fn flatten_workspace_tree(&mut self, workspace: &AerospaceWorkspaceId) -> Result<()> {
        self.backend.flatten_workspace_tree(workspace)
    }
}
