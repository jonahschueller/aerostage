use anyhow::{Result, anyhow};

use crate::aerospace::{
    AerospaceApp, AerospaceLayout, AerospaceWindow, AerospaceWindowId, AerospaceWorkspace,
    AerospaceWorkspaceId,
    backend::{AerospaceBackend, AerospaceCliBackend, AerospaceSocketBackend},
};

pub struct Aerospace {
    backend: Box<dyn AerospaceBackend>,
}

impl Default for Aerospace {
    fn default() -> Self {
        let backend: Box<dyn AerospaceBackend> =
            match AerospaceSocketBackend::with_aerospace_socket().and_then(|be| be.do_handshake()) {
                Ok(socket) => Box::new(socket),
                Err(_) => Box::new(AerospaceCliBackend::default()),
            };

        Self { backend }
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

    pub fn list_apps(&self) -> Result<Vec<AerospaceApp>> {
        self.backend.list_apps()
    }

    pub fn list_workspaces(&self) -> Result<Vec<AerospaceWorkspace>> {
        self.backend.list_workspaces()
    }

    pub fn list_windows(&self) -> Result<Vec<AerospaceWindow>> {
        self.backend.list_windows()
    }

    pub fn move_node_to_workspace(
        &self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> Result<()> {
        self.backend.move_node_to_workspace(workspace, window_id)
    }

    pub fn change_layout(
        &self,
        workspace: &AerospaceWorkspaceId,
        layout: &AerospaceLayout,
    ) -> Result<()> {
        self.backend.layout(workspace, layout)
    }

    pub fn flatten_workspace_tree(&self, workspace: &AerospaceWorkspaceId) -> Result<()> {
        self.backend.flatten_workspace_tree(workspace)
    }
}
