mod cli;
mod common;
mod error;
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
    fn list_apps(&mut self) -> Result<Vec<AerospaceApp>>;
    fn list_windows(&mut self) -> Result<Vec<AerospaceWindow>>;
    fn list_workspaces(&mut self) -> Result<Vec<AerospaceWorkspace>>;
    fn move_node_to_workspace(
        &mut self,
        workspace: &AerospaceWorkspaceId,
        window_id: AerospaceWindowId,
    ) -> Result<()>;
    fn layout(&mut self, workspace: &AerospaceWorkspaceId, layout: &AerospaceLayout) -> Result<()>;
    fn flatten_workspace_tree(&mut self, workspace: &AerospaceWorkspaceId) -> Result<()>;
}
