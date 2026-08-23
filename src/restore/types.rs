use crate::{
    aerospace::{AerospaceWindow, AerospaceWindowId, AerospaceWorkspaceId},
    stage::{StageWindow, StageWorkspace},
};

pub struct ResolveTarget<'a> {
    pub target_workspace: &'a StageWorkspace,
    pub target_window: &'a StageWindow,
}

impl<'a> ResolveTarget<'a> {
    pub fn matches_window_app(&self, window: &AerospaceWindow) -> bool {
        let app_matches = self
            .target_window
            .app
            .as_deref()
            .map_or(false, |app| app == window.app_name);

        let bundle_id_matches = self
            .target_window
            .bundle_id
            .as_deref()
            .map_or(false, |bundle_id| bundle_id == window.app_bundle_id);

        app_matches || bundle_id_matches
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
