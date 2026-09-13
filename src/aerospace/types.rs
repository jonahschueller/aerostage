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

#[derive(Debug, Deserialize)]
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
