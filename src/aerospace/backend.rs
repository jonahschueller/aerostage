mod cli;
mod common;
mod error;
#[cfg(test)]
pub mod mock;
mod socket;

use anyhow::Result;

use crate::aerospace::{
    AerospaceApp, AerospaceEventStream, AerospaceLayout, AerospaceWindow, AerospaceWindowId,
    AerospaceWorkspace, AerospaceWorkspaceId,
};

pub use cli::*;
pub use socket::*;

pub trait AerospaceBackend {
    #[allow(unused)]
    fn list_apps(&self) -> Result<Vec<AerospaceApp>>;
    fn list_windows(&self) -> Result<Vec<AerospaceWindow>>;
    fn list_workspaces(&self) -> Result<Vec<AerospaceWorkspace>>;
    fn list_workspace_focused(&self) -> Result<AerospaceWorkspace>;
    /// Fails when no window has focus.
    fn list_window_focused(&self) -> Result<AerospaceWindow>;
    fn move_node_to_workspace(
        &self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> Result<()>;
    fn move_node_to_workspace_and_focus(
        &self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> Result<()>;
    fn layout(&self, workspace: &AerospaceWorkspaceId, layout: &AerospaceLayout) -> Result<()>;
    fn flatten_workspace_tree(&self, workspace: &AerospaceWorkspaceId) -> Result<()>;
    fn subscribe(&self) -> Result<AerospaceEventStream>;
}
