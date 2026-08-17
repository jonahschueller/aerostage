use crate::{
    aerospace::{AerospaceWindow, AerospaceWindowId, AerospaceWorkspaceId},
    arrangement::{ArrangementWindow, ArrangementWorkspace},
};

pub struct ResolveTarget<'a> {
    pub target_workspace: &'a ArrangementWorkspace,
    pub target_window: &'a ArrangementWindow,
}

impl<'a> ResolveTarget<'a> {
    pub fn matches_window_app(&self, window: &AerospaceWindow) -> bool {
        self.target_window.app == window.app_name
            || self.target_window.bundle_id == window.app_bundle_id
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedWindowMatch {
    pub target_workspace: AerospaceWorkspaceId,
    pub window_id: AerospaceWindowId,
}

#[derive(Debug)]
pub struct UnresolvedWindow {
    pub window_id: AerospaceWindowId,
}
