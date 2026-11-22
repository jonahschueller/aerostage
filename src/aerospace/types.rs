use std::fmt::Display;

use serde::{Deserialize, Deserializer};

pub type AerospaceWindowId = u32;
pub type AerospaceWorkspaceId = String;

fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}

/// An event streamed by `aerospace subscribe`. Only the kind is kept, the
/// payload is dropped.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(tag = "_event", rename_all = "kebab-case")]
pub enum AerospaceEvent {
    FocusChanged,
    FocusedMonitorChanged,
    FocusedWorkspaceChanged,
    ModeChanged,
    WindowDetected,
    BindingTriggered,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct AerospaceApp {
    #[allow(dead_code)]
    #[serde(
        rename = "app-bundle-id",
        default,
        deserialize_with = "null_to_default"
    )]
    pub app_bundle_id: String,
    #[allow(dead_code)]
    #[serde(rename = "app-name", default, deserialize_with = "null_to_default")]
    pub app_name: String,
    #[allow(dead_code)]
    #[serde(rename = "app-pid")]
    pub app_pid: u32,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AerospaceLayout {
    HTiles,
    VTiles,
    HAccordion,
    VAccordion,
}

impl Display for AerospaceLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            AerospaceLayout::HTiles => "h_tiles",
            AerospaceLayout::VTiles => "v_tiles",
            AerospaceLayout::HAccordion => "h_accordion",
            AerospaceLayout::VAccordion => "v_accordion",
        };

        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AerospaceWorkspace {
    pub workspace: AerospaceWorkspaceId,
    #[serde(rename = "workspace-root-container-layout")]
    pub layout: AerospaceLayout,
}

#[derive(Debug, Deserialize)]
pub struct AerospaceMonitor {
    #[allow(dead_code)]
    #[serde(rename = "monitor-id")]
    pub monitor_id: i32,
    #[allow(dead_code)]
    #[serde(rename = "monitor-name", default, deserialize_with = "null_to_default")]
    pub monitor_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AerospaceWindow {
    #[serde(rename = "window-id")]
    pub window_id: AerospaceWindowId,
    #[serde(rename = "window-title", default, deserialize_with = "null_to_default")]
    pub window_title: String,
    #[serde(rename = "app-name", default, deserialize_with = "null_to_default")]
    pub app_name: String,
    #[serde(
        rename = "app-bundle-id",
        default,
        deserialize_with = "null_to_default"
    )]
    pub app_bundle_id: String,
    #[serde(rename = "workspace", default, deserialize_with = "null_to_default")]
    pub workspace: AerospaceWorkspaceId,
}

impl AerospaceWindow {
    #[cfg(test)]
    pub fn dummy() -> Self {
        AerospaceWindow {
            app_bundle_id: "com.example.test".into(),
            app_name: "Test App".into(),
            window_id: 0,
            workspace: "1".into(),
            window_title: "Test Title".into(),
        }
    }

    #[cfg(test)]
    pub fn with_bundle_id(mut self, bundle_id: &str) -> Self {
        self.app_bundle_id = bundle_id.to_string();
        self
    }

    #[cfg(test)]
    pub fn with_window_id(mut self, window_id: u32) -> Self {
        self.window_id = window_id;
        self
    }

    #[cfg(test)]
    pub fn with_app_name(mut self, app_name: &str) -> Self {
        self.app_name = app_name.to_string();
        self
    }

    #[cfg(test)]
    pub fn with_workspace(mut self, workspace: &str) -> Self {
        self.workspace = workspace.to_string();
        self
    }

    #[cfg(test)]
    pub fn with_window_title(mut self, window_title: &str) -> Self {
        self.window_title = window_title.to_string();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> AerospaceEvent {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn parses_events_by_their_name() {
        assert_eq!(
            parse(r#"{"_event":"focus-changed","windowId":20481,"workspace":"2"}"#),
            AerospaceEvent::FocusChanged
        );
        assert_eq!(
            parse(r#"{"_event":"focus-changed","workspace":"2"}"#),
            AerospaceEvent::FocusChanged
        );
        assert_eq!(
            parse(r#"{"_event":"focused-monitor-changed","monitorId":2,"workspace":"2"}"#),
            AerospaceEvent::FocusedMonitorChanged
        );
        assert_eq!(
            parse(r#"{"_event":"focused-workspace-changed","prevWorkspace":"1","workspace":"2"}"#),
            AerospaceEvent::FocusedWorkspaceChanged
        );
        assert_eq!(
            parse(r#"{"_event":"mode-changed","mode":"main"}"#),
            AerospaceEvent::ModeChanged
        );
        assert_eq!(
            parse(
                r#"{"_event":"window-detected","windowId":7,"appBundleId":"com.apple.Safari","appName":"Safari"}"#
            ),
            AerospaceEvent::WindowDetected
        );
        assert_eq!(
            parse(r#"{"_event":"binding-triggered","binding":"alt-1","mode":"main"}"#),
            AerospaceEvent::BindingTriggered
        );
    }

    #[test]
    fn parses_unknown_events_as_unknown() {
        assert_eq!(
            parse(r#"{"_event":"window-closed","windowId":7}"#),
            AerospaceEvent::Unknown
        );
    }

    #[test]
    fn rejects_payloads_without_an_event_name() {
        let result = serde_json::from_str::<AerospaceEvent>(r#"{"exitCode":2,"stderr":"ERROR"}"#);

        assert!(result.is_err());
    }
}
