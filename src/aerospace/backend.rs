mod cli;
mod common;
#[cfg(test)]
pub mod mock;
mod socket;
mod types;

use anyhow::Result;

use crate::aerospace::{
    AerospaceApp, AerospaceLayout, AerospaceWindow, AerospaceWindowId, AerospaceWorkspace,
    AerospaceWorkspaceId,
};

pub use cli::*;
pub use socket::*;
pub use types::*;

pub trait AerospaceBackend {
    fn list_apps(&self) -> Result<Vec<AerospaceApp>>;
    fn list_windows(&self) -> Result<Vec<AerospaceWindow>>;
    fn list_workspaces(&self) -> Result<Vec<AerospaceWorkspace>>;
    fn move_node_to_workspace(
        &self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> Result<()>;
    fn layout(&self, workspace: &AerospaceWorkspaceId, layout: &AerospaceLayout) -> Result<()>;
    fn flatten_workspace_tree(&self, workspace: &AerospaceWorkspaceId) -> Result<()>;
}
