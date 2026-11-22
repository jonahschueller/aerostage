use anyhow::{Result, anyhow};

use crate::aerospace::{
    AerospaceApp, AerospaceEventStream, AerospaceLayout, AerospaceWindow, AerospaceWindowId,
    AerospaceWorkspace, AerospaceWorkspaceId,
    backend::{AerospaceBackend, AerospaceCliBackend, AerospaceSocketBackend},
};

pub struct Aerospace {
    backend: Box<dyn AerospaceBackend>,
}

impl Aerospace {
    #[cfg(test)]
    pub fn mock() -> Self {
        Self::mock_with_windows(Vec::new())
    }

    #[cfg(test)]
    pub fn mock_with_windows(windows: Vec<AerospaceWindow>) -> Self {
        Self {
            backend: Box::new(
                crate::aerospace::backend::mock::MockAerospaceBackend::with_windows(windows),
            ),
        }
    }

    pub fn connect() -> Result<Self> {
        match AerospaceSocketBackend::with_aerospace_socket().and_then(|be| be.do_handshake()) {
            Ok(socket) => Ok(Self {
                backend: Box::new(socket),
            }),
            Err(socket_err) => {
                if which::which("aerospace").is_ok() {
                    Ok(Self {
                        backend: Box::new(AerospaceCliBackend::default()),
                    })
                } else {
                    Err(anyhow!(
                        "Unable to connect to AeroSpace ({socket_err}). Start AeroSpace.app, or install the `aerospace` CLI and put it on your PATH."
                    ))
                }
            }
        }
    }

    pub fn list_apps(&self) -> Result<Vec<AerospaceApp>> {
        self.backend.list_apps()
    }

    pub fn list_workspaces(&self) -> Result<Vec<AerospaceWorkspace>> {
        self.backend.list_workspaces()
    }

    pub fn list_workspace_focused(&self) -> Result<AerospaceWorkspace> {
        self.backend.list_workspace_focused()
    }

    pub fn list_windows(&self) -> Result<Vec<AerospaceWindow>> {
        self.backend.list_windows()
    }

    /// Fails when no window has focus.
    pub fn list_window_focused(&self) -> Result<AerospaceWindow> {
        self.backend.list_window_focused()
    }

    pub fn move_node_to_workspace(
        &self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> Result<()> {
        self.backend.move_node_to_workspace(workspace, window_id)
    }

    pub fn move_node_to_workspace_and_focus(
        &self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> Result<()> {
        self.backend
            .move_node_to_workspace_and_focus(workspace, window_id)
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

    pub fn subscribe(&self) -> Result<AerospaceEventStream> {
        self.backend.subscribe()
    }
}
