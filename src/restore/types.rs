use crate::{
    aerospace::{AerospaceWindow, AerospaceWindowId, AerospaceWorkspaceId},
    stage::{StageWindow, StageWorkspace},
};

pub struct ResolveTarget<'a> {
    pub target_workspace: &'a StageWorkspace,
    pub target_window: &'a StageWindow,
}

pub(crate) fn present_text(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|text| !text.is_empty())
}

impl<'a> ResolveTarget<'a> {
    pub fn matches_window_app(&self, window: &AerospaceWindow) -> bool {
        let app = present_text(self.target_window.app.as_deref());
        let bundle_id = present_text(self.target_window.bundle_id.as_deref());

        match (app, bundle_id) {
            (None, None) => true,
            (Some(app), None) => app == window.app_name,
            (None, Some(bundle_id)) => bundle_id == window.app_bundle_id,
            (Some(app), Some(bundle_id)) => {
                app == window.app_name && bundle_id == window.app_bundle_id
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stage::{StageWindow, StageWorkspace};

    fn target<'a>(window: &'a StageWindow, workspace: &'a StageWorkspace) -> ResolveTarget<'a> {
        ResolveTarget {
            target_window: window,
            target_workspace: workspace,
        }
    }

    #[test]
    fn matches_when_both_app_and_bundle_id_agree() {
        let window = StageWindow::dummy()
            .with_app("Safari")
            .with_bundle_id("com.apple.Safari");
        let workspace = StageWorkspace::dummy();
        let live = AerospaceWindow::dummy()
            .with_app_name("Safari")
            .with_bundle_id("com.apple.Safari");

        assert!(target(&window, &workspace).matches_window_app(&live));
    }

    #[test]
    fn rejects_when_app_matches_but_bundle_id_does_not() {
        let window = StageWindow::dummy()
            .with_app("Code")
            .with_bundle_id("com.microsoft.VSCode");
        let workspace = StageWorkspace::dummy();
        let live = AerospaceWindow::dummy()
            .with_app_name("Code")
            .with_bundle_id("com.visualstudio.code");

        assert!(!target(&window, &workspace).matches_window_app(&live));
    }

    #[test]
    fn matches_on_app_alone_when_bundle_id_is_absent() {
        let window = StageWindow {
            app: Some("Safari".into()),
            title: Some("Inbox".into()),
            bundle_id: None,
        };
        let workspace = StageWorkspace::dummy();
        let live = AerospaceWindow::dummy()
            .with_app_name("Safari")
            .with_bundle_id("com.other.Safari");

        assert!(target(&window, &workspace).matches_window_app(&live));
    }

    #[test]
    fn title_only_target_matches_any_app() {
        let window = StageWindow {
            app: None,
            title: Some("Inbox".into()),
            bundle_id: None,
        };
        let workspace = StageWorkspace::dummy();
        let live = AerospaceWindow::dummy().with_app_name("Mail");

        assert!(target(&window, &workspace).matches_window_app(&live));
    }
}
